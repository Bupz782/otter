use async_trait::async_trait;
use domain::ports::storage_port::{
    DelegationRecord, ExecutionRecord, IntentRecord, StorageError, StoragePort, StrategyRecord,
};
use sqlx::{Pool, Postgres, Row};

use crate::storage::migrations;

/// PostgreSQL-backed implementation of [`StoragePort`].
///
/// The connection pool is managed by `sqlx`. Pending migrations from
/// `crates/infrastructure/migrations` are run automatically when the storage is
/// created; the runner records each applied version in `schema_migrations` with
/// the current Unix timestamp.
pub struct PgStorage {
    pool: Pool<Postgres>,
}

impl PgStorage {
    /// Connect to a PostgreSQL database at `database_url`, run pending
    /// migrations, and return a storage handle.
    pub async fn new(database_url: &str) -> Result<Self, StorageError> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await
            .map_err(|e| StorageError::InitFailed(e.to_string()))?;

        run_migrations(&pool).await?;

        Ok(Self { pool })
    }
}

async fn run_migrations(pool: &Pool<Postgres>) -> Result<(), StorageError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        )",
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| StorageError::InitFailed(e.to_string()))?;

    for path in migrations::migration_files()? {
        let version = migrations::migration_version(&path)?;
        let applied: bool =
            sqlx::query_scalar::<_, i64>("SELECT 1 FROM schema_migrations WHERE version = $1")
                .bind(version)
                .fetch_optional(&mut *tx)
                .await
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
            sqlx::raw_sql(&sql).execute(&mut *tx).await.map_err(|e| {
                StorageError::InitFailed(format!(
                    "failed to run migration {}: {}",
                    path.display(),
                    e
                ))
            })?;

            sqlx::query("INSERT INTO schema_migrations (version, applied_at) VALUES ($1, $2)")
                .bind(version)
                .bind(migrations::unix_now())
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    StorageError::InitFailed(format!(
                        "failed to record migration {}: {}",
                        version, e
                    ))
                })?;
        }
    }

    tx.commit()
        .await
        .map_err(|e| StorageError::InitFailed(e.to_string()))?;

    Ok(())
}

fn row_to_record(row: &sqlx::postgres::PgRow) -> Result<IntentRecord, StorageError> {
    let id: String = row
        .try_get("id")
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
    let text: String = row
        .try_get("text")
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
    let intent_json: String = row
        .try_get("intent_json")
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
    let state: String = row
        .try_get("state")
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
    let created_at: i64 = row
        .try_get("created_at")
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
    let updated_at: i64 = row
        .try_get("updated_at")
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
    let user_address: Option<String> = row
        .try_get("user_address")
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

    let intent =
        serde_json::from_str(&intent_json).map_err(|e| StorageError::ReadFailed(e.to_string()))?;

    Ok(IntentRecord {
        id,
        text,
        intent,
        state,
        created_at,
        updated_at,
        user_address,
    })
}

#[async_trait]
impl StoragePort for PgStorage {
    async fn save_intent(&self, record: &IntentRecord) -> Result<(), StorageError> {
        let intent_json = serde_json::to_string(&record.intent)
            .map_err(|e| StorageError::SaveFailed(e.to_string()))?;

        sqlx::query(
            "INSERT INTO intents (id, text, intent_json, state, created_at, updated_at, user_address)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (id) DO UPDATE SET
                 text = EXCLUDED.text,
                 intent_json = EXCLUDED.intent_json,
                 state = EXCLUDED.state,
                 created_at = EXCLUDED.created_at,
                 updated_at = EXCLUDED.updated_at,
                 user_address = EXCLUDED.user_address",
        )
        .bind(&record.id)
        .bind(&record.text)
        .bind(&intent_json)
        .bind(&record.state)
        .bind(record.created_at)
        .bind(record.updated_at)
        .bind(record.user_address.as_ref())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::SaveFailed(e.to_string()))?;

