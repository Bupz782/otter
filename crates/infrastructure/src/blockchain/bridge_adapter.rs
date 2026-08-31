use std::str::FromStr;

use alloy::network::EthereumWallet;
use alloy::primitives::{Address, B256, U256};
use alloy::providers::ProviderBuilder;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use async_trait::async_trait;
use domain::ports::{BridgeError, BridgeLockEvent, BridgeLockResult, BridgePort};

sol! {
    #[sol(rpc)]
    interface IOtterBridge {
        function lock(uint256 amount, uint256 destinationChainId) external returns (bytes32 bridgeId);
        function mint(address user, uint256 amount, bytes32 bridgeId) external;

        event Lock(address indexed user, uint256 amount, uint256 indexed destinationChainId, bytes32 indexed bridgeId, uint256 nonce);
    }
}

pub struct OtterBridgeAdapter {
    rpc_url: String,
    bridge_address: Address,
    private_key_hex: String,
}

impl OtterBridgeAdapter {
    /// The chain id is not pinned on the provider: the recommended fillers
    /// fetch it from the node, which also covers nonce and gas.
    pub fn new(
        rpc_url: &str,
        bridge_address: &str,
        private_key_hex: &str,
    ) -> Result<Self, BridgeError> {
        let bridge_address = Address::from_str(bridge_address)
            .map_err(|e| BridgeError::InvalidInput(format!("invalid bridge address: {}", e)))?;

        Ok(Self {
            rpc_url: rpc_url.to_string(),
            bridge_address,
            private_key_hex: private_key_hex.to_string(),
        })
    }
}

#[async_trait]
impl BridgePort for OtterBridgeAdapter {
    async fn lock(
        &self,
        _user: String,
        amount: String,
        destination_chain_id: u64,
    ) -> Result<BridgeLockResult, BridgeError> {
        let amount = amount
            .parse::<U256>()
            .map_err(|e| BridgeError::InvalidInput(format!("invalid amount: {}", e)))?;

        let signer = PrivateKeySigner::from_str(&self.private_key_hex)
            .map_err(|e| BridgeError::InvalidInput(format!("invalid private key: {}", e)))?;
        let url = self
            .rpc_url
            .parse::<reqwest::Url>()
            .map_err(|e| BridgeError::InvalidInput(e.to_string()))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer))
            .on_http(url);

        let contract = IOtterBridge::new(self.bridge_address, provider);
        let call = contract.lock(amount, U256::from(destination_chain_id));
        let tx = call
            .send()
            .await
            .map_err(|e| BridgeError::Contract(e.to_string()))?;
        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| BridgeError::Contract(e.to_string()))?;

        let bridge_id = receipt
            .inner
            .logs()
            .iter()
            .find_map(|log| {
                if log.address() != self.bridge_address {
                    return None;
                }
                log.log_decode::<IOtterBridge::Lock>()
                    .ok()
                    .map(|decoded| format!("{:#x}", decoded.data().bridgeId))
            })
            .ok_or_else(|| BridgeError::Contract("Lock event not found in receipt".to_string()))?;

        Ok(BridgeLockResult {
            bridge_id,
            tx_hash: format!("{:#x}", receipt.transaction_hash),
        })
    }

    async fn mint(
        &self,
        user: String,
        amount: String,
        bridge_id: String,
    ) -> Result<String, BridgeError> {
        let user = Address::from_str(&user)
            .map_err(|e| BridgeError::InvalidInput(format!("invalid user address: {}", e)))?;
        let amount = amount
            .parse::<U256>()
            .map_err(|e| BridgeError::InvalidInput(format!("invalid amount: {}", e)))?;
        let bridge_id = B256::from_str(&bridge_id)
            .map_err(|e| BridgeError::InvalidInput(format!("invalid bridge id: {}", e)))?;

        let signer = PrivateKeySigner::from_str(&self.private_key_hex)
            .map_err(|e| BridgeError::InvalidInput(format!("invalid private key: {}", e)))?;
        let url = self
            .rpc_url
            .parse::<reqwest::Url>()
            .map_err(|e| BridgeError::InvalidInput(e.to_string()))?;
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(EthereumWallet::from(signer))
            .on_http(url);

        let contract = IOtterBridge::new(self.bridge_address, provider);
        let call = contract.mint(user, amount, bridge_id);
        let tx = call
            .send()
            .await
            .map_err(|e| BridgeError::Contract(e.to_string()))?;
        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| BridgeError::Contract(e.to_string()))?;
        Ok(format!("{:#x}", receipt.transaction_hash))
    }

    async fn pending_locks(&self) -> Result<Vec<BridgeLockEvent>, BridgeError> {
        Ok(vec![])
    }
}
