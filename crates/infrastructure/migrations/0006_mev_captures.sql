-- Simulated MEV captures (V1, off-chain). profit_wei is stored as TEXT to
-- accommodate u128 values beyond SQLite's integer range.
CREATE TABLE IF NOT EXISTS mev_captures (
    tx_hash TEXT PRIMARY KEY,
    block_number INTEGER NOT NULL,
    profit_wei TEXT NOT NULL,
    owner_address TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_mev_captures_owner ON mev_captures(owner_address);
