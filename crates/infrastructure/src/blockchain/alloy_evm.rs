use domain::models::delegation::{DelegationMessage, DelegationProof, PublicDelegationInputs};
use domain::models::transaction::{Transaction, TransactionReceipt};
use domain::ports::evm_port::{EvmError, EvmPort};

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, B256, Bytes, FixedBytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use std::time::Duration;

use super::retry::{retry_any, with_retry};

sol! {
    #[sol(rpc)]
    interface DelegationVault {
        function delegate(
            bytes32 delegationHash,
            uint256 allowedIntents,
            uint256[10] maxAmounts,
            uint256[5] allowedProtocols,
            uint256 expiry,
            uint256 nonce
        ) external;

        function executeWithProof(bytes calldata proof, bytes32[] calldata publicInputs) external;

        function setProtocolRouter(uint256 protocol, address router) external;

        function verifier() external view returns (address);
    }

    #[sol(rpc)]
    interface IVerifier {
        function verify(bytes calldata proof, bytes32[] calldata publicInputs) external view returns (bool);
    }
}

/// EVM adapter backed by Alloy.
///
/// Signs and submits `DelegationVault.executeWithProof` transactions to an
/// Ethereum-compatible RPC. A new Tokio runtime is created for each call so
/// that the adapter can implement the synchronous `EvmPort` trait.
#[derive(Clone)]
pub struct AlloyEvmAdapter {
    rpc_url: String,
    signer: PrivateKeySigner,
    vault_address: Address,
}

impl AlloyEvmAdapter {
    /// Create an adapter from an RPC URL, a hex private key and a vault address.
    pub fn new(
        rpc_url: String,
        private_key_hex: &str,
        vault_address: &str,
    ) -> Result<Self, EvmError> {
        let cleaned = private_key_hex
            .trim()
            .strip_prefix("0x")
            .unwrap_or(private_key_hex);
        let mut key_bytes = [0u8; 32];
        let decoded = hex::decode(cleaned)
            .map_err(|e| EvmError::InvalidInput(format!("invalid private key hex: {}", e)))?;
        if decoded.len() != 32 {
            return Err(EvmError::InvalidInput(format!(
                "private key must be 32 bytes, got {}",
                decoded.len()
            )));
        }
        key_bytes.copy_from_slice(&decoded);

        let signer = PrivateKeySigner::from_slice(&key_bytes)
            .map_err(|e| EvmError::InvalidInput(format!("invalid private key: {}", e)))?;

        let vault_address: Address = vault_address
            .parse()
            .map_err(|e| EvmError::InvalidInput(format!("invalid vault address: {}", e)))?;

        Ok(Self {
            rpc_url,
            signer,
            vault_address,
        })
    }

