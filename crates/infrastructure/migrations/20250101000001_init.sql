CREATE TABLE IF NOT EXISTS intents (
    id TEXT PRIMARY KEY,
    text TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    state TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_intents_updated_at ON intents(updated_at DESC);

CREATE TABLE IF NOT EXISTS delegations (
    hash TEXT PRIMARY KEY,
    payload_json TEXT NOT NULL,
    signature TEXT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_delegations_created_at ON delegations(created_at DESC);

CREATE TABLE IF NOT EXISTS executions (
    id TEXT PRIMARY KEY,
    intent_id TEXT NOT NULL,
    tx_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    gas_used BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_executions_intent_id ON executions(intent_id);
