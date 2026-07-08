use serde::Deserialize;
use std::path::Path;

/// Application configuration loaded from TOML and overridable via
/// environment variables.
///
/// Configuration file path defaults to `config.toml` in the working directory.
/// Environment variables use the `OTTER_` prefix and double underscores for
/// nested fields, e.g. `OTTER_RPC__URL`.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Human-readable name for this agent instance.
    pub agent_name: String,

    /// RPC endpoint for the primary EVM chain.
    pub rpc_url: String,

    /// Chain ID for the primary EVM chain.
    pub chain_id: u64,

    /// Path to the local GGUF model used for intent parsing.
    pub model_path: String,

    /// Monitoring interval in seconds.
    #[serde(default = "default_monitoring_interval")]
    pub monitoring_interval_secs: u64,

    /// Logging level.
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Logging format: `text` or `json`.
    #[serde(default = "default_log_format")]
    pub log_format: String,

    /// Path or connection string for the SQLite database or PostgreSQL URL
    /// (e.g. `postgres://user:pass@host/db`).
    #[serde(default = "default_database_url")]
    pub database_url: String,

    /// Port on which the HTTP API server listens.
    #[serde(default = "default_api_port")]
    pub api_port: u16,

    /// Network used for on-chain oracles and protocol adapters.
    /// "sepolia" or "mainnet".
    #[serde(default)]
    pub network: Option<String>,

    /// DelegationVault contract address.
    #[serde(default)]
    pub vault_address: Option<String>,

    /// Hex private key used to sign delegations and transactions. For AWS KMS
    /// deployments set this to the base64-encoded KMS ciphertext blob so the
    /// configured provider can decrypt it.
    #[serde(default)]
    pub private_key: Option<String>,

    /// Path to the Noir circuit directory.
    #[serde(default)]
    pub circuit_dir: Option<String>,

    /// Path to the Barretenberg `bb` binary.
    #[serde(default)]
    pub bb_bin: Option<String>,

    /// Whether to register the delegation on-chain when an intent is created.
    #[serde(default = "default_delegate_on_create")]
    pub delegate_on_create: bool,

    /// Whether the agent should execute intents when conditions are met.
    #[serde(default = "default_execution_enabled")]
    pub execution_enabled: bool,

    /// Whether to expose Prometheus metrics on `/metrics`.
    #[serde(default = "default_metrics_enabled")]
    pub metrics_enabled: bool,

    /// Path to a file used to persist the agent's delegation nonce across
    /// restarts. Defaults to `otter-nonce.txt` in the working directory.
    #[serde(default = "default_nonce_store_path")]
    pub nonce_store_path: String,

    /// Path to a file containing the hex-encoded agent private key. Takes
    /// precedence over `private_key` when both are set.
    #[serde(default)]
    pub private_key_file: Option<String>,

    /// Path to an encrypted Ethereum keystore file (EIP-2335). Takes
    /// precedence over `private_key` and `private_key_file` when set.
    #[serde(default)]
    pub keystore_file: Option<String>,

    /// Password for the encrypted keystore file.
    #[serde(default)]
    pub keystore_password: Option<String>,

    /// Human-readable description of where the private key is sourced. Used in
    /// logs and health output to audit secret management.
    #[serde(default)]
    pub private_key_source: Option<String>,

    /// Whether JWT authentication is required on mutating API endpoints.
    #[serde(default = "default_auth_enabled")]
    pub auth_enabled: bool,

    /// Secret used to sign JWTs. If empty, a random secret is generated at startup
    /// (dev only; set explicitly in production).
    #[serde(default)]
    pub jwt_secret: String,

    /// JWT token lifetime in hours.
    #[serde(default = "default_jwt_ttl_hours")]
    pub jwt_ttl_hours: i64,

    /// CORS allowed origins. "*" means any origin. Comma-separated list otherwise.
    #[serde(default = "default_cors_allowed_origins")]
    pub cors_allowed_origins: String,

    /// Maximum number of requests per minute per IP. 0 disables rate limiting.
    #[serde(default = "default_rate_limit_per_minute")]
    pub rate_limit_per_minute: u32,
}

fn default_auth_enabled() -> bool {
    false
}

fn default_jwt_ttl_hours() -> i64 {
    24
}

fn default_cors_allowed_origins() -> String {
    "*".to_string()
}

fn default_rate_limit_per_minute() -> u32 {
    100
}

fn default_monitoring_interval() -> u64 {
    60
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "text".to_string()
}

fn default_database_url() -> String {
    "otter.db".to_string()
}

fn default_api_port() -> u16 {
    3001
}

fn default_delegate_on_create() -> bool {
    false
}

fn default_execution_enabled() -> bool {
    false
}

fn default_metrics_enabled() -> bool {
    false
}

fn default_nonce_store_path() -> String {
    "otter-nonce.txt".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            agent_name: "otter-agent".to_string(),
            rpc_url: "http://localhost:8545".to_string(),
            chain_id: 1,
            model_path: "models/Qwen3-8B-Q4_K_M.gguf".to_string(),
            monitoring_interval_secs: default_monitoring_interval(),
            log_level: default_log_level(),
            log_format: default_log_format(),
            database_url: default_database_url(),
            api_port: default_api_port(),
            network: None,
            vault_address: None,
            private_key: None,
            circuit_dir: None,
            bb_bin: None,
            delegate_on_create: default_delegate_on_create(),
            execution_enabled: default_execution_enabled(),
            metrics_enabled: default_metrics_enabled(),
            nonce_store_path: default_nonce_store_path(),
            private_key_file: None,
            keystore_file: None,
            keystore_password: None,
            private_key_source: None,
            auth_enabled: default_auth_enabled(),
            jwt_secret: String::new(),
            jwt_ttl_hours: default_jwt_ttl_hours(),
            cors_allowed_origins: default_cors_allowed_origins(),
            rate_limit_per_minute: default_rate_limit_per_minute(),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    ReadFailed(String),
    #[error("failed to parse config: {0}")]
    ParseFailed(String),
}

