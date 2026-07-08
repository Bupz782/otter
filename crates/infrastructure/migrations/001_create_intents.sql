CREATE TABLE IF NOT EXISTS intents (
    id TEXT PRIMARY KEY,
    text TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    state TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_intents_updated_at ON intents(updated_at DESC);
