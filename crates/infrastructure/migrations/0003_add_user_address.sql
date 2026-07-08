-- `user_address` is part of the consolidated 0001_init.sql schema.
-- For databases that existed before that consolidation, the runner adds the
-- column from Rust because the bundled SQLite does not support
-- `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`.
INSERT INTO schema_migrations (version, applied_at) VALUES (3, strftime('%s','now'));
