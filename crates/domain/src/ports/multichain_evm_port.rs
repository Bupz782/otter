//! Multi-chain EVM port: route calls to a named network.

use crate::models::delegation::{DelegationMessage, DelegationProof};
use crate::ports::evm_port::EvmError;

/// Errors surfaced by multi-network routing.
#[derive(Debug)]
pub enum MultichainError {
    /// No adapter configured for the requested network name.
    NetworkNotFound(String),
    /// Underlying EVM error from the routed adapter.
    Evm(EvmError),
}

impl std::fmt::Display for MultichainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultichainError::NetworkNotFound(name) => write!(f, "network not found: {}", name),
            MultichainError::Evm(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for MultichainError {}

impl From<EvmError> for MultichainError {
    fn from(e: EvmError) -> Self {
        MultichainError::Evm(e)
    }
}

/// Route per-network operations through the registry.
///
/// `network: None` selects the default (single-network) configuration.
pub trait MultiChainEvmPort: Send + Sync {
    /// Chain id configured for `network` (or the default when `None`).
    fn chain_id_of(&self, network: Option<&str>) -> Result<u64, MultichainError>;

    /// Register (or refresh) the delegation on the target network.
    fn ensure_delegated_on(
        &self,
        network: Option<&str>,
        delegation: &DelegationMessage,
    ) -> Result<String, MultichainError>;

    /// Whitelist the protocol router on the target network.
    fn set_protocol_router_on(
        &self,
        network: Option<&str>,
        protocol: u64,
        router: &str,
    ) -> Result<String, MultichainError>;

    /// Submit `executeWithProof` on the target network; returns the tx hash.
    fn execute_with_proof_on(
        &self,
        network: Option<&str>,
        proof: &DelegationProof,
        public_inputs: &crate::models::delegation::PublicDelegationInputs,
    ) -> Result<String, MultichainError>;

    /// Verify a proof on the target network's verifier (view call).
    fn verify_onchain_on(
        &self,
        network: Option<&str>,
        proof: &DelegationProof,
        public_inputs: &crate::models::delegation::PublicDelegationInputs,
    ) -> Result<bool, MultichainError>;
}
