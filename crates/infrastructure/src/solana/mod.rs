//! Solana attestation adapter.
//!
//! The real implementation is behind the `solana` Cargo feature. When the
//! feature is disabled, the exported adapter returns a configuration error for
//! every call, keeping the rest of the workspace compilable without the Solana
//! toolchain.

#[cfg(feature = "solana")]
mod adapter;
#[cfg(feature = "solana")]
pub use adapter::SolanaAttestationAdapter;

pub mod scheduler;

#[cfg(not(feature = "solana"))]
mod disabled;
#[cfg(not(feature = "solana"))]
pub use disabled::SolanaAttestationAdapter;
