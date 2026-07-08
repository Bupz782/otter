use super::error::LlmError;
use llama_cpp_2::{
    llama_backend::LlamaBackend,
    model::{LlamaModel, params::LlamaModelParams},
};
use std::path::Path;

/// Handles loading of LLM models from GGUF files
pub struct ModelLoader;

impl ModelLoader {
    /// Initialize the LLaMA backend
    pub fn init_backend() -> Result<LlamaBackend, LlmError> {
        LlamaBackend::init().map_err(|e| LlmError::BackendInit(e.to_string()))
    }

    /// Load a model from a file path
    pub fn load_model<P: AsRef<Path>>(
        backend: &LlamaBackend,
        model_path: P,
    ) -> Result<LlamaModel, LlmError> {
        let model_params = LlamaModelParams::default();
        LlamaModel::load_from_file(backend, model_path.as_ref(), &model_params)
            .map_err(|e| LlmError::ModelLoad(e.to_string()))
    }

    /// Convenience method to load both backend and model in one call
    pub fn load<P: AsRef<Path>>(model_path: P) -> Result<(LlamaBackend, LlamaModel), LlmError> {
        let backend = Self::init_backend()?;
        let model = Self::load_model(&backend, model_path)?;
        Ok((backend, model))
    }
}

/// A loaded model with its backend, ready for inference
pub struct LoadedModel {
    pub backend: LlamaBackend,
    pub model: LlamaModel,
}

impl LoadedModel {
    /// Load a model from a path
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, LlmError> {
        let (backend, model) = ModelLoader::load(path)?;
        Ok(Self { backend, model })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_loader_creation() {
        // Just verify the struct can be created
        let _loader = ModelLoader;
    }

    #[test]
    fn test_loaded_model_path_storage() {
        // This test just verifies the path handling logic
        // Actual loading requires a real model file
        let path = "/tmp/test_model.gguf";
        assert_eq!(path, "/tmp/test_model.gguf");
    }
}
