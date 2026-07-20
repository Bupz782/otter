# Otter 🦦

> **Describe your DeFi strategy in plain English. Delegate with cryptographic limits. The agent executes across chains, captures MEV, and pays you. Share your strategy. Copy others. Every action is proven valid. The vault is proven solvent. The proof is forever.**

---

## What Is Otter?

Otter is a **trustless DeFi automation protocol**. You deposit funds into a vault, describe your strategy in natural language, and delegate execution authority to an agent — with signed, cryptographically-enforced limits. The agent executes across Ethereum and Arbitrum, captures MEV from the transaction flow, and rebates it to you. Every action requires a zero-knowledge proof. The vault periodically proves its solvency without revealing individual balances.

**The combination doesn't exist anywhere else:** natural language intents + persistent ZKP delegation + MEV rebates + proof-of-solvency + social strategy sharing.

---

## Core Flow

```
Connect Wallet → Deposit → Type Intent → Set Delegation → Agent Monitors
                                    ↓
                    Condition Met → Generate ZKP → Capture MEV → Execute
                                    ↓
                    Real-Time Status + Permanent Proof + MEV Rebate
```

**Example:**
> *"Lend 1000 USDC on Aave if yield > 3%"*

1. **Parse:** Local LLM converts text to structured intent
2. **Delegate:** You sign limits (max 5k, protocols, expiry)
3. **Monitor:** Agent checks yield every 60s
4. **Prove:** When yield = 3.2%, agent generates ZKP proving action respects your delegation
5. **Execute:** Vault verifies proof → lends on Aave → rebates captured MEV
6. **Verify:** Anyone can independently verify the proof on-chain, forever

---

## Key Features

| Feature | Description |
|---------|-------------|
| **Natural Language Intents** | Type what you want. Local LLM parses it live. |
| **ZKP Delegation** | Sign limits. Math enforces them. Server can't cheat. |
| **MEV Rebates** | Agent captures MEV from execution. You get paid to automate. |
| **Proof-of-Solvency** | Vault proves `Assets ≥ Deposits` without revealing balances. |
| **Agent Marketplace** | Choose agents by reputation, bond size, and performance. |
| **Social Strategies** | Publish, share, and copy strategies. Creators earn fees. |
| **Multi-Chain** | Ethereum + Arbitrum. Cross-chain intents supported. |

---

## Architecture

```
┌──────────────────────────────────────────────┐
│           React + Vite Frontend               │
│  (Intent input, Delegation wizard, Dashboard, │
│   Marketplace, Social feed, Proof explorer)   │
└──────────────┬───────────────────────────────┘
               │ HTTP / WebSocket
┌──────────────▼───────────────────────────────┐
│           Rust Backend (Axum)                 │
│  ┌─────────────┐  ┌──────────────────────┐   │
│  │  LLM Parser │  │   Orchestrator       │   │
│  │  (local)    │  │   (state machine)    │   │
│  └─────────────┘  └──────────────────────┘   │
│  ┌─────────────┐  ┌──────────────────────┐   │
│  │ ZKP Prover  │  │   Blockchain         │   │
│  │ (Noir)      │  │   Adapters (Alloy)   │   │
│  └─────────────┘  └──────────────────────┘   │
│  ┌─────────────┐  ┌──────────────────────┐   │
│  │ MEV Capture │  │   Social / Strategy  │   │
│  │ (Searcher)  │  │   Registry           │   │
│  └─────────────┘  └──────────────────────┘   │
└──────────────┬───────────────────────────────┘
               │ RPC / Transactions
┌──────────────▼───────────────────────────────┐
│           Blockchain Layer                    │
│  ┌──────────────┐  ┌────────────────────┐   │
│  │ StrategyVault│  │ DelegationVerifier │   │
│  │  (holds funds│  │  (verifies ZKPs)   │   │
│  │   + executes)│  └────────────────────┘   │
│  └──────────────┘                            │
│       ┌──────────┐  ┌──────────┐            │
│       │  Aave    │  │ Compound │            │
│       │ Uniswap  │  │ Arbitrum │  L2)       │
│       └──────────┘  └──────────┘            │
└──────────────────────────────────────────────┘
```

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Frontend** | React 18 + Vite + Tailwind CSS + RainbowKit + wagmi/viem |
| **Backend API** | Rust (Axum) + tokio |
| **LLM** | Local llama.cpp (GGUF models) |
| **ZKP** | Noir circuits + Barretenberg verifier |
| **Smart Contracts** | Solidity + Foundry |
| **Blockchain** | Ethereum mainnet + Arbitrum |

---

## Project Status

| Wave | Theme | Scope | Progress |
|------|-------|-------|----------|
| 0 | Setup & Architecture | **MVP** | ~95% |
| 1 | Intent Parsing & LLM | **MVP** | ~80% |
| 2 | ZKP Delegation | **MVP** | ~60% |
| 3 | FHE | **CUT** | — |
| 4 | Encrypted Mempool | **CUT** | — |
| 5 | Blockchain + MEV + Solvency | **MVP** | ~2% |
| 6 | Orchestrator + Marketplace | **MVP** | ~3% |
| 6.5 | Web UI + Social | **MVP** | ~0% |
| 7 | Production | **MVP** | ~0% |
| 8+ | Advanced / Community | **Future** | — |

