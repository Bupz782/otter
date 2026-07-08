-- Migration 3 consolidated the `user_address` column into `0001_init.sql`.
-- The migration runner records version 3 in `schema_migrations` automatically
-- after the preceding migrations succeed, and adds the columns from Rust for
-- SQLite deployments that predate the consolidation.

