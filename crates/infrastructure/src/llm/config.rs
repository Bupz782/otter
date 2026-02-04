/// Configuration for LLM inference parameters
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Context window size (number of tokens)
    pub n_ctx: u32,
    /// Batch size for token processing
    pub batch_size: usize,
    /// Temperature for sampling (0.0 = deterministic, higher = more random)
    pub temperature: f32,
    /// Top-p sampling (nucleus sampling)
    pub top_p: f32,
    /// Top-k sampling (limit to k most likely tokens)
    pub top_k: i32,
    /// Repetition penalty
    pub repeat_penalty: f32,
    /// Random seed (None for random)
    pub seed: Option<u32>,
    /// Number of threads to use (None = auto)
    pub threads: Option<usize>,
    /// Number of GPU layers to offload (0 = CPU only)
    pub gpu_layers: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            n_ctx: 2048,
            batch_size: 1024,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
            seed: None,
            threads: None,
            gpu_layers: 0,
        }
    }
}

impl LlmConfig {
    /// Create a new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set context window size
    pub fn with_n_ctx(mut self, n_ctx: u32) -> Self {
        self.n_ctx = n_ctx;
        self
    }

    /// Set batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 2.0);
        self
    }

    /// Set top-p sampling
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p.clamp(0.0, 1.0);
        self
    }

    /// Set top-k sampling
    pub fn with_top_k(mut self, top_k: i32) -> Self {
        self.top_k = top_k.max(1);
        self
    }

    /// Set repetition penalty
    pub fn with_repeat_penalty(mut self, repeat_penalty: f32) -> Self {
        self.repeat_penalty = repeat_penalty.max(1.0);
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set number of threads
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = Some(threads);
        self
    }

    /// Set number of GPU layers
    pub fn with_gpu_layers(mut self, gpu_layers: u32) -> Self {
        self.gpu_layers = gpu_layers;
        self
    }

    /// Create a deterministic configuration (for reproducible results)
    pub fn deterministic() -> Self {
        Self {
            temperature: 0.0,
            seed: Some(42),
            ..Default::default()
        }
    }

    /// Create a creative configuration (for varied outputs)
    pub fn creative() -> Self {
        Self {
            temperature: 0.9,
            top_p: 0.95,
            top_k: 100,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert_eq!(config.n_ctx, 2048);
        assert_eq!(config.batch_size, 1024);
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.top_p, 0.9);
        assert_eq!(config.top_k, 40);
        assert_eq!(config.repeat_penalty, 1.1);
        assert_eq!(config.seed, None);
        assert_eq!(config.threads, None);
        assert_eq!(config.gpu_layers, 0);
    }

    #[test]
    fn test_builder_pattern() {
        let config = LlmConfig::new()
            .with_n_ctx(4096)
            .with_temperature(0.5)
            .with_seed(123)
            .with_gpu_layers(35);

        assert_eq!(config.n_ctx, 4096);
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.seed, Some(123));
        assert_eq!(config.gpu_layers, 35);
    }

    #[test]
    fn test_temperature_clamping() {
        let config = LlmConfig::new().with_temperature(3.0);
        assert_eq!(config.temperature, 2.0);

        let config = LlmConfig::new().with_temperature(-0.5);
        assert_eq!(config.temperature, 0.0);
    }

    #[test]
    fn test_top_p_clamping() {
        let config = LlmConfig::new().with_top_p(1.5);
        assert_eq!(config.top_p, 1.0);

        let config = LlmConfig::new().with_top_p(-0.5);
        assert_eq!(config.top_p, 0.0);
    }

    #[test]
    fn test_deterministic_preset() {
        let config = LlmConfig::deterministic();
        assert_eq!(config.temperature, 0.0);
        assert_eq!(config.seed, Some(42));
    }

    #[test]
    fn test_creative_preset() {
        let config = LlmConfig::creative();
        assert_eq!(config.temperature, 0.9);
        assert_eq!(config.top_p, 0.95);
        assert_eq!(config.top_k, 100);
    }
}
