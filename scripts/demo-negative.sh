#!/usr/bin/env bash
# demo-negative.sh — Soutenance demo (bonus): show that the on-chain
# delegation constraints BITE.
#
# Setup identique à demo.sh, mais on enregistre la délégation avec des
# maxAmounts minuscules puis on soumet une preuve RÉELLE dont les public
# inputs portent un amount bien supérieur -> la tx DOIT revert avec
# `AmountExceedsMax`.
#
# Usage: bash scripts/demo-negative.sh
set -euo pipefail

cd "$(dirname "$0")/.."
ROOT="$(pwd)"

RPC_URL="${OTTER_TEST_RPC_URL:-http://localhost:8545}"
PRIVATE_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
OWNER="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
BB_BIN="${BB_BIN:-${HOME}/.bb/bb}"
CIRCUIT_DIR="${OTTER_CIRCUIT_DIR:-${ROOT}/delegation_circuit}"
OUT_DIR="${ROOT}/.tmp/demo-negative"
INTENT="swap 1000 USDC for ETH on Uniswap"

step() { printf '\n\033[1;36m=== [%s] %s\033[0m\n' "$1" "$2"; }
fail() { printf '\n\033[1;31m✗ %s\033[0m\n' "$1" >&2; exit 1; }

START=$(date +%s)

# Early detection (avant de démarrer anvil).
for cmd in anvil forge cast cargo nargo curl xxd python3; do
  command -v "$cmd" >/dev/null 2>&1 || fail "outil manquant: $cmd"
