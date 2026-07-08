use async_trait::async_trait;
use domain::models::intent::ConditionalIntent;
use domain::ports::storage_port::{
    DelegationRecord, ExecutionRecord, IntentRecord, StorageError, StoragePort,
};
use rusqlite::{Connection, OptionalExtension};
use std::sync::{Arc, Mutex};

#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

/// SQLite-backed implementation of [`StoragePort`].
///
/// The connection is protected by a mutex so the adapter can be shared safely
/// across async tasks. Blocking SQLite operations are executed on a dedicated
/// thread pool so the async runtime is not starved.
pub struct SqliteStorage {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStorage {
    /// Open (or create) a SQLite database at the given path and initialize the
    /// schema.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let conn = Connection::open(path).map_err(|e| StorageError::InitFailed(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS intents (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                intent_json TEXT NOT NULL,
                state TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_intents_updated_at ON intents(updated_at DESC)",
            [],
        )
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS delegations (
                hash TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL,
                signature TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_delegations_created_at ON delegations(created_at DESC)",
            [],
        )
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS executions (
                id TEXT PRIMARY KEY,
                intent_id TEXT NOT NULL,
                tx_hash TEXT NOT NULL,
                status TEXT NOT NULL,
                gas_used INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_executions_intent_id ON executions(intent_id)",
            [],
        )
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create an in-memory SQLite storage, useful for tests.
    #[cfg(test)]
    pub fn in_memory() -> Result<Self, StorageError> {
        let conn =
            Connection::open_in_memory().map_err(|e| StorageError::InitFailed(e.to_string()))?;
        conn.execute(
            "CREATE TABLE intents (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                intent_json TEXT NOT NULL,
                state TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
        conn.execute(
            "CREATE TABLE delegations (
                hash TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL,
                signature TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
        conn.execute(
            "CREATE TABLE executions (
                id TEXT PRIMARY KEY,
                intent_id TEXT NOT NULL,
                tx_hash TEXT NOT NULL,
                status TEXT NOT NULL,
                gas_used INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl StoragePort for SqliteStorage {
    async fn save_intent(&self, record: &IntentRecord) -> Result<(), StorageError> {
        let conn = self.conn.clone();
        let record = record.clone();
        tokio::task::spawn_blocking(move || {
            let intent_json = serde_json::to_string(&record.intent)
                .map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            let conn = conn.lock().map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO intents (id, text, intent_json, state, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                [
                    &record.id,
                    &record.text,
                    &intent_json,
                    &record.state,
                    &record.created_at.to_string(),
                    &record.updated_at.to_string(),
                ],
            )
            .map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            Ok(())
        })
        .await
        .map_err(|e| StorageError::SaveFailed(e.to_string()))?
    }

    async fn list_intents(&self) -> Result<Vec<IntentRecord>, StorageError> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, text, intent_json, state, created_at, updated_at
                     FROM intents
                     ORDER BY updated_at DESC",
                )
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    let id: String = row.get(0)?;
                    let text: String = row.get(1)?;
                    let intent_json: String = row.get(2)?;
                    let state: String = row.get(3)?;
                    let created_at: i64 = row.get(4)?;
                    let updated_at: i64 = row.get(5)?;
                    let intent: ConditionalIntent =
                        serde_json::from_str(&intent_json).map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                0,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?;
                    Ok(IntentRecord {
                        id,
                        text,
                        intent,
                        state,
                        created_at,
                        updated_at,
                    })
                })
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }

    async fn get_intent(&self, id: &str) -> Result<Option<IntentRecord>, StorageError> {
        let conn = self.conn.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let row = conn
                .query_row(
                    "SELECT id, text, intent_json, state, created_at, updated_at
                     FROM intents WHERE id = ?1",
                    [&id],
                    |row| {
                        let intent_json: String = row.get(2)?;
                        let intent: ConditionalIntent = serde_json::from_str(&intent_json)
                            .map_err(|e| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    0,
                                    rusqlite::types::Type::Text,
                                    Box::new(e),
                                )
                            })?;
                        Ok(IntentRecord {
                            id: row.get(0)?,
                            text: row.get(1)?,
                            intent,
                            state: row.get(3)?,
                            created_at: row.get(4)?,
                            updated_at: row.get(5)?,
                        })
                    },
                )
                .optional()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            Ok(row)
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }

    async fn delete_intent(&self, id: &str) -> Result<(), StorageError> {
        let conn = self.conn.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::DeleteFailed(e.to_string()))?;
            let affected = conn
                .execute("DELETE FROM intents WHERE id = ?1", [&id])
                .map_err(|e| StorageError::DeleteFailed(e.to_string()))?;
            if affected == 0 {
                return Err(StorageError::NotFound(id));
            }
            Ok(())
        })
        .await
        .map_err(|e| StorageError::DeleteFailed(e.to_string()))?
    }

    async fn health_check(&self) -> Result<(), StorageError> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            conn.query_row("SELECT 1", [], |_row| Ok(()))
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            Ok(())
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }

    async fn save_delegation(&self, record: &DelegationRecord) -> Result<(), StorageError> {
        let conn = self.conn.clone();
        let record = record.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO delegations (hash, payload_json, signature, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
                [
                    &record.hash,
                    &record.payload_json,
                    &record.signature,
                    &record.created_at.to_string(),
                ],
            )
            .map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            Ok(())
        })
        .await
        .map_err(|e| StorageError::SaveFailed(e.to_string()))?
    }

    async fn list_delegations(&self) -> Result<Vec<DelegationRecord>, StorageError> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let mut stmt = conn
                .prepare(
                    "SELECT hash, payload_json, signature, created_at
                     FROM delegations
                     ORDER BY created_at DESC",
                )
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(DelegationRecord {
                        hash: row.get(0)?,
                        payload_json: row.get(1)?,
                        signature: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                })
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }

    async fn get_delegation(&self, hash: &str) -> Result<Option<DelegationRecord>, StorageError> {
        let conn = self.conn.clone();
        let hash = hash.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let row = conn
                .query_row(
                    "SELECT hash, payload_json, signature, created_at
                     FROM delegations WHERE hash = ?1",
                    [&hash],
                    |row| {
                        Ok(DelegationRecord {
                            hash: row.get(0)?,
                            payload_json: row.get(1)?,
                            signature: row.get(2)?,
                            created_at: row.get(3)?,
                        })
                    },
                )
                .optional()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            Ok(row)
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }

    async fn save_execution(&self, record: &ExecutionRecord) -> Result<(), StorageError> {
        let conn = self.conn.clone();
        let record = record.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO executions (id, intent_id, tx_hash, status, gas_used, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                [
                    &record.id,
                    &record.intent_id,
                    &record.tx_hash,
                    &record.status,
                    &record.gas_used.to_string(),
                    &record.created_at.to_string(),
                ],
            )
            .map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            Ok(())
        })
        .await
        .map_err(|e| StorageError::SaveFailed(e.to_string()))?
    }

    async fn list_executions(&self) -> Result<Vec<ExecutionRecord>, StorageError> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, intent_id, tx_hash, status, gas_used, created_at
                     FROM executions
                     ORDER BY created_at DESC",
                )
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(ExecutionRecord {
                        id: row.get(0)?,
                        intent_id: row.get(1)?,
                        tx_hash: row.get(2)?,
                        status: row.get(3)?,
                        gas_used: row.get::<_, i64>(4)? as u64,
                        created_at: row.get(5)?,
                    })
                })
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }

    async fn get_executions_for_intent(
        &self,
        intent_id: &str,
    ) -> Result<Vec<ExecutionRecord>, StorageError> {
        let conn = self.conn.clone();
        let intent_id = intent_id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, intent_id, tx_hash, status, gas_used, created_at
                     FROM executions
                     WHERE intent_id = ?1
                     ORDER BY created_at DESC",
                )
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let rows = stmt
                .query_map([&intent_id], |row| {
                    Ok(ExecutionRecord {
                        id: row.get(0)?,
                        intent_id: row.get(1)?,
                        tx_hash: row.get(2)?,
                        status: row.get(3)?,
                        gas_used: row.get::<_, i64>(4)? as u64,
                        created_at: row.get(5)?,
                    })
                })
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }
}

