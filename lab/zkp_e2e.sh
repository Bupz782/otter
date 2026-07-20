#!/usr/bin/env bash
set -euo pipefail

# End-to-end ZKP demo on a local Anvil node.
# Requires: anvil, forge, nargo, bb, cargo, cast

RPC_URL="http://localhost:8545"
PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
OWNER="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
INTENT="swap 1000 USDC for ETH on Uniswap"
CIRCUIT_DIR="${OTTER_CIRCUIT_DIR:-./delegation_circuit}"
BB_BIN="${BB_BIN:-${HOME}/.bb/bb}"

for cmd in anvil forge cast cargo nargo; do
  if ! command -v "$cmd" &> /dev/null; then
    echo "Missing required tool: $cmd"
    exit 1
  fi
done

if [[ ! -x "$BB_BIN" ]]; then
  echo "Barretenberg binary not found or not executable: $BB_BIN"
  echo "Set BB_BIN or install bb to ~/.bb/bb"
  exit 1
fi

echo "[1/6] Starting Anvil..."
anvil --fork-url "" --block-time 1 &
ANVIL_PID=$!
trap 'echo "[cleanup] stopping Anvil"; kill $ANVIL_PID 2>/dev/null || true' EXIT
sleep 3

echo "[2/6] Deploying DelegationVerifier + DelegationVault..."
DEPLOY_OUTPUT=$(cd contracts && PRIVATE_KEY=$PRIVATE_KEY forge script script/DeployDelegationVault.s.sol \
  --rpc-url "$RPC_URL" \
  --broadcast \
  --sender "$OWNER")

VAULT=$(echo "$DEPLOY_OUTPUT" | grep -oE 'DelegationVault deployed at: (0x[0-9a-fA-F]+)' | awk '{print $NF}')
VERIFIER=$(echo "$DEPLOY_OUTPUT" | grep -oE 'DelegationVerifier deployed at: (0x[0-9a-fA-F]+)' | awk '{print $NF}')

echo "  Verifier: $VERIFIER"
echo "  Vault:    $VAULT"

echo "[3/6] Depositing 10 ETH into the vault..."
cast send "$VAULT" "deposit()" --value 10ether \
  --rpc-url "$RPC_URL" \
  --private-key "$PRIVATE_KEY" \
  >/dev/null

echo "[4/6] Generating delegation proof..."
cargo run -p interfaces --bin otter_cli -- prove "$INTENT" \
  --private-key "$PRIVATE_KEY" \
  --output-dir ./lab/zkp_e2e_out \
  --circuit-dir "$CIRCUIT_DIR" \
  --bb-bin "$BB_BIN"

echo "[5/6] Executing intent on-chain with proof..."
cargo run -p interfaces --bin otter_cli -- execute "$INTENT" \
  --rpc-url "$RPC_URL" \
  --private-key "$PRIVATE_KEY" \
  --vault "$VAULT" \
  --delegate \
  --circuit-dir "$CIRCUIT_DIR" \
  --bb-bin "$BB_BIN"

echo "[6/6] Verifying proof on-chain..."
cargo run -p interfaces --bin otter_cli -- verify-onchain \
  --vault "$VAULT" \
  --rpc-url "$RPC_URL" \
  --private-key "$PRIVATE_KEY" \
  --proof ./lab/zkp_e2e_out/proof.bin \
  --public-inputs ./lab/zkp_e2e_out/public_inputs.bin

echo "✅ ZKP end-to-end demo completed successfully."
