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

    /// Load the Vault token from the `VAULT_TOKEN` environment variable.
    /// A missing or empty token is treated as an error and logged.
    #[cfg(feature = "vault")]
    fn load_token() -> Option<String> {
        match std::env::var("VAULT_TOKEN") {
            Ok(token) if !token.is_empty() => Some(token),
            Ok(_) => {
                tracing::error!(
                    "VAULT_TOKEN environment variable is set but empty; cannot authenticate to HashiCorp Vault"
                );
                None
            }
            Err(_) => {
                tracing::error!(
                    "VAULT_TOKEN environment variable is not set; cannot authenticate to HashiCorp Vault"
                );
                None
            }
        }
    }

    #[cfg(feature = "vault")]
    async fn do_get(addr: String, mount: String, path: String, name: String) -> Option<String> {
        use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
        use vaultrs::kv2;

        let token = Self::load_token()?;

        let settings = match VaultClientSettingsBuilder::default()
            .address(&addr)
            .token(&token)
            .build()
        {
            Ok(settings) => settings,
            Err(error) => {
                tracing::error!(%error, %addr, "failed to build HashiCorp Vault client settings");
                return None;
            }
        };

        let client = match VaultClient::new(settings) {
            Ok(client) => client,
            Err(error) => {
                tracing::error!(%error, "failed to create HashiCorp Vault client");
                return None;
            }
        };

        let secret: serde_json::Value = match kv2::read(&client, &mount, &path).await {
            Ok(secret) => secret,
            Err(error) => {
                tracing::error!(%error, %mount, %path, "failed to read secret from HashiCorp Vault");
                return None;
            }
        };

        match secret.get(&name).and_then(|v| v.as_str()) {
            Some(value) => Some(value.to_string()),
            None => {
                tracing::error!(field = %name, "requested field not found in HashiCorp Vault secret");
                None
            }
        }
    }
}

impl SecretProvider for HashiCorpVaultSecretProvider {
    #[cfg(feature = "vault")]
    fn get(&self, name: &str) -> Option<String> {
        block_on_secret(Self::do_get(
            self.addr.clone(),
            self.mount.clone(),
            self.path.clone(),
            name.to_string(),
        ))
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

/// Decrypts secrets using AWS KMS.
///
/// The `name` argument of [`SecretProvider::get`] is treated as a **base64-encoded
/// ciphertext blob** (the output of a previous AWS KMS `Encrypt` call against the
/// configured `key_id`). This differs from the usual secret-provider contract where
/// `name` is a human-readable identifier. Use [`Self::decrypt_base64_blob`] for an
/// explicitly-named helper.
///
/// # Key ID semantics
///
/// `key_id` is passed to every [`Decrypt`](https://docs.aws.amazon.com/kms/latest/APIReference/API_Decrypt.html)
/// request. For symmetric keys it is optional and must match the key that produced
/// the ciphertext if provided. For asymmetric keys it is required and must match
/// the key pair used for encryption. Passing it explicitly avoids ambiguity and
/// validates that ciphertexts were encrypted with the expected key.
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

    /// Decrypt a base64-encoded KMS ciphertext blob.
    ///
    /// This is a convenience alias for [`SecretProvider::get`] with a clearer name.
    pub fn decrypt_base64_blob(&self, ciphertext_blob_base64: &str) -> Option<String> {
        self.get(ciphertext_blob_base64)
    }

    async fn do_decrypt(key_id: String, region: String, ciphertext: Vec<u8>) -> Option<String> {
        use aws_config::BehaviorVersion;
        use aws_sdk_kms::primitives::Blob;
        use aws_sdk_kms::Client;

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(aws_sdk_kms::config::Region::new(region.clone()))
            .load()
            .await;
        let client = Client::new(&config);

        let response = match client
            .decrypt()
            .key_id(&key_id)
            .ciphertext_blob(Blob::new(ciphertext))
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                tracing::error!(%error, %key_id, %region, "failed to decrypt with AWS KMS");
                return None;
            }
        };

        match response.plaintext {
            Some(blob) => Some(hex::encode(blob.as_ref())),
            None => {
                tracing::error!("AWS KMS decrypt response did not contain plaintext");
                None
            }
        }
    }
}

#[cfg(feature = "aws-kms")]
impl SecretProvider for AwsKmsSecretProvider {
    fn get(&self, name: &str) -> Option<String> {
        use base64::Engine;

        let ciphertext = match base64::engine::general_purpose::STANDARD.decode(name) {
            Ok(ciphertext) => ciphertext,
            Err(error) => {
                tracing::error!(%error, "failed to decode AWS KMS ciphertext blob from base64; the 'name' argument must be a base64-encoded ciphertext blob");
                return None;
            }
        };

        block_on_secret(Self::do_decrypt(
            self.key_id.clone(),
            self.region.clone(),
            ciphertext,
        ))
    }
}

