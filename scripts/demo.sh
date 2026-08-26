#!/usr/bin/env bash
# demo.sh — Soutenance demo: reproducible end-to-end Otter flow, timed < 3 min.
#
# Flow: anvil -> deploy DelegationVault -> delegation + deposit +
# executeWithProof with a REAL Noir/Barretenberg proof (cargo e2e test).
#
# Usage: bash scripts/demo.sh
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$(pwd)"

RPC_URL="${OTTER_TEST_RPC_URL:-http://localhost:8545}"
PRIVATE_KEY="${OTTER_TEST_PRIVATE_KEY:-0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80}"
BB_BIN="${BB_BIN:-${HOME}/.bb/bb}"
CIRCUIT_DIR="${OTTER_CIRCUIT_DIR:-${ROOT}/delegation_circuit}"

step() { printf '\n\033[1;36m=== [%s] %s\033[0m\n' "$1" "$2"; }
fail() { printf '\n\033[1;31m✗ %s\033[0m\n' "$1" >&2; exit 1; }

START=$(date +%s)

# ---------------------------------------------------------------------------
# Early detection: fail BEFORE starting anvil if the ZK toolchain/fixtures
# are missing.
# ---------------------------------------------------------------------------
for cmd in anvil forge cast cargo nargo curl; do
  command -v "$cmd" >/dev/null 2>&1 || fail "outil manquant: $cmd — installer puis relancer"
done
[[ -x "$BB_BIN" ]] || fail "Barretenberg introuvable: $BB_BIN (export BB_BIN=... ou installer bb)"
[[ -f "${CIRCUIT_DIR}/Nargo.toml" ]] || fail "circuit introuvable: ${CIRCUIT_DIR}"
if ! ls "${CIRCUIT_DIR}"/target/*.json >/dev/null 2>&1; then
  fail "fixtures requis: lancer generate-fixture (compilation du circuit Noir)"
fi

ANVIL_PID=""

cleanup() {
  if [[ -n "$ANVIL_PID" ]] && kill -0 "$ANVIL_PID" 2>/dev/null; then
    echo ""
    echo "[cleanup] arrêt d'anvil (pid $ANVIL_PID)"
    kill "$ANVIL_PID" 2>/dev/null || true
    wait "$ANVIL_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
step "1/5" "Démarrage d'anvil local (${RPC_URL})"
mkdir -p "${ROOT}/.tmp"
anvil --port 8545 >"${ROOT}/.tmp/demo-anvil.log" 2>&1 &
ANVIL_PID=$!

for _ in $(seq 1 50); do
  if chain_id=$(curl -sf -X POST -H 'Content-Type: application/json' \
      --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
      "$RPC_URL" 2>/dev/null); then
    [[ "$chain_id" == *'"0x"'* ]] && break
  fi
  sleep 0.2
done
curl -sf -X POST -H 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  "$RPC_URL" >/dev/null || fail "anvil ne répond pas sur ${RPC_URL}"
echo "  anvil prêt (pid $ANVIL_PID, log: .tmp/demo-anvil.log)"

# ---------------------------------------------------------------------------
step "2/5" "Déploiement de DelegationVault sur anvil"
DEPLOY_OUTPUT=$(cd contracts && \
  forge script script/DeployDelegationVault.s.sol \
  --rpc-url "$RPC_URL" --broadcast \
  --private-key "$PRIVATE_KEY" \
  --sender 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266)
VAULT_ADDRESS=$(echo "$DEPLOY_OUTPUT" | grep -oE 'DelegationVault deployed at: (0x[0-9a-fA-F]+)' | awk '{print $NF}')
[[ -n "$VAULT_ADDRESS" ]] || fail "adresse du vault non trouvée dans la sortie forge"
echo "  Vault: $VAULT_ADDRESS"

# ---------------------------------------------------------------------------
step "3/5" "Export des variables d'environnement"
export OTTER_TEST_RPC_URL="$RPC_URL"
export OTTER_TEST_VAULT_ADDRESS="$VAULT_ADDRESS"
export OTTER_TEST_PRIVATE_KEY="$PRIVATE_KEY"
export BB_BIN
echo "  OTTER_TEST_RPC_URL=$OTTER_TEST_RPC_URL"
echo "  OTTER_TEST_VAULT_ADDRESS=$VAULT_ADDRESS"

# ---------------------------------------------------------------------------
step "4/5" "Test e2e: delegation + deposit + executeWithProof (preuve réelle bb)"
cargo test -p infrastructure --test e2e_anvil_flow -- --nocapture

# ---------------------------------------------------------------------------
step "5/5" "Cleanup"
TOTAL=$(( $(date +%s) - START ))
echo ""
echo -e "\033[1;32m✅ Démo end-to-end réussie en ${TOTAL}s (< 180s visé pour la soutenance)\033[0m"
