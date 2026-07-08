use domain::models::condition::Metric;
use domain::models::intent::Asset;
use domain::ports::price_oracle_port::{OracleError, PriceOraclePort};
use std::collections::HashMap;

/// In-memory oracle for development and testing.
///
/// Configure values with `set` before evaluating conditions. Production
/// implementations will fetch from Chainlink or protocol contracts.
#[derive(Clone)]
pub struct MockOracleAdapter {
    values: HashMap<(Metric, Option<Asset>), u128>,
}

impl MockOracleAdapter {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn set(&mut self, metric: Metric, asset: Option<Asset>, value: u128) -> &mut Self {
        self.values.insert((metric, asset), value);
        self
    }

    pub fn with_default(metric: Metric, value: u128) -> Self {
        let mut adapter = Self::new();
        adapter.set(metric, None, value);
        adapter
    }
}

impl Default for MockOracleAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PriceOraclePort for MockOracleAdapter {
    fn fetch(&self, metric: &Metric, asset: Option<&Asset>) -> Result<u128, OracleError> {
        let key = (*metric, asset.cloned());
        self.values
            .get(&key)
            .copied()
            .or_else(|| self.values.get(&(*metric, None)).copied())
            .ok_or_else(|| OracleError::FetchFailed(format!("No value set for {:?}", metric)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::condition::Metric;

    #[test]
    fn returns_configured_value() {
        let mut oracle = MockOracleAdapter::new();
        oracle.set(Metric::Yield, None, 42);
        assert_eq!(oracle.fetch(&Metric::Yield, None).unwrap(), 42);
    }

    #[test]
    fn falls_back_to_asset_agnostic_value() {
        let oracle = MockOracleAdapter::with_default(Metric::Price, 2000_000000);
        assert_eq!(
            oracle.fetch(&Metric::Price, Some(&Asset::Eth)).unwrap(),
            2000_000000
        );
    }
}
