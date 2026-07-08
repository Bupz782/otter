/// Port for a local signing wallet used to sign delegation messages and
/// blockchain transactions.
///
/// Implementations are expected to hold a private key securely. This MVP
/// version reads the key from an environment variable or CLI argument;
/// a production implementation should use a keystore or HSM.
pub trait WalletPort {
    /// Return the wallet's EVM address as a checksummed hex string.
    fn address(&self) -> Result<String, WalletError>;

    /// Sign an arbitrary 32-byte message hash and return the 64-byte compact
    /// secp256k1 signature (r || s).
    fn sign_hash(&self, hash: &[u8; 32]) -> Result<[u8; 64], WalletError>;

    /// Expose the secp256k1 public key coordinates (32 bytes each) so that
    /// the delegation message can include the signer pubkey.
    fn pubkey(&self) -> Result<([u8; 32], [u8; 32]), WalletError>;
}

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("signing failed: {0}")]
    SigningFailed(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
