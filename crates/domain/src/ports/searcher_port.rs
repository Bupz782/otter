use crate::ports::evm_port::EvmError;

/// Errors returned by a bundle searcher.
#[derive(Debug, thiserror::Error)]
pub enum SearcherError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("submission failed: {0}")]
    SubmissionFailed(String),
}

impl From<SearcherError> for EvmError {
    fn from(e: SearcherError) -> Self {
        match e {
            SearcherError::InvalidInput(msg) => EvmError::InvalidInput(msg),
            SearcherError::SubmissionFailed(msg) => EvmError::SubmissionFailed(msg),
        }
    }
}

/// A transaction bundle ready to be sent to a private block builder.
#[derive(Debug, Clone)]
pub struct Bundle {
    /// Signed raw transactions (RLP-encoded).
    pub txs: Vec<Vec<u8>>,
    /// Target block number (optional; omit for "next block").
    pub block_number: Option<u64>,
    /// Minimum timestamp for bundle validity.
    pub min_timestamp: Option<u64>,
    /// Maximum timestamp for bundle validity.
    pub max_timestamp: Option<u64>,
}

/// Port for submitting MEV bundles to private relays.
#[async_trait::async_trait]
pub trait BundleSearcherPort: Send + Sync {
    /// Submit a bundle to the relay. Returns the bundle hash.
    async fn submit_bundle(&self, bundle: Bundle) -> Result<String, SearcherError>;
}
