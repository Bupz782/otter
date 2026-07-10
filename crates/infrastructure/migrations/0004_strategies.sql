CREATE TABLE IF NOT EXISTS strategies (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    raw_text TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    creator_address TEXT,
    agent_id TEXT NOT NULL,
    risk_profile TEXT NOT NULL,
    copies INTEGER NOT NULL DEFAULT 0,
    total_volume INTEGER NOT NULL DEFAULT 0,
    apy REAL NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_strategies_agent_id ON strategies(agent_id);
CREATE INDEX IF NOT EXISTS idx_strategies_creator_address ON strategies(creator_address);
