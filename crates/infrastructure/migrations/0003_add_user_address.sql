-- Migration 3: add `user_address` column to intents and delegations.
--
-- PostgreSQL deployments apply the DDL below. SQLite does not support
-- `ALTER TABLE ... ADD COLUMN IF NOT EXISTS`, so the SQLite runner applies
-- the equivalent idempotent column addition from Rust while still recording
-- version 3 in `schema_migrations`.
-- New deployments already have the column from `0001_init.sql`.

ALTER TABLE intents ADD COLUMN IF NOT EXISTS user_address TEXT;
ALTER TABLE delegations ADD COLUMN IF NOT EXISTS user_address TEXT;
