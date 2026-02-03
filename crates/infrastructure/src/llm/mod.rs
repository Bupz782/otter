//! LLM (Large Language Model) module for local inference using llama.cpp
//!
//! This module provides components for loading and running LLMs locally:
//!
//! - `LocalLlmClient`: Main client for text generation
//! - `LlmConfig`: Configuration for inference parameters
//! - `ModelLoader`: Handles loading of GGUF model files
//! - `PromptBuilder`: Formats prompts for the model
//! - `ResponseParser`/`TokenGenerator`: Handles token generation and decoding
//! - `Cache`: Response caching for improved performance
//!
//! # Example
//!
//! ```rust,no_run
//! use infrastructure::llm::{LocalLlmClient, LlmConfig};
//!
//! let mut client = LocalLlmClient::new(
//!     "path/to/model.gguf",
//!     "You are a helpful assistant."
//! );
//!
//! client.load().expect("Failed to load model");
//! let response = client.generate("Hello!", 100).expect("Generation failed");
//! ```

mod cache;
mod config;
mod error;
mod local_client;
mod model_loader;
mod prompt_builder;
mod response_parser;

// Main exports
pub use cache::{CachedClient, CacheStats, PromptCache, ResponseCache};
pub use config::LlmConfig;
pub use error::LlmError;
pub use local_client::LocalLlmClient;

// Component exports (for advanced usage)
pub use model_loader::{LoadedModel, ModelLoader};
pub use prompt_builder::{ChatMessage, MessageRole, PromptBuilder};
pub use response_parser::{ResponseParser, Tokenizer};
