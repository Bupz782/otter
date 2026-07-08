#!/usr/bin/env bash
set -euo pipefail

: "${OTTER_API_PORT:=3001}"
: "${OTTER_DATABASE_URL:=/data/otter.db}"
: "${OTTER_CIRCUIT_DIR:=/app/delegation_circuit}"
: "${OTTER_NARGO_BIN:=/usr/local/bin/nargo}"
: "${OTTER_BB_BIN:=/usr/local/bin/bb}"

# If the container is invoked to run a tool rather than the API binary, skip
# the runtime setup and execute it directly (e.g. nargo --version, bb --version).
if [[ "${1:-}" != "metis_api" ]]; then
    exec "$@"
fi

# Validate required settings
if [[ -z "${OTTER_RPC_URL:-}" ]]; then
    echo "ERROR: OTTER_RPC_URL is required" >&2
    exit 1
fi

if [[ -z "${OTTER_CHAIN_ID:-}" ]]; then
    echo "ERROR: OTTER_CHAIN_ID is required" >&2
    exit 1
fi

# Wait for Postgres if configured
if [[ "$OTTER_DATABASE_URL" == postgres* ]]; then
    db_host=$(echo "$OTTER_DATABASE_URL" | sed -n 's/.*@\([^:/]*\).*/\1/p')
    echo "Waiting for Postgres at $db_host:5432..."
    until nc -z "$db_host" 5432; do
        sleep 1
    done
else
    # Ensure parent directory exists for SQLite
    mkdir -p "$(dirname "$OTTER_DATABASE_URL")"
fi

# Migrations are applied automatically by the API when it starts up (see
# PgStorage::new / SqliteStorage::new). No separate migration command is needed
# here as long as OTTER_MIGRATIONS_DIR points at the bundled SQL files.

# Validate ZKP tooling if execution is enabled
if [[ "${OTTER_EXECUTION_ENABLED:-false}" == "true" ]]; then
    if ! command -v "$OTTER_NARGO_BIN" >/dev/null 2>&1; then
        echo "ERROR: nargo not found at $OTTER_NARGO_BIN" >&2
        exit 1
    fi
    if ! command -v "$OTTER_BB_BIN" >/dev/null 2>&1; then
        echo "ERROR: bb not found at $OTTER_BB_BIN" >&2
        exit 1
    fi
    if [[ -z "${OTTER_VAULT_ADDRESS:-}" ]]; then
        echo "ERROR: OTTER_VAULT_ADDRESS is required when execution is enabled" >&2
        exit 1
    fi
fi

echo "Starting metis_api..."
exec "$@"