    /// Register the given delegation on-chain. This must be called by the
    /// delegation owner before any agent can execute intents against it.
    pub fn ensure_delegated(&self, delegation: &DelegationMessage) -> Result<String, EvmError> {
        with_retry(
            || {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| EvmError::SubmissionFailed(format!("tokio runtime: {}", e)))?;
                rt.block_on(self.ensure_delegated_async(delegation))
            },
            3,
            Duration::from_millis(500),
            Duration::from_secs(5),
            is_retryable_evm_error,
        )
    }

    /// Whitelist the router address for a protocol id. Must be called by the
    /// vault owner before intents targeting that protocol can execute.
    pub fn set_protocol_router(&self, protocol: u64, router: &str) -> Result<String, EvmError> {
        with_retry(
            || {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| EvmError::SubmissionFailed(format!("tokio runtime: {}", e)))?;
                rt.block_on(self.set_protocol_router_async(protocol, router))
            },
            3,
            Duration::from_millis(500),
            Duration::from_secs(5),
            is_retryable_evm_error,
        )
    }

    async fn set_protocol_router_async(
        &self,
        protocol: u64,
        router: &str,
    ) -> Result<String, EvmError> {
        let wallet = EthereumWallet::from(self.signer.clone());
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid rpc url: {}", e)))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(url);

        let vault = DelegationVault::new(self.vault_address, provider);
        let router_address = router
            .parse::<Address>()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid router address: {}", e)))?;

        let tx_hash = vault
            .setProtocolRouter(U256::from(protocol), router_address)
            .send()
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("router set failed: {}", e)))?
            .watch()
            .await
            .map_err(|e| {
                EvmError::SubmissionFailed(format!("router set confirmation failed: {}", e))
            })?;

        Ok(tx_hash.to_string())
    }

    async fn ensure_delegated_async(
        &self,
        delegation: &DelegationMessage,
    ) -> Result<String, EvmError> {
        let wallet = EthereumWallet::from(self.signer.clone());
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid rpc url: {}", e)))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(url);

        let vault = DelegationVault::new(self.vault_address, provider);

        let delegation_hash =
            FixedBytes::from_slice(&domain::models::delegation::hash_delegation(delegation));
        let allowed_intents = U256::from_be_bytes(delegation.allowed_intents);
        let max_amounts = array_to_u256::<10>(&delegation.max_amounts);
        let allowed_protocols = array_to_u256::<5>(&delegation.allowed_protocols);
        let expiry = U256::from_be_bytes(delegation.expiry);
        let nonce = U256::from_be_bytes(delegation.nonce);

        let tx_hash = vault
            .delegate(
                delegation_hash,
                allowed_intents,
                max_amounts,
                allowed_protocols,
                expiry,
                nonce,
            )
            .send()
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("delegate send failed: {}", e)))?
            .watch()
            .await
            .map_err(|e| {
                EvmError::SubmissionFailed(format!("delegate confirmation failed: {}", e))
            })?;

        Ok(tx_hash.to_string())
    }

    async fn verify_async(&self, proof: &DelegationProof) -> Result<bool, EvmError> {
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid rpc url: {}", e)))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(url);

        let vault = DelegationVault::new(self.vault_address, &provider);
        let verifier_address = vault
            .verifier()
            .call()
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("failed to read verifier: {}", e)))?
            ._0;

        let verifier = IVerifier::new(verifier_address, &provider);
        let proof_bytes = Bytes::copy_from_slice(&proof.proof);
        let public_inputs_vec = chunk_public_inputs(&proof.public_inputs)?;

        let valid = verifier
            .verify(proof_bytes, public_inputs_vec)
            .call()
            .await
            .map_err(|e| EvmError::Reverted(format!("verify call failed: {}", e)))?
            ._0;

        Ok(valid)
    }

    async fn execute_async(
        &self,
        proof: &DelegationProof,
        _public_inputs: &PublicDelegationInputs,
    ) -> Result<String, EvmError> {
        let wallet = EthereumWallet::from(self.signer.clone());
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid rpc url: {}", e)))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(url);

        let vault = DelegationVault::new(self.vault_address, provider);

        let proof_bytes = Bytes::copy_from_slice(&proof.proof);
        let public_inputs_vec = chunk_public_inputs(&proof.public_inputs)?;

        let tx_hash = vault
            .executeWithProof(proof_bytes, public_inputs_vec)
            .send()
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("send failed: {}", e)))?
            .watch()
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("confirmation failed: {}", e)))?;

        Ok(tx_hash.to_string())
    }
}

