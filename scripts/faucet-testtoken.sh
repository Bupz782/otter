#!/usr/bin/env bash
# Mint TestToken (tTST) on Sepolia for a demo tester.
#
# TestToken.mint(address,uint256) is public — anyone can mint (it's a
# testnet-only token). The deployer key (or any funded key) pays the gas.
#
# Usage:
#   scripts/faucet-testtoken.sh <recipient-0x-address> [amount]
#
# Env:
#   SEPOLIA_RPC_URL  (default: https://ethereum-sepolia-rpc.publicnode.com)
#   MINTER_KEY       private key paying the gas (0x-prefixed)
#   OTTER_TEST_TOKEN TestToken address (default: read from .env.sepolia)
#
# Default amount: 10 000 tTST (18 decimals).

set -euo pipefail

TO="${1:-}"
AMOUNT_HUMAN="${2:-10000}"

if [[ -z "$TO" ]]; then
    echo "Usage: $0 <recipient-0x-address> [amount]" >&2
    exit 1
fi

cd "$(dirname "$0")/.."

RPC_URL="${SEPOLIA_RPC_URL:-https://ethereum-sepolia-rpc.publicnode.com}"
KEY="${MINTER_KEY:-${DEPLOYER_KEY:-}}"
TOKEN="${OTTER_TEST_TOKEN:-$(grep -E '^OTTER_TEST_TOKEN=' .env.sepolia 2>/dev/null | cut -d= -f2 || true)}"

[[ -n "$TOKEN" ]] || { echo "ERROR: OTTER_TEST_TOKEN unknown (set it or run scripts/deploy-sepolia.sh first)." >&2; exit 1; }
[[ -n "$KEY" ]] || { echo "ERROR: MINTER_KEY (private key) is required." >&2; exit 1; }

command -v cast >/dev/null || { echo "ERROR: cast not found (install foundry)." >&2; exit 1; }

AMOUNT_WEI=$(cast parse-units "$AMOUNT_HUMAN" 18)

echo "Minting $AMOUNT_HUMAN tTST ($AMOUNT_WEI wei) to $TO…"
cast send "$TOKEN" "mint(address,uint256)" "$TO" "$AMOUNT_WEI" \
    --rpc-url "$RPC_URL" \
    --private-key "${KEY#0x}"

echo "Balance after mint:"
cast call "$TOKEN" "balanceOf(address)(uint256)" "$TO" --rpc-url "$RPC_URL"
