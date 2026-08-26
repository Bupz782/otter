-- Social/sharing fields for strategies (visibility + fork tracking).
ALTER TABLE strategies ADD COLUMN visibility TEXT NOT NULL DEFAULT 'private';
ALTER TABLE strategies ADD COLUMN fork_count INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_strategies_visibility ON strategies(visibility);
