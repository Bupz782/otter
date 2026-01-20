#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidAmount(String),
    UnknownAsset(String),
    UnknownProtocol(String),
    UnknownMetric(String),
    InvalidFormat(String),
    MissingField(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::InvalidAmount(s) => write!(f, "Invalid amount: {}", s),
            ParseError::UnknownAsset(s) => write!(f, "Unknown asset: {}", s),
            ParseError::UnknownProtocol(s) => write!(f, "Unknown protocol: {}", s),
            ParseError::UnknownMetric(s) => write!(f, "Unknown metric: {}", s),
            ParseError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            ParseError::MissingField(s) => write!(f, "Missing field: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}