impl AlloyEvmAdapter {
    /// Verify a proof on-chain against the verifier contract linked to the vault.
    /// This is a view call and does not modify chain state.
    pub fn verify_onchain(&self, proof: &DelegationProof) -> Result<bool, EvmError> {
        with_retry(
            || {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| EvmError::SubmissionFailed(format!("tokio runtime: {}", e)))?;
                rt.block_on(self.verify_async(proof))
            },
            3,
            Duration::from_millis(500),
            Duration::from_secs(5),
            retry_any,
        )
    }

    /// Poll the RPC provider until the transaction receipt is available.
    ///
    /// Retries with backoff to tolerate transient RPC latency or mempool
    /// propagation delays. Returns `true` only when the receipt exists and
    /// reports a successful status (status code == 1).
    pub fn confirm_transaction(&self, tx_hash: &str) -> Result<bool, EvmError> {
        with_retry(
            || {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| EvmError::SubmissionFailed(format!("tokio runtime: {}", e)))?;
                rt.block_on(self.confirm_transaction_async(tx_hash))
            },
            5,
            Duration::from_millis(500),
            Duration::from_secs(15),
            is_retryable_evm_error,
        )
    }

    async fn confirm_transaction_async(&self, tx_hash: &str) -> Result<bool, EvmError> {
        let cleaned = tx_hash.strip_prefix("0x").unwrap_or(tx_hash);
        let mut hash_bytes = [0u8; 32];
        hex::decode_to_slice(cleaned, &mut hash_bytes)
            .map_err(|e| EvmError::InvalidInput(format!("invalid tx hash: {}", e)))?;
        let hash = B256::from(hash_bytes);

        let url = self
            .rpc_url
            .parse()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid rpc url: {}", e)))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(url);

        let receipt = provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("receipt fetch failed: {}", e)))?;

        Ok(receipt.is_some_and(|r| r.status()))
    }

    /// Return the Ethereum address of the configured signer.
    pub fn signer_address(&self) -> String {
        self.signer.address().to_string()
    }

    /// Return the current on-chain transaction count for the adapter's signer.
    pub async fn get_transaction_count(&self) -> Result<u64, EvmError> {
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid rpc url: {}", e)))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(url);

        let count = provider
            .get_transaction_count(self.signer.address())
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("failed to get nonce: {}", e)))?;
        Ok(count)
    }

    async fn get_balance_async(&self, address: &str) -> Result<u128, EvmError> {
        let addr: Address = address
            .parse()
            .map_err(|e| EvmError::InvalidInput(format!("invalid address: {}", e)))?;
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid rpc url: {}", e)))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(url);

        let balance = provider
            .get_balance(addr)
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("balance fetch failed: {}", e)))?;
        Ok(balance.to::<u128>())
    }

    async fn estimate_gas_async(&self, tx: &Transaction) -> Result<u64, EvmError> {
        let to: Address = tx
            .to
            .parse()
            .map_err(|e| EvmError::InvalidInput(format!("invalid to address: {}", e)))?;
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid rpc url: {}", e)))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(url);

        let tx_request = alloy::rpc::types::TransactionRequest::default()
            .to(to)
            .value(U256::from(tx.value))
            .input(alloy::primitives::Bytes::copy_from_slice(&tx.data).into());

        let gas = provider
            .estimate_gas(&tx_request)
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("gas estimation failed: {}", e)))?;
        Ok(gas as u64)
    }

    async fn send_transaction_async(&self, tx: &Transaction) -> Result<String, EvmError> {
        let to: Address = tx
            .to
            .parse()
            .map_err(|e| EvmError::InvalidInput(format!("invalid to address: {}", e)))?;
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid rpc url: {}", e)))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(self.signer.clone()))
            .on_http(url);

        let tx_request = alloy::rpc::types::TransactionRequest::default()
            .to(to)
            .value(U256::from(tx.value))
            .input(alloy::primitives::Bytes::copy_from_slice(&tx.data).into())
            .gas_limit(tx.gas_limit.into());

        let tx_hash = provider
            .send_transaction(tx_request)
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("send failed: {}", e)))?
            .watch()
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("confirmation failed: {}", e)))?;
        Ok(tx_hash.to_string())
    }

    async fn get_transaction_receipt_async(
        &self,
        tx_hash: &str,
    ) -> Result<Option<TransactionReceipt>, EvmError> {
        let cleaned = tx_hash.strip_prefix("0x").unwrap_or(tx_hash);
        let mut hash_bytes = [0u8; 32];
        hex::decode_to_slice(cleaned, &mut hash_bytes)
            .map_err(|e| EvmError::InvalidInput(format!("invalid tx hash: {}", e)))?;
        let hash = B256::from(hash_bytes);

        let url = self
            .rpc_url
            .parse()
            .map_err(|e| EvmError::SubmissionFailed(format!("invalid rpc url: {}", e)))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_http(url);

        let receipt = provider
            .get_transaction_receipt(hash)
            .await
            .map_err(|e| EvmError::SubmissionFailed(format!("receipt fetch failed: {}", e)))?;

        Ok(receipt.map(|r| TransactionReceipt {
            tx_hash: tx_hash.to_string(),
            block_number: r.block_number.unwrap_or_default(),
            status: r.status(),
            gas_used: r.gas_used as u64,
        }))
    }
}

