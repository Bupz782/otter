use crate::models::intent::{Asset, DexType, LendingType, Protocol};
use crate::models::transaction::Transaction;
use std::collections::HashMap;

/// Errors that can occur when interacting with a DeFi protocol adapter.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("unsupported asset: {0}")]
    UnsupportedAsset(String),
    #[error("unsupported protocol: {0}")]
    UnsupportedProtocol(String),
    #[error("invalid amount: {0}")]
    InvalidAmount(String),
    #[error("operation failed: {0}")]
    OperationFailed(String),
}

/// Common interface for lending protocols such as Aave or Compound.
pub trait LendingProtocol: std::fmt::Debug {
    /// Fetch the current supply APY for `asset` as a percentage (e.g. 3.5 for 3.5%).
    fn get_apy(&self, asset: &Asset) -> Result<f64, ProtocolError>;

    /// Build a transaction that supplies `amount` of `asset` to the protocol.
    fn supply(&self, asset: &Asset, amount: u128) -> Result<Transaction, ProtocolError>;

    /// Build a transaction that withdraws `amount` of `asset` from the protocol.
    fn withdraw(&self, asset: &Asset, amount: u128) -> Result<Transaction, ProtocolError>;

    /// Build a transaction that borrows `amount` of `asset` against collateral.
    fn borrow(
        &self,
        asset: &Asset,
        amount: u128,
        collateral: &Asset,
        collateral_amount: u128,
    ) -> Result<Transaction, ProtocolError>;

    /// Build a transaction that repays `amount` of `asset`.
    fn repay(&self, asset: &Asset, amount: u128) -> Result<Transaction, ProtocolError>;
}

/// Common interface for DEX protocols such as Uniswap or Sushiswap.
pub trait DexProtocol: std::fmt::Debug {
    /// Estimate the output amount for swapping `amount` of `from` into `to`.
    fn get_quote(&self, from: &Asset, to: &Asset, amount: u128) -> Result<u128, ProtocolError>;

    /// Build a transaction that swaps `amount` of `from` into `to`.
    ///
    /// `slippage_bps` is the maximum acceptable slippage in basis points
    /// (e.g. 100 for 1%).
    fn swap(
        &self,
        from: &Asset,
        to: &Asset,
        amount: u128,
        slippage_bps: u16,
    ) -> Result<Transaction, ProtocolError>;
}

/// Registry that maps domain protocol identifiers to concrete adapters.
///
/// The registry holds trait-object references so the orchestrator can resolve
/// the right adapter for a given intent without knowing the concrete type.
pub struct ProtocolRegistry<'a> {
    lending: HashMap<LendingType, &'a dyn LendingProtocol>,
    dex: HashMap<DexType, &'a dyn DexProtocol>,
}

impl<'a> Default for ProtocolRegistry<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ProtocolRegistry<'a> {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            lending: HashMap::new(),
            dex: HashMap::new(),
        }
    }

    /// Register a lending protocol adapter.
    pub fn register_lending(
        &mut self,
        protocol: LendingType,
        adapter: &'a dyn LendingProtocol,
    ) -> &mut Self {
        self.lending.insert(protocol, adapter);
        self
    }

    /// Register a DEX protocol adapter.
    pub fn register_dex(&mut self, protocol: DexType, adapter: &'a dyn DexProtocol) -> &mut Self {
        self.dex.insert(protocol, adapter);
        self
    }

    /// Look up a lending adapter by protocol type.
    pub fn lending(&self, protocol: &LendingType) -> Option<&'a dyn LendingProtocol> {
        self.lending.get(protocol).copied()
    }

    /// Look up a DEX adapter by protocol type.
    pub fn dex(&self, protocol: &DexType) -> Option<&'a dyn DexProtocol> {
        self.dex.get(protocol).copied()
    }

    /// Look up any protocol adapter.
    ///
    /// Returns `None` if the protocol is not registered.
    pub fn resolve(&self, protocol: &Protocol) -> Result<ProtocolAdapter<'a>, ProtocolError> {
        match protocol {
            Protocol::Lending(lending) => self
                .lending(lending)
                .map(ProtocolAdapter::Lending)
                .ok_or_else(|| ProtocolError::UnsupportedProtocol(format!("{:?}", lending))),
            Protocol::Dex(dex) => self
                .dex(dex)
                .map(ProtocolAdapter::Dex)
                .ok_or_else(|| ProtocolError::UnsupportedProtocol(format!("{:?}", dex))),
        }
    }
}

/// Discriminated union returned by [`ProtocolRegistry::resolve`].
#[derive(Debug, Clone, Copy)]
pub enum ProtocolAdapter<'a> {
    Lending(&'a dyn LendingProtocol),
    Dex(&'a dyn DexProtocol),
}
