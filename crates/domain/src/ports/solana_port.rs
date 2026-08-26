use async_trait::async_trait;

/// Errors returned by a Solana attestation adapter.
#[derive(Debug, thiserror::Error)]
pub enum SolanaError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("submission failed: {0}")]
    SubmissionFailed(String),
    #[error("attestation not found")]
    NotFound,
}

/// A single on-chain attestation record.
#[derive(Debug, Clone)]
pub struct AttestationRecord {
    pub authority: String,
    pub payload_hash: [u8; 32],
    pub timestamp: i64,
}

/// Minimal Solana attestation port. V1 is intentionally small: it stores a
/// 32-byte payload hash under an authority and allows anyone to verify it.
#[async_trait]
pub trait SolanaPort: Send + Sync {
    /// Store or overwrite an attestation for the configured authority.
    async fn attest(&self, payload_hash: [u8; 32]) -> Result<String, SolanaError>;

    /// Read the current attestation for `authority`.
    async fn get_attestation(&self, authority: &str) -> Result<AttestationRecord, SolanaError>;

    /// Return true if `authority` has attested to `payload_hash`.
    async fn verify(&self, authority: &str, payload_hash: [u8; 32]) -> Result<bool, SolanaError>;
}
