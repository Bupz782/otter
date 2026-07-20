#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$REPO_ROOT"

# Optional fork configuration for Anvil. If unset, Anvil starts from a blank local chain.
: "${SEPOLIA_RPC_URL:=}"
: "${FORK_BLOCK:=}"

# Make sure tools installed by dev-setup.sh are available in this shell.
export PATH="$HOME/.foundry/bin:$HOME/.nargo/bin:$HOME/.bb:$PATH"

ANVIL_PID=""
FRONTEND_PID=""

cleanup() {
    echo "==> Shutting down dev stack..."
    if [[ -n "$FRONTEND_PID" ]]; then
        kill "$FRONTEND_PID" 2>/dev/null || true
    fi
    if [[ -n "$ANVIL_PID" ]]; then
        kill "$ANVIL_PID" 2>/dev/null || true
    fi
    docker compose stop postgres 2>/dev/null || true
}
trap cleanup EXIT

# Build Anvil CLI arguments.
ANVIL_ARGS=()
if [[ -n "$SEPOLIA_RPC_URL" ]]; then
    ANVIL_ARGS+=(--fork-url "$SEPOLIA_RPC_URL")
    if [[ -n "$FORK_BLOCK" ]]; then
        ANVIL_ARGS+=(--fork-block-number "$FORK_BLOCK")
    fi
fi

# Start Anvil in the background.
echo "==> Starting Anvil..."
if [[ ${#ANVIL_ARGS[@]} -eq 0 ]]; then
    anvil &
else
    anvil "${ANVIL_ARGS[@]}" &
fi
ANVIL_PID=$!
sleep 3

# Start Postgres via docker compose (detached).
echo "==> Starting Postgres..."
docker compose up postgres -d

# Wait briefly for Postgres to accept connections.
for _ in {1..30}; do
    if nc -z localhost 5432 2>/dev/null; then
        break
    fi
    sleep 1
done

# Require a deployer private key via environment variable. Do not hardcode keys.
if [[ -z "${DEPLOYER_PRIVATE_KEY:-}" ]]; then
    echo "ERROR: DEPLOYER_PRIVATE_KEY is not set." >&2
    echo "Set it to a funded Anvil private key, e.g.:" >&2
    echo "  DEPLOYER_PRIVATE_KEY=0x... ./scripts/dev.sh" >&2
    exit 1
fi
DEPLOYER_PK="$DEPLOYER_PRIVATE_KEY"
OTTER_VAULT_ADDRESS=""

# Deploy contracts to the local Anvil chain and try to capture the vault address.
echo "==> Deploying contracts..."
DEPLOY_LOG=$(mktemp /tmp/otter-deploy.XXXXXX)
if forge script contracts/script/DeployDelegationVault.s.sol \
    --rpc-url http://localhost:8545 \
    --private-key "$DEPLOYER_PK" \
    --broadcast >"$DEPLOY_LOG" 2>&1; then
    OTTER_VAULT_ADDRESS=$(grep -oE "DelegationVault deployed at: (0x[0-9a-fA-F]{40})" "$DEPLOY_LOG" | awk '{print $NF}' || true)
    if [[ -n "$OTTER_VAULT_ADDRESS" ]]; then
        echo "Deployed vault: $OTTER_VAULT_ADDRESS"
    else
        echo "WARN: Could not parse vault address from deploy output." >&2
    fi
else
    echo "WARN: Contract deployment failed or script not found; set OTTER_VAULT_ADDRESS manually if execution is enabled." >&2
fi
rm -f "$DEPLOY_LOG"

# Local development environment.
export OTTER_RPC_URL=http://localhost:8545
export OTTER_CHAIN_ID=31337
export OTTER_DATABASE_URL=postgres://otter:otter@localhost:5432/otter
export OTTER_NETWORK=local
export OTTER_EXECUTION_ENABLED=false
export OTTER_DELEGATE_ON_CREATE=false
export OTTER_METRICS_ENABLED=true
export OTTER_MONITORING_INTERVAL_SECS=60
export OTTER_API_PORT=3001
export OTTER_PRIVATE_KEY=$DEPLOYER_PK
export OTTER_NONCE_STORE_PATH=./otter-nonce.txt
export OTTER_CIRCUIT_DIR=./delegation_circuit
export OTTER_NARGO_BIN="${HOME}/.nargo/bin/nargo"
export OTTER_BB_BIN="${HOME}/.bb/bb"
export RUST_LOG=debug
export OTTER_LOG_FORMAT=text
export OTTER_CORS_ALLOWED_ORIGINS="*"

# Generate repo-root .env.local for documentation and justfile dotenv-load.
cat > .env.local <<EOF
OTTER_RPC_URL=$OTTER_RPC_URL
OTTER_CHAIN_ID=$OTTER_CHAIN_ID
OTTER_DATABASE_URL=$OTTER_DATABASE_URL
OTTER_NETWORK=$OTTER_NETWORK
OTTER_EXECUTION_ENABLED=$OTTER_EXECUTION_ENABLED
OTTER_DELEGATE_ON_CREATE=$OTTER_DELEGATE_ON_CREATE
OTTER_METRICS_ENABLED=$OTTER_METRICS_ENABLED
OTTER_MONITORING_INTERVAL_SECS=$OTTER_MONITORING_INTERVAL_SECS
OTTER_API_PORT=$OTTER_API_PORT
OTTER_PRIVATE_KEY=$OTTER_PRIVATE_KEY
OTTER_VAULT_ADDRESS=${OTTER_VAULT_ADDRESS:-}
OTTER_NONCE_STORE_PATH=$OTTER_NONCE_STORE_PATH
OTTER_CIRCUIT_DIR=$OTTER_CIRCUIT_DIR
OTTER_NARGO_BIN=$OTTER_NARGO_BIN
OTTER_BB_BIN=$OTTER_BB_BIN
RUST_LOG=$RUST_LOG
OTTER_LOG_FORMAT=$OTTER_LOG_FORMAT
OTTER_CORS_ALLOWED_ORIGINS=$OTTER_CORS_ALLOWED_ORIGINS
VITE_API_URL=http://localhost:${OTTER_API_PORT}
EOF

# Generate frontend .env.local so Vite picks up the API URL automatically.
cat > frontend/.env.local <<EOF
VITE_API_URL=http://localhost:${OTTER_API_PORT}
EOF

# Start Vite frontend in the background.
echo "==> Starting Vite frontend..."
(
    cd frontend
    npm run dev
) &
FRONTEND_PID=$!

# Start API with cargo watch in the foreground so the script blocks until the user stops it.
echo "==> Starting API (cargo watch)..."
if command -v cargo-watch >/dev/null 2>&1; then
    cargo watch -x "run --bin otter_api"
else
    echo "WARN: cargo-watch not found; falling back to 'cargo run --bin otter_api'. Install cargo-watch for auto-reload." >&2
    cargo run --bin otter_api
fi
