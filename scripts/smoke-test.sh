#!/usr/bin/env bash
set -euo pipefail

API_URL="${OTTER_API_URL:-http://localhost:3001}"

echo "==> Smoke testing $API_URL"

echo "--- /ready"
curl -fsS "$API_URL/ready" | jq .

echo "--- /health"
curl -fsS "$API_URL/api/v1/health" | jq .

echo "--- parse intent"
curl -fsS -X POST "$API_URL/api/v1/intents/parse" \
  -H 'Content-Type: application/json' \
  -d '{"text":"lend 100 USDC on Aave if yield > 1"}' | jq .

echo "==> Smoke tests passed"
