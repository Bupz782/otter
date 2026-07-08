#!/usr/bin/env bash
set -euo pipefail

API_URL="${OTTER_API_URL:-http://localhost:3001}"

pretty_json() {
    if command -v jq >/dev/null 2>&1; then
        jq .
    elif command -v python3 >/dev/null 2>&1; then
        python3 -m json.tool
    elif command -v python >/dev/null 2>&1; then
        python -m json.tool
    else
        cat
    fi
}

echo "==> Smoke testing $API_URL"

echo "--- /ready (polling until warm, 503 is retryable)"
ready_status=""
for _ in {1..30}; do
    ready_status=$(curl -s -o /tmp/otter-ready.json -w "%{http_code}" "$API_URL/ready" || true)
    if [[ "$ready_status" == "200" ]]; then
        break
    fi
    if [[ "$ready_status" != "503" && -n "$ready_status" ]]; then
        echo "ERROR: /ready returned unexpected status $ready_status" >&2
        exit 1
    fi
    sleep 1
done
if [[ "$ready_status" != "200" ]]; then
    echo "ERROR: /ready did not become ready within 30 seconds (last status: $ready_status)" >&2
    exit 1
fi
cat /tmp/otter-ready.json | pretty_json

echo "--- /health"
curl -fsS "$API_URL/api/v1/health" | pretty_json

echo "--- parse intent"
curl -fsS -X POST "$API_URL/api/v1/intents/parse" \
  -H 'Content-Type: application/json' \
  -d '{"text":"lend 100 USDC on Aave if yield > 1"}' | pretty_json

echo "==> Smoke tests passed"