done
[[ -x "$BB_BIN" ]] || fail "Barretenberg introuvable: $BB_BIN"
[[ -f "${CIRCUIT_DIR}/Nargo.toml" ]] || fail "circuit introuvable: ${CIRCUIT_DIR}"
ls "${CIRCUIT_DIR}"/target/*.json >/dev/null 2>&1 \
  || fail "fixtures requis: lancer generate-fixture"

ANVIL_PID=""
cleanup() {
  if [[ -n "$ANVIL_PID" ]] && kill -0 "$ANVIL_PID" 2>/dev/null; then
    echo ""; echo "[cleanup] arrêt d'anvil"; kill "$ANVIL_PID" 2>/dev/null || true
    wait "$ANVIL_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

step "1/6" "Démarrage d'anvil local (${RPC_URL})"
mkdir -p "$OUT_DIR"
anvil --port 8545 >"${OUT_DIR}/anvil.log" 2>&1 &
ANVIL_PID=$!
for _ in $(seq 1 50); do
  curl -sf -X POST -H 'Content-Type: application/json' \
    --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
    "$RPC_URL" >/dev/null 2>&1 && break
  sleep 0.2
done
curl -sf -X POST -H 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
  "$RPC_URL" >/dev/null || fail "anvil ne répond pas"
echo "  anvil prêt"

step "2/6" "Déploiement de DelegationVault"
DEPLOY_OUTPUT=$(cd contracts && \
  forge script script/DeployDelegationVault.s.sol \
  --rpc-url "$RPC_URL" --broadcast \
  --private-key "$PRIVATE_KEY" --sender "$OWNER")
VAULT=$(echo "$DEPLOY_OUTPUT" | grep -oE 'DelegationVault deployed at: (0x[0-9a-fA-F]+)' | awk '{print $NF}')
[[ -n "$VAULT" ]] || fail "adresse du vault non trouvée"
echo "  Vault: $VAULT"

step "3/6" "Génération d'une preuve RÉELLE (nargo + bb)"
cargo run -q -p interfaces --bin otter_cli -- prove "$INTENT" \
  --private-key "$PRIVATE_KEY" \
  --output-dir "$OUT_DIR" \
  --circuit-dir "$CIRCUIT_DIR" \
  --bb-bin "$BB_BIN"
[[ -f "${OUT_DIR}/proof.bin" && -f "${OUT_DIR}/public_inputs.bin" ]] \
  || fail "preuve non générée"

step "4/6" "Enregistrement de la délégation avec maxAmounts minuscules"
# Parse des public inputs (38 champs de 32 octets):
#   [0..31]=hash bytes | 32=intent_type | 33=amount | 34=protocol |
#   35=target_contract | 36=timestamp | 37=nonce
read -r HASH NONCE AMOUNT <<<"$(python3 - "$OUT_DIR/public_inputs.bin" <<'PY'
import sys
data = open(sys.argv[1], "rb").read()
assert len(data) == 38 * 32, f"taille inattendue: {len(data)}"
fields = [data[i*32:(i+1)*32] for i in range(38)]
h = 0
for i in range(32):          # même reconstruction que _reconstructHash
    h = (h << 8) | int.from_bytes(fields[i], "big")
print(hex(h), int.from_bytes(fields[37], "big"), int.from_bytes(fields[33], "big"))
PY
)"
[[ -n "$HASH" ]] || fail "parsing des public inputs échoué"
echo "  delegationHash: $HASH"
echo "  nonce: $NONCE   amount dans la preuve: $AMOUNT"
echo "  -> on enregistre maxAmounts[type]=1 pour chaque type"

EXPIRY=$(( $(date +%s) + 3600 ))
cast send "$VAULT" "delegate(bytes32,uint256,uint256[10],uint256[5],uint256,uint256)" \
  "$HASH" "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff" \
  "[1,1,1,1,1,1,1,1,1,1]" "[1,2,0,0,0]" "$EXPIRY" "$NONCE" \
  --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" --json >"${OUT_DIR}/delegate.json"
cast send "$VAULT" "setProtocolRouter(uint256,address)" 1 0x2222222222222222222222222222222222222222 \
  --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" >/dev/null
echo "  délégation enregistrée (limites volontairement ridicules)"

step "5/6" "Soumission de executeWithProof -> la tx DOIT revert (AmountExceedsMax)"
PROOF_HEX="0x$(xxd -p "${OUT_DIR}/proof.bin" | tr -d '\n')"
PUB_INPUTS="$(python3 - "$OUT_DIR/public_inputs.bin" <<'PY'
import sys
data = open(sys.argv[1], "rb").read()
print("[" + ",".join("0x" + data[i*32:(i+1)*32].hex() for i in range(38)) + "]")
PY
)"

REVERT_MSG=""
if cast send "$VAULT" "executeWithProof(bytes,bytes32[])" \
     "$PROOF_HEX" "$PUB_INPUTS" \
     --rpc-url "$RPC_URL" --private-key "$PRIVATE_KEY" \
     >"${OUT_DIR}/execute.log" 2>&1; then
  fail "la tx a PASSÉ — les contraintes n'ont pas mordu !"
fi
# Récupère le motif de revert (sortie cast ou trace anvil).
REVERT_MSG=$(grep -oiE '(AmountExceedsMax|InvalidProof|IntentNotAllowed|ProtocolNotAllowed|StaleProof|InvalidNonce|DelegationNotFound|DelegationExpired)[^"]*' \
  "${OUT_DIR}/execute.log" "${OUT_DIR}/anvil.log" 2>/dev/null | head -1 | cut -d: -f2- || true)
echo "  tx REVERT ✓${REVERT_MSG:+  (motif: $REVERT_MSG)}"

step "6/6" "Cleanup"
TOTAL=$(( $(date +%s) - START ))
if [[ "$REVERT_MSG" == *AmountExceedsMax* ]]; then
  echo ""
  echo -e "\033[1;32m✅ Contrainte maxAmounts vérifiée on-chain: la preuve réelle est rejetée quand l'amount dépasse la limite (${TOTAL}s)\033[0m"
else
  echo -e "\033[1;33m⚠ tx bien rejetée mais motif non confirmé comme AmountExceedsMax (voir ${OUT_DIR}/execute.log)\033[0m"
  exit 1
fi
