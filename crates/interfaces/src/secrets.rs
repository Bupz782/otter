use infrastructure::config::Config;
use std::fs;

/// Strategy used to retrieve a sensitive configuration value such as the agent
/// private key. The trait is intentionally small so that production deployments
/// can swap the plain-env/file implementations for a KMS integration without
/// changing the rest of the codebase.
pub trait SecretProvider: Send + Sync {
    fn get(&self, name: &str) -> Option<String>;
}

/// Reads secrets from environment variables. Convenient for local development
/// and testnet, but leaves the value in the process environment.
pub struct EnvSecretProvider;

impl SecretProvider for EnvSecretProvider {
    fn get(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}

/// Reads secrets from a file on disk. The file should be readable only by the
/// service user (mode `0600`) and must never be checked into version control.
pub struct FileSecretProvider {
    path: String,
}

impl FileSecretProvider {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl SecretProvider for FileSecretProvider {
    fn get(&self, _name: &str) -> Option<String> {
        fs::read_to_string(&self.path).ok()
    }
}

/// Placeholder for HashiCorp Vault integration. In production this would use
/// the Vault API (e.g. `v1/secret/data/{path}`) with AppRole / Kubernetes auth
/// and return the value stored at the configured key.
pub struct HashiCorpVaultSecretProvider {
    #[allow(dead_code)]
    addr: String,
    #[allow(dead_code)]
    mount: String,
    #[allow(dead_code)]
    path: String,
}

impl HashiCorpVaultSecretProvider {
    pub fn new(addr: impl Into<String>, mount: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            mount: mount.into(),
            path: path.into(),
        }
    }
}

impl SecretProvider for HashiCorpVaultSecretProvider {
    fn get(&self, _name: &str) -> Option<String> {
        tracing::warn!(
            addr = %self.addr,
            mount = %self.mount,
            path = %self.path,
            "HashiCorp Vault secret provider is not implemented in V1; returning None"
        );
        None
    }
}

/// Load the agent private key according to the configured secret strategy.
///
/// Returns the decoded 32-byte key together with a human-readable source label
/// used for audit logging. The precedence is:
/// 1. `keystore_file` + `keystore_password` (encrypted keystore)
/// 2. `private_key_file` (read the file contents)
/// 3. `private_key` (environment variable / config value)
///
/// A clear warning is emitted when the key is read from the environment so
/// operators know to rotate to a keystore- or KMS-based approach for production.
pub fn load_private_key(config: &Config) -> Result<([u8; 32], String), String> {
    if let Some(path) = &config.keystore_file {
        let password = config.keystore_password.as_deref().unwrap_or("");
        let key_bytes = eth_keystore::decrypt_key(path, password)
            .map_err(|e| format!("failed to decrypt keystore {path}: {e}"))?;
        if key_bytes.len() != 32 {
            return Err(format!(
                "keystore private key must be 32 bytes, got {}",
                key_bytes.len()
            ));
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&key_bytes);
        let source = config
            .private_key_source
            .clone()
            .unwrap_or_else(|| format!("keystore:{path}"));
        return Ok((bytes, source));
    }

    let (raw, source) = if let Some(path) = &config.private_key_file {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("failed to read private key file {path}: {e}"))?;
        (
            content.trim().to_string(),
            config
                .private_key_source
                .clone()
                .unwrap_or_else(|| format!("file:{path}")),
        )
    } else if let Some(key) = &config.private_key {
        tracing::warn!(
            "Private key loaded from environment/config. This is acceptable for testnet but NOT recommended for production. Use OTTER_PRIVATE_KEY_FILE, OTTER_KEYSTORE_FILE or integrate a KMS/Vault provider."
        );
        (
            key.clone(),
            config
                .private_key_source
                .clone()
                .unwrap_or_else(|| "environment-variable".to_string()),
        )
    } else {
        return Err(
            "execution is enabled but no private key was provided (set OTTER_PRIVATE_KEY, OTTER_PRIVATE_KEY_FILE or OTTER_KEYSTORE_FILE)"
                .to_string(),
        );
    };

    decode_private_key(&raw).map(|key| (key, source))
}

fn decode_private_key(hex_str: &str) -> Result<[u8; 32], String> {
    let cleaned = hex_str.trim().strip_prefix("0x").unwrap_or(hex_str);
    let decoded = hex::decode(cleaned).map_err(|e| format!("invalid private key hex: {e}"))?;
    if decoded.len() != 32 {
        return Err(format!(
            "private key must be 32 bytes, got {}",
            decoded.len()
        ));
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&decoded);
    Ok(bytes)
}
