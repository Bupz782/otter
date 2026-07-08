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

echo "--- /ready"
curl -fsS "$API_URL/ready" | pretty_json

echo "--- /health"
curl -fsS "$API_URL/api/v1/health" | pretty_json

echo "--- parse intent"
curl -fsS -X POST "$API_URL/api/v1/intents/parse" \
  -H 'Content-Type: application/json' \
  -d '{"text":"lend 100 USDC on Aave if yield > 1"}' | pretty_json

echo "==> Smoke tests passed"
