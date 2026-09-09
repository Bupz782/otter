use super::error::LlmError;
use domain::models::ConditionalIntent;
use llama_cpp_2::{
    context::LlamaContext,
    llama_batch::LlamaBatch,
    model::{AddBos, LlamaModel, Special},
    token::LlamaToken,
};
use tracing::debug;

/// Parses LLM response tokens into strings
pub struct ResponseParser;

impl ResponseParser {
    pub fn decode_token(model: &LlamaModel, token: LlamaToken) -> Result<String, LlmError> {
        model
            .token_to_str(token, Special::Plaintext)
            .map_err(|e| LlmError::Generation(format!("Token conversion failed: {}", e)))
    }

    pub fn is_eos(model: &LlamaModel, token: LlamaToken) -> bool {
        token == model.token_eos()
    }

    pub fn sample_greedy(ctx: &LlamaContext) -> Result<LlamaToken, LlmError> {
        ctx.candidates()
            .max_by(|a, b| {
                a.logit()
                    .partial_cmp(&b.logit())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|c| c.id())
            .ok_or_else(|| LlmError::Generation("No candidates found".to_string()))
    }
}

pub fn add_tokens_to_batch(batch: &mut LlamaBatch, tokens: &[LlamaToken]) -> Result<(), LlmError> {
    for (i, token) in tokens.iter().enumerate() {
        batch
            .add(*token, i as i32, &[0], i == tokens.len() - 1)
            .map_err(|e| LlmError::Generation(format!("Failed to add token: {}", e)))?;
    }
    Ok(())
}

pub fn decode_batch(ctx: &mut LlamaContext, batch: &mut LlamaBatch) -> Result<(), LlmError> {
    ctx.decode(batch)
        .map_err(|e| LlmError::Generation(format!("Decode failed: {}", e)))?;
    batch.clear();
    Ok(())
}

pub fn generate_tokens(
    model: &LlamaModel,
    ctx: &mut LlamaContext,
    batch: &mut LlamaBatch,
    n_cur: &mut usize,
    max_tokens: usize,
) -> Result<String, LlmError> {
    let mut output = String::new();
    let initial_tokens = *n_cur;
    let start = std::time::Instant::now();
    let mut generated_count = 0usize;

    // ChatML models (Qwen…) end an assistant turn with <|im_end|>, which is
    // not necessarily the GGUF eos token: without this the generation runs
    // past the answer and appends a new hallucinated turn to the output.
    let im_end_token = model
        .str_to_token("<|im_end|>", AddBos::Never)
        .ok()
        .and_then(|t| {
            if t.len() == 1 {
                t.first().copied()
            } else {
                None
            }
        });

    for _ in 0..max_tokens {
        let token = ResponseParser::sample_greedy(ctx)?;

        if ResponseParser::is_eos(model, token) || Some(token) == im_end_token {
            break;
        }
        // Fallback when the special token did not resolve to a single id:
        // compare its decoded string form instead.
        if im_end_token.is_none()
            && let Ok(piece) = model.token_to_str(token, Special::Tokenize)
            && piece == "<|im_end|>"
        {
            break;
        }

        let piece = ResponseParser::decode_token(model, token)?;
        output.push_str(&piece);

        batch
            .add(token, *n_cur as i32, &[0], true)
            .map_err(|e| LlmError::Generation(format!("Failed to add token: {}", e)))?;

        ctx.decode(batch)
            .map_err(|e| LlmError::Generation(format!("Decode failed: {}", e)))?;
        batch.clear();

        *n_cur += 1;
        generated_count += 1;
    }

    let duration = start.elapsed();
    debug!("Generated {} tokens in {:?}", generated_count, duration);
    debug!(
        "Input tokens: {}, Output tokens: {}",
        initial_tokens, generated_count
    );

    Ok(output)
}

/// Extract the JSON object from a chatty model output: trims markdown fences
/// and surrounding prose by keeping the span from the first `{` to the last
/// `}`. Returns the input trimmed when it contains no braces.
pub fn extract_json_candidate(output: &str) -> &str {
    let trimmed = output.trim();
    match (trimmed.find('{'), trimmed.rfind('}')) {
        (Some(start), Some(end)) if end >= start => &trimmed[start..=end],
        _ => trimmed,
    }
}

pub fn parse_intent(json_response: &str) -> Result<ConditionalIntent, serde_json::Error> {
    serde_json::from_str(extract_json_candidate(json_response))
}

pub struct Tokenizer<'a>(pub &'a LlamaModel);

