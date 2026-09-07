-- Migration 9: add `delegation_id` column to intents.
-- Links an intent to the signed delegation it runs under, so the UI can
-- show "runs under delegation 0x…" and a delegation can list its intents.
--
-- PostgreSQL deployments apply the DDL below. SQLite does not support
-- `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`, so the SQLite runner applies
-- the equivalent idempotent column addition from Rust (same pattern as
-- migration 0003) while still recording version 9 in `schema_migrations`.

ALTER TABLE intents ADD COLUMN IF NOT EXISTS delegation_id TEXT;
