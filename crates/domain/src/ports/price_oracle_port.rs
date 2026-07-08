use crate::models::condition::Metric;
use crate::models::intent::Asset;

/// Port for fetching normalized market metrics used to evaluate conditions.
///
/// Values are returned as `u128` in the metric's canonical unit:
/// - Yield: integer percent (e.g. 3 for 3%)
/// - Price: USD with 6 decimals (e.g. 2000_000000 for $2,000)
/// - GasCost: gwei (integer)
/// - Volume: token units with the asset's decimals
pub trait PriceOraclePort {
    fn fetch(&self, metric: &Metric, asset: Option<&Asset>) -> Result<u128, OracleError>;
}

impl<T: PriceOraclePort + ?Sized> PriceOraclePort for &T {
    fn fetch(&self, metric: &Metric, asset: Option<&Asset>) -> Result<u128, OracleError> {
        (*self).fetch(metric, asset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OracleError {
    UnsupportedMetric(String),
    AssetRequired(String),
    FetchFailed(String),
}

impl std::fmt::Display for OracleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedMetric(m) => write!(f, "unsupported metric: {}", m),
            Self::AssetRequired(m) => write!(f, "metric {} requires an asset", m),
            Self::FetchFailed(msg) => write!(f, "fetch failed: {}", msg),
        }
    }
}

impl std::error::Error for OracleError {}
