-- Bundle-based MEV submissions (V2). Records every bundle submitted to a
-- private relay, whether from the backrun monitor or the manual API endpoint.
CREATE TABLE IF NOT EXISTS mev_bundles (
    bundle_hash TEXT PRIMARY KEY,
    target_tx_hash TEXT,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL
);
