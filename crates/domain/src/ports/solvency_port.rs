use crate::models::delegation::FieldBytes;

/// Number of leaves committed in the solvency Merkle-sum tree.
pub const SOLVENCY_LEAF_COUNT: usize = 16;

/// A single depositor leaf of the Merkle-sum tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolvencyLeaf {
    /// blake2s(secret ++ balance) computed off-circuit.
    pub commitment_hash: FieldBytes,
    /// Balance covered by this leaf (u128, smallest on-chain unit).
    pub balance: u128,
}

/// Public inputs of the solvency circuit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicSolvencyInputs {
    /// Root the operator commits to on-chain.
    pub merkle_root: FieldBytes,
    /// Aggregate balance announced to users.
    pub total_deposits: u128,
    /// Proof generation timestamp (seconds since epoch).
    pub timestamp: u64,
}

/// Private inputs of the solvency circuit.
#[derive(Debug, Clone)]
pub struct PrivateSolvencyInputs {
    /// The N individual depositor records.
    pub leaves: [SolvencyLeaf; SOLVENCY_LEAF_COUNT],
}

/// A generated solvency proof with its serialized public inputs.
#[derive(Debug, Clone)]
pub struct SolvencyProof {
    /// Raw UltraHonk proof bytes (empty when only witness validation ran).
    pub proof: Vec<u8>,
    /// Serialized public inputs accompanying the proof.
    pub public_inputs: Vec<u8>,
}

/// Errors that can occur when interacting with a ZKP backend for solvency
/// proofs. Mirrors [`crate::ports::zkp_port::ZkpError`].
#[derive(Debug, thiserror::Error)]
pub enum SolvencyError {
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

/// Port for generating and verifying Merkle-sum proof-of-solvency proofs.
pub trait SolvencyPort {
    /// Generate a proof that the committed leaves sum to `total_deposits`
    /// and hash to the published root.
    fn prove_solvency(
        &self,
        public_inputs: &PublicSolvencyInputs,
        private_inputs: &PrivateSolvencyInputs,
    ) -> Result<SolvencyProof, SolvencyError>;

    /// Verify a solvency proof against its public inputs.
    fn verify_solvency(
        &self,
        proof: &SolvencyProof,
        public_inputs: &PublicSolvencyInputs,
    ) -> Result<bool, SolvencyError>;
}
