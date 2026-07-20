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
    /// Recognized values: `environment-variable`, `file`, `keystore`, `aws-kms`, `vault`.
    #[serde(default)]
    pub private_key_source: Option<String>,

    /// AWS KMS key ID (ARN or alias) used when `private_key_source` is `aws-kms`.
    #[serde(default)]
    pub aws_kms_key_id: Option<String>,

    /// AWS region for the KMS key.
    #[serde(default)]
    pub aws_kms_region: Option<String>,

    /// HashiCorp Vault address (e.g. `https://vault.example.com:8200`).
    #[serde(default)]
    pub vault_addr: Option<String>,

    /// HashiCorp Vault KV mount (e.g. `secret`).
    #[serde(default)]
    pub vault_mount: Option<String>,

    /// HashiCorp Vault secret path (e.g. `otter/agent`).
    #[serde(default)]
    pub vault_path: Option<String>,

    /// HashiCorp Vault secret key containing the private key.
    #[serde(default)]
    pub vault_key: Option<String>,

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
            aws_kms_key_id: None,
            aws_kms_region: None,
            vault_addr: None,
            vault_mount: None,
            vault_path: None,
            vault_key: None,
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
        if let Ok(val) = std::env::var("OTTER_AWS_KMS_KEY_ID") {
            self.aws_kms_key_id = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_AWS_KMS_REGION") {
            self.aws_kms_region = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_VAULT_ADDR") {
            self.vault_addr = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_VAULT_MOUNT") {
            self.vault_mount = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_VAULT_PATH") {
            self.vault_path = Some(val);
        }
        if let Ok(val) = std::env::var("OTTER_VAULT_KEY") {
            self.vault_key = Some(val);
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
    use serial_test::serial;

    fn temp_config_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("otter-config-test-{}.toml", rand::random::<u64>()))
    }

    fn write_temp_config(contents: &str) -> std::path::PathBuf {
        let path = temp_config_path();
        std::fs::write(&path, contents).unwrap();
        path
    }

    /// Remove every `OTTER_`/`RUST_LOG` variable that could leak from the
    /// developer shell or another test into `Config::from_env`.
    fn clear_otter_env() {
        let keys: Vec<String> = std::env::vars()
            .map(|(key, _)| key)
            .filter(|key| key.starts_with("OTTER_") || key == "RUST_LOG")
            .collect();
        for key in keys {
            unsafe { std::env::remove_var(&key) };
        }
    }

    #[test]
    #[serial]
    fn default_config_is_valid() {
        clear_otter_env();
        let config = Config::default();
        assert_eq!(config.agent_name, "otter-agent");
        assert_eq!(config.rpc_url, "http://localhost:8545");
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.model_path, "models/Qwen3-8B-Q4_K_M.gguf");
        assert_eq!(config.monitoring_interval_secs, 60);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.log_format, "text");
        assert_eq!(config.database_url, "otter.db");
        assert_eq!(config.api_port, 3001);
        assert_eq!(config.nonce_store_path, "otter-nonce.txt");
        assert_eq!(config.jwt_ttl_hours, 24);
        assert_eq!(config.cors_allowed_origins, "*");
        assert_eq!(config.rate_limit_per_minute, 100);
        assert!(!config.delegate_on_create);
        assert!(!config.execution_enabled);
        assert!(!config.metrics_enabled);
        assert!(!config.auth_enabled);
        assert!(config.network.is_none());
        assert!(config.vault_address.is_none());
        assert!(config.private_key.is_none());
        assert!(config.jwt_secret.is_empty());
        // The default config must pass validation (execution disabled).
        config.validate().unwrap();
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

    #[test]
    #[serial]
    fn from_file_applies_serde_defaults_for_missing_fields() {
        clear_otter_env();
        let path = write_temp_config(
            r#"
agent_name = "minimal"
rpc_url = "http://localhost:8545"
chain_id = 11155111
model_path = "model.gguf"
"#,
        );

        let config = Config::from_file(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(config.agent_name, "minimal");
        assert_eq!(config.chain_id, 11155111);
        // Fields absent from the file fall back to the serde defaults.
        assert_eq!(config.monitoring_interval_secs, 60);
        assert_eq!(config.api_port, 3001);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.database_url, "otter.db");
        assert_eq!(config.nonce_store_path, "otter-nonce.txt");
        assert!(!config.execution_enabled);
        assert!(config.private_key_file.is_none());
        assert!(config.keystore_file.is_none());
        assert!(config.aws_kms_key_id.is_none());
        assert!(config.vault_addr.is_none());
    }

    #[test]
    #[serial]
    fn from_file_parses_optional_sections() {
        clear_otter_env();
        let path = write_temp_config(
            r#"
agent_name = "full"
rpc_url = "http://localhost:8545"
chain_id = 1
model_path = "model.gguf"
network = "sepolia"
vault_address = "0xVault"
private_key = "0xdeadbeef"
circuit_dir = "delegation_circuit"
bb_bin = "/usr/local/bin/bb"
delegate_on_create = true
execution_enabled = true
metrics_enabled = true
auth_enabled = true
jwt_secret = "s3cret"
jwt_ttl_hours = 12
cors_allowed_origins = "https://app.example.com"
rate_limit_per_minute = 42
private_key_source = "vault"
aws_kms_key_id = "alias/otter"
aws_kms_region = "eu-west-1"
vault_addr = "https://vault.example.com:8200"
vault_mount = "secret"
vault_path = "otter/agent"
vault_key = "private_key"
"#,
        );

        let config = Config::from_file(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(config.network.as_deref(), Some("sepolia"));
        assert_eq!(config.vault_address.as_deref(), Some("0xVault"));
        assert_eq!(config.private_key.as_deref(), Some("0xdeadbeef"));
        assert_eq!(config.circuit_dir.as_deref(), Some("delegation_circuit"));
        assert_eq!(config.bb_bin.as_deref(), Some("/usr/local/bin/bb"));
        assert!(config.delegate_on_create);
        assert!(config.execution_enabled);
        assert!(config.metrics_enabled);
        assert!(config.auth_enabled);
        assert_eq!(config.jwt_secret, "s3cret");
        assert_eq!(config.jwt_ttl_hours, 12);
        assert_eq!(config.cors_allowed_origins, "https://app.example.com");
        assert_eq!(config.rate_limit_per_minute, 42);
        assert_eq!(config.private_key_source.as_deref(), Some("vault"));
        assert_eq!(config.aws_kms_key_id.as_deref(), Some("alias/otter"));
        assert_eq!(config.aws_kms_region.as_deref(), Some("eu-west-1"));
        assert_eq!(
            config.vault_addr.as_deref(),
            Some("https://vault.example.com:8200")
        );
        assert_eq!(config.vault_mount.as_deref(), Some("secret"));
        assert_eq!(config.vault_path.as_deref(), Some("otter/agent"));
        assert_eq!(config.vault_key.as_deref(), Some("private_key"));
    }

    #[test]
    fn from_file_fails_when_file_is_missing() {
        let err = Config::from_file("/nonexistent/otter-config-zzz.toml").unwrap_err();
        assert!(matches!(err, ConfigError::ReadFailed(_)));
    }

    #[test]
    fn from_file_fails_on_invalid_toml() {
        let path = write_temp_config("this is = = not valid toml [[[");
        let err = Config::from_file(&path).unwrap_err();
        std::fs::remove_file(&path).ok();
        assert!(matches!(err, ConfigError::ParseFailed(_)));
    }

    #[test]
    fn from_file_fails_on_wrong_field_type() {
        let path = write_temp_config(
            r#"
agent_name = "x"
rpc_url = "http://localhost:8545"
chain_id = "not-a-number"
model_path = "model.gguf"
"#,
        );
        let err = Config::from_file(&path).unwrap_err();
        std::fs::remove_file(&path).ok();
        assert!(matches!(err, ConfigError::ParseFailed(_)));
    }

    #[test]
    fn from_file_fails_when_required_field_is_missing() {
        // `agent_name` has no serde default; omitting it must fail.
        let path = write_temp_config(
            r#"
rpc_url = "http://localhost:8545"
chain_id = 1
model_path = "model.gguf"
"#,
        );
        let err = Config::from_file(&path).unwrap_err();
        std::fs::remove_file(&path).ok();
        assert!(matches!(err, ConfigError::ParseFailed(_)));
    }

    #[test]
    #[serial]
    fn env_overrides_scalars_and_strings() {
        clear_otter_env();
        unsafe {
            std::env::set_var("OTTER_AGENT_NAME", "env-agent");
            std::env::set_var("OTTER_RPC_URL", "http://env:8545");
            std::env::set_var("OTTER_CHAIN_ID", "8453");
            std::env::set_var("OTTER_MODEL_PATH", "env-model.gguf");
            std::env::set_var("OTTER_MONITORING_INTERVAL_SECS", "15");
            std::env::set_var("OTTER_LOG_FORMAT", "json");
            std::env::set_var("OTTER_DATABASE_URL", "postgres://localhost/otter");
            std::env::set_var("OTTER_API_PORT", "8080");
            std::env::set_var("OTTER_NETWORK", "mainnet");
            std::env::set_var("OTTER_VAULT_ADDRESS", "0xEnvVault");
            std::env::set_var("OTTER_PRIVATE_KEY", "0xenvkey");
            std::env::set_var("OTTER_NONCE_STORE_PATH", "/tmp/nonce.txt");
            std::env::set_var("OTTER_PRIVATE_KEY_FILE", "/tmp/key.hex");
            std::env::set_var("OTTER_KEYSTORE_FILE", "/tmp/keystore.json");
            std::env::set_var("OTTER_KEYSTORE_PASSWORD", "pw");
            std::env::set_var("OTTER_PRIVATE_KEY_SOURCE", "keystore");
            std::env::set_var("OTTER_AWS_KMS_KEY_ID", "alias/env");
            std::env::set_var("OTTER_AWS_KMS_REGION", "us-east-1");
            std::env::set_var("OTTER_VAULT_ADDR", "https://vault:8200");
            std::env::set_var("OTTER_VAULT_MOUNT", "kv");
            std::env::set_var("OTTER_VAULT_PATH", "otter/env");
            std::env::set_var("OTTER_VAULT_KEY", "pk");
            std::env::set_var("OTTER_JWT_SECRET", "env-secret");
            std::env::set_var("OTTER_JWT_TTL_HOURS", "1");
            std::env::set_var("OTTER_CORS_ALLOWED_ORIGINS", "https://a.com,https://b.com");
            std::env::set_var("OTTER_RATE_LIMIT_PER_MINUTE", "7");
            std::env::set_var("OTTER_CIRCUIT_DIR", "/tmp/circuits");
            std::env::set_var("OTTER_BB_BIN", "/opt/bb");
        }

        let config = Config::from_env();
        clear_otter_env();

        assert_eq!(config.agent_name, "env-agent");
        assert_eq!(config.rpc_url, "http://env:8545");
        assert_eq!(config.chain_id, 8453);
        assert_eq!(config.model_path, "env-model.gguf");
        assert_eq!(config.monitoring_interval_secs, 15);
        assert_eq!(config.log_format, "json");
        assert_eq!(config.database_url, "postgres://localhost/otter");
        assert_eq!(config.api_port, 8080);
        assert_eq!(config.network.as_deref(), Some("mainnet"));
        assert_eq!(config.vault_address.as_deref(), Some("0xEnvVault"));
        assert_eq!(config.private_key.as_deref(), Some("0xenvkey"));
        assert_eq!(config.nonce_store_path, "/tmp/nonce.txt");
        assert_eq!(config.private_key_file.as_deref(), Some("/tmp/key.hex"));
        assert_eq!(config.keystore_file.as_deref(), Some("/tmp/keystore.json"));
        assert_eq!(config.keystore_password.as_deref(), Some("pw"));
        assert_eq!(config.private_key_source.as_deref(), Some("keystore"));
        assert_eq!(config.aws_kms_key_id.as_deref(), Some("alias/env"));
        assert_eq!(config.aws_kms_region.as_deref(), Some("us-east-1"));
        assert_eq!(config.vault_addr.as_deref(), Some("https://vault:8200"));
        assert_eq!(config.vault_mount.as_deref(), Some("kv"));
        assert_eq!(config.vault_path.as_deref(), Some("otter/env"));
        assert_eq!(config.vault_key.as_deref(), Some("pk"));
        assert_eq!(config.jwt_secret, "env-secret");
        assert_eq!(config.jwt_ttl_hours, 1);
        assert_eq!(config.cors_allowed_origins, "https://a.com,https://b.com");
        assert_eq!(config.rate_limit_per_minute, 7);
        assert_eq!(config.circuit_dir.as_deref(), Some("/tmp/circuits"));
        assert_eq!(config.bb_bin.as_deref(), Some("/opt/bb"));
    }

    #[test]
    #[serial]
    fn env_overrides_booleans() {
        clear_otter_env();
        unsafe {
            std::env::set_var("OTTER_DELEGATE_ON_CREATE", "true");
            std::env::set_var("OTTER_EXECUTION_ENABLED", "true");
            std::env::set_var("OTTER_METRICS_ENABLED", "true");
            std::env::set_var("OTTER_AUTH_ENABLED", "true");
        }

        let config = Config::from_env();
        clear_otter_env();

        assert!(config.delegate_on_create);
        assert!(config.execution_enabled);
        assert!(config.metrics_enabled);
        assert!(config.auth_enabled);
    }

    #[test]
    #[serial]
    fn invalid_numeric_env_values_are_ignored() {
        clear_otter_env();
        unsafe {
            std::env::set_var("OTTER_CHAIN_ID", "not-a-number");
            std::env::set_var("OTTER_API_PORT", "99999999");
            std::env::set_var("OTTER_MONITORING_INTERVAL_SECS", "-5");
            std::env::set_var("OTTER_JWT_TTL_HOURS", "abc");
            std::env::set_var("OTTER_RATE_LIMIT_PER_MINUTE", "fast");
            std::env::set_var("OTTER_AUTH_ENABLED", "maybe");
        }

        let config = Config::from_env();
        clear_otter_env();

        // Unparsable values leave the defaults untouched.
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.api_port, 3001);
        assert_eq!(config.monitoring_interval_secs, 60);
        assert_eq!(config.jwt_ttl_hours, 24);
        assert_eq!(config.rate_limit_per_minute, 100);
        assert!(!config.auth_enabled);
    }

    #[test]
    #[serial]
    fn rust_log_takes_precedence_over_otter_log_level() {
        clear_otter_env();
        unsafe {
            std::env::set_var("OTTER_LOG_LEVEL", "debug");
        }
        let config = Config::from_env();
        assert_eq!(config.log_level, "debug");

        unsafe {
            std::env::set_var("RUST_LOG", "trace");
        }
        let config = Config::from_env();
        clear_otter_env();
        // RUST_LOG is applied after OTTER_LOG_LEVEL and wins.
        assert_eq!(config.log_level, "trace");
    }

    #[test]
    fn validate_rejects_execution_without_vault_address() {
        let config = Config {
            execution_enabled: true,
            private_key: Some("0xkey".to_string()),
            network: Some("sepolia".to_string()),
            ..Config::default()
        };

        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("vault_address"));
    }

    #[test]
    fn validate_rejects_execution_without_any_key_source() {
        let config = Config {
            execution_enabled: true,
            vault_address: Some("0xVault".to_string()),
            network: Some("sepolia".to_string()),
            ..Config::default()
        };

        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("private_key"));
    }

    #[test]
    fn validate_rejects_execution_without_network() {
        let config = Config {
            execution_enabled: true,
            vault_address: Some("0xVault".to_string()),
            private_key: Some("0xkey".to_string()),
            ..Config::default()
        };

        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("network"));
    }

    #[test]
    fn validate_accepts_execution_with_keystore_instead_of_raw_key() {
        let config = Config {
            execution_enabled: true,
            vault_address: Some("0xVault".to_string()),
            keystore_file: Some("/tmp/keystore.json".to_string()),
            network: Some("sepolia".to_string()),
            ..Config::default()
        };

        config.validate().unwrap();
    }

    #[test]
    fn validate_accepts_execution_with_key_file() {
        let config = Config {
            execution_enabled: true,
            vault_address: Some("0xVault".to_string()),
            private_key_file: Some("/tmp/key.hex".to_string()),
            network: Some("mainnet".to_string()),
            ..Config::default()
        };

        config.validate().unwrap();
    }

    #[test]
    fn validate_passes_when_execution_disabled_without_onchain_fields() {
        let config = Config::default();
        config.validate().unwrap();
    }
}
