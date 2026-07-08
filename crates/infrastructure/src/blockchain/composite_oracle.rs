use domain::LendingProtocol;
use domain::models::condition::Metric;
use domain::models::intent::Asset;
use domain::ports::price_oracle_port::{OracleError, PriceOraclePort};

use super::chainlink_oracle::{ChainlinkPriceOracle, Network as OracleNetwork};
use crate::protocols::AaveAdapter;
use crate::protocols::aave::DUMMY_ON_BEHALF_OF;

/// Composite oracle that dispatches market-data queries to the right on-chain
/// source.
///
/// - `Metric::Price` → Chainlink price feeds (USD, 6 decimals).
/// - `Metric::Yield` → Aave V3 supply APY (integer percent).
#[derive(Debug, Clone)]
pub struct CompositeOracle {
    price_oracle: ChainlinkPriceOracle,
    yield_oracle: AaveAdapter,
}

impl CompositeOracle {
    /// Create a composite oracle using the same RPC URL for both Chainlink and
    /// Aave. `network` selects the known mainnet/Sepolia contract addresses.
    pub fn new(rpc_url: impl Into<String>, network: OracleNetwork) -> Result<Self, OracleError> {
        let rpc_url = rpc_url.into();
        let price_oracle = ChainlinkPriceOracle::new(rpc_url.clone(), network);
        let yield_oracle = match network {
            OracleNetwork::Mainnet => AaveAdapter::mainnet(&rpc_url, DUMMY_ON_BEHALF_OF)
                .map_err(|e| OracleError::FetchFailed(e.to_string()))?,
            OracleNetwork::Sepolia => AaveAdapter::sepolia(&rpc_url, DUMMY_ON_BEHALF_OF)
                .map_err(|e| OracleError::FetchFailed(e.to_string()))?,
        };
        Ok(Self {
            price_oracle,
            yield_oracle,
        })
    }

    /// Oracle configured for Ethereum mainnet.
    pub fn mainnet(rpc_url: impl Into<String>) -> Result<Self, OracleError> {
        Self::new(rpc_url, OracleNetwork::Mainnet)
    }

    /// Oracle configured for Sepolia testnet.
    pub fn sepolia(rpc_url: impl Into<String>) -> Result<Self, OracleError> {
        Self::new(rpc_url, OracleNetwork::Sepolia)
    }
}

impl PriceOraclePort for CompositeOracle {
    fn fetch(&self, metric: &Metric, asset: Option<&Asset>) -> Result<u128, OracleError> {
        match metric {
            Metric::Price => self.price_oracle.fetch(metric, asset),
            Metric::Yield => {
                let asset = asset.ok_or_else(|| {
                    OracleError::AssetRequired("yield requires an asset".to_string())
                })?;
                let apy = self
                    .yield_oracle
                    .get_apy(asset)
                    .map_err(|e| OracleError::FetchFailed(e.to_string()))?;
                Ok(apy.trunc() as u128)
            }
            Metric::GasCost | Metric::Volume => Err(OracleError::UnsupportedMetric(format!(
                "CompositeOracle does not support {:?}",
                metric
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsupported_metrics() {
        let oracle = CompositeOracle::sepolia("http://localhost:8545").unwrap();
        assert!(oracle.fetch(&Metric::GasCost, Some(&Asset::Eth)).is_err());
        assert!(oracle.fetch(&Metric::Volume, Some(&Asset::Eth)).is_err());
    }

    #[test]
    fn yield_requires_asset() {
        let oracle = CompositeOracle::sepolia("http://localhost:8545").unwrap();
        assert!(matches!(
            oracle.fetch(&Metric::Yield, None),
            Err(OracleError::AssetRequired(_))
        ));
    }

    #[test]
    fn price_requires_asset() {
        let oracle = CompositeOracle::sepolia("http://localhost:8545").unwrap();
        assert!(matches!(
            oracle.fetch(&Metric::Price, None),
            Err(OracleError::AssetRequired(_))
        ));
    }

    #[test]
    fn price_rejects_unsupported_asset() {
        let oracle = CompositeOracle::sepolia("http://localhost:8545").unwrap();
        assert!(matches!(
            oracle.fetch(&Metric::Price, Some(&Asset::Sol)),
            Err(OracleError::UnsupportedMetric(_))
        ));
    }
}
