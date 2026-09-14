use domain::models::ConditionalIntent;
use domain::ports::intent_parser_port::{IntentParserError, IntentParserPort};
use std::sync::{Mutex, MutexGuard};

use crate::llm::{IntentOutput, LlmConfig, LocalLlmClient};

/// Adapter that wires the local LLM client to the domain's `IntentParserPort`.
///
/// The client must already be loaded (`client.load()`) before calling `parse`.
/// Caching is handled inside the underlying `LocalLlmClient`.
pub struct LlmIntentParser {
    client: Mutex<LocalLlmClient>,
}

impl LlmIntentParser {
    /// System prompt that instructs the model to emit JSON matching
    /// `domain::models::ConditionalIntent`.
    ///
    /// The vocabulary is CLOSED: the model must never invent enum values
    /// (a live failure mapped "utilization" to an unknown metric variant).
    /// Unsupported requests must be refused with `{"error": "unsupported"}`
    /// instead of approximated — the parse error then surfaces honestly.
    pub const SYSTEM_PROMPT: &'static str = r#"You are a DeFi intent parser.
Convert the user's instruction (English or French) into ONE JSON object matching this Rust structure:

{ "intent": { "<Variant>": { ...fields } }, "condition": { "Comparison": { "metric": "...", "comparator": "...", "value": 0 } } | null }

Closed vocabulary — use ONLY these values, never invent new ones:
- Intent variants and their fields:
  - Lend:      { "asset", "amount", "protocol" }
  - Stake:     { "asset", "amount", "protocol" }
  - Borrow:    { "asset", "amount", "collateral", "collateral_amount", "protocol" }
  - Swap:      { "from_asset", "to_asset", "amount", "protocol" }
  - Composite: { "intents": [ <intent variant objects> ] } — for multi-action requests ("do X and Y")
- Lending/staking protocols: Aave, Compound. DEX protocols: Uniswap, Sushiswap, Balancer.
- Assets: Eth, Dai, Usdc, Wbtc, Link, Sol.
- Metrics: Yield, Price, GasCost, Volume.
- Comparators: GreaterThan, LessThan, EqualTo, GreaterThanOrEqualTo, LessThanOrEqualTo.

Amounts are integers in the asset's base units. Decimals per asset: Eth/Dai/Link = 18, Usdc = 6, Wbtc = 8, Sol = 9.
Examples: 1 ETH = 1000000000000000000 ; 0.5 ETH = 500000000000000000 ; 1000 USDC = 1000000000 ; 0.1 WBTC = 10000000.

If the request cannot be expressed with this exact vocabulary (unsupported action like "withdraw", unsupported metric like "utilization"), output exactly:
{"error": "unsupported"}
Never approximate: do not map unknown actions or metrics to the closest known one.

Examples:
"swap 250 USDC to ETH" ->
{"intent":{"Swap":{"from_asset":"Usdc","to_asset":"Eth","amount":250000000,"protocol":"Uniswap"}},"condition":null}

"Swap 1 ETH for USDC on Uniswap when gas < 20 gwei" ->
{"intent":{"Swap":{"from_asset":"Eth","to_asset":"Usdc","amount":1000000000000000000,"protocol":"Uniswap"}},"condition":{"Comparison":{"metric":"GasCost","comparator":"LessThan","value":20}}}

"lend 1000 USDC on Aave if yield > 4%" ->
{"intent":{"Lend":{"asset":"Usdc","amount":1000000000,"protocol":"Aave"}},"condition":{"Comparison":{"metric":"Yield","comparator":"GreaterThan","value":4}}}

"withdraw 2000 USDC from Compound if utilization > 85%" ->
{"error":"unsupported"}

Your entire reply is ONE JSON object: first character "{", last character "}". No markdown, no explanation, no text before or after.
"#;

    /// Create a parser with default configuration. The model is NOT loaded yet;
    /// call `load()` on the underlying client before parsing.
    pub fn new(model_path: impl Into<String>) -> Self {
        let client = LocalLlmClient::new(model_path, Self::SYSTEM_PROMPT);
        Self {
            client: Mutex::new(client),
        }
    }

    /// Create a parser with a custom `LlmConfig`.
    pub fn with_config(model_path: impl Into<String>, config: LlmConfig) -> Self {
        let client = LocalLlmClient::with_config(model_path, Self::SYSTEM_PROMPT, config);
        Self {
            client: Mutex::new(client),
        }
    }

    /// Expose the underlying client (e.g. to load the model).
    pub fn client(&self) -> MutexGuard<'_, LocalLlmClient> {
        self.client.lock().expect("llm client mutex poisoned")
    }

    /// Expose the underlying client mutably.
    pub fn client_mut(&self) -> MutexGuard<'_, LocalLlmClient> {
        self.client.lock().expect("llm client mutex poisoned")
    }
}

impl IntentParserPort for LlmIntentParser {
    fn parse(&self, text: &str) -> Result<ConditionalIntent, IntentParserError> {
        let mut client = self
            .client
            .lock()
            .map_err(|_| IntentParserError::LlmError("llm client mutex poisoned".to_string()))?;

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

    /// The prompt is a contract with the model: every enum value the domain
    /// accepts must be listed, and the refusal rule for out-of-vocabulary
    /// requests must stay (live failure: "utilization" became an unknown
    /// `Utilization` metric variant instead of a clean rejection).
    #[test]
    fn system_prompt_lists_the_full_closed_vocabulary() {
        let prompt = LlmIntentParser::SYSTEM_PROMPT;
        for variant in ["Lend", "Stake", "Borrow", "Swap", "Composite"] {
            assert!(prompt.contains(variant), "missing intent variant {variant}");
        }
        for asset in ["Eth", "Dai", "Usdc", "Wbtc", "Link", "Sol"] {
            assert!(prompt.contains(asset), "missing asset {asset}");
        }
        for protocol in ["Aave", "Compound", "Uniswap", "Sushiswap", "Balancer"] {
            assert!(prompt.contains(protocol), "missing protocol {protocol}");
        }
        for metric in ["Yield", "Price", "GasCost", "Volume"] {
            assert!(prompt.contains(metric), "missing metric {metric}");
        }
        for comparator in [
            "GreaterThan",
            "LessThan",
            "EqualTo",
            "GreaterThanOrEqualTo",
            "LessThanOrEqualTo",
        ] {
            assert!(
                prompt.contains(comparator),
                "missing comparator {comparator}"
            );
        }
        assert!(
            prompt.contains(r#"{"error": "unsupported"}"#),
            "the refusal contract for out-of-vocabulary requests must stay"
        );
    }
}
