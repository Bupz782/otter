/// A raw EVM transaction ready to be signed and submitted.
///
/// This is intentionally minimal for the MVP. It mirrors the fields needed by
/// an Ethereum transaction while remaining backend-agnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub to: String,
    pub value: u128,
    pub data: Vec<u8>,
    pub gas_limit: u64,
}

impl Transaction {
    /// Create a new transaction with empty calldata.
    pub fn new(to: impl Into<String>, value: u128, gas_limit: u64) -> Self {
        Self {
            to: to.into(),
            value,
            data: Vec::new(),
            gas_limit,
        }
    }

    /// Attach calldata to the transaction.
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }
}

/// Receipt returned after a transaction has been mined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionReceipt {
    pub tx_hash: String,
    pub block_number: u64,
    pub status: bool,
    pub gas_used: u64,
}