impl<'a> Tokenizer<'a> {
    pub fn tokenize(&self, text: &str, add_bos: bool) -> Result<Vec<LlamaToken>, LlmError> {
        let add_bos_flag = if add_bos {
            AddBos::Always
        } else {
            AddBos::Never
        };
        self.0
            .str_to_token(text, add_bos_flag)
            .map_err(|e| LlmError::Tokenization(e.to_string()))
    }

    pub fn count_tokens(&self, text: &str, add_bos: bool) -> Result<usize, LlmError> {
        self.tokenize(text, add_bos)
            .map(|tokens: Vec<LlamaToken>| tokens.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::condition::{Comparator, Condition, Metric};
    use domain::models::intent::{Asset, DexType, Intent, LendingType};

    #[test]
    fn response_parser_creation() {
        let _parser = ResponseParser;
    }

    #[test]
    fn extract_json_candidate_strips_markdown_fence() {
        let output = "```json\n{\"a\": 1}\n```";
        assert_eq!(extract_json_candidate(output), "{\"a\": 1}");
    }

    #[test]
    fn extract_json_candidate_strips_surrounding_prose() {
        let output = "Here is the parsed intent:\n{\"a\": 1}\nI hope this helps!";
        assert_eq!(extract_json_candidate(output), "{\"a\": 1}");
    }

    #[test]
    fn extract_json_candidate_keeps_trailing_garbage_out() {
        let output = "{\"a\": 1}\n<|im_end|>\n<|im_start|>user\nanother turn";
        assert_eq!(extract_json_candidate(output), "{\"a\": 1}");
    }

    #[test]
    fn extract_json_candidate_without_braces_returns_trimmed_input() {
        assert_eq!(extract_json_candidate("  no json here  "), "no json here");
        assert_eq!(extract_json_candidate(""), "");
    }

    #[test]
    fn parse_intent_accepts_fenced_and_chatty_output() {
        let output = "```json\n{\n  \"intent\": { \"Lend\": { \"asset\": \"Usdc\", \"amount\": 1000000000, \"protocol\": \"Aave\" } },\n  \"condition\": null\n}\n```";

        let parsed = parse_intent(output).unwrap();
        assert!(matches!(parsed.intent, Intent::Lend { .. }));
    }

    #[test]
    fn parse_intent_accepts_output_with_trailing_turn() {
        let output = "{\"intent\": { \"Stake\": { \"asset\": \"Eth\", \"amount\": 5, \"protocol\": \"Aave\" } }, \"condition\": null}\n<|im_end|>\n<|im_start|>user\nignore me";

        let parsed = parse_intent(output).unwrap();
        assert!(matches!(parsed.intent, Intent::Stake { .. }));
    }

    #[test]
    fn parse_intent_accepts_swap_with_condition() {
        let json = r#"{
            "intent": {
                "Swap": {
                    "from_asset": "Eth",
                    "to_asset": "Usdc",
                    "amount": 1000000000000000000,
                    "protocol": "Uniswap"
                }
            },
            "condition": {
                "Comparison": {
                    "metric": "Price",
                    "comparator": "GreaterThan",
                    "value": 2000
                }
            }
        }"#;

        let parsed = parse_intent(json).unwrap();
        match parsed.intent {
            Intent::Swap {
                from_asset,
                to_asset,
                amount,
                protocol,
            } => {
                assert_eq!(from_asset, Asset::Eth);
                assert_eq!(to_asset, Asset::Usdc);
                assert_eq!(amount, 1_000_000_000_000_000_000);
                assert_eq!(protocol, DexType::Uniswap);
            }
            other => panic!("expected Swap intent, got {other:?}"),
        }
        assert_eq!(
            parsed.condition,
            Some(Condition::Comparison {
                metric: Metric::Price,
                comparator: Comparator::GreaterThan,
                value: 2_000,
            })
        );
    }

    #[test]
    fn parse_intent_accepts_null_condition() {
        let json = r#"{
            "intent": {
                "Stake": {
                    "asset": "Eth",
                    "amount": 500,
                    "protocol": "Aave"
                }
            },
            "condition": null
        }"#;

        let parsed = parse_intent(json).unwrap();
        assert!(matches!(
            parsed.intent,
            Intent::Stake {
                asset: Asset::Eth,
                amount: 500,
                protocol: LendingType::Aave,
            }
        ));
        assert!(parsed.condition.is_none());
    }