impl EvmPort for AlloyEvmAdapter {
    fn execute_with_proof(
        &self,
        proof: &DelegationProof,
        public_inputs: &PublicDelegationInputs,
    ) -> Result<String, EvmError> {
        let _ = public_inputs;
        with_retry(
            || {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| EvmError::SubmissionFailed(format!("tokio runtime: {}", e)))?;
                rt.block_on(self.execute_async(proof, public_inputs))
            },
            3,
            Duration::from_millis(500),
            Duration::from_secs(5),
            is_retryable_evm_error,
        )
    }

    fn get_balance(&self, address: &str) -> Result<u128, EvmError> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| EvmError::SubmissionFailed(format!("tokio runtime: {}", e)))?;
        rt.block_on(self.get_balance_async(address))
    }

    fn estimate_gas(&self, tx: &Transaction) -> Result<u64, EvmError> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| EvmError::SubmissionFailed(format!("tokio runtime: {}", e)))?;
        rt.block_on(self.estimate_gas_async(tx))
    }

    fn send_transaction(&self, tx: &Transaction) -> Result<String, EvmError> {
        with_retry(
            || {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| EvmError::SubmissionFailed(format!("tokio runtime: {}", e)))?;
                rt.block_on(self.send_transaction_async(tx))
            },
            3,
            Duration::from_millis(500),
            Duration::from_secs(5),
            is_retryable_evm_error,
        )
    }

    fn get_transaction_receipt(
        &self,
        tx_hash: &str,
    ) -> Result<Option<TransactionReceipt>, EvmError> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| EvmError::SubmissionFailed(format!("tokio runtime: {}", e)))?;
        rt.block_on(self.get_transaction_receipt_async(tx_hash))
    }
}

fn is_retryable_evm_error(err: &EvmError) -> bool {
    match err {
        EvmError::SubmissionFailed(msg) | EvmError::Reverted(msg) => {
            let lower = msg.to_lowercase();
            lower.contains("submission failed")
                || lower.contains("underpriced")
                || lower.contains("max fee per gas")
                || lower.contains("max priority fee")
                || lower.contains("replacement transaction")
                || lower.contains("nonce too low")
        }
        EvmError::Io(_) => true,
        _ => false,
    }
}

fn array_to_u256<const N: usize>(fields: &[[u8; 32]; N]) -> [U256; N] {
    let mut result = [U256::ZERO; N];
    for i in 0..N {
        result[i] = U256::from_be_bytes(fields[i]);
    }
    result
}

fn chunk_public_inputs(public_inputs: &[u8]) -> Result<Vec<FixedBytes<32>>, EvmError> {
    if !public_inputs.len().is_multiple_of(32) {
        return Err(EvmError::InvalidInput(format!(
            "public inputs length {} is not a multiple of 32",
            public_inputs.len()
        )));
    }

    let (chunks, _remainder) = public_inputs.as_chunks::<32>();
    let result = chunks
        .iter()
        .map(|chunk| FixedBytes::from(*chunk))
        .collect();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunks_public_inputs_correctly() {
        let bytes = vec![0u8; 1216]; // 38 * 32
        let chunks = chunk_public_inputs(&bytes).unwrap();
        assert_eq!(chunks.len(), 38);
    }

    #[test]
    fn rejects_misaligned_public_inputs() {
        let bytes = vec![0u8; 100];
        assert!(chunk_public_inputs(&bytes).is_err());
    }

    #[test]
    fn retryable_predicate_catches_gas_errors() {
        assert!(is_retryable_evm_error(&EvmError::SubmissionFailed(
            "replacement transaction underpriced".to_string()
        )));
        assert!(is_retryable_evm_error(&EvmError::SubmissionFailed(
            "max fee per gas too low".to_string()
        )));
        assert!(is_retryable_evm_error(&EvmError::SubmissionFailed(
            "nonce too low".to_string()
        )));
        assert!(!is_retryable_evm_error(&EvmError::InvalidInput(
            "bad address".to_string()
        )));
    }
}
