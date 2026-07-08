use crate::use_cases::execute_intent::ExecutionError;
use domain::models::delegation::DelegationMessage;

/// Result of confirming an on-chain transaction.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
}

/// Port for executing an intent end-to-end once its condition is met.
///
/// The orchestrator delegates the heavy lifting (condition re-evaluation,
/// proof generation and on-chain submission) to this port so that it stays
/// independent from any concrete ZKP or EVM adapter.
pub trait ExecutionPort: Send + Sync {
    /// Execute the intent described by `input` and return the on-chain
    /// transaction hash or identifier on success.
    fn execute(&self, input: &str) -> Result<String, ExecutionError>;

    /// Provide a user-signed delegation and its signature for upcoming
    /// executions. The service may fall back to an agent-generated delegation
    /// when none has been supplied.
    fn set_delegation(&self, delegation: DelegationMessage, signature: [u8; 64]);

    /// Poll the chain until the transaction identified by `tx_hash` has been
    /// included in a block. Returns the confirmation result (success + gas used).
    ///
    /// Adapters that do not support on-chain confirmation may keep the default
    /// implementation, which returns `success=true` and `gas_used=0` immediately.
    fn confirm(&self, _tx_hash: &str) -> Result<ExecutionResult, ExecutionError> {
        Ok(ExecutionResult::default())
    }
}
