use super::evm_port::{EvmError, EvmPort};
use crate::models::transaction::{Transaction, TransactionReceipt};

/// Port for blockchain interactions.
///
/// Extends [`EvmPort`] with generic read/write operations needed by protocol
/// adapters and the orchestrator. Default implementations delegate to the
/// underlying [`EvmPort`] methods so existing adapters keep working.
pub trait BlockchainPort: EvmPort {
    /// Read the native balance of an address.
    fn get_balance(&self, address: &str) -> Result<u128, EvmError> {
        <Self as EvmPort>::get_balance(self, address)
    }

    /// Estimate the gas required for a raw transaction.
    fn estimate_gas(&self, tx: &Transaction) -> Result<u64, EvmError> {
        <Self as EvmPort>::estimate_gas(self, tx)
    }

    /// Sign and broadcast a raw transaction.
    fn send_transaction(&self, tx: &Transaction) -> Result<String, EvmError> {
        <Self as EvmPort>::send_transaction(self, tx)
    }

    /// Fetch the on-chain receipt for a transaction hash.
    fn get_transaction_receipt(
        &self,
        tx_hash: &str,
    ) -> Result<Option<TransactionReceipt>, EvmError> {
        <Self as EvmPort>::get_transaction_receipt(self, tx_hash)
    }
}

impl<T: EvmPort + ?Sized> BlockchainPort for T {}
