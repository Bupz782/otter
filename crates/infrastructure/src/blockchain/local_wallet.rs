use domain::ports::wallet_port::{WalletError, WalletPort};
use k256::ecdsa::signature::hazmat::PrehashSigner;
use k256::ecdsa::{Signature, SigningKey};

/// Local secp256k1 wallet loaded from a raw 32-byte private key.
///
/// # Security warning
/// This adapter stores the private key in memory. Do not use it in production
/// with high-value keys; prefer a keystore or HSM adapter instead.
#[derive(Clone)]
pub struct LocalWalletAdapter {
    signing_key: SigningKey,
}

impl LocalWalletAdapter {
    /// Load a wallet from a 32-byte private key.
    pub fn from_bytes(private_key: &[u8; 32]) -> Result<Self, WalletError> {
        let signing_key = SigningKey::from_slice(private_key.as_slice())
            .map_err(|e| WalletError::InvalidPrivateKey(e.to_string()))?;
        Ok(Self { signing_key })
    }

    /// Load a wallet from a hex-encoded private key (with or without 0x prefix).
    pub fn from_hex(hex_str: &str) -> Result<Self, WalletError> {
        let cleaned = hex_str.trim().strip_prefix("0x").unwrap_or(hex_str);
        let mut bytes = [0u8; 32];
        let decoded = hex::decode(cleaned)
            .map_err(|e| WalletError::InvalidPrivateKey(format!("invalid hex: {}", e)))?;
        if decoded.len() != 32 {
            return Err(WalletError::InvalidPrivateKey(format!(
                "private key must be 32 bytes, got {}",
                decoded.len()
            )));
        }
        bytes.copy_from_slice(&decoded);
        Self::from_bytes(&bytes)
    }

    /// Load a wallet from an encrypted Ethereum keystore file (EIP-2335 / Web3 secret storage).
    pub fn from_keystore(path: &str, password: &str) -> Result<Self, WalletError> {
        let key_bytes = eth_keystore::decrypt_key(path, password).map_err(|e| {
            WalletError::InvalidPrivateKey(format!("keystore decrypt failed: {}", e))
        })?;
        if key_bytes.len() != 32 {
            return Err(WalletError::InvalidPrivateKey(format!(
                "keystore private key must be 32 bytes, got {}",
                key_bytes.len()
            )));
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&key_bytes);
        Self::from_bytes(&bytes)
    }
}

impl WalletPort for LocalWalletAdapter {
    fn address(&self) -> Result<String, WalletError> {
        let verifying_key = self.signing_key.verifying_key();
        let encoded = verifying_key.to_encoded_point(false);
        let pubkey_bytes = encoded.as_bytes();
        // ethereum address = keccak256(pubkey_without_prefix)[12..]
        let hash = keccak256(&pubkey_bytes[1..]);
        Ok(format!("0x{}", hex::encode(&hash[12..])))
    }

    fn sign_hash(&self, hash: &[u8; 32]) -> Result<[u8; 64], WalletError> {
        let signature: Signature = self
            .signing_key
            .sign_prehash(hash)
            .map_err(|e| WalletError::SigningFailed(e.to_string()))?;
        let bytes: [u8; 64] = signature.to_bytes().into();
        Ok(bytes)
    }

    fn pubkey(&self) -> Result<([u8; 32], [u8; 32]), WalletError> {
        let verifying_key = self.signing_key.verifying_key();
        let encoded = verifying_key.to_encoded_point(false);
        let pubkey_bytes = encoded.as_bytes();
        if pubkey_bytes.len() != 65 || pubkey_bytes[0] != 0x04 {
            return Err(WalletError::SigningFailed(
                "unexpected encoded public key format".to_string(),
            ));
        }
        let mut x = [0u8; 32];
        let mut y = [0u8; 32];
        x.copy_from_slice(&pubkey_bytes[1..33]);
        y.copy_from_slice(&pubkey_bytes[33..65]);
        Ok((x, y))
    }
}

fn keccak256(input: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut hasher = Keccak::v256();
    hasher.update(input);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_key_derivation() {
        let mut key = [0u8; 32];
        key[31] = 0x01;
        let wallet = LocalWalletAdapter::from_bytes(&key).unwrap();
        let (x, y) = wallet.pubkey().unwrap();
        assert_ne!(x, [0u8; 32]);
        assert_ne!(y, [0u8; 32]);
    }

    #[test]
    fn signs_and_verifies() {
        let mut key = [0u8; 32];
        key[31] = 0x42;
        let wallet = LocalWalletAdapter::from_bytes(&key).unwrap();
        let hash = [0xabu8; 32];
        let signature = wallet.sign_hash(&hash).unwrap();
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn loads_from_encrypted_keystore() {
        let mut key = [0u8; 32];
        key[31] = 0x42;

        let dir = std::env::temp_dir();
        let path =
            eth_keystore::encrypt_key(&dir, &mut rand::rngs::OsRng, key, "test-password", None)
                .expect("encrypt_key should succeed");

        let full_path = dir.join(&path);
        let wallet =
            LocalWalletAdapter::from_keystore(full_path.to_str().unwrap(), "test-password")
                .expect("from_keystore should succeed");
        let hash = [0xabu8; 32];
        let signature = wallet.sign_hash(&hash).unwrap();
        assert_eq!(signature.len(), 64);

        let _ = std::fs::remove_file(&full_path);
    }
}
