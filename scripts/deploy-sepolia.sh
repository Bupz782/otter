#!/usr/bin/env bash
# Deploy the full Otter demo stack on Sepolia in ONE broadcast.
#
# Deploys (in this order): TestToken, OtterBridge, BridgeToken,
# DelegationVerifier, DelegationVault, SolvencyVerifier, SolvencyRegistry,
# then wires bridge.setBridgeToken().
#
# Unlike the anvil flow, addresses on Sepolia are NOT deterministic — the
# script extracts them from the forge broadcast log and writes an env
# fragment you can source or paste into .env.
#
# Prerequisites:
#   - forge + cast installed (foundry)
#   - a SEPOLIA PRIVATE KEY funded with Sepolia ETH (faucet:
#     https://sepoliafaucet.com or https://cloud.google.com/application/web3/faucet/ethereum/sepolia)
#   - a Sepolia RPC endpoint (free, no key: https://ethereum-sepolia-rpc.publicnode.com)
#
# Usage:
#   SEPOLIA_RPC_URL=https://ethereum-sepolia-rpc.publicnode.com \
#   DEPLOYER_KEY=0x<private-key> \
#   scripts/deploy-sepolia.sh [--verify]
#
# With --verify, ETHERSCAN_API_KEY must be set (free at etherscan.io).
#
# Output: prints the addresses and writes .env.sepolia (gitignored-safe
# fragment, no secrets) in the repo root.

set -euo pipefail

cd "$(dirname "$0")/../contracts"

RPC_URL="${SEPOLIA_RPC_URL:-https://ethereum-sepolia-rpc.publicnode.com}"
KEY="${DEPLOYER_KEY:-}"
VERIFY="${1:-}"

if [[ -z "$KEY" ]]; then
    echo "ERROR: DEPLOYER_KEY (private key, 0x-prefixed) is required." >&2
    exit 1
fi

for bin in forge cast; do
    command -v "$bin" >/dev/null || { echo "ERROR: $bin not found (install foundry)."; exit 1; }
done

# --- Preflight ---------------------------------------------------------------
CHAIN_ID=$(cast chain-id --rpc-url "$RPC_URL")
if [[ "$CHAIN_ID" != "11155111" ]]; then
    echo "ERROR: RPC chain id is $CHAIN_ID, expected 11155111 (Sepolia)." >&2
    exit 1
fi

DEPLOYER=$(cast wallet address --private-key "$KEY")
BALANCE=$(cast balance "$DEPLOYER" --rpc-url "$RPC_URL")
echo "Deployer: $DEPLOYER"
echo "Balance:  $(cast from-wei "$BALANCE") ETH"
# Full-stack deploy costs ~23M gas (docs/DEPLOIEMENT.md); at 5 gwei ≈ 0.12 ETH.
if (( BALANCE < 100000000000000000 )); then
    echo "ERROR: balance < 0.1 ETH — fund the deployer via a Sepolia faucet first." >&2
    exit 1
fi

# --- Deploy ------------------------------------------------------------------
echo
echo "Deploying full stack to Sepolia (one broadcast)…"
VERIFY_ARGS=()
if [[ "$VERIFY" == "--verify" ]]; then
    : "${ETHERSCAN_API_KEY:?--verify requires ETHERSCAN_API_KEY}"
    VERIFY_ARGS=(--verify --etherscan-api-key "$ETHERSCAN_API_KEY")
fi

LOG=$(mktemp)
# forge script expects the key WITHOUT the 0x prefix.
forge script script/DeployLocalStack.s.sol \
    --rpc-url "$RPC_URL" \
    --private-key "${KEY#0x}" \
    --broadcast \
    "${VERIFY_ARGS[@]}" \
    | tee "$LOG"

# --- Extract addresses from the == Logs == section ---------------------------
extract() { grep "$1:" "$LOG" | grep -oE '0x[0-9a-fA-F]{40}' | head -1; }

TOKEN=$(extract "TestToken")
BRIDGE=$(extract "OtterBridge")
BRIDGED=$(extract "BridgeToken")
DELEGATION_VERIFIER=$(extract "DelegationVerifier")
VAULT=$(extract "DelegationVault")
SOLVENCY_VERIFIER=$(extract "SolvencyVerifier")
REGISTRY=$(extract "SolvencyRegistry")

if [[ -z "$VAULT" || -z "$REGISTRY" || -z "$TOKEN" ]]; then
    echo "ERROR: could not extract all addresses from the forge output (see $LOG)." >&2
    exit 1
fi

ENV_FILE="../.env.sepolia"
cat > "$ENV_FILE" <<EOF
# Otter Sepolia deployment — $(date -u +%Y-%m-%dT%H:%M:%SZ)
# Deployer: $DEPLOYER
# Chain: sepolia (11155111)

OTTER_NETWORKS=sepolia=$RPC_URL|$VAULT|11155111|$BRIDGE
OTTER_SOLVENCY_REGISTRY=$REGISTRY
OTTER_NETWORK=sepolia
OTTER_EXECUTION_ENABLED=true

OTTER_TEST_TOKEN=$TOKEN
OTTER_BRIDGE_TOKEN=$BRIDGED
OTTER_DELEGATION_VERIFIER=$DELEGATION_VERIFIER
OTTER_SOLVENCY_VERIFIER=$SOLVENCY_VERIFIER
EOF

echo
echo "=== Deployment complete ==="
echo "TestToken:           $TOKEN"
echo "OtterBridge:         $BRIDGE"
echo "BridgeToken:         $BRIDGED"
echo "DelegationVerifier:  $DELEGATION_VERIFIER"
echo "DelegationVault:     $VAULT"
echo "SolvencyVerifier:    $SOLVENCY_VERIFIER"
echo "SolvencyRegistry:    $REGISTRY"
echo
echo "Env fragment written to .env.sepolia — merge it into your .env (or export it"
echo "before docker compose up). The deployer key must be set separately as"
echo "OTTER_PRIVATE_KEY (or better: OTTER_PRIVATE_KEY_FILE)."
echo
echo "Next: fund testers with scripts/faucet-testtoken.sh"
