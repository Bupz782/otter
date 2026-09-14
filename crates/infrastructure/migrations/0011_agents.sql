-- Agents operated by Otter (anomaly A2): replaces the hardcoded
-- `default_agents()` seed. Seeded at boot from the configured signer key
-- (real BabyJubJub pubkey); stats are computed from executions/proofs.
-- Dual-dialect SQLite/Postgres: TEXT primary key, no AUTOINCREMENT/SERIAL.
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    operator TEXT NOT NULL,
    pubkey_x TEXT,
    pubkey_y TEXT,
    bond_wei TEXT NOT NULL DEFAULT '0',
    status TEXT NOT NULL DEFAULT 'active',
    created_at INTEGER NOT NULL
);