impl Config {
    /// Load configuration from a TOML file, then apply environment variable
    /// overrides.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents =
            std::fs::read_to_string(path).map_err(|e| ConfigError::ReadFailed(e.to_string()))?;

        let mut config: Config =
            toml::from_str(&contents).map_err(|e| ConfigError::ParseFailed(e.to_string()))?;

        config.apply_env_overrides();
        Ok(config)
    }

    /// Load configuration from environment variables only, using defaults for
    /// missing fields.
    pub fn from_env() -> Self {
        let mut config = Self::default();
        config.apply_env_overrides();
        config
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("OTTER_AGENT_NAME") {
            self.agent_name = val;
        }
        if let Ok(val) = std::env::var("OTTER_RPC_URL") {
            self.rpc_url = val;
        }
        if let Ok(val) = std::env::var("OTTER_CHAIN_ID")
            && let Ok(chain_id) = val.parse()
        {
            self.chain_id = chain_id;
        }
        if let Ok(val) = std::env::var("OTTER_MODEL_PATH") {
            self.model_path = val;
        }
        if let Ok(val) = std::env::var("OTTER_MONITORING_INTERVAL_SECS")
            && let Ok(interval) = val.parse()
        {
            self.monitoring_interval_secs = interval;
        }

        if let Ok(val) = std::env::var("OTTER_LOG_LEVEL") {
            self.log_level = val;
        }
        if let Ok(val) = std::env::var("RUST_LOG") {
            self.log_level = val;
        }
        if let Ok(val) = std::env::var("OTTER_LOG_FORMAT") {
            self.log_format = val;
        }
        if let Ok(val) = std::env::var("OTTER_DATABASE_URL") {
            self.database_url = val;
        }
        if let Ok(val) = std::env::var("OTTER_API_PORT")
            && let Ok(port) = val.parse()
        {
            self.api_port = port;
        }
        if let Ok(val) = std::env::var("OTTER_NETWORK") {
            self.network = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_VAULT_ADDRESS") {
            self.vault_address = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_PRIVATE_KEY") {
            self.private_key = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_CIRCUIT_DIR") {
            self.circuit_dir = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_BB_BIN") {
            self.bb_bin = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_DELEGATE_ON_CREATE")
            && let Ok(enabled) = val.parse()
        {
            self.delegate_on_create = enabled;
        }
        if let Ok(val) = std::env::var("OTTER_EXECUTION_ENABLED")
            && let Ok(enabled) = val.parse()
        {
            self.execution_enabled = enabled;
        }
        if let Ok(val) = std::env::var("OTTER_METRICS_ENABLED")
            && let Ok(enabled) = val.parse()
        {
            self.metrics_enabled = enabled;
        }
        if let Ok(val) = std::env::var("OTTER_NONCE_STORE_PATH") {
            self.nonce_store_path = val;
        }
        if let Ok(val) = std::env::var("OTTER_PRIVATE_KEY_FILE") {
            self.private_key_file = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_KEYSTORE_FILE") {
            self.keystore_file = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_KEYSTORE_PASSWORD") {
            self.keystore_password = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_PRIVATE_KEY_SOURCE") {
            self.private_key_source = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_AUTH_ENABLED")
            && let Ok(enabled) = val.parse()
        {
            self.auth_enabled = enabled;
        }
        if let Ok(val) = std::env::var("OTTER_JWT_SECRET") {
            self.jwt_secret = val;
        }
        if let Ok(val) = std::env::var("OTTER_JWT_TTL_HOURS")
            && let Ok(hours) = val.parse()
        {
            self.jwt_ttl_hours = hours;
        }
        if let Ok(val) = std::env::var("OTTER_CORS_ALLOWED_ORIGINS") {
            self.cors_allowed_origins = val;
        }
        if let Ok(val) = std::env::var("OTTER_RATE_LIMIT_PER_MINUTE")
            && let Ok(limit) = val.parse()
        {
            self.rate_limit_per_minute = limit;
        }
    }

    /// Validate configuration when on-chain execution is enabled.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.execution_enabled {
            if self.vault_address.is_none() {
                return Err(ConfigError::ParseFailed(
                    "execution_enabled requires vault_address".to_string(),
                ));
            }
            if self.private_key.is_none()
                && self.private_key_file.is_none()
                && self.keystore_file.is_none()
            {
                return Err(ConfigError::ParseFailed(
                    "execution_enabled requires private_key, private_key_file or keystore_file"
                        .to_string(),
                ));
            }
            if self.network.is_none() {
                return Err(ConfigError::ParseFailed(
                    "execution_enabled requires network".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = Config::default();
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.monitoring_interval_secs, 60);
    }

    #[test]
    fn config_from_toml_parses() {
        let toml = r#"
agent_name = "prod-agent"
rpc_url = "https://arb-sepolia.example.com"
chain_id = 421614
model_path = "models/model.gguf"
monitoring_interval_secs = 30
log_level = "debug"
"#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.agent_name, "prod-agent");
        assert_eq!(config.chain_id, 421614);
        assert_eq!(config.monitoring_interval_secs, 30);
    }
}
