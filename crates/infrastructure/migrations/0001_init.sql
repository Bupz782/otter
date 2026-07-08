CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS intents (
    id TEXT PRIMARY KEY,
    text TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    state TEXT NOT NULL,
    user_address TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS delegations (
    hash TEXT PRIMARY KEY,
    payload_json TEXT NOT NULL,
    signature TEXT NOT NULL,
    user_address TEXT,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS executions (
    id TEXT PRIMARY KEY,
    intent_id TEXT NOT NULL,
    tx_hash TEXT,
    status TEXT NOT NULL,
    gas_used INTEGER,
    created_at INTEGER NOT NULL
);

INSERT INTO schema_migrations (version, applied_at) VALUES (1, strftime('%s','now'));
