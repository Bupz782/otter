use crate::models::delegation::{DelegationProof, PublicDelegationInputs};
use crate::models::transaction::{Transaction, TransactionReceipt};

/// Errors that can occur when interacting with the EVM chain.
#[derive(Debug, thiserror::Error)]
pub enum EvmError {
    #[error("transaction submission failed: {0}")]
    SubmissionFailed(String),
    #[error("transaction reverted: {0}")]
    Reverted(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Port for submitting verified delegation proofs on-chain.
pub trait EvmPort {
    /// Submit a delegation proof to the vault contract.
    ///
    /// Returns a transaction identifier (e.g. transaction hash) on success.
    fn execute_with_proof(
        &self,
        proof: &DelegationProof,
        public_inputs: &PublicDelegationInputs,
    ) -> Result<String, EvmError>;

    /// Read the native balance of an EVM address.
    fn get_balance(&self, address: &str) -> Result<u128, EvmError>;

    /// Estimate the gas required to execute a raw transaction.
    fn estimate_gas(&self, tx: &Transaction) -> Result<u64, EvmError>;

    /// Sign and broadcast a raw transaction.
    fn send_transaction(&self, tx: &Transaction) -> Result<String, EvmError>;

    /// Fetch the on-chain receipt for a transaction hash, if already mined.
    fn get_transaction_receipt(
        &self,
        tx_hash: &str,
    ) -> Result<Option<TransactionReceipt>, EvmError>;
}
