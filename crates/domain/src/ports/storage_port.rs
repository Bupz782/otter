use crate::models::intent::ConditionalIntent;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A persisted intent record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentRecord {
    pub id: String,
    /// Original natural-language text.
    pub text: String,
    /// Parsed conditional intent, serialized as JSON.
    pub intent: ConditionalIntent,
    /// High-level state: `active`, `executed`, `cancelled`, `error`.
    pub state: String,
    /// Unix timestamp (seconds) when the record was created.
    pub created_at: i64,
    /// Unix timestamp (seconds) of the last update.
    pub updated_at: i64,
    /// Authenticated user address that created the intent, if auth is enabled.
    pub user_address: Option<String>,
}

/// A persisted delegation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRecord {
    pub hash: String,
    /// Serialized JSON of the delegation message.
    pub payload_json: String,
    /// Hex-encoded 64-byte signature.
    pub signature: String,
    /// Unix timestamp (seconds) when the record was created.
    pub created_at: i64,
    /// Authenticated user address that created the delegation, if auth is enabled.
    pub user_address: Option<String>,
}

/// A persisted execution / transaction record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: String,
    pub intent_id: String,
    /// Transaction hash or receipt identifier.
    pub tx_hash: String,
    /// Final status: `success`, `failed`, `error`.
    pub status: String,
    /// Gas used by the transaction, if known.
    pub gas_used: u64,
    /// Unix timestamp (seconds) when the record was created.
    pub created_at: i64,
}

/// A persisted strategy template record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRecord {
    pub id: String,
    pub title: String,
    pub description: String,
    pub raw_text: String,
    pub intent_json: String,
    pub creator_address: Option<String>,
    pub agent_id: String,
    pub risk_profile: String,
    pub copies: u64,
    pub total_volume: u64,
    pub apy: f64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Errors returned by storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage init failed: {0}")]
    InitFailed(String),
    #[error("save failed: {0}")]
    SaveFailed(String),
    #[error("read failed: {0}")]
    ReadFailed(String),
    #[error("delete failed: {0}")]
    DeleteFailed(String),
    #[error("record not found: {0}")]
    NotFound(String),
}

/// Port for persisting intents and execution history.
#[async_trait]
pub trait StoragePort: Send + Sync {
    /// Persist a new or updated intent record.
    async fn save_intent(&self, record: &IntentRecord) -> Result<(), StorageError>;

    /// Return all stored intent records, most recent first.
    async fn list_intents(&self) -> Result<Vec<IntentRecord>, StorageError>;

    /// Return a single record by id, if it exists.
    async fn get_intent(&self, id: &str) -> Result<Option<IntentRecord>, StorageError>;

    /// Remove a record by id.
    async fn delete_intent(&self, id: &str) -> Result<(), StorageError>;

    /// Check that the storage backend is reachable.
    async fn health_check(&self) -> Result<(), StorageError>;

    /// Persist a delegation record.
    async fn save_delegation(&self, record: &DelegationRecord) -> Result<(), StorageError>;

    /// Return all stored delegation records, most recent first.
    async fn list_delegations(&self) -> Result<Vec<DelegationRecord>, StorageError>;

    /// Return a single delegation by hash, if it exists.
    async fn get_delegation(&self, hash: &str) -> Result<Option<DelegationRecord>, StorageError>;

    /// Persist an execution / transaction record.
    async fn save_execution(&self, record: &ExecutionRecord) -> Result<(), StorageError>;

    /// Return all execution records, most recent first.
    async fn list_executions(&self) -> Result<Vec<ExecutionRecord>, StorageError>;

    /// Return execution records for a given intent.
    async fn get_executions_for_intent(
        &self,
        intent_id: &str,
    ) -> Result<Vec<ExecutionRecord>, StorageError>;

    /// Persist a strategy template record.
    async fn save_strategy(&self, record: &StrategyRecord) -> Result<(), StorageError>;

    /// Return all stored strategy records, most recently updated first.
    async fn list_strategies(&self) -> Result<Vec<StrategyRecord>, StorageError>;

    /// Return a single strategy by id, if it exists.
    async fn get_strategy(&self, id: &str) -> Result<Option<StrategyRecord>, StorageError>;

    /// Increment the copy count for a strategy and refresh its updated_at timestamp.
    async fn increment_strategy_copies(&self, id: &str) -> Result<(), StorageError>;
}
