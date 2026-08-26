//! Multi-network EVM adapter registry.
//!
//! Routes delegation/execution calls to the network named by the intent
//! (`OTTER_NETWORKS`), falling back to the `default` network. V1 keeps one
//! private key shared across networks and performs no cross-chain bridging.

use std::collections::HashMap;
use std::sync::Mutex;

use alloy::providers::{Provider, ProviderBuilder};

use crate::blockchain::AlloyEvmAdapter;
use crate::config::NetworkSpec;

/// Cached healthcheck entry for `/api/v1/networks`.
#[derive(Debug, Clone)]
pub struct HealthEntry {
    pub healthy: bool,
    pub checked_at: std::time::Instant,
}

impl HealthEntry {
    /// Consider a healthcheck fresh for 60 seconds.
    pub fn is_fresh(&self) -> bool {
        self.checked_at.elapsed() < std::time::Duration::from_secs(60)
    }
}

/// Read-only view of a configured network for API responses.
#[derive(Debug, Clone)]
pub struct NetworkSummary {
    pub name: String,
    pub chain_id: u64,
    pub vault_address: String,
}

/// Errors surfaced by multi-network routing.
#[derive(Debug)]
pub enum MultichainError {
    NetworkNotFound(String),
    Evm(domain::ports::evm_port::EvmError),
}

impl std::fmt::Display for MultichainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultichainError::NetworkNotFound(name) => write!(f, "network not found: {}", name),
            MultichainError::Evm(e) => write!(f, "{}", e),
        }
    }
}

/// Registry of per-network [`AlloyEvmAdapter`]s sharing one private key.
#[derive(Clone)]
pub struct MultiChainAdapter {
    adapters: Arc<Mutex<HashMap<String, (AlloyEvmAdapter, NetworkSpec)>>>,
}

// AlloyEvmAdapter is Clone; the map is behind a mutex so lookups stay cheap.
type Arc<M> = std::sync::Arc<M>;

impl MultiChainAdapter {
    /// Build the registry from parsed `OTTER_NETWORKS` specs.
    ///
    /// `bb_bin` is forwarded to proof verification helpers; pass `None` when
    /// unknown (the adapter then skips on-chain verification paths that need it).
    pub fn new(
        networks: &[NetworkSpec],
        private_key_hex: &str,
        _bb_bin: Option<&str>,
    ) -> Result<Self, MultichainError> {
        let mut adapters = HashMap::new();
        for spec in networks {
            let adapter =
                AlloyEvmAdapter::new(spec.rpc_url.clone(), private_key_hex, &spec.vault_address)
                    .map_err(MultichainError::Evm)?;
            adapters.insert(spec.name.clone(), (adapter, spec.clone()));
        }
        Ok(Self {
            adapters: Arc::new(Mutex::new(adapters)),
        })
    }

    /// An empty registry (no agent key configured).
    pub fn empty() -> Self {
        Self {
            adapters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Look up the adapter for a network name.
    pub fn adapter_for(&self, network: &str) -> Result<AlloyEvmAdapter, MultichainError> {
        self.adapters
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(network)
            .map(|(a, _)| a.clone())
            .ok_or_else(|| MultichainError::NetworkNotFound(network.to_string()))
    }

    /// Chain id for a network (or the default single-network chain id when
    /// `network` is None and exactly one network is configured).
    pub fn chain_id_of(&self, network: Option<&str>) -> Result<u64, MultichainError> {
        let map = self.adapters.lock().unwrap_or_else(|e| e.into_inner());
        match network {
            Some(name) => map
                .get(name)
                .map(|(_, s)| s.chain_id)
                .ok_or_else(|| MultichainError::NetworkNotFound(name.to_string())),
            None => {
                let mut values = map.values();
                values
                    .next()
                    .filter(|_| values.next().is_none())
                    .map(|(_, s)| s.chain_id)
                    .ok_or_else(|| MultichainError::NetworkNotFound("default".to_string()))
            }
        }
    }

    /// Names of all configured networks.
    pub fn network_names(&self) -> Vec<String> {
        self.adapters
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .keys()
            .cloned()
            .collect()
    }

    /// All configured networks for `/api/v1/networks`.
    pub fn network_summaries(&self) -> Vec<NetworkSummary> {
        self.adapters
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .map(|(_, s)| NetworkSummary {
                name: s.name.clone(),
                chain_id: s.chain_id,
                vault_address: s.vault_address.clone(),
            })
            .collect()
    }

    /// Healthcheck round-trip: `eth_chainId` against the network's RPC.
    pub async fn rpc_chain_id(&self, network: Option<&str>) -> Result<u64, MultichainError> {
        let spec = {
            let map = self.adapters.lock().unwrap_or_else(|e| e.into_inner());
            match network {
                Some(name) => map
                    .get(name)
                    .map(|(_, s)| s.clone())
                    .ok_or_else(|| MultichainError::NetworkNotFound(name.to_string()))?,
                None => map
                    .values()
                    .next()
                    .map(|(_, s)| s.clone())
                    .ok_or_else(|| MultichainError::NetworkNotFound("default".to_string()))?,
            }
        };
        let provider = ProviderBuilder::new().on_http(spec.rpc_url.parse().map_err(|_| {
            MultichainError::Evm(domain::ports::evm_port::EvmError::SubmissionFailed(
                format!("invalid rpc url: {}", spec.rpc_url),
            ))
        })?);
        provider.get_chain_id().await.map_err(|e| {
            MultichainError::Evm(domain::ports::evm_port::EvmError::SubmissionFailed(
                e.to_string(),
            ))
        })
    }
}
