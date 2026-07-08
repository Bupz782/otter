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

/// Reads secrets from HashiCorp Vault using the KV v2 secrets engine. When the
/// `vault` feature is enabled the provider makes an authenticated HTTP call to
/// Vault and returns the requested field. Without the feature it logs a warning
/// and returns `None` so the crate still compiles.
pub struct HashiCorpVaultSecretProvider {
    addr: String,
    mount: String,
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
    #[cfg(feature = "vault")]
    fn get(&self, name: &str) -> Option<String> {
        use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
        use vaultrs::kv2;

        let token = std::env::var("VAULT_TOKEN").ok().unwrap_or_default();
        let settings = VaultClientSettingsBuilder::default()
            .address(&self.addr)
            .token(&token)
            .build()
            .ok()?;
        let client = VaultClient::new(settings).ok()?;

        let runtime = tokio::runtime::Runtime::new().ok()?;
        runtime.block_on(async {
            let secret: serde_json::Value =
                kv2::read(&client, &self.mount, &self.path).await.ok()?;
            secret
                .get(name)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
    }

    #[cfg(not(feature = "vault"))]
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

/// Decrypts secrets using AWS KMS. The `name` passed to [`SecretProvider::get`]
/// is interpreted as a base64-encoded ciphertext blob that was produced by a
/// previous `Encrypt` call against the configured KMS key.
#[cfg(feature = "aws-kms")]
pub struct AwsKmsSecretProvider {
    key_id: String,
    region: String,
}

#[cfg(feature = "aws-kms")]
impl AwsKmsSecretProvider {
    pub fn new(key_id: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            key_id: key_id.into(),
            region: region.into(),
        }
    }
}

#[cfg(feature = "aws-kms")]
impl SecretProvider for AwsKmsSecretProvider {
    fn get(&self, name: &str) -> Option<String> {
        use aws_config::BehaviorVersion;
        use aws_sdk_kms::primitives::Blob;
        use aws_sdk_kms::Client;
        use base64::Engine;

        let ciphertext = base64::engine::general_purpose::STANDARD
            .decode(name)
            .ok()?;

        let runtime = tokio::runtime::Runtime::new().ok()?;
        runtime.block_on(async {
            let config = aws_config::defaults(BehaviorVersion::latest())
                .region(aws_sdk_kms::config::Region::new(self.region.clone()))
                .load()
                .await;
            let client = Client::new(&config);
            let response = client
                .decrypt()
                .key_id(&self.key_id)
                .ciphertext_blob(Blob::new(ciphertext))
                .send()
                .await
                .ok()?;
            let plaintext = response.plaintext?;
            Some(hex::encode(plaintext.as_ref()))
        })
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
