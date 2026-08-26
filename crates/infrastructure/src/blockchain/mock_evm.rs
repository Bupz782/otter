use domain::models::delegation::{DelegationMessage, DelegationProof, PublicDelegationInputs};
use domain::models::transaction::{Transaction, TransactionReceipt};
use domain::ports::evm_port::{EvmError, EvmPort};

/// In-memory EVM adapter for development and testing.
///
/// Records submitted proofs and returns deterministic fake transaction hashes.
/// It does **not** interact with a real chain.
pub struct MockEvmAdapter {
    submissions: std::sync::Mutex<Vec<(Vec<u8>, Vec<u8>)>>,
    fail_next: std::sync::Mutex<bool>,
}

impl Clone for MockEvmAdapter {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl MockEvmAdapter {
    pub fn new() -> Self {
        Self {
            submissions: std::sync::Mutex::new(Vec::new()),
            fail_next: std::sync::Mutex::new(false),
        }
    }

    pub fn failing() -> Self {
        Self {
            submissions: std::sync::Mutex::new(Vec::new()),
            fail_next: std::sync::Mutex::new(true),
        }
    }

    pub fn with_failure(self, fail: bool) -> Self {
        *self.fail_next.lock().unwrap() = fail;
        self
    }

    pub fn submission_count(&self) -> usize {
        self.submissions.lock().unwrap().len()
    }

    pub fn last_submission(&self) -> Option<(Vec<u8>, Vec<u8>)> {
        self.submissions.lock().unwrap().last().cloned()
    }
}

impl Default for MockEvmAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl EvmPort for MockEvmAdapter {
    fn execute_with_proof(
        &self,
        proof: &DelegationProof,
        public_inputs: &PublicDelegationInputs,
    ) -> Result<String, EvmError> {
        if *self.fail_next.lock().unwrap() {
            *self.fail_next.lock().unwrap() = false;
            return Err(EvmError::Reverted("mock revert".to_string()));
        }

        let mut public_inputs_bytes = Vec::new();
        public_inputs_bytes.extend_from_slice(&public_inputs.delegation_hash);
        public_inputs_bytes.extend_from_slice(&public_inputs.proposed_intent.intent_type);
        public_inputs_bytes.extend_from_slice(&public_inputs.proposed_intent.amount);
        public_inputs_bytes.extend_from_slice(&public_inputs.proposed_intent.protocol);
        public_inputs_bytes.extend_from_slice(&public_inputs.proposed_intent.target_contract);
        public_inputs_bytes.extend_from_slice(&public_inputs.timestamp);
        public_inputs_bytes.extend_from_slice(&public_inputs.nonce);

        self.submissions
            .lock()
            .unwrap()
            .push((proof.proof.clone(), public_inputs_bytes));

        Ok(format!("0x{}", hex::encode([0u8; 32])))
    }

    fn get_balance(&self, _address: &str) -> Result<u128, EvmError> {
        Ok(0)
    }

    fn estimate_gas(&self, _tx: &Transaction) -> Result<u64, EvmError> {
        Ok(21000)
    }

    fn send_transaction(&self, _tx: &Transaction) -> Result<String, EvmError> {
        if *self.fail_next.lock().unwrap() {
            *self.fail_next.lock().unwrap() = false;
            return Err(EvmError::SubmissionFailed(
                "mock submission failed".to_string(),
            ));
        }
        Ok(format!("0x{}", hex::encode([1u8; 32])))
    }

    fn get_transaction_receipt(
        &self,
        tx_hash: &str,
    ) -> Result<Option<TransactionReceipt>, EvmError> {
        Ok(Some(TransactionReceipt {
            tx_hash: tx_hash.to_string(),
            block_number: 1,
            status: true,
            gas_used: 21000,
        }))
    }
}

impl domain::ports::multichain_evm_port::MultiChainEvmPort for MockEvmAdapter {
    fn ensure_delegated_on(
        &self,
        _network: Option<&str>,
        _delegation: &DelegationMessage,
    ) -> Result<String, domain::ports::multichain_evm_port::MultichainError> {
        Ok(format!("0x{}", hex::encode([3u8; 32])))
    }