**Overall: ~15% complete. MVP target: ~384 stories.**

See [`BACKLOG.md`](BACKLOG.md) for the full catalog of 580 user stories with per-story status tracking.
See [`PRODUCT.md`](PRODUCT.md) for the detailed product specification.

---

## Quick Start

### Prerequisites

- [Rust nightly](https://rustup.rs/) with `rustfmt` and `clippy`
- [Noir (nargo)](https://noir-lang.org/docs/getting_started/installation/)
- [Foundry](https://book.getfoundry.sh/getting-started/installation) (`forge`, `cast`, `anvil`)
- `gh` CLI (optional, for issue sync)

### Build

```bash
# Build the entire workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run lints
cargo clippy --workspace --all-targets -- -D warnings
```

### Configuration

```bash
cp config.example.toml config.toml
# Edit config.toml with your RPC URL, model path, etc.
```

Configuration can also be overridden via environment variables:
`OTTER_RPC_URL`, `OTTER_CHAIN_ID`, `OTTER_MODEL_PATH`, `OTTER_LOG_LEVEL`,
`OTTER_DATABASE_URL`, `OTTER_API_PORT`, etc.

Use `OTTER_CONFIG_PATH` to point to a custom config file location.

### Run Examples

```bash
# Parse a natural-language intent
cargo run -p interfaces --bin otter_cli -- parse "lend 1000 USDC on Aave"

# Build an execution plan from an intent
cargo run -p interfaces --bin otter_cli -- plan "swap 1 ETH for USDC on Uniswap"

# Start the orchestrator daemon and monitor a conditional price intent.
# Fetches real-time ETH/USD from Chainlink on Sepolia (default).
cargo run -p interfaces --bin otter_cli -- start \
  "swap 1 ETH for USDC on Uniswap if price > 2_000_000000" \
  --interval 5

# Monitor a yield intent (fetches Aave supply APY on the chosen network)
cargo run -p interfaces --bin otter_cli -- start \
  "lend 1000 USDC on Aave if yield > 3" \
  --network sepolia \
  --interval 60

# Query the daemon state
cargo run -p interfaces --bin otter_cli -- status

# Start the daemon in on-chain execution mode
# It will monitor the condition, generate a ZKP, and submit executeWithProof
# to the vault when the condition is met.
cargo run -p interfaces --bin otter_cli -- start \
  "swap 1000 USDC for ETH on Uniswap if price > 2_000_000000" \
  --network sepolia \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --vault $VAULT \
  --delegate \
  --interval 2

# Test intent parsing
cargo run --example test_plan_use_case -p application

# Test strategy planner
cargo run --example test_strategy_planner -p application

# Test condition evaluation with mock oracle
cargo run --example test_evaluate_use_case -p application

# Start the React + Vite frontend
cd frontend
npm install
npm run dev
# Then open http://localhost:5173
```

The frontend provides three views:
- **Create intent** — parse, preview the execution plan, and submit an intent.
- **My intents** — list persisted intents with live polling.
- **Status** — API health/ready and current orchestrator state.

### End-to-end CLI demo on a local Anvil node

```bash
# 1. Start a local Anvil node
anvil

# 2. Deploy DelegationVerifier + DelegationVault
cd contracts
forge script script/DeployDelegationVault.s.sol \
  --rpc-url http://localhost:8545 \
  --broadcast \
  --sender 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Save the DelegationVault address printed by the script (example below).
export VAULT=0x0165878A594ca255338adfa4d48449f69242Eb8F

# 3. Deposit native ETH into the vault for the owner
cast send $VAULT "deposit()" --value 10ether \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# 4. Execute a natural-language intent with a real Noir proof
cargo run -p interfaces --bin otter_cli -- execute \
  "swap 1000 USDC for ETH on Uniswap" \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --vault $VAULT \
  --delegate
```

The `--delegate` flag registers the delegation on-chain before executing. The CLI
signs the delegation locally, generates an UltraHonk proof with Noir + Barretenberg,
and submits `executeWithProof` to the vault.

You can also generate and verify a proof independently:

```bash
# Generate a proof for an intent (writes proof.bin + public_inputs.bin)
cargo run -p interfaces --bin otter_cli -- prove \
  "lend 1000 USDC on Aave" \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --output-dir ./tmp

# Verify the proof on-chain (view call against the verifier linked to the vault)
cargo run -p interfaces --bin otter_cli -- verify-onchain \
  --proof ./tmp/proof.bin \
  --public-inputs ./tmp/public_inputs.bin \
  --rpc-url http://localhost:8545 \
  --vault $VAULT \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

### Automated ZKP end-to-end demo

A single script starts Anvil, deploys the contracts, deposits ETH, generates a
real UltraHonk proof and submits it to the vault:

```bash
export BB_BIN="$HOME/.bb/bb"
./lab/zkp_e2e.sh
```

Prerequisites: `anvil`, `forge`, `cast`, `nargo`, `bb` and the Rust toolchain.

### REST API

The Rust backend exposes a minimal Axum server (`otter_api`) on port `3001`
(configurable via `OTTER_API_PORT`).

```bash
# Start the API server
cargo run -p interfaces --bin otter_api

# Parse an intent
curl -X POST http://localhost:3001/api/v1/intents/parse \
  -H 'Content-Type: application/json' \
  -d '{"text":"lend 1000 USDC on Aave"}'

# Submit an intent to the orchestrator
curl -X POST http://localhost:3001/api/v1/intents \
  -H 'Content-Type: application/json' \
  -d '{"text":"swap 1 ETH for USDC on Uniswap"}'

# List active intents
curl http://localhost:3001/api/v1/intents
```

### Frontend

A React 18 + Vite frontend lives in `frontend/`. It talks to the Rust backend
over HTTP instead of shelling out to the CLI.

```bash
# 1. Start the API server (in a separate terminal)
cargo run -p interfaces --bin otter_api

# 2. Start the frontend
cd frontend
npm install

# Dev server (Vite proxies /api to http://localhost:3001)
npm run dev

# Production build
npm run build
```

The frontend includes:
- Wallet connection via RainbowKit + wagmi/viem
- A simple intent input form
- Calls `/api/v1/intents/parse` relative to the frontend origin (Vite and nginx
  proxy `/api/*` to the Rust backend)

The API URL can be overridden with `VITE_OTTER_API_URL` (default: empty).

Open [http://localhost:5173](http://localhost:5173), connect your wallet, type an
intent like *"lend 1000 USDC on Aave"* and click **Parse intent**.

### Docker Compose (local production-like)

A `docker-compose.yml` starts the API server and the React + Vite frontend
together, with SQLite persistence in a Docker volume.

```bash
# Build and start everything
docker compose up --build

# Or detached
docker compose up -d --build

# View logs
docker compose logs -f

# The API is available at http://localhost:3001
# The frontend is available at http://localhost:3000
```

Services:
- `api` — Rust Axum backend (`otter_api`), persists intents in `/data/otter.db`
- `frontend` — React 18 + Vite app served by nginx; nginx proxies `/api/*` to
  the backend service

Copy `.env.example` to `.env` and fill in the values. For a full Sepolia
deployment with on-chain execution, see [`DEPLOYMENT.md`](DEPLOYMENT.md).

### Protocol Adapters

Otter abstracts DeFi protocols behind domain traits so the orchestrator can build
execution plans without hard-coding contract logic.

```rust
use domain::protocols::{LendingProtocol, DexProtocol, ProtocolRegistry};
use domain::models::intent::{LendingType, DexType};
use infrastructure::protocols::{AaveAdapter, UniswapAdapter};

let rpc_url = std::env::var("OTTER_RPC_URL").unwrap_or_else(|_| "https://rpc.sepolia.org".to_string());

let aave = AaveAdapter::sepolia(&rpc_url).unwrap();
let uniswap = UniswapAdapter::sepolia(&rpc_url).unwrap();

let mut registry = ProtocolRegistry::new();
registry
    .register_lending(LendingType::Aave, &aave)
    .register_dex(DexType::Uniswap, &uniswap);

// Read on-chain supply APY
let apy = aave.get_apy(&Asset::Usdc).unwrap();

// Get a swap quote from Uniswap QuoterV2
let out = uniswap.get_quote(&Asset::Eth, &Asset::Usdc, 1_000_000).unwrap();

// Build real Aave/Uniswap calldata
let supply_tx = aave.supply(&Asset::Usdc, 1_000_000).unwrap();
let swap_tx = uniswap.swap(&Asset::Eth, &Asset::Usdc, 1_000_000, 100).unwrap();
```

Supported adapters:
- **Aave V3** — reads `getReserveData` and encodes `supply`, `withdraw`, `borrow`, `repay`.
- **Uniswap V3** — reads quotes via `QuoterV2` and encodes `exactInputSingle` via the SwapRouter.

Integration tests against a live testnet RPC can be run with:

```bash
OTTER_TEST_RPC_URL=https://rpc.sepolia.org cargo test -p infrastructure --test protocol_integration
```

---

## Backlog & Issue Tracking

The complete backlog lives in [`BACKLOG.md`](BACKLOG.md). It contains:
- 580 user stories organized by wave and epic
- Statut par story : [FAIT] / [EN COURS] / [EN ATTENTE] / [CUT] / [FUTUR]
- Scope tags: MVP / Future / Cut

### Sync stories to GitHub Issues

```bash
# 1. Create labels (run once)
./scripts/setup-labels.sh

# 2. Preview what would be created
./scripts/sync-issues.sh

# 3. Create all MVP issues
./scripts/sync-issues.sh --create

# 4. Or create only one wave at a time
./scripts/sync-issues.sh --create --wave 2
```

---

## License

TBD — See [US-559](BACKLOG.md#epic-101-open-source-release)

---

> *Built with 🦀 Rust, 🔒 Noir, and 🧮 zero-knowledge proofs.*
