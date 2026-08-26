//! MEV capture port (simulated V1 — see `infrastructure::mev` for the
//! deterministic profit model and its documented limitations).

use crate::ports::evm_port::EvmError;

/// Records simulated MEV profit captured during intent execution.
pub trait MevPort: Send + Sync {
    /// Record the capture attached to a successful execution transaction.
    ///
    /// Returns the recorded capture, or `None` when the backend cannot store
    /// captures (e.g. non-SQLite storage).
    fn capture_from_execution(
        &self,
        tx_hash: &str,
        block_number: u64,
        amount: u128,
        owner_address: &str,
    ) -> Result<Option<crate::ports::mev_port::MevCapture>, EvmError>;
}

/// One recorded capture.
#[derive(Debug, Clone)]
pub struct MevCapture {
    pub tx_hash: String,
    pub block_number: u64,
    /// Captured profit in wei.
    pub profit_wei: u128,
}
