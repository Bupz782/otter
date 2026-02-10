use domain::models::intent::{Intent, ValidationError};

pub struct ValidateIntentUseCase;

#[derive(Debug)]
pub enum ValidateIntentError {
    InvalidAmount(String),
    InvalidAsset(String),
    InvalidProtocol(String),
    MissingField(String),
}

impl From<ValidationError> for ValidateIntentError {
    fn from(err: ValidationError) -> Self {
        match err {
            ValidationError::InvalidAmount(msg) => ValidateIntentError::InvalidAmount(msg),
            ValidationError::InvalidAsset(msg) => ValidateIntentError::InvalidAsset(msg),
            ValidationError::InvalidProtocol(msg) => ValidateIntentError::InvalidProtocol(msg),
            ValidationError::MissingField(msg) => ValidateIntentError::MissingField(msg),
        }
    }
}

impl ValidateIntentUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, intent: &Intent) -> Result<(), ValidateIntentError> {
        intent.validate()?;
        Ok(())
    }
}

impl Default for ValidateIntentUseCase {
    fn default() -> Self {
        Self::new()
    }
}
