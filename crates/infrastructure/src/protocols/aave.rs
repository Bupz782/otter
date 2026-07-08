use alloy::primitives::{Address, U256};
use alloy::providers::ProviderBuilder;
use alloy::sol;
use alloy::sol_types::SolCall;
use domain::models::intent::Asset;
use domain::models::transaction::Transaction;
use domain::protocols::{LendingProtocol, ProtocolError};
use std::time::Duration;

use crate::blockchain::retry::{retry_any, with_retry};

sol! {
    #[sol(rpc)]
    interface IPool {
        /// Simplified return: skip the full ReserveData struct by decoding its
        /// fields as a tuple. We only need `currentLiquidityRate` (3rd uint128).
        function getReserveData(address asset)
            external
            view
            returns (
                uint256 configuration,
                uint128 liquidityIndex,
                uint128 currentLiquidityRate,
                uint128 variableBorrowIndex,
                uint128 currentVariableBorrowRate,
                uint128 currentStableBorrowRate,
                uint40 lastUpdateTimestamp,
                uint16 id,
                address aTokenAddress,
                address stableDebtTokenAddress,
                address variableDebtTokenAddress,
                address interestRateStrategyAddress,
                uint128 accruedToTreasury,
                uint128 unbacked,
                uint128 isolationModeTotalDebt
            );

        function supply(address asset, uint256 amount, address onBehalfOf, uint16 referralCode) external;
        function withdraw(address asset, uint256 amount, address to) external returns (uint256);
        function borrow(address asset, uint256 amount, uint256 interestRateMode, uint16 referralCode, address onBehalfOf) external;
        function repay(address asset, uint256 amount, uint256 interestRateMode, address onBehalfOf) external returns (uint256);
    }
}

/// Number of seconds in a standard year.
const SECONDS_PER_YEAR: f64 = 31_536_000.0;

/// Aave V3 protocol adapter.
///
/// Reads supply APYs from the Pool and encodes real Aave Pool calldata for
/// supply, withdraw, borrow and repay operations.
#[derive(Debug, Clone)]
pub struct AaveAdapter {
    rpc_url: String,
    pool_address: Address,
    /// Address used as `onBehalfOf` for supply/borrow/repay. Defaults to the
    /// zero address if not provided (use the caller in production).
    on_behalf_of: Address,
}

impl AaveAdapter {
    /// Create an adapter targeting the given Aave V3 Pool.
    pub fn new(
        rpc_url: impl Into<String>,
        pool_address: impl Into<String>,
        on_behalf_of: Option<impl Into<String>>,
    ) -> Result<Self, ProtocolError> {
        let pool_address = pool_address
            .into()
            .parse()
            .map_err(|e| ProtocolError::OperationFailed(format!("invalid pool address: {e}")))?;
        let on_behalf_of = on_behalf_of
            .map(|a| a.into().parse())
            .transpose()
            .map_err(|e| {
                ProtocolError::OperationFailed(format!("invalid onBehalfOf address: {e}"))
            })?
            .unwrap_or(Address::ZERO);
        Ok(Self {
            rpc_url: rpc_url.into(),
            pool_address,
            on_behalf_of,
        })
    }

    /// Adapter pointing at the official Aave V3 Sepolia pool.
    pub fn sepolia(rpc_url: impl Into<String>) -> Result<Self, ProtocolError> {
        Self::new(
            rpc_url,
            "0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951",
            None::<String>,
        )
    }

    /// Adapter pointing at the official Aave V3 mainnet pool.
    pub fn mainnet(rpc_url: impl Into<String>) -> Result<Self, ProtocolError> {
        Self::new(
            rpc_url,
            "0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2",
            None::<String>,
        )
    }

    fn asset_address(asset: &Asset, network: Network) -> Result<Address, ProtocolError> {
        address_for(asset, network)
    }

    fn network(&self) -> Network {
        // A simple heuristic: Sepolia pool address is well-known.
        if self.pool_address.to_string().to_lowercase()
            == "0x6ae43d3271ff6888e7fc43fd7321a503ff738951"
        {
            Network::Sepolia
        } else {
            Network::Mainnet
        }
    }
}

impl LendingProtocol for AaveAdapter {
    fn get_apy(&self, asset: &Asset) -> Result<f64, ProtocolError> {
        with_retry(
            || self.get_apy_once(asset),
            3,
            Duration::from_millis(500),
            Duration::from_secs(5),
            retry_any,
        )
    }

    fn supply(&self, asset: &Asset, amount: u128) -> Result<Transaction, ProtocolError> {
        if amount == 0 {
            return Err(ProtocolError::InvalidAmount(
                "amount must be > 0".to_string(),
            ));
        }
        let asset_address = Self::asset_address(asset, self.network())?;
        let call = IPool::supplyCall {
            asset: asset_address,
            amount: U256::from(amount),
            onBehalfOf: self.on_behalf_of,
            referralCode: 0,
        };
        Ok(
            Transaction::new(self.pool_address.to_string(), 0, 300_000)
                .with_data(call.abi_encode()),
        )
    }

    fn withdraw(&self, asset: &Asset, amount: u128) -> Result<Transaction, ProtocolError> {
        if amount == 0 {
            return Err(ProtocolError::InvalidAmount(
                "amount must be > 0".to_string(),
            ));
        }
        let asset_address = Self::asset_address(asset, self.network())?;
        let call = IPool::withdrawCall {
            asset: asset_address,
            amount: U256::from(amount),
            to: self.on_behalf_of,
        };
        Ok(
            Transaction::new(self.pool_address.to_string(), 0, 300_000)
                .with_data(call.abi_encode()),
        )
    }

