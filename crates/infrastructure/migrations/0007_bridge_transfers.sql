-- Cross-chain bridge V1 lock/mint transfers.
-- status: pending -> minted | failed
CREATE TABLE IF NOT EXISTS bridge_transfers (
    bridge_id TEXT PRIMARY KEY,
    source_chain_id INTEGER NOT NULL,
    destination_chain_id INTEGER NOT NULL,
    user_address TEXT NOT NULL,
    amount_wei TEXT NOT NULL,
    lock_tx_hash TEXT,
    mint_tx_hash TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_bridge_transfers_user ON bridge_transfers(user_address);
CREATE INDEX IF NOT EXISTS idx_bridge_transfers_status ON bridge_transfers(status);
