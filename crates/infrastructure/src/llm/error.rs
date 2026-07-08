/// Errors that can occur during LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Model not loaded")]
    ModelNotLoaded,
    #[error("Backend not loaded")]
    BackendNotLoaded,
    #[error("Failed to initialize backend: {0}")]
    BackendInit(String),
    #[error("Failed to load model: {0}")]
    ModelLoad(String),
    #[error("Failed to create context: {0}")]
    ContextCreation(String),
    #[error("Tokenization failed: {0}")]
    Tokenization(String),
    #[error("Generation failed: {0}")]
    Generation(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("JSON parsing failed: {0}")]
    JsonParsing(#[from] serde_json::Error),
}
