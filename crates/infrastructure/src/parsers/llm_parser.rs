use domain::models::ConditionalIntent;
use domain::ports::intent_parser_port::{IntentParserError, IntentParserPort};
use std::cell::RefCell;

use crate::llm::{IntentOutput, LlmConfig, LocalLlmClient};

/// Adapter that wires the local LLM client to the domain's `IntentParserPort`.
///
/// The client must already be loaded (`client.load()`) before calling `parse`.
/// Caching is handled inside the underlying `LocalLlmClient`.
pub struct LlmIntentParser {
    client: RefCell<LocalLlmClient>,
}

impl LlmIntentParser {
    /// System prompt that instructs the model to emit JSON matching
    /// `domain::models::ConditionalIntent`.
    pub const SYSTEM_PROMPT: &'static str = r#"You are a DeFi intent parser.
Convert the user's instruction into a compact JSON object matching this Rust structure:

{
  "intent": { "Lend": { "asset": "Usdc", "amount": 1000000000, "protocol": "Aave" } },
  "condition": { "Comparison": { "metric": "Yield", "comparator": "GreaterThan", "value": 3 } }
}

Rules:
- Output ONLY valid JSON. No markdown, no explanations.
- Intent variants: Lend, Borrow, Swap, Stake, Composite.
- Asset values: Eth, Dai, Usdc, Wbtc, Link, Sol.
- Lending protocol values: Aave, Compound.
- DEX protocol values: Uniswap, Sushiswap, Balancer.
- Metric values: Yield, Price, GasCost, Volume.
- Comparator values: GreaterThan, LessThan, EqualTo, GreaterThanOrEqualTo, LessThanOrEqualTo.
- Amounts must be integers in the asset's base units (e.g. USDC has 6 decimals, so 1000 USDC = 1000000000).
- If there is no condition, set "condition": null.
- For borrow, include "collateral" and "collateral_amount" fields.
- For swap, include "from_asset", "to_asset", and "protocol" fields.
"#;

    /// Create a parser with default configuration. The model is NOT loaded yet;
    /// call `load()` on the underlying client before parsing.
    pub fn new(model_path: impl Into<String>) -> Self {
        let client = LocalLlmClient::new(model_path, Self::SYSTEM_PROMPT);
        Self {
            client: RefCell::new(client),
        }
    }

    /// Create a parser with a custom `LlmConfig`.
    pub fn with_config(model_path: impl Into<String>, config: LlmConfig) -> Self {
        let client = LocalLlmClient::with_config(model_path, Self::SYSTEM_PROMPT, config);
        Self {
            client: RefCell::new(client),
        }
    }

    /// Expose the underlying client (e.g. to load the model).
    pub fn client(&self) -> std::cell::Ref<'_, LocalLlmClient> {
        self.client.borrow()
    }

    /// Expose the underlying client mutably.
    pub fn client_mut(&self) -> std::cell::RefMut<'_, LocalLlmClient> {
        self.client.borrow_mut()
    }
}

impl IntentParserPort for LlmIntentParser {
    fn parse(&self, text: &str) -> Result<ConditionalIntent, IntentParserError> {
        let mut client = self.client.borrow_mut();

        if !client.is_loaded() {
            return Err(IntentParserError::LlmError(
                "LLM model is not loaded. Call client.load() first.".to_string(),
            ));
        }

        match client.generate(text, 256) {
            Ok(IntentOutput::Conditional(intent)) => Ok(intent),
            Ok(IntentOutput::Raw(raw)) => Err(IntentParserError::InvalidFormat(format!(
                "LLM returned raw text: {}",
                raw
            ))),
            Err(e) => Err(IntentParserError::LlmError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_reports_error_when_model_not_loaded() {
        let parser = LlmIntentParser::new("/dev/null/model.gguf");
        let result = parser.parse("lend 100 USDC on Aave");
        assert!(matches!(result, Err(IntentParserError::LlmError(_))));
    }
}
