use crate::ports::evm_port::EvmError;

/// Errors returned by bridge operations.
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("contract error: {0}")]
    Contract(String),
    #[error("not authorized")]
    NotAuthorized,
    #[error("already minted")]
    AlreadyMinted,
    #[error(transparent)]
    Evm(#[from] EvmError),
}

/// Result of a successful token lock on the source chain.
#[derive(Debug, Clone)]
pub struct BridgeLockResult {
    pub bridge_id: String,
    pub tx_hash: String,
}

/// A recorded Lock event observed or emitted by the bridge.
#[derive(Debug, Clone)]
pub struct BridgeLockEvent {
    pub bridge_id: String,
    pub user: String,
    pub amount: String,
    pub source_chain_id: u64,
    pub destination_chain_id: u64,
}

/// Operations for cross-chain lock/mint bridges.
#[async_trait::async_trait]
pub trait BridgePort: Send + Sync {
    /// Lock `amount` of the underlying token on the source chain and emit a Lock event.
    async fn lock(
        &self,
        user: String,
        amount: String,
        destination_chain_id: u64,
    ) -> Result<BridgeLockResult, BridgeError>;

    /// Mint wrapped tokens on the destination chain for a previously locked bridge id.
    async fn mint(
        &self,
        user: String,
        amount: String,
        bridge_id: String,
    ) -> Result<String, BridgeError>;

    /// List pending lock events that have not yet been minted on this side.
    async fn pending_locks(&self) -> Result<Vec<BridgeLockEvent>, BridgeError>;
}