    fn borrow(
        &self,
        asset: &Asset,
        amount: u128,
        collateral: &Asset,
        collateral_amount: u128,
    ) -> Result<Transaction, ProtocolError> {
        let _ = collateral;
        let _ = collateral_amount;
        if amount == 0 {
            return Err(ProtocolError::InvalidAmount(
                "borrow amount must be > 0".to_string(),
            ));
        }
        let asset_address = Self::asset_address(asset, self.network())?;
        let call = IPool::borrowCall {
            asset: asset_address,
            amount: U256::from(amount),
            interestRateMode: U256::from(2), // variable rate
            referralCode: 0,
            onBehalfOf: self.on_behalf_of,
        };
        Ok(
            Transaction::new(self.pool_address.to_string(), 0, 350_000)
                .with_data(call.abi_encode()),
        )
    }

    fn repay(&self, asset: &Asset, amount: u128) -> Result<Transaction, ProtocolError> {
        if amount == 0 {
            return Err(ProtocolError::InvalidAmount(
                "amount must be > 0".to_string(),
            ));
        }
        let asset_address = Self::asset_address(asset, self.network())?;
        let call = IPool::repayCall {
            asset: asset_address,
            amount: U256::from(amount),
            interestRateMode: U256::from(2), // variable rate
            onBehalfOf: self.on_behalf_of,
        };
        Ok(
            Transaction::new(self.pool_address.to_string(), 0, 300_000)
                .with_data(call.abi_encode()),
        )
    }
}

impl AaveAdapter {
    fn get_apy_once(&self, asset: &Asset) -> Result<f64, ProtocolError> {
        let asset_address = Self::asset_address(asset, self.network())?;

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| ProtocolError::OperationFailed(format!("tokio runtime: {e}")))?;
        let rate = rt.block_on(async {
            let url = self
                .rpc_url
                .parse()
                .map_err(|e| ProtocolError::OperationFailed(format!("invalid rpc url: {e}")))?;
            let provider = ProviderBuilder::new().on_http(url);
            let pool = IPool::new(self.pool_address, provider);
            let result = tokio::time::timeout(
                Duration::from_secs(10),
                pool.getReserveData(asset_address).call(),
            )
            .await
            .map_err(|_| ProtocolError::OperationFailed("getReserveData timed out".to_string()))?
            .map_err(|e| ProtocolError::OperationFailed(format!("getReserveData failed: {e}")))?;
            Ok::<_, ProtocolError>(result.currentLiquidityRate)
        })?;

        let ray = 1e27;
        let rate_f = rate as f64 / ray;
        let apy = (1.0 + rate_f).powf(SECONDS_PER_YEAR) - 1.0;
        Ok(apy * 100.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Sepolia,
}

/// Resolve a domain asset to its ERC20 address on a given network.
pub fn address_for(asset: &Asset, network: Network) -> Result<Address, ProtocolError> {
    let addr = match (network, asset) {
        (Network::Mainnet, Asset::Eth) => "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
        (Network::Mainnet, Asset::Usdc) => "0xA0b86a33E6441c3be33C6d8eFA5A0c2e55d2fE52", // USDC
        (Network::Mainnet, Asset::Dai) => "0x6B175474E89094C44Da98b954EedeAC495271d0F",
        (Network::Mainnet, Asset::Wbtc) => "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599",
        (Network::Mainnet, Asset::Link) => "0x514910771AF9Ca656af840dff83E8264EcF986CA",
        (Network::Sepolia, Asset::Eth) => "0xC558DBdd856501FCd9aaF1E62eae57A9F0629a3c", // WETH
        (Network::Sepolia, Asset::Usdc) => "0x94a9D9AC8a22534E3FaCa9F4e7F2E2cf85d5E4C8",
        (Network::Sepolia, Asset::Dai) => "0x68194a729C2450ad26072b3D33ADaCbcef39D574",
        (Network::Sepolia, Asset::Wbtc) => "0x29f2D40B060420436f7d5897A08C2fB77b32833F",
        (Network::Sepolia, Asset::Link) => "0x779877A7B0D9E8603169DdbD7836e478b4624789",
        (_, Asset::Sol) => {
            return Err(ProtocolError::UnsupportedAsset(format!("{:?}", asset)));
        }
    };
    addr.parse().map_err(|e| {
        ProtocolError::OperationFailed(format!("invalid token address for {asset:?}: {e}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supply_encodes_valid_calldata() {
        let aave = AaveAdapter::sepolia("http://localhost:8545").unwrap();
        let tx = aave.supply(&Asset::Usdc, 1_000_000).unwrap();
        assert_eq!(
            tx.to.to_lowercase(),
            aave.pool_address.to_string().to_lowercase()
        );
        assert!(!tx.data.is_empty());
        assert_eq!(&tx.data[0..4], &alloy::primitives::hex!("617ba037")); // supply selector
    }

    #[test]
    fn borrow_encodes_variable_rate_calldata() {
        let aave = AaveAdapter::sepolia("http://localhost:8545").unwrap();
        let tx = aave
            .borrow(
                &Asset::Usdc,
                1_000_000,
                &Asset::Eth,
                2_000_000_000_000_000_000,
            )
            .unwrap();
        assert!(!tx.data.is_empty());
        assert_eq!(&tx.data[0..4], IPool::borrowCall::SELECTOR);
    }

    #[test]
    fn get_apy_rejects_unsupported_asset() {
        let aave = AaveAdapter::sepolia("http://localhost:8545").unwrap();
        assert!(aave.get_apy(&Asset::Sol).is_err());
    }
}
