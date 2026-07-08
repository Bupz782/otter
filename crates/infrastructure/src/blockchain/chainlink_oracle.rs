use alloy::primitives::{Address, I256};
use alloy::providers::ProviderBuilder;
use alloy::sol;
use domain::models::condition::Metric;
use domain::models::intent::Asset;
use domain::ports::price_oracle_port::{OracleError, PriceOraclePort};
use std::time::Duration;

use super::retry::{retry_any, with_retry};

sol! {
    #[sol(rpc)]
    interface AggregatorV3Interface {
        function latestRoundData()
            external
            view
            returns (
                uint80 roundId,
                int256 answer,
                uint256 startedAt,
                uint256 updatedAt,
                uint80 answeredInRound
            );
    }
}

/// On-chain price oracle backed by Chainlink Data Feeds.
///
/// Prices are returned as USD with 6 decimals (e.g. 2_000_000000 for $2,000),
/// matching the canonical unit expected by [`PriceOraclePort`].
#[derive(Debug, Clone)]
pub struct ChainlinkPriceOracle {
    rpc_url: String,
    network: Network,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Sepolia,
}

impl ChainlinkPriceOracle {
    /// Create a new Chainlink price oracle.
    pub fn new(rpc_url: impl Into<String>, network: Network) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            network,
        }
    }

    /// Oracle pointing at Ethereum mainnet feeds.
    pub fn mainnet(rpc_url: impl Into<String>) -> Self {
        Self::new(rpc_url, Network::Mainnet)
    }

    /// Oracle pointing at Sepolia testnet feeds.
    pub fn sepolia(rpc_url: impl Into<String>) -> Self {
        Self::new(rpc_url, Network::Sepolia)
    }

    fn feed_address(&self, asset: &Asset) -> Result<Address, OracleError> {
        let addr = match (self.network, asset) {
            (Network::Mainnet, Asset::Eth) => "0x5f4eC3Df9cbd43714FE2740f5E3616155c5b8419",
            (Network::Mainnet, Asset::Usdc) => "0x8fFfFfd4AfB6115b954Bd326cbe7B4BA576818f6",
            (Network::Mainnet, Asset::Dai) => "0xAed0c38402a5d19df6E4c03F4E2DceD6e29c1e96",
            (Network::Mainnet, Asset::Wbtc) => "0xF4030086522a5bEEa4988F8cA5B36dbC97BeE88c",
            (Network::Mainnet, Asset::Link) => "0x2c1d072e956AFFC0D435Cb7AC38EF18d24d9127c",
            (Network::Sepolia, Asset::Eth) => "0x694AA1769357215DE4FAC081bf1f309aDC325306",
            (Network::Sepolia, Asset::Usdc) => "0xA2F78ab2355fe2f984D808B5CeE7FD0A93D5270E",
            (Network::Sepolia, Asset::Dai) => "0x14866185B1962B63C3Ea9E03Bc1da838bab34C19",
            (Network::Sepolia, Asset::Wbtc) => "0x1b44F3514812d835EB1BDB0acB33d3fA3351Ee43",
            (Network::Sepolia, Asset::Link) => "0xc59E3633BAAC79493d908e63626716e204A45EdF",
            (_, Asset::Sol) => {
                return Err(OracleError::UnsupportedMetric(format!(
                    "no Chainlink feed for {:?}",
                    asset
                )));
            }
        };
        addr.parse()
            .map_err(|e| OracleError::FetchFailed(format!("invalid feed address: {e}")))
    }
}

impl PriceOraclePort for ChainlinkPriceOracle {
    fn fetch(&self, metric: &Metric, asset: Option<&Asset>) -> Result<u128, OracleError> {
        if *metric != Metric::Price {
            return Err(OracleError::UnsupportedMetric(format!(
                "ChainlinkPriceOracle only supports Price, got {:?}",
                metric
            )));
        }
        let asset = asset
            .ok_or_else(|| OracleError::AssetRequired("price requires an asset".to_string()))?;
        let feed_address = self.feed_address(asset)?;

        with_retry(
            || self.fetch_once(feed_address),
            3,
            Duration::from_millis(500),
            Duration::from_secs(5),
            retry_any,
        )
    }
}

impl ChainlinkPriceOracle {
    fn fetch_once(&self, feed_address: Address) -> Result<u128, OracleError> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| OracleError::FetchFailed(format!("tokio runtime: {e}")))?;
        let answer = rt.block_on(async {
            let url = self
                .rpc_url
                .parse()
                .map_err(|e| OracleError::FetchFailed(format!("invalid rpc url: {e}")))?;
            let provider = ProviderBuilder::new().on_http(url);
            let feed = AggregatorV3Interface::new(feed_address, provider);
            let result =
                tokio::time::timeout(Duration::from_secs(10), feed.latestRoundData().call())
                    .await
                    .map_err(|_| OracleError::FetchFailed("latestRoundData timed out".to_string()))?
                    .map_err(|e| {
                        OracleError::FetchFailed(format!("latestRoundData failed: {e}"))
                    })?;
            Ok::<_, OracleError>(result.answer)
        })?;

        if answer < I256::ZERO {
            return Err(OracleError::FetchFailed(
                "negative price from Chainlink".to_string(),
            ));
        }

        // Chainlink USD feeds use 8 decimals; normalize to 6 decimals.
        let raw: u128 = answer
            .try_into()
            .map_err(|e| OracleError::FetchFailed(format!("price overflow: {e}")))?;
        let normalized = raw
            .checked_div(100)
            .ok_or_else(|| OracleError::FetchFailed("price normalization failed".to_string()))?;
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_yield_metric() {
        let oracle = ChainlinkPriceOracle::sepolia("http://localhost:8545");
        let result = oracle.fetch(&Metric::Yield, Some(&Asset::Eth));
        assert!(matches!(result, Err(OracleError::UnsupportedMetric(_))));
    }

    #[test]
    fn rejects_missing_asset() {
        let oracle = ChainlinkPriceOracle::sepolia("http://localhost:8545");
        let result = oracle.fetch(&Metric::Price, None);
        assert!(matches!(result, Err(OracleError::AssetRequired(_))));
    }
}
