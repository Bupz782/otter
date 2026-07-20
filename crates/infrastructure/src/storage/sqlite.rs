use async_trait::async_trait;
use domain::models::intent::ConditionalIntent;
use domain::ports::storage_port::{
    DelegationRecord, ExecutionRecord, IntentRecord, StorageError, StoragePort, StrategyRecord,
};
use rusqlite::{Connection, OptionalExtension};

use crate::storage::migrations;
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
    /// schema by running pending migrations from `crates/infrastructure/migrations`.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        let conn = Connection::open(path).map_err(|e| StorageError::InitFailed(e.to_string()))?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Create an in-memory SQLite storage, useful for tests.
    #[cfg(test)]
    pub fn in_memory() -> Result<Self, StorageError> {
        let conn =
            Connection::open_in_memory().map_err(|e| StorageError::InitFailed(e.to_string()))?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

/// Run every `.sql` file in the migrations directory in lexicographic order,
/// skipping migrations already recorded in `schema_migrations`.
///
/// The SQL files contain only DDL/DML. After each migration succeeds the runner
/// records the version in `schema_migrations` with the current Unix timestamp.
fn run_migrations(conn: &Connection) -> Result<(), StorageError> {
    // Ensure the tracking table exists before we query it.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| StorageError::InitFailed(e.to_string()))?;

    for path in migrations::migration_files()? {
        let version = migrations::migration_version(&path)?;
        let applied: bool = conn
            .query_row(
                "SELECT 1 FROM schema_migrations WHERE version = ?1",
                [version],
                |_row| Ok(()),
            )
            .optional()
            .map_err(|e| StorageError::InitFailed(e.to_string()))?
            .is_some();

        if !applied {
            let sql = std::fs::read_to_string(&path).map_err(|e| {
                StorageError::InitFailed(format!(
                    "failed to read migration {}: {}",
                    path.display(),
                    e
                ))
            })?;

            // Version 3 adds the `user_address` column using PostgreSQL's
            // `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` syntax. SQLite does not
            // support `IF NOT EXISTS` on `ADD COLUMN`, so we apply the equivalent
            // idempotent change from Rust for SQLite deployments.
            if version == 3 {
                add_column_if_missing(conn, "intents", "user_address", "TEXT")?;
                add_column_if_missing(conn, "delegations", "user_address", "TEXT")?;
            } else {
                conn.execute_batch(&sql).map_err(|e| {
                    StorageError::InitFailed(format!(
                        "failed to run migration {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }

            conn.execute(
                "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
                [version, migrations::unix_now()],
            )
            .map_err(|e| {
                StorageError::InitFailed(format!("failed to record migration {}: {}", version, e))
            })?;
        }
    }
    Ok(())
}

/// Add a column to a table only if it is not already present.
fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    column_type: &str,
) -> Result<(), StorageError> {
    let mut stmt = conn
        .prepare("SELECT name FROM pragma_table_info(?1) WHERE name = ?2")
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
    let exists = stmt
        .exists(rusqlite::params![table, column])
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
    if !exists {
        conn.execute(
            &format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, column_type),
            [],
        )
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;
    }
    Ok(())
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
                "INSERT OR REPLACE INTO intents (id, text, intent_json, state, created_at, updated_at, user_address)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    &record.id,
                    &record.text,
                    &intent_json,
                    &record.state,
                    &record.created_at.to_string(),
                    &record.updated_at.to_string(),
                    record.user_address.as_deref(),
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
                    "SELECT id, text, intent_json, state, created_at, updated_at, user_address
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
                    let user_address: Option<String> = row.get(6)?;
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
                        user_address,
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
                    "SELECT id, text, intent_json, state, created_at, updated_at, user_address
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
                            user_address: row.get(6)?,
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
                "INSERT OR REPLACE INTO delegations (hash, payload_json, signature, created_at, user_address)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    &record.hash,
                    &record.payload_json,
                    &record.signature,
                    &record.created_at.to_string(),
                    record.user_address.as_deref(),
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
                    "SELECT hash, payload_json, signature, created_at, user_address
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
                        user_address: row.get(4)?,
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
                    "SELECT hash, payload_json, signature, created_at, user_address
                     FROM delegations WHERE hash = ?1",
                    [&hash],
                    |row| {
                        Ok(DelegationRecord {
                            hash: row.get(0)?,
                            payload_json: row.get(1)?,
                            signature: row.get(2)?,
                            created_at: row.get(3)?,
                            user_address: row.get(4)?,
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

    async fn save_strategy(&self, record: &StrategyRecord) -> Result<(), StorageError> {
        let conn = self.conn.clone();
        let record = record.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            conn.execute(
                "INSERT OR REPLACE INTO strategies
                 (id, title, description, raw_text, intent_json, creator_address, agent_id,
                  risk_profile, copies, total_volume, apy, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    &record.id,
                    &record.title,
                    &record.description,
                    &record.raw_text,
                    &record.intent_json,
                    record.creator_address.as_deref(),
                    &record.agent_id,
                    &record.risk_profile,
                    record.copies as i64,
                    record.total_volume as i64,
                    record.apy,
                    record.created_at,
                    record.updated_at,
                ],
            )
            .map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            Ok(())
        })
        .await
        .map_err(|e| StorageError::SaveFailed(e.to_string()))?
    }

    async fn list_strategies(&self) -> Result<Vec<StrategyRecord>, StorageError> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let mut stmt = conn
                .prepare(
                    "SELECT id, title, description, raw_text, intent_json, creator_address, agent_id,
                            risk_profile, copies, total_volume, apy, created_at, updated_at
                     FROM strategies
                     ORDER BY updated_at DESC",
                )
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(StrategyRecord {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: row.get(2)?,
                        raw_text: row.get(3)?,
                        intent_json: row.get(4)?,
                        creator_address: row.get(5)?,
                        agent_id: row.get(6)?,
                        risk_profile: row.get(7)?,
                        copies: row.get::<_, i64>(8)? as u64,
                        total_volume: row.get::<_, i64>(9)? as u64,
                        apy: row.get(10)?,
                        created_at: row.get(11)?,
                        updated_at: row.get(12)?,
                    })
                })
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))
        })
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?
    }

    async fn get_strategy(&self, id: &str) -> Result<Option<StrategyRecord>, StorageError> {
        let conn = self.conn.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
            let row = conn
                .query_row(
                    "SELECT id, title, description, raw_text, intent_json, creator_address, agent_id,
                            risk_profile, copies, total_volume, apy, created_at, updated_at
                     FROM strategies WHERE id = ?1",
                    [&id],
                    |row| {
                        Ok(StrategyRecord {
                            id: row.get(0)?,
                            title: row.get(1)?,
                            description: row.get(2)?,
                            raw_text: row.get(3)?,
                            intent_json: row.get(4)?,
                            creator_address: row.get(5)?,
                            agent_id: row.get(6)?,
                            risk_profile: row.get(7)?,
                            copies: row.get::<_, i64>(8)? as u64,
                            total_volume: row.get::<_, i64>(9)? as u64,
                            apy: row.get(10)?,
                            created_at: row.get(11)?,
                            updated_at: row.get(12)?,
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

    async fn increment_strategy_copies(&self,
        id: &str,
    ) -> Result<(), StorageError> {
        let conn = self.conn.clone();
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn.lock().map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            conn.execute(
                "UPDATE strategies SET copies = copies + 1, updated_at = ?1 WHERE id = ?2",
                rusqlite::params![migrations::unix_now(), id],
            )
            .map_err(|e| StorageError::SaveFailed(e.to_string()))?;
            Ok(())
        })
        .await
        .map_err(|e| StorageError::SaveFailed(e.to_string()))?
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
            user_address: None,
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

    fn sample_strategy(id: &str) -> StrategyRecord {
        StrategyRecord {
            id: id.to_string(),
            title: "Delta-neutral LP".to_string(),
            description: "A strategy description.".to_string(),
            raw_text: "delta neutral lp on aerodrome".to_string(),
            intent_json: "{}".to_string(),
            creator_address: Some("0xCreator".to_string()),
            agent_id: "agent-1".to_string(),
            risk_profile: "moderate".to_string(),
            copies: 0,
            total_volume: 0,
            apy: 0.12,
            created_at: now_secs(),
            updated_at: now_secs(),
        }
    }

    #[tokio::test]
    async fn save_and_list_strategy() {
        let storage = SqliteStorage::in_memory().unwrap();
        let record = sample_strategy("strategy-1");
        storage.save_strategy(&record).await.unwrap();

        let strategies = storage.list_strategies().await.unwrap();
        assert_eq!(strategies.len(), 1);
        assert_eq!(strategies[0].id, "strategy-1");
        assert_eq!(strategies[0].title, "Delta-neutral LP");
    }

    #[tokio::test]
    async fn get_strategy_by_id() {
        let storage = SqliteStorage::in_memory().unwrap();
        storage.save_strategy(&sample_strategy("a")).await.unwrap();

        let found = storage.get_strategy("a").await.unwrap().unwrap();
        assert_eq!(found.id, "a");
        assert!(storage.get_strategy("missing").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn increment_strategy_copies_bumps_count() {
        let storage = SqliteStorage::in_memory().unwrap();
        storage.save_strategy(&sample_strategy("forkable")).await.unwrap();

        storage.increment_strategy_copies("forkable").await.unwrap();
        let found = storage.get_strategy("forkable").await.unwrap().unwrap();
        assert_eq!(found.copies, 1);
        assert!(found.updated_at >= found.created_at);
    }
}
