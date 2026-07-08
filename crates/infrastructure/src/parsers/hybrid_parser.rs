use domain::models::ConditionalIntent;
use domain::ports::intent_parser_port::{IntentParserError, IntentParserPort};

use super::llm_parser::LlmIntentParser;
use super::regex_parser::RegexParser;

/// Hybrid parser: try the LLM first, fall back to the rule-based regex parser
/// on failure or invalid output.
///
/// This gives the flexibility of natural-language understanding while keeping
/// deterministic behavior for well-known patterns.
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
        match self.parse_llm(text) {
            Ok(intent) => Ok(intent),
            Err(llm_err) => {
                // Fallback to regex; preserve the LLM error if regex also fails.
                self.parse_regex(text).map_err(|regex_err| {
                    IntentParserError::ParsingFailed(format!(
                        "LLM failed ({}); regex fallback failed ({})",
                        llm_err, regex_err
                    ))
                })
            }
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
