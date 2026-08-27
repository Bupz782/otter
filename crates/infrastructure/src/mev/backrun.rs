//! Minimal backrun handler for the bundle-based MEV searcher.
//!
//! When the mempool monitor reports a transaction targeting the watched
//! address, this handler builds a signed rebate transfer (searcher key to
//! beneficiary, zero value — profit extraction is the strategy's job, see
//! `docs/MEV_SEARCHER.md`) and submits it as a bundle for the next block via
//! the [`BundleSearcherPort`]. Every submission attempt is recorded in the
//! `mev_bundles` table through the [`StoragePort`].

use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use alloy::eips::eip2718::Encodable2718;
use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use domain::ports::searcher_port::{Bundle, BundleSearcherPort};
use domain::ports::storage_port::{MevBundleRecord, StoragePort};
use tracing::{error, info, warn};

use super::mempool_monitor::{MempoolHandler, MempoolMonitor, PendingTransaction};

/// Builds and submits a backrun bundle for each detected target transaction.
pub struct BackrunHandler {
    searcher: Arc<dyn BundleSearcherPort>,
    storage: Arc<dyn StoragePort>,
    rpc_url: String,
    private_key_hex: String,
    beneficiary: Address,
    chain_id: u64,
}

impl BackrunHandler {
    pub fn new(
        searcher: Arc<dyn BundleSearcherPort>,
        storage: Arc<dyn StoragePort>,
        rpc_url: String,
        private_key_hex: String,
        beneficiary: Address,
        chain_id: u64,
    ) -> Self {
        Self {
            searcher,
            storage,
            rpc_url,
            private_key_hex,
            beneficiary,
            chain_id,
        }
    }

    fn unix_now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    /// Build and sign the zero-value rebate transfer to the beneficiary.
    async fn build_rebate_tx(&self) -> Result<Vec<u8>, String> {
        let signer = PrivateKeySigner::from_str(&self.private_key_hex)
            .map_err(|e| format!("invalid searcher private key: {e}"))?;
        let url = self
            .rpc_url
            .parse::<reqwest::Url>()
            .map_err(|e| format!("invalid rpc url: {e}"))?;
        let provider = ProviderBuilder::new().on_http(url);

        let nonce = provider
            .get_transaction_count(signer.address())
            .pending()
            .await
            .map_err(|e| format!("nonce lookup: {e}"))?;
        let fees = provider
            .estimate_eip1559_fees(None)
            .await
            .map_err(|e| format!("fee estimation: {e}"))?;

        let wallet = EthereumWallet::from(signer);
        let tx = TransactionRequest::default()
            .with_to(self.beneficiary)
            .with_value(U256::ZERO)
            .with_nonce(nonce)
            .with_chain_id(self.chain_id)
            .with_gas_limit(21_000)
            .with_max_fee_per_gas(fees.max_fee_per_gas)
            .with_max_priority_fee_per_gas(fees.max_priority_fee_per_gas);
        let envelope = tx
            .build(&wallet)
            .await
            .map_err(|e| format!("sign rebate tx: {e}"))?;

        Ok(envelope.encoded_2718())
    }

    async fn record(&self, bundle_hash: String, target_tx_hash: Option<String>, status: &str) {
        let record = MevBundleRecord {
            bundle_hash,
            target_tx_hash,
            status: status.to_string(),
            created_at: Self::unix_now(),
        };
        if let Err(err) = self.storage.save_mev_bundle(&record).await {
            warn!(%err, "failed to persist mev bundle record");
        }
    }
}

#[async_trait::async_trait]
impl MempoolHandler for BackrunHandler {
    async fn on_target_tx(&self, tx: PendingTransaction) {
        let target = format!("{:#x}", tx.hash);

        let raw_tx = match self.build_rebate_tx().await {
            Ok(raw) => raw,
            Err(err) => {
                error!(%target, %err, "failed to build rebate transaction");
                self.record(format!("failed-{target}"), Some(target), "failed")
                    .await;
                return;
            }
        };

        // Omit the block number: the relay targets the next block.
        let bundle = Bundle {
            txs: vec![raw_tx],
            block_number: None,
            min_timestamp: None,
            max_timestamp: None,
        };

        match self.searcher.submit_bundle(bundle).await {
            Ok(bundle_hash) => {
                info!(%target, %bundle_hash, "backrun bundle submitted");
                self.record(bundle_hash, Some(target), "submitted").await;
            }
            Err(err) => {
                error!(%target, %err, "backrun bundle submission failed");
                self.record(format!("failed-{target}"), Some(target), "failed")
                    .await;
            }
        }
    }
}

/// Spawn the mempool monitor + backrun handler as a background task.
///
/// Returns `None` (with a warning) when the target address or searcher key is
/// invalid. When `beneficiary` is unset, rebate transfers default to the
/// searcher address itself.
#[allow(clippy::too_many_arguments)]
pub fn spawn_backrun_monitor(
    rpc_url: String,
    chain_id: u64,
    target_address: &str,
    beneficiary: Option<&str>,
    searcher_private_key: &str,
    poll_interval: std::time::Duration,
    searcher: Arc<dyn BundleSearcherPort>,
    storage: Arc<dyn StoragePort>,
) -> Option<tokio::task::JoinHandle<()>> {
    let target = match Address::from_str(target_address) {
        Ok(addr) => addr,
        Err(err) => {
            warn!(%err, target_address, "invalid mev bundle target address; backrun disabled");
            return None;
        }
    };
    let signer = match PrivateKeySigner::from_str(searcher_private_key) {
        Ok(signer) => signer,
        Err(err) => {
            warn!(%err, "invalid mev bundle private key; backrun disabled");
            return None;
        }
    };
    let beneficiary = beneficiary
        .and_then(|b| Address::from_str(b).ok())
        .unwrap_or_else(|| signer.address());

    let monitor = MempoolMonitor::new(rpc_url.clone(), target, poll_interval);
    let handler = BackrunHandler::new(
        searcher,
        storage,
        rpc_url,
        searcher_private_key.to_string(),
        beneficiary,
        chain_id,
    );
    info!(%target, %beneficiary, "spawning mev backrun monitor");
    Some(tokio::spawn(async move { monitor.run(handler).await }))
}
