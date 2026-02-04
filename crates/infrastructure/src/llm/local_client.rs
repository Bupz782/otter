use super::{
    cache::{CachedClient, PromptCache},
    config::LlmConfig,
    error::LlmError,
    model_loader::LoadedModel,
    prompt_builder::PromptBuilder,
    response_parser::{
        Tokenizer, add_tokens_to_batch, decode_batch, generate_tokens, parse_intent,
    },
};
use domain::models::ConditionalIntent;
use llama_cpp_2::{context::params::LlamaContextParams, llama_batch::LlamaBatch};
use std::num::NonZeroU32;

/// Local LLM client using llama.cpp with modular components
///
/// This client combines:
/// - `ModelLoader` for loading GGUF models
/// - `PromptBuilder` for formatting prompts
/// - Token generation functions from `response_parser`
/// - `PromptCache` for response caching
pub struct LocalLlmClient {
    model_path: String,
    config: LlmConfig,
    prompt_builder: PromptBuilder,
    loaded_model: Option<LoadedModel>,
    cache: PromptCache,
    use_cache: bool,
}

#[derive(Debug, Clone)]
pub enum IntentOutput {
    Conditional(ConditionalIntent),
    Raw(String),
}

impl LocalLlmClient {
    /// Create a new LLM client with default configuration
    pub fn new(model: impl Into<String>, system_prompt: impl Into<String>) -> Self {
        Self {
            model_path: model.into(),
            config: LlmConfig::default(),
            prompt_builder: PromptBuilder::new(system_prompt),
            loaded_model: None,
            cache: PromptCache::default(),
            use_cache: true,
        }
    }

    /// Create a new LLM client with custom configuration
    pub fn with_config(
        model: impl Into<String>,
        system_prompt: impl Into<String>,
        config: LlmConfig,
    ) -> Self {
        Self {
            model_path: model.into(),
            config,
            prompt_builder: PromptBuilder::new(system_prompt),
            loaded_model: None,
            cache: PromptCache::new(100),
            use_cache: true,
        }
    }

    /// Load the model into memory
    pub fn load(&mut self) -> Result<(), LlmError> {
        let loaded = LoadedModel::from_path(&self.model_path)?;
        self.loaded_model = Some(loaded);
        Ok(())
    }

    /// Generate text from a prompt
    pub fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<IntentOutput, LlmError> {
        // Check cache first if enabled
        if self.use_cache
            && let Some(cached) = self.cache.get(&prompt.to_string()) {
                return Ok(IntentOutput::Raw(cached.clone()));
            }

        let loaded = self.loaded_model.as_ref().ok_or(LlmError::ModelNotLoaded)?;

        // Build the prompt
        let full_prompt = self.prompt_builder.build(prompt);

        // Create context
        let n_ctx = NonZeroU32::new(self.config.n_ctx)
            .ok_or(LlmError::InvalidConfig("n_ctx must be > 0".to_string()))?;
        let ctx_params = LlamaContextParams::default().with_n_ctx(Some(n_ctx));

        let mut ctx = loaded
            .model
            .new_context(&loaded.backend, ctx_params)
            .map_err(|e| LlmError::ContextCreation(e.to_string()))?;

        // Tokenize
        let tokenizer = Tokenizer(&loaded.model);
        let tokens = tokenizer.tokenize(&full_prompt, true)?;

        // Setup batch
        let mut batch = LlamaBatch::new(self.config.batch_size, 1);
        let mut n_cur = tokens.len();

        // Add tokens and initial decode
        add_tokens_to_batch(&mut batch, &tokens)?;
        decode_batch(&mut ctx, &mut batch)?;

        // Generate response
        let output = generate_tokens(&loaded.model, &mut ctx, &mut batch, &mut n_cur, max_tokens)?;

        if self.use_cache {
            self.cache.insert(prompt.to_string(), output.clone());
        }

        let output_intent = parse_intent(&output)?;

        Ok(IntentOutput::Conditional(output_intent))
    }

    /// Generate text with system prompt, returns raw string (no JSON parsing)
    pub fn generate_text(&self, prompt: &str, max_tokens: usize) -> Result<String, LlmError> {
        let loaded = self.loaded_model.as_ref().ok_or(LlmError::ModelNotLoaded)?;

        // Build the prompt with system prompt
        let full_prompt = self.prompt_builder.build(prompt);

        // Create context
        let n_ctx = NonZeroU32::new(self.config.n_ctx)
            .ok_or(LlmError::InvalidConfig("n_ctx must be > 0".to_string()))?;
        let ctx_params = LlamaContextParams::default().with_n_ctx(Some(n_ctx));

        let mut ctx = loaded
            .model
            .new_context(&loaded.backend, ctx_params)
            .map_err(|e| LlmError::ContextCreation(e.to_string()))?;

        // Tokenize
        let tokenizer = Tokenizer(&loaded.model);
        let tokens = tokenizer.tokenize(&full_prompt, true)?;

        // Setup batch
        let mut batch = LlamaBatch::new(self.config.batch_size, 1);
        let mut n_cur = tokens.len();

        // Add tokens and initial decode
        add_tokens_to_batch(&mut batch, &tokens)?;
        decode_batch(&mut ctx, &mut batch)?;

        // Generate response
        generate_tokens(&loaded.model, &mut ctx, &mut batch, &mut n_cur, max_tokens)
    }

