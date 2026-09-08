-- Per-intent lifecycle timeline events (parsed -> condition_met -> proof ->
-- submitted -> confirmed). Was an in-memory ring buffer lost on restart.
-- id is "{epoch_millis:013}-{uuid}": lexicographic order == chronological
-- order, portable across SQLite and Postgres (no AUTOINCREMENT/SERIAL).
CREATE TABLE IF NOT EXISTS intent_events (
    id TEXT PRIMARY KEY,
    intent_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    detail TEXT,
    at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_intent_events_intent ON intent_events(intent_id, id);
