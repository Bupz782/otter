#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$REPO_ROOT"

# Make sure tools installed by dev-setup.sh are available in this shell.
export PATH="$HOME/.foundry/bin:$HOME/.nargo/bin:$HOME/.bb:$PATH"

NOIR_VERSION=$(grep compiler_version delegation_circuit/Nargo.toml | sed 's/.*= "\(.*\)".*/\1/')
BB_VERSION=$(cat .bb-version)

# Minimum Node major version required by the frontend toolchain.
MIN_NODE_MAJOR=20

echo "==> Otter dev setup"
echo "Noir version: $NOIR_VERSION"
echo "BB version: $BB_VERSION"

# Rust nightly (required by the workspace).
if ! command -v rustup >/dev/null 2>&1; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
fi
if ! rustup toolchain list | grep -q nightly; then
    echo "Installing nightly toolchain..."
    rustup toolchain install nightly
fi
rustup component add rustfmt clippy --toolchain nightly

# Foundry (forge, cast, anvil).
if ! command -v forge >/dev/null 2>&1; then
    echo "Installing Foundry..."
    curl -L https://foundry.paradigm.xyz | bash
    "$HOME/.foundry/bin/foundryup"
fi

# Noir (nargo + noirup).
NOIRUP_BIN="$HOME/.nargo/bin/noirup"
NARGO_BIN="$HOME/.nargo/bin/nargo"
if ! command -v nargo >/dev/null 2>&1 || [[ "$(nargo --version 2>/dev/null)" != *"$NOIR_VERSION"* ]]; then
    echo "Installing Noir $NOIR_VERSION..."
    curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
    "$NOIRUP_BIN" -v "$NOIR_VERSION"
fi

# Barretenberg (bb + bbup).
BBUP_BIN="$HOME/.bb/bbup"
BB_BIN="$HOME/.bb/bb"
if ! command -v bb >/dev/null 2>&1 || [[ "$(bb --version 2>/dev/null)" != *"$BB_VERSION"* ]]; then
    echo "Installing bb $BB_VERSION..."
    curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/bbup/install | bash
    "$BBUP_BIN" -v "$BB_VERSION"
fi

# Node.js (frontend build tooling).
if ! command -v node >/dev/null 2>&1; then
    echo "ERROR: Node.js is required. Please install Node $MIN_NODE_MAJOR (https://nodejs.org) and re-run." >&2
    exit 1
fi
NODE_MAJOR=$(node --version | sed 's/v\([0-9]*\).*/\1/')
if [[ "$NODE_MAJOR" -lt "$MIN_NODE_MAJOR" ]]; then
    echo "ERROR: Node $MIN_NODE_MAJOR+ is required (found $(node --version))." >&2
    exit 1
fi

# cargo-watch (used by dev.sh to auto-restart the API).
if ! command -v cargo-watch >/dev/null 2>&1; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch --locked
fi

# Frontend dependencies.
echo "==> Installing frontend dependencies..."
cd frontend
npm install

cd "$REPO_ROOT"

echo "==> Setup complete"
echo "Run 'just dev' or './scripts/dev.sh' to start the stack."
