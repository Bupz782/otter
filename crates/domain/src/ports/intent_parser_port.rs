use crate::models::intent::ConditionalIntent;

pub trait IntentParserPort {
    fn parse(&self, text: &str) -> Result<ConditionalIntent, IntentParserError>;
}

impl<T: IntentParserPort + ?Sized> IntentParserPort for &T {
    fn parse(&self, text: &str) -> Result<ConditionalIntent, IntentParserError> {
        (*self).parse(text)
    }
}

impl<T: IntentParserPort + ?Sized> IntentParserPort for std::sync::Arc<T> {
    fn parse(&self, text: &str) -> Result<ConditionalIntent, IntentParserError> {
        (**self).parse(text)
    }
}

#[derive(Debug, Clone)]
pub enum IntentParserError {
    ParsingFailed(String),
    InvalidFormat(String),
    LlmError(String),
    /// The model explicitly refused the request as out-of-vocabulary
    /// (`{"error": "unsupported"}`): a definitive answer, not a failure —
    /// callers must NOT fall back to another parser.
    Unsupported(String),
}

impl std::fmt::Display for IntentParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParsingFailed(msg) => write!(f, "parsing failed: {}", msg),
            Self::InvalidFormat(msg) => write!(f, "invalid format: {}", msg),
            Self::LlmError(msg) => write!(f, "LLM error: {}", msg),
            Self::Unsupported(msg) => write!(f, "unsupported intent: {}", msg),
        }
    }
}

impl std::error::Error for IntentParserError {}
