#!/usr/bin/env bash
# End-to-end local UI demo: an intent created through the API fires, gets a
# real bb proof, and executes on the local vault — the full timeline is then
# visible in the frontend (http://localhost:5173/app/intents/<id>).
#
# Idempotent: safe to re-run. After an anvil reset, re-run this script (it
# redeploys the contracts when the vault address has no code).
#
# Requires: docker compose stack running (anvil + api), cast, curl, python3.
set -euo pipefail

RPC_URL="${RPC_URL:-http://localhost:8545}"
API_URL="${API_URL:-http://localhost:3001}"
VAULT="${OTTER_VAULT_ADDRESS:-0xa513E6E4b8f2a923D98304ec87F64353C4D5C853}"
KEY="${ANVIL_KEY:-ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80}"
DUMMY_ROUTER="0x2222222222222222222222222222222222222222"
INTENT_TEXT="${INTENT_TEXT:-Swap 1 ETH for USDC on Uniswap when gas < 20 gwei}"

cd "$(dirname "$0")/.."

echo "== 1/4 Contracts =="
if [[ -z "$(cast code "$VAULT" --rpc-url "$RPC_URL" 2>/dev/null)" ]]; then
    echo "no code at $VAULT — deploying the local stack"
    echo "(the api container must NOT be mid-execution: it shares the deployer key)"
    (cd contracts && forge script script/DeployLocalStack.s.sol \
        --rpc-url "$RPC_URL" --private-key "$KEY" --broadcast 2>/dev/null | grep -E "Vault|Registry")
else
    echo "vault already deployed at $VAULT"
fi

echo "== 2/4 Vault setup (router + 5 ETH float) =="
cast send "$VAULT" "setProtocolRouter(uint256,address)" 1 "$DUMMY_ROUTER" \
    --rpc-url "$RPC_URL" --private-key "$KEY" >/dev/null
cast send "$VAULT" "deposit()" --value 5ether \
    --rpc-url "$RPC_URL" --private-key "$KEY" >/dev/null
echo "router 1 -> $DUMMY_ROUTER, vault balance: $(cast balance "$VAULT" --rpc-url "$RPC_URL") wei"

echo "== 3/4 Intent: $INTENT_TEXT =="
INTENT_ID=$(curl -sf -X POST "$API_URL/api/v1/intents" \
    -H 'content-type: application/json' \
    -d "{\"text\":\"$INTENT_TEXT\"}" | python3 -c "import json,sys; print(json.load(sys.stdin)['id'])")
echo "created: $INTENT_ID"

echo "== 4/4 Waiting for execution (monitoring tick is 60s) =="
for i in $(seq 1 20); do
    sleep 15
    EVENTS=$(curl -sf "$API_URL/api/v1/intents/$INTENT_ID/events" \
        | python3 -c "import json,sys; print(' '.join(e['kind'] for e in json.load(sys.stdin)['events']))")
    echo "  t+$((i*15))s: $EVENTS"
    case " $EVENTS " in
        *" confirmed "*)
            STATE=$(curl -sf "$API_URL/api/v1/intents/$INTENT_ID" | python3 -c "import json,sys; print(json.load(sys.stdin)['state'])")
            echo ""
            echo "OK — intent executed: $STATE"
            echo "Timeline: http://localhost:5173/app/intents/$INTENT_ID"
            exit 0
            ;;
        *" failed "*)
            echo ""
            echo "FAILED — check the timeline for the reason:"
            curl -sf "$API_URL/api/v1/intents/$INTENT_ID/events" | python3 -m json.tool
            exit 1
            ;;
    esac
done
echo "TIMEOUT after 5 minutes — last events: $EVENTS"
exit 1
