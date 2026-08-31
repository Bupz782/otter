//! Periodic Solana attestation scheduler.
//!
//! When both the Solana adapter and an on-chain `SolvencyRegistry` are
//! configured, the API spawns this task at boot. Each tick reads the current
//! proven Merkle root from the EVM registry and anchors it on Solana through
//! the [`SolanaPort`], so the attestation registry mirrors the EVM solvency
//! state. Not feature-gated: it only uses trait objects.

use std::sync::Arc;
use std::time::Duration;

use domain::ports::SolanaPort;
use tracing::{debug, info, warn};

use crate::blockchain::MultiChainAdapter;

/// Spawn a background loop attesting the solvency Merkle root on Solana every
/// `interval`. Skips the tick (with a debug log) while the registry has no
/// proven root yet; transient failures are logged and retried next tick.
pub fn spawn_attestation_scheduler(
    solana: Arc<dyn SolanaPort>,
    multichain: Arc<MultiChainAdapter>,
    registry_address: String,
    interval: Duration,
) -> tokio::task::JoinHandle<()> {
    info!(?interval, %registry_address, "spawning solana attestation scheduler");
    tokio::spawn(async move {
        loop {
            match multichain.solvency_state(&registry_address, None).await {
                Ok(state) if state.merkle_root != [0u8; 32] => {
                    match solana.attest(state.merkle_root).await {
                        Ok(sig) => info!(%sig, "solvency merkle root attested on solana"),
                        Err(err) => warn!(%err, "solana attestation failed"),
                    }
                }
                Ok(_) => debug!("solvency registry has no proven root yet; skipping attestation"),
                Err(err) => warn!(%err, "failed to read solvency state for solana attestation"),
            }
            tokio::time::sleep(interval).await;
        }
    })
}