        Ok(())
    }

    async fn list_intents(&self) -> Result<Vec<IntentRecord>, StorageError> {
        let rows = sqlx::query(
            "SELECT id, text, intent_json, state, created_at, updated_at, user_address
             FROM intents
             ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        rows.iter().map(row_to_record).collect()
    }

    async fn get_intent(&self, id: &str) -> Result<Option<IntentRecord>, StorageError> {
        let row = sqlx::query(
            "SELECT id, text, intent_json, state, created_at, updated_at, user_address
             FROM intents
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_record(&r)?)),
            None => Ok(None),
        }
    }

    async fn delete_intent(&self, id: &str) -> Result<(), StorageError> {
        let result = sqlx::query("DELETE FROM intents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DeleteFailed(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<(), StorageError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
        Ok(())
    }

    async fn save_delegation(&self, record: &DelegationRecord) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO delegations (hash, payload_json, signature, created_at, user_address)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (hash) DO UPDATE SET
                 payload_json = EXCLUDED.payload_json,
                 signature = EXCLUDED.signature,
                 created_at = EXCLUDED.created_at,
                 user_address = EXCLUDED.user_address",
        )
        .bind(&record.hash)
        .bind(&record.payload_json)
        .bind(&record.signature)
        .bind(record.created_at)
        .bind(record.user_address.as_ref())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::SaveFailed(e.to_string()))?;

        Ok(())
    }

    async fn list_delegations(&self) -> Result<Vec<DelegationRecord>, StorageError> {
        let rows = sqlx::query(
            "SELECT hash, payload_json, signature, created_at, user_address
             FROM delegations
             ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        rows.iter()
            .map(|row| {
                let payload_json: String = row
                    .try_get("payload_json")
                    .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
                Ok(DelegationRecord {
                    hash: row
                        .try_get("hash")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    payload_json,
                    signature: row
                        .try_get("signature")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    created_at: row
                        .try_get("created_at")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    user_address: row
                        .try_get("user_address")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                })
            })
            .collect()
    }

    async fn get_delegation(&self, hash: &str) -> Result<Option<DelegationRecord>, StorageError> {
        let row = sqlx::query(
            "SELECT hash, payload_json, signature, created_at, user_address
             FROM delegations
             WHERE hash = $1",
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        match row {
            Some(r) => {
                let payload_json: String = r
                    .try_get("payload_json")
                    .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
                Ok(Some(DelegationRecord {
                    hash: r
                        .try_get("hash")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    payload_json,
                    signature: r
                        .try_get("signature")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    created_at: r
                        .try_get("created_at")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    user_address: r
                        .try_get("user_address")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                }))
            }
            None => Ok(None),
        }
    }

    async fn save_execution(&self, record: &ExecutionRecord) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO executions (id, intent_id, tx_hash, status, gas_used, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET
                 intent_id = EXCLUDED.intent_id,
                 tx_hash = EXCLUDED.tx_hash,
                 status = EXCLUDED.status,
                 gas_used = EXCLUDED.gas_used,
                 created_at = EXCLUDED.created_at",
        )
        .bind(&record.id)
        .bind(&record.intent_id)
        .bind(&record.tx_hash)
        .bind(&record.status)
        .bind(record.gas_used as i64)
        .bind(record.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::SaveFailed(e.to_string()))?;

        Ok(())
    }

    async fn list_executions(&self) -> Result<Vec<ExecutionRecord>, StorageError> {
        let rows = sqlx::query(
            "SELECT id, intent_id, tx_hash, status, gas_used, created_at
             FROM executions
             ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        rows.iter()
            .map(|row| {
                let gas_used: i64 = row
                    .try_get("gas_used")
                    .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
                Ok(ExecutionRecord {
                    id: row
                        .try_get("id")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    intent_id: row
                        .try_get("intent_id")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    tx_hash: row
                        .try_get("tx_hash")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    status: row
                        .try_get("status")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    gas_used: gas_used as u64,
                    created_at: row
                        .try_get("created_at")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                })
            })
            .collect()
    }

    async fn get_executions_for_intent(
        &self,
        intent_id: &str,
    ) -> Result<Vec<ExecutionRecord>, StorageError> {
        let rows = sqlx::query(
            "SELECT id, intent_id, tx_hash, status, gas_used, created_at
             FROM executions
             WHERE intent_id = $1
             ORDER BY created_at DESC",
        )
        .bind(intent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        rows.iter()
            .map(|row| {
                let gas_used: i64 = row
                    .try_get("gas_used")
                    .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
                Ok(ExecutionRecord {
                    id: row
                        .try_get("id")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    intent_id: row
                        .try_get("intent_id")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    tx_hash: row
                        .try_get("tx_hash")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    status: row
                        .try_get("status")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    gas_used: gas_used as u64,
                    created_at: row
                        .try_get("created_at")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                })
            })
            .collect()
    }

    async fn save_strategy(&self, record: &StrategyRecord) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO strategies
             (id, title, description, raw_text, intent_json, creator_address, agent_id,
              risk_profile, copies, total_volume, apy, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT (id) DO UPDATE SET
                 title = EXCLUDED.title,
                 description = EXCLUDED.description,
                 raw_text = EXCLUDED.raw_text,
                 intent_json = EXCLUDED.intent_json,
                 creator_address = EXCLUDED.creator_address,
                 agent_id = EXCLUDED.agent_id,
                 risk_profile = EXCLUDED.risk_profile,
                 copies = EXCLUDED.copies,
                 total_volume = EXCLUDED.total_volume,
                 apy = EXCLUDED.apy,
                 created_at = EXCLUDED.created_at,
                 updated_at = EXCLUDED.updated_at",
        )
        .bind(&record.id)
        .bind(&record.title)
        .bind(&record.description)
        .bind(&record.raw_text)
        .bind(&record.intent_json)
        .bind(record.creator_address.as_ref())
        .bind(&record.agent_id)
        .bind(&record.risk_profile)
        .bind(record.copies as i64)
        .bind(record.total_volume as i64)
        .bind(record.apy)
        .bind(record.created_at)
        .bind(record.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::SaveFailed(e.to_string()))?;

        Ok(())
    }

    async fn list_strategies(&self) -> Result<Vec<StrategyRecord>, StorageError> {
        let rows = sqlx::query(
            "SELECT id, title, description, raw_text, intent_json, creator_address, agent_id,
                    risk_profile, copies, total_volume, apy, created_at, updated_at
             FROM strategies
             ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        rows.iter()
            .map(|row| {
                let copies: i64 = row
                    .try_get("copies")
                    .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
                let total_volume: i64 = row
                    .try_get("total_volume")
                    .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
                Ok(StrategyRecord {
                    id: row
                        .try_get("id")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    title: row
                        .try_get("title")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    description: row
                        .try_get("description")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    raw_text: row
                        .try_get("raw_text")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    intent_json: row
                        .try_get("intent_json")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    creator_address: row
                        .try_get("creator_address")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    agent_id: row
                        .try_get("agent_id")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    risk_profile: row
                        .try_get("risk_profile")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    copies: copies as u64,
                    total_volume: total_volume as u64,
                    apy: row
                        .try_get("apy")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    created_at: row
                        .try_get("created_at")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    updated_at: row
                        .try_get("updated_at")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                })
            })
            .collect()
    }

    async fn get_strategy(&self, id: &str) -> Result<Option<StrategyRecord>, StorageError> {
        let row = sqlx::query(
            "SELECT id, title, description, raw_text, intent_json, creator_address, agent_id,
                    risk_profile, copies, total_volume, apy, created_at, updated_at
             FROM strategies
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::ReadFailed(e.to_string()))?;

        match row {
            Some(r) => {
                let copies: i64 = r
                    .try_get("copies")
                    .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
                let total_volume: i64 = r
                    .try_get("total_volume")
                    .map_err(|e| StorageError::ReadFailed(e.to_string()))?;
                Ok(Some(StrategyRecord {
                    id: r.try_get("id")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    title: r.try_get("title")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    description: r.try_get("description")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    raw_text: r.try_get("raw_text")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    intent_json: r.try_get("intent_json")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    creator_address: r.try_get("creator_address")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    agent_id: r.try_get("agent_id")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    risk_profile: r.try_get("risk_profile")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    copies: copies as u64,
                    total_volume: total_volume as u64,
                    apy: r.try_get("apy")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    created_at: r.try_get("created_at")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                    updated_at: r.try_get("updated_at")
                        .map_err(|e| StorageError::ReadFailed(e.to_string()))?,
                }))
            }
            None => Ok(None),
        }
    }

    async fn increment_strategy_copies(&self, id: &str) -> Result<(), StorageError> {
        sqlx::query(
            "UPDATE strategies SET copies = copies + 1, updated_at = $1 WHERE id = $2",
        )
        .bind(migrations::unix_now())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::SaveFailed(e.to_string()))?;

        Ok(())
    }
}
