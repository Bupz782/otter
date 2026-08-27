//! Mempool monitor for bundle-based MEV.
//!
//! V2 polls a pending-transaction filter and emits transactions whose `to`
//! address matches a configured target (e.g., the vault or a protocol router).
//! Callers can use these events to build and submit backrun bundles via a
//! [`BundleSearcherPort`].

use std::time::Duration;

use alloy::primitives::{Address, B256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::Transaction as RpcTransaction;
use futures::StreamExt;
use tracing::{debug, error, info};

/// A pending transaction together with its arrival metadata.
#[derive(Debug, Clone)]
pub struct PendingTransaction {
    pub hash: B256,
    pub from: alloy::primitives::Address,
    pub to: Option<Address>,
    pub value: alloy::primitives::U256,
    pub input: alloy::primitives::Bytes,
}

/// Callback invoked for every matching pending transaction.
#[async_trait::async_trait]
pub trait MempoolHandler: Send + Sync {
    async fn on_target_tx(&self, tx: PendingTransaction);
}

/// Polls the mempool for transactions targeting a specific address.
pub struct MempoolMonitor {
    rpc_url: String,
    target_address: Address,
    poll_interval: Duration,
}

impl MempoolMonitor {
    /// Create a new monitor.
    pub fn new(rpc_url: String, target_address: Address, poll_interval: Duration) -> Self {
        Self {
            rpc_url,
            target_address,
            poll_interval,
        }
    }

    /// Start monitoring. Runs indefinitely.
    pub async fn run<H: MempoolHandler>(self, handler: H) {
        let url = match self.rpc_url.parse() {
            Ok(u) => u,
            Err(err) => {
                error!(%err, "invalid mempool rpc url");
                return;
            }
        };
        let provider = ProviderBuilder::new().on_http(url);

        let mut stream = match provider.watch_pending_transactions().await {
            Ok(p) => p.into_stream(),
            Err(err) => {
                error!(%err, "watch_pending_transactions not supported by rpc");
                return;
            }
        };

        info!(%self.target_address, "mempool monitor started");

        loop {
            let Some(batch) = tokio::time::timeout(self.poll_interval, stream.next())
                .await
                .ok()
                .flatten()
            else {
                continue;
            };

            for hash in batch {
                let tx: Option<RpcTransaction> = match provider.get_transaction_by_hash(hash).await
                {
                    Ok(t) => t,
                    Err(err) => {
                        debug!(%hash, %err, "failed to fetch pending transaction");
                        continue;
                    }
                };

                if let Some(tx) = tx && tx.to == Some(self.target_address) {
                    info!(%hash, "target transaction detected in mempool");
                    handler
                        .on_target_tx(PendingTransaction {
                            hash,
                            from: tx.from,
                            to: tx.to,
                            value: tx.value,
                            input: tx.input,
                        })
                        .await;
                }
            }
        }
    }
}