#[cfg(test)]
fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::condition::{Comparator, Condition, Metric};
    use domain::models::intent::{Asset, DexType, Intent};

    fn sample_record(id: &str) -> IntentRecord {
        IntentRecord {
            id: id.to_string(),
            text: "swap 1 ETH for USDC on Uniswap".to_string(),
            intent: ConditionalIntent {
                intent: Intent::Swap {
                    from_asset: Asset::Eth,
                    to_asset: Asset::Usdc,
                    amount: 1_000_000_000_000_000_000,
                    protocol: DexType::Uniswap,
                },
                condition: Some(Condition::Comparison {
                    metric: Metric::Price,
                    comparator: Comparator::GreaterThan,
                    value: 2_000,
                }),
            },
            state: "active".to_string(),
            created_at: now_secs(),
            updated_at: now_secs(),
        }
    }

    #[tokio::test]
    async fn save_and_list_intent() {
        let storage = SqliteStorage::in_memory().unwrap();
        let record = sample_record("intent-1");
        storage.save_intent(&record).await.unwrap();

        let intents = storage.list_intents().await.unwrap();
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].id, "intent-1");
        assert!(matches!(intents[0].intent.intent, Intent::Swap { .. }));
    }

    #[tokio::test]
    async fn get_intent_by_id() {
        let storage = SqliteStorage::in_memory().unwrap();
        storage.save_intent(&sample_record("a")).await.unwrap();

        let found = storage.get_intent("a").await.unwrap().unwrap();
        assert_eq!(found.id, "a");
        assert!(storage.get_intent("missing").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn delete_intent() {
        let storage = SqliteStorage::in_memory().unwrap();
        storage.save_intent(&sample_record("x")).await.unwrap();
        storage.delete_intent("x").await.unwrap();
        assert!(storage.get_intent("x").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn health_check_succeeds() {
        let storage = SqliteStorage::in_memory().unwrap();
        storage.health_check().await.unwrap();
    }
}
