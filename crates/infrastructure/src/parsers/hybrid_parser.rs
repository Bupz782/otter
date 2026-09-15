use domain::models::ConditionalIntent;
use domain::ports::intent_parser_port::{IntentParserError, IntentParserPort};

use super::llm_parser::LlmIntentParser;
use super::regex_parser::RegexParser;

/// Hybrid parser: try the deterministic regex parser first, fall back to the
/// LLM for everything it cannot express.
///
/// The demo runs the LLM on CPU (Qwen3-8B ≈ 60 s per inference in Docker), so
/// sending canonical phrases through the LLM first made every parse look
/// broken in the UI. Regex hits are exact by construction (the grammar
/// requires action + amounts), so canonical phrases answer in milliseconds;
/// free-form text still gets full LLM understanding.
pub struct HybridParser {
    llm: LlmIntentParser,
    regex: RegexParser,
}

impl HybridParser {
    /// Create a new hybrid parser backed by the given LLM client adapter.
    pub fn new(llm: LlmIntentParser) -> Self {
        Self {
            llm,
            regex: RegexParser::new(),
        }
    }

    /// Create a hybrid parser from a model path and custom LLM config.
    pub fn with_config(model_path: impl Into<String>, config: crate::llm::LlmConfig) -> Self {
        Self::new(LlmIntentParser::with_config(model_path, config))
    }

    /// Attempt LLM parsing only.
    pub fn parse_llm(&self, text: &str) -> Result<ConditionalIntent, IntentParserError> {
        self.llm.parse(text)
    }

    /// Attempt regex parsing only.
    pub fn parse_regex(&self, text: &str) -> Result<ConditionalIntent, IntentParserError> {
        self.regex
            .parse_conditional_intent(text)
            .map_err(|e| IntentParserError::ParsingFailed(e.to_string()))
    }
}

impl IntentParserPort for HybridParser {
    fn parse(&self, text: &str) -> Result<ConditionalIntent, IntentParserError> {
        if let Ok(intent) = self.parse_regex(text) {
            return Ok(intent);
        }
        match self.parse_llm(text) {
            Ok(intent) => Ok(intent),
            // An explicit refusal is a definitive answer, not a failure.
            Err(err @ IntentParserError::Unsupported(_)) => Err(err),
            Err(llm_err) => Err(IntentParserError::ParsingFailed(format!(
                "regex failed; LLM fallback failed ({})",
                llm_err
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::LlmConfig;

    #[test]
    fn regex_fallback_works_when_llm_is_unloaded() {
        let parser = HybridParser::with_config("/dev/null/model.gguf", LlmConfig::default());
        let result = parser.parse("lend 1000 USDC on Aave if yield > 5");

        assert!(
            result.is_ok(),
            "Expected regex fallback to succeed: {:?}",
            result
        );
        let ci = result.unwrap();
        assert!(matches!(ci.intent, domain::models::Intent::Lend { .. }));
        assert!(ci.condition.is_some());
    }

    #[test]
    fn regex_fallback_handles_unconditional_intent() {
        let parser = HybridParser::with_config("/dev/null/model.gguf", LlmConfig::default());
        let result = parser.parse("swap 1 ETH for USDC on Uniswap");

        assert!(result.is_ok());
        let ci = result.unwrap();
        assert!(matches!(ci.intent, domain::models::Intent::Swap { .. }));
        assert!(ci.condition.is_none());
    }
}
