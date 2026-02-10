use domain::models::intent::ConditionalIntent;
use domain::ports::intent_parser_port::{IntentParserError, IntentParserPort};

pub struct ParseIntentUseCase<P> {
    parser: P,
}

#[derive(Debug)]
pub enum ParseIntentError {
    ParsingFailed(String),
    InvalidIntent(String),
    LlmError(String),
}

impl From<IntentParserError> for ParseIntentError {
    fn from(err: IntentParserError) -> Self {
        match err {
            IntentParserError::ParsingFailed(msg) => ParseIntentError::ParsingFailed(msg),
            IntentParserError::InvalidFormat(msg) => ParseIntentError::InvalidIntent(msg),
            IntentParserError::LlmError(msg) => ParseIntentError::LlmError(msg),
        }
    }
}

impl<P> ParseIntentUseCase<P>
where
    P: IntentParserPort,
{
    pub fn new(parser: P) -> Self {
        Self { parser }
    }

    pub fn execute(&self, text: &str) -> Result<ConditionalIntent, ParseIntentError> {
        // 1. Validate input
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(ParseIntentError::InvalidIntent(
                "Input text cannot be empty".to_string(),
            ));
        }

        // 2. Parse with the parser (LLM or regex)
        let conditional_intent = self.parser.parse(trimmed)?;

        // 3. Validate the parsed intent
        conditional_intent
            .intent
            .validate()
            .map_err(|e| ParseIntentError::InvalidIntent(format!("{:?}", e)))?;

        Ok(conditional_intent)
    }
}