    /// Generate text with a raw prompt (no system prompt prepended)
    pub fn generate_raw(&self, raw_prompt: &str, max_tokens: usize) -> Result<String, LlmError> {
        let loaded = self.loaded_model.as_ref().ok_or(LlmError::ModelNotLoaded)?;

        // Create context
        let n_ctx = NonZeroU32::new(self.config.n_ctx)
            .ok_or(LlmError::InvalidConfig("n_ctx must be > 0".to_string()))?;
        let ctx_params = LlamaContextParams::default().with_n_ctx(Some(n_ctx));

        let mut ctx = loaded
            .model
            .new_context(&loaded.backend, ctx_params)
            .map_err(|e| LlmError::ContextCreation(e.to_string()))?;

        // Tokenize
        let tokenizer = Tokenizer(&loaded.model);
        let tokens = tokenizer.tokenize(raw_prompt, true)?;

        // Setup batch
        let mut batch = LlamaBatch::new(self.config.batch_size, 1);
        let mut n_cur = tokens.len();

        // Add tokens and initial decode
        add_tokens_to_batch(&mut batch, &tokens)?;
        decode_batch(&mut ctx, &mut batch)?;

        // Generate response
        generate_tokens(&loaded.model, &mut ctx, &mut batch, &mut n_cur, max_tokens)
    }

    pub fn set_config(&mut self, config: LlmConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }

    /// Check if model is loaded
    pub fn is_loaded(&self) -> bool {
        self.loaded_model.is_some()
    }

    /// Unload the model to free memory
    pub fn unload(&mut self) {
        self.loaded_model = None;
    }

    /// Get the model path
    pub fn model_path(&self) -> &str {
        &self.model_path
    }

    /// Get the prompt builder
    pub fn prompt_builder(&self) -> &PromptBuilder {
        &self.prompt_builder
    }

    /// Get mutable prompt builder
    pub fn prompt_builder_mut(&mut self) -> &mut PromptBuilder {
        &mut self.prompt_builder
    }

    /// Enable/disable response caching
    pub fn set_caching(&mut self, enabled: bool) {
        self.use_cache = enabled;
    }

    /// Check if caching is enabled
    pub fn is_caching_enabled(&self) -> bool {
        self.use_cache
    }

    /// Clear the response cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Wrap this client in a CachedClient wrapper
    pub fn with_cache(self, max_size: usize) -> CachedClient<Self> {
        CachedClient::new(self, max_size)
    }
}

impl CachedClient<LocalLlmClient> {
    /// Convenience method to generate with caching stats
    pub fn generate_cached(
        &mut self,
        prompt: &str,
        max_tokens: usize,
    ) -> Result<IntentOutput, LlmError> {
        if let Some(cached) = self.get_cached(prompt) {
            return Ok(IntentOutput::Raw(cached));
        }

        let response = self.inner_mut().generate(prompt, max_tokens)?;

        if let IntentOutput::Raw(ref text) = response {
            self.cache_response(prompt.to_string(), text.clone());
        }

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LocalLlmClient::new("/path/to/model.gguf", "System prompt");
        assert_eq!(client.model_path(), "/path/to/model.gguf");
        assert!(!client.is_loaded());
        assert!(client.is_caching_enabled());
    }

    #[test]
    fn test_client_with_config() {
        let config = LlmConfig::new().with_n_ctx(4096).with_temperature(0.5);

        let client = LocalLlmClient::with_config("/path/to/model.gguf", "System prompt", config);

        assert_eq!(client.config().n_ctx, 4096);
        assert_eq!(client.config().temperature, 0.5);
    }

    #[test]
    fn test_set_config() {
        let mut client = LocalLlmClient::new("/path/to/model.gguf", "System prompt");

        let new_config = LlmConfig::creative();
        client.set_config(new_config);

        assert_eq!(client.config().temperature, 0.9);
    }

    #[test]
    fn test_unload() {
        let mut client = LocalLlmClient::new("/path/to/model.gguf", "System prompt");
        client.unload();
        assert!(!client.is_loaded());
    }

    #[test]
    fn test_caching_controls() {
        let mut client = LocalLlmClient::new("/path/to/model.gguf", "System prompt");

        assert!(client.is_caching_enabled());

        client.set_caching(false);
        assert!(!client.is_caching_enabled());

        client.clear_cache();
        assert_eq!(client.cache_size(), 0);
    }

    #[test]
    fn test_prompt_builder_access() {
        let client = LocalLlmClient::new("/path/to/model.gguf", "You are a parser");

        assert_eq!(client.prompt_builder().system_prompt(), "You are a parser");
    }
}