    fn set_protocol_router_on(
        &self,
        _network: Option<&str>,
        protocol: u64,
        router: &str,
    ) -> Result<String, domain::ports::multichain_evm_port::MultichainError> {
        let _ = (protocol, router);
        Ok(format!("0x{}", hex::encode([2u8; 32])))
    }

    fn execute_with_proof_on(
        &self,
        network: Option<&str>,
        proof: &DelegationProof,
        public_inputs: &PublicDelegationInputs,
    ) -> Result<String, domain::ports::multichain_evm_port::MultichainError> {
        // Record the routed network in the fake tx hash so tests can assert
        // which adapter was selected.
        let tag = match network.unwrap_or("default") {
            n if n.contains("arbitrum") => [0xAAu8; 32],
            _ => [0u8; 32],
        };
        EvmPort::execute_with_proof(self, proof, public_inputs)?;
        Ok(format!("0x{}", hex::encode(tag)))
    }

    fn verify_onchain_on(
        &self,
        _network: Option<&str>,
        _proof: &DelegationProof,
        _public_inputs: &PublicDelegationInputs,
    ) -> Result<bool, domain::ports::multichain_evm_port::MultichainError> {
        Ok(true)
    }

    fn chain_id_of(
        &self,
        network: Option<&str>,
    ) -> Result<u64, domain::ports::multichain_evm_port::MultichainError> {
        match network.unwrap_or("default") {
            n if n.contains("arbitrum") => Ok(421614),
            "mainnet" => Ok(1),
            _ => Ok(11155111),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::delegation::{
        ProposedDelegationIntent, field_from_u32, field_from_u64, field_from_u128,
    };

    fn sample_public_inputs() -> PublicDelegationInputs {
        PublicDelegationInputs {
            delegation_hash: [0u8; 32],
            proposed_intent: ProposedDelegationIntent {
                intent_type: field_from_u32(1),
                amount: field_from_u128(1000),
                protocol: field_from_u32(1),
                target_contract: field_from_u32(0),
            },
            timestamp: field_from_u64(1234567890),
            nonce: field_from_u64(42),
        }
    }

    #[test]
    fn records_submission_and_returns_tx_hash() {
        let adapter = MockEvmAdapter::new();
        let proof = DelegationProof {
            proof: b"proof".to_vec(),
            public_inputs: Vec::new(),
        };

        let tx = adapter
            .execute_with_proof(&proof, &sample_public_inputs())
            .unwrap();
        assert_eq!(
            tx,
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(adapter.submission_count(), 1);
    }

    #[test]
    fn failing_adapter_returns_error() {
        let adapter = MockEvmAdapter::failing();
        let proof = DelegationProof {
            proof: Vec::new(),
            public_inputs: Vec::new(),
        };

        assert!(
            adapter
                .execute_with_proof(&proof, &sample_public_inputs())
                .is_err()
        );
    }

    #[test]
    fn get_balance_returns_zero() {
        let adapter = MockEvmAdapter::new();
        assert_eq!(adapter.get_balance("0x...").unwrap(), 0);
    }

    #[test]
    fn estimate_gas_returns_default() {
        let adapter = MockEvmAdapter::new();
        let tx = Transaction::new("0x0000000000000000000000000000000000000000", 0, 100_000);
        assert_eq!(adapter.estimate_gas(&tx).unwrap(), 21000);
    }

    #[test]
    fn send_transaction_returns_hash() {
        let adapter = MockEvmAdapter::new();
        let tx = Transaction::new("0x0000000000000000000000000000000000000000", 0, 100_000);
        let hash = adapter.send_transaction(&tx).unwrap();
        assert!(hash.starts_with("0x"));
    }

    #[test]
    fn send_transaction_failure_is_retryable() {
        let adapter = MockEvmAdapter::failing();
        let tx = Transaction::new("0x0000000000000000000000000000000000000000", 0, 100_000);
        assert!(adapter.send_transaction(&tx).is_err());
    }

    #[test]
    fn get_transaction_receipt_returns_success() {
        let adapter = MockEvmAdapter::new();
        let receipt = adapter
            .get_transaction_receipt("0x1234")
            .unwrap()
            .expect("receipt exists");
        assert_eq!(receipt.tx_hash, "0x1234");
        assert!(receipt.status);
    }
}
