#!/usr/bin/env bash
set -euo pipefail

: "${OTTER_API_PORT:=3001}"
: "${OTTER_DATABASE_URL:=/data/otter.db}"
: "${OTTER_CIRCUIT_DIR:=/app/delegation_circuit}"
: "${OTTER_NARGO_BIN:=/usr/local/bin/nargo}"
: "${OTTER_BB_BIN:=/usr/local/bin/bb}"

# If the container is invoked to run a tool rather than the API binary, skip
# the runtime setup and execute it directly (e.g. nargo --version, bb --version).
if [[ "${1:-}" != "otter_api" ]]; then
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

# Optional LLM model for the hybrid intent parser. The API selects the hybrid
# parser only when OTTER_MODEL_PATH exists at boot, so the download runs in
# the background: the stack comes up on the regex parser immediately, and a
# later container restart picks the model up.
if [[ "${OTTER_MODEL_DOWNLOAD:-false}" == "true" ]]; then
    : "${OTTER_MODEL_PATH:=/app/models/Qwen3-8B-Q4_K_M.gguf}"
    MODEL_URL="${OTTER_MODEL_URL:-https://huggingface.co/Qwen/Qwen3-8B-GGUF/resolve/main/Qwen3-8B-Q4_K_M.gguf}"
    if [[ ! -f "$OTTER_MODEL_PATH" ]]; then
        mkdir -p "$(dirname "$OTTER_MODEL_PATH")"
        (
            echo "Downloading LLM model in the background (~4.5GB)..."
            if curl -sfL -o "$OTTER_MODEL_PATH.part" "$MODEL_URL"; then
                mv "$OTTER_MODEL_PATH.part" "$OTTER_MODEL_PATH"
                echo "Model ready at $OTTER_MODEL_PATH — restart the api container to enable the hybrid parser."
            else
                rm -f "$OTTER_MODEL_PATH.part"
                echo "Model download failed; regex parser stays active." >&2
            fi
        ) &
    fi
fi

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

echo "Starting otter_api..."
exec "$@"
