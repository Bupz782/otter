use crate::models::delegation::{DelegationProof, PrivateDelegationInputs, PublicDelegationInputs};

/// Errors that can occur when interacting with a ZKP backend.
#[derive(Debug, thiserror::Error)]
pub enum ZkpError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("witness generation failed: {0}")]
    WitnessGenerationFailed(String),
    #[error("proof generation failed: {0}")]
    ProofGenerationFailed(String),
    #[error("verification failed: {0}")]
    VerificationFailed(String),
    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Port for generating and verifying delegation proofs.
pub trait ZkpPort {
    /// Generate a proof that a proposed intent is authorized by a delegation.
    ///
    /// The default implementation in this crate is a port; concrete adapters
    /// (e.g. `NoirAdapter`) handle the backend-specific proof system calls.
    fn prove_delegation(
        &self,
        public_inputs: &PublicDelegationInputs,
        private_inputs: &PrivateDelegationInputs,
    ) -> Result<DelegationProof, ZkpError>;

    /// Verify a delegation proof against its public inputs.
    fn verify_delegation(
        &self,
        proof: &DelegationProof,
        public_inputs: &PublicDelegationInputs,
    ) -> Result<bool, ZkpError>;
}
