//! Simulated MEV capture and user rebates (V1, off-chain).
//!
//! LIMITATION (by design for V1): there is no real searcher/MEV pipeline.
//! Profit is a deterministic function of the executed intent's amount —
//! documented in [`SimulatedMevCapture::profit_for`] — persisted off-chain,
//! and partially rebated to the vault owner. Contracts and the Noir circuit
//! are untouched.

use std::sync::Arc;

use rusqlite::Connection;

use domain::ports::evm_port::EvmError;

/// Share of captured profit rebated to the vault owner when
/// `OTTER_MEV_REBATE_BPS` is unset (50%).
pub const DEFAULT_REBATE_BPS: u64 = 5_000;

/// Read the rebate share (basis points) from the environment.
pub fn rebate_bps_from_env() -> u64 {
    std::env::var("OTTER_MEV_REBATE_BPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&bps| bps <= 10_000)
        .unwrap_or(DEFAULT_REBATE_BPS)
}

/// Compute the rebate owed from a captured profit.
///
/// `rebate = profit * bps / 10_000`, truncating towards zero so the protocol
/// never pays out more than was captured.
pub fn rebate_of(profit_wei: u128, bps: u64) -> u128 {
    profit_wei * u128::from(bps) / 10_000
}

/// One recorded capture.
#[derive(Debug, Clone)]
pub struct MevCapture {
    pub tx_hash: String,
    pub block_number: u64,
    pub profit_wei: u128,
}

/// Deterministic simulated profit for an executed intent.
///
/// Formula (documented so results are reproducible): 3 bps of the traded
/// amount — i.e. `amount * 3 / 100_000` wei — standing in for the spread a
/// private-orderflow execution would have saved. Never random.
pub fn profit_for(amount: u128) -> u128 {
    amount * 3 / 100_000
}

/// SQLite-backed capture store. Shares the main database file.
#[derive(Clone)]
pub struct SimulatedMevCapture {
    conn: Arc<std::sync::Mutex<Connection>>,
}

impl SimulatedMevCapture {
    /// Open (and lazily create the table in) the SQLite database at `path`.
    pub fn new(database_url: &str) -> Result<Self, String> {
        let path = database_url
            .strip_prefix("sqlite://")
            .unwrap_or(database_url);
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS mev_captures (
                tx_hash TEXT PRIMARY KEY,
                block_number INTEGER NOT NULL,
                profit_wei TEXT NOT NULL,
                owner_address TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_mev_captures_owner ON mev_captures(owner_address);",
        )
        .map_err(|e| e.to_string())?;
        Ok(Self {
            conn: Arc::new(std::sync::Mutex::new(conn)),
        })
    }

    /// Record one capture; idempotent per tx hash.
    pub fn record(
        &self,
        tx_hash: &str,
        block_number: u64,
        amount: u128,
        owner_address: &str,
    ) -> Result<u128, String> {
        let profit = profit_for(amount);
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR IGNORE INTO mev_captures
             (tx_hash, block_number, profit_wei, owner_address, created_at)
             VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))",
            rusqlite::params![
                tx_hash,
                block_number as i64,
                profit.to_string(),
                owner_address
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(profit)
    }

    /// Total rebated profit for an owner (sum of captures × rebate share).
    pub fn total_rebate(&self, owner_address: &str) -> Result<u128, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT COALESCE(SUM(CAST(profit_wei AS INTEGER)), 0)
                 FROM mev_captures WHERE owner_address = ?1",
            )
            .map_err(|e| e.to_string())?;
        let total: i64 = stmt
            .query_row([owner_address], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(rebate_of(total as u128, rebate_bps_from_env()))
    }
}

impl domain::ports::mev_port::MevPort for SimulatedMevCapture {
    fn capture_from_execution(
        &self,
        tx_hash: &str,
        block_number: u64,
        amount: u128,
        owner_address: &str,
    ) -> Result<Option<domain::ports::mev_port::MevCapture>, EvmError> {
        self.record(tx_hash, block_number, amount, owner_address)
            .map(|_| {
                Some(domain::ports::mev_port::MevCapture {
                    tx_hash: tx_hash.to_string(),
                    block_number,
                    profit_wei: profit_for(amount),
                })
            })
            .map_err(|e| EvmError::SubmissionFailed(format!("mev capture: {}", e)))
    }
}
