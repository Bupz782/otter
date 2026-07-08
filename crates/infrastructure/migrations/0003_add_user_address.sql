-- Migration 3: add `user_address` column to intents and delegations.
--
-- This migration uses PostgreSQL's `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`
-- syntax, which is idempotent. New deployments already have the column from
-- `0001_init.sql`; existing PostgreSQL deployments receive it here.
--
-- SQLite does not support `IF NOT EXISTS` on `ALTER TABLE ... ADD COLUMN`, so
-- the SQLite migration runner applies the equivalent idempotent column addition
-- from Rust while still recording version 3 in `schema_migrations`.

ALTER TABLE intents ADD COLUMN IF NOT EXISTS user_address TEXT;
ALTER TABLE delegations ADD COLUMN IF NOT EXISTS user_address TEXT;