/// Run an async secret lookup from a synchronous context without panicking,
/// regardless of whether the current thread is inside a Tokio runtime.
///
/// On a multi-threaded Tokio runtime this uses `block_in_place` for efficiency.
/// On a single-threaded runtime or when no runtime is active it spawns a
/// dedicated thread and creates a fresh runtime there, which avoids both the
/// "cannot start a runtime from within a runtime" panic and the
/// "block_in_place on single-threaded runtime" panic.
#[cfg(any(feature = "aws-kms", feature = "vault"))]
fn block_on_secret<F, R>(fut: F) -> Option<R>
where
    F: std::future::Future<Output = Option<R>> + Send + 'static,
    R: Send + 'static,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| handle.block_on(fut))
        }
        _ => std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().ok()?;
            rt.block_on(fut)
        })
        .join()
        .ok()?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialize tests that mutate process-wide environment variables.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn env_secret_provider_reads_variable() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe { std::env::set_var("OTTER_TEST_SECRET", "hunter2") };
        let provider = EnvSecretProvider;
        assert_eq!(
            provider.get("OTTER_TEST_SECRET"),
            Some("hunter2".to_string())
        );
        unsafe { std::env::remove_var("OTTER_TEST_SECRET") };
    }

    #[test]
    fn file_secret_provider_reads_file() {
        let path = std::env::temp_dir().join(format!("otter-secret-{}", std::process::id()));
        std::fs::write(&path, "secret-value").unwrap();
        let provider = FileSecretProvider::new(path.to_string_lossy().as_ref());
        assert_eq!(provider.get("ignored"), Some("secret-value".to_string()));
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn vault_provider_can_be_constructed() {
        let provider =
            HashiCorpVaultSecretProvider::new("http://localhost:8200", "secret", "otter/agent");
        assert_eq!(provider.addr, "http://localhost:8200");
        assert_eq!(provider.mount, "secret");
        assert_eq!(provider.path, "otter/agent");
    }

    #[cfg(feature = "vault")]
    #[test]
    fn vault_load_token_rejects_missing_and_empty() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe { std::env::remove_var("VAULT_TOKEN") };
        assert!(HashiCorpVaultSecretProvider::load_token().is_none());

        unsafe { std::env::set_var("VAULT_TOKEN", "") };
        assert!(HashiCorpVaultSecretProvider::load_token().is_none());

        unsafe { std::env::set_var("VAULT_TOKEN", "valid-token") };
        assert_eq!(
            HashiCorpVaultSecretProvider::load_token(),
            Some("valid-token".to_string())
        );

        unsafe { std::env::remove_var("VAULT_TOKEN") };
    }

    #[cfg(feature = "vault")]
    #[tokio::test(flavor = "multi_thread")]
    async fn vault_provider_returns_none_when_token_missing_inside_runtime() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe { std::env::remove_var("VAULT_TOKEN") };
        let provider =
            HashiCorpVaultSecretProvider::new("http://localhost:8200", "secret", "otter/agent");
        // Should not panic when called from inside a multi-threaded Tokio runtime.
        assert!(provider.get("private_key").is_none());
    }

    #[cfg(feature = "vault")]
    #[tokio::test]
    async fn vault_provider_returns_none_when_token_missing_inside_single_threaded_runtime() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe { std::env::remove_var("VAULT_TOKEN") };
        let provider =
            HashiCorpVaultSecretProvider::new("http://localhost:8200", "secret", "otter/agent");
        // Should not panic on a single-threaded runtime either.
        assert!(provider.get("private_key").is_none());
    }

    #[cfg(feature = "vault")]
    #[test]
    fn vault_provider_returns_none_when_token_missing_outside_runtime() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe { std::env::remove_var("VAULT_TOKEN") };
        let provider =
            HashiCorpVaultSecretProvider::new("http://localhost:8200", "secret", "otter/agent");
        assert!(provider.get("private_key").is_none());
    }

    #[cfg(feature = "aws-kms")]
    #[test]
    fn aws_kms_provider_can_be_constructed() {
        let provider = AwsKmsSecretProvider::new("alias/otter-agent", "us-east-1");
        assert_eq!(provider.key_id, "alias/otter-agent");
        assert_eq!(provider.region, "us-east-1");
    }

    #[cfg(feature = "aws-kms")]
    #[tokio::test(flavor = "multi_thread")]
    async fn aws_kms_provider_returns_none_for_invalid_base64_inside_runtime() {
        let provider = AwsKmsSecretProvider::new("alias/otter-agent", "us-east-1");
        // Should not panic when called from inside a multi-threaded Tokio runtime.
        assert!(provider.get("not-valid-base64!!!").is_none());
    }

    #[cfg(feature = "aws-kms")]
    #[tokio::test]
    async fn aws_kms_provider_returns_none_for_invalid_base64_inside_single_threaded_runtime() {
        let provider = AwsKmsSecretProvider::new("alias/otter-agent", "us-east-1");
        // Should not panic on a single-threaded runtime either.
        assert!(provider.get("not-valid-base64!!!").is_none());
    }

    #[cfg(feature = "aws-kms")]
    #[test]
    fn aws_kms_provider_returns_none_for_invalid_base64_outside_runtime() {
        let provider = AwsKmsSecretProvider::new("alias/otter-agent", "us-east-1");
        assert!(provider.get("not-valid-base64!!!").is_none());
    }

    #[cfg(feature = "aws-kms")]
    #[test]
    fn aws_kms_decrypt_base64_blob_helper_works() {
        let provider = AwsKmsSecretProvider::new("alias/otter-agent", "us-east-1");
        assert!(provider.decrypt_base64_blob("not-valid-base64!!!").is_none());
    }
}
