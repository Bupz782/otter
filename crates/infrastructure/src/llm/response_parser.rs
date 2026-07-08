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

    for _ in 0..max_tokens {
        let token = ResponseParser::sample_greedy(ctx)?;

        if ResponseParser::is_eos(model, token) {
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

pub fn parse_intent(json_response: &str) -> Result<ConditionalIntent, serde_json::Error> {
    let result: Result<ConditionalIntent, _> = serde_json::from_str(json_response);
    result
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

    #[test]
    fn test_response_parser_creation() {
        let _parser = ResponseParser;
    }
}
