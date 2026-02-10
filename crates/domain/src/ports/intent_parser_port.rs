use crate::models::intent::ConditionalIntent;

pub trait IntentParserPort {
    fn parse(&self, text: &str) -> Result<ConditionalIntent, IntentParserError>;
}

#[derive(Debug, Clone)]
pub enum IntentParserError {
    ParsingFailed(String),
    InvalidFormat(String),
    LlmError(String),
}
