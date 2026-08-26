use async_trait::async_trait;
use domain::ports::solana_port::{AttestationRecord, SolanaError, SolanaPort};

/// Stub adapter used when the `solana` feature is disabled.
pub struct SolanaAttestationAdapter;

impl SolanaAttestationAdapter {
    pub fn new(
        _rpc_url: &str,
        _program_id: &str,
        _authority_keypair: &str,
    ) -> Result<Self, SolanaError> {
        Ok(Self)
    }
}

#[async_trait]
impl SolanaPort for SolanaAttestationAdapter {
    async fn attest(&self, _payload_hash: [u8; 32]) -> Result<String, SolanaError> {
        Err(SolanaError::SubmissionFailed(
            "solana feature is not enabled".to_string(),
        ))
    }

    async fn get_attestation(&self, _authority: &str) -> Result<AttestationRecord, SolanaError> {
        Err(SolanaError::SubmissionFailed(
            "solana feature is not enabled".to_string(),
        ))
    }

    async fn verify(&self, _authority: &str, _payload_hash: [u8; 32]) -> Result<bool, SolanaError> {
        Err(SolanaError::SubmissionFailed(
            "solana feature is not enabled".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn disabled_adapter_returns_feature_error() {
        let adapter = SolanaAttestationAdapter::new("rpc", "prog", "key").unwrap();
        let err = adapter.attest([0u8; 32]).await.unwrap_err();
        assert!(err.to_string().contains("solana feature is not enabled"));
    }
}