    #[test]
    fn parse_intent_accepts_composite_intent() {
        let json = r#"{
            "intent": {
                "Composite": {
                    "intents": [
                        {
                            "Swap": {
                                "from_asset": "Eth",
                                "to_asset": "Dai",
                                "amount": 10,
                                "protocol": "Sushiswap"
                            }
                        },
                        {
                            "Lend": {
                                "asset": "Dai",
                                "amount": 10,
                                "protocol": "Compound"
                            }
                        }
                    ]
                }
            },
            "condition": null
        }"#;

        let parsed = parse_intent(json).unwrap();
        match parsed.intent {
            Intent::Composite { intents } => {
                assert_eq!(intents.len(), 2);
                assert!(matches!(intents[0], Intent::Swap { .. }));
                assert!(matches!(intents[1], Intent::Lend { .. }));
            }
            other => panic!("expected Composite intent, got {other:?}"),
        }
    }

    #[test]
    fn parse_intent_round_trips_serialized_intent() {
        let original = ConditionalIntent {
            intent: Intent::Borrow {
                asset: Asset::Usdc,
                amount: 1_000_000,
                collateral: Asset::Wbtc,
                collateral_amount: 50_000_000,
                protocol: LendingType::Aave,
            },
            condition: Some(Condition::Comparison {
                metric: Metric::Yield,
                comparator: Comparator::LessThanOrEqualTo,
                value: 750,
            }),
            network: None,
        };
        let json = serde_json::to_string(&original).unwrap();

        let parsed = parse_intent(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn parse_intent_rejects_malformed_json() {
        assert!(parse_intent("{ not json").is_err());
        assert!(parse_intent("").is_err());
        assert!(parse_intent("[]").is_err());
    }

    #[test]
    fn parse_intent_rejects_missing_intent_field() {
        let json = r#"{"condition": null}"#;
        assert!(parse_intent(json).is_err());
    }

    #[test]
    fn parse_intent_rejects_unknown_intent_variant() {
        let json = r#"{
            "intent": { "Fly": { "amount": 1 } },
            "condition": null
        }"#;
        assert!(parse_intent(json).is_err());
    }

    #[test]
    fn parse_intent_rejects_wrong_field_type() {
        // `amount` is a u128; a string must be rejected.
        let json = r#"{
            "intent": {
                "Swap": {
                    "from_asset": "Eth",
                    "to_asset": "Usdc",
                    "amount": "one ether",
                    "protocol": "Uniswap"
                }
            },
            "condition": null
        }"#;
        assert!(parse_intent(json).is_err());
    }

    #[test]
    fn parse_intent_rejects_unknown_asset() {
        let json = r#"{
            "intent": {
                "Swap": {
                    "from_asset": "Doge",
                    "to_asset": "Usdc",
                    "amount": 1,
                    "protocol": "Uniswap"
                }
            },
            "condition": null
        }"#;
        assert!(parse_intent(json).is_err());
    }
}
