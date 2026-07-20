# Otter — Product Specification

> **Previously known as "Otter"**
>
> *Describe your DeFi strategy in plain English. Delegate with cryptographic limits. The agent executes across chains, captures MEV, and pays you. Share your strategy. Copy others. Every action is proven valid. The vault is proven solvent. The proof is forever.*

---

## 1. Product Thesis

Otter is a **trustless DeFi automation protocol** where users describe strategies in natural language, delegate execution authority with signed limits, and an agent executes them across multiple chains. Every action requires a zero-knowledge proof proving it respects the user's delegation. The agent captures MEV from execution and rebates it to users. The vault periodically proves its solvency without revealing individual balances.

**Why this matters:**
- Current automation (Gelato, Keep3r) requires **code trust** — you trust the operator's smart contracts.
- Current intent platforms (Aperture, UniswapX) are **one-shot execution** with no persistent delegation.
- Otter combines **natural language input**, **persistent delegation with user-defined constraints**, **cryptographic enforcement**, **MEV rebates**, **proof-of-solvency**, and **social strategy sharing** — none of which exist together today.

---

## 2. User Flow

### 2.1 Connect & Deposit
1. User connects wallet (MetaMask / Rainbow / WalletConnect)
2. Deposits USDC / ETH into the **StrategyVault** smart contract
3. Vault holds funds but cannot move them without a valid ZKP

### 2.2 Create an Intent
1. User navigates to **Create Intent** — a large textarea
2. Types: *"Lend 1000 USDC on Aave if yield > 3%"*
3. UI shows live parsing feedback:
   - **Detected:** Lend 1000 USDC on Aave
   - **Condition:** if yield > 3%
   - **Current yield:** 2.8% — *en attente...*
4. User can edit the parsed JSON manually if the LLM misinterpreted

### 2.3 Delegate & Sign
1. User clicks **"Set Delegation"**
2. Configures limits:
   - Max lend per action: 5,000 USDC
   - Max swap per action: 2,000 USDC
   - Allowed protocols: Aave, Compound, Uniswap
   - Expiry: 30 days
   - Agent selection: browse agent marketplace by reputation/bond
3. Signs delegation message with wallet (EIP-712)
4. Delegation is stored on-chain in the vault

### 2.4 Monitoring & Execution
1. Backend agent monitors conditions (yield, price, gas) every 60s
2. When condition is met:
   - Constructs ProposedIntent
   - Fetches user's Delegation from chain
   - Generates ZKP proving intent respects delegation
   - Captures MEV from execution flow
   - Submits transaction to vault with proof attached
3. Vault verifies proof → executes action → distributes MEV rebate

### 2.5 Real-Time Status
User sees live updates:
- **Monitoring** → checking yield every 60s
- **Condition Met** → yield is now 3.2%
- **Generating Proof** → 1.2s (847 constraints)
- **Submitted** → 0xabc...def
- **Confirmed** → 1000 USDC lent on Aave (+ 0.45 USDC MEV rebate)

### 2.6 Social Layer
- **Share Strategy:** Publish delegation + intent as a shareable link
- **Copy Strategy:** Others delegate to the same agent with one click
- **Leaderboard:** Top agents by proof count, yield generated, uptime
- **Follow:** Get notified when someone publishes a new strategy

---

## 3. Architecture

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
│       │ Uniswap  │  │ (Arbitrum│  L2)       │
│       └──────────┘  └──────────┘            │
└──────────────────────────────────────────────┘
```

---

## 4. Core Components

### 4.1 Natural Language Intent Layer
- **Local LLM** (llama.cpp, GGUF models) parses English into structured `Intent`
- **Regex fallback** for common patterns when LLM is uncertain
- **Live feedback** in UI as user types
- **Manual override** — edit parsed JSON before confirming

### 4.2 ZKP Delegation Circuit (Noir)
**Public Inputs:**
- Delegation hash (committed on-chain)
- ProposedIntent (intent_type, amount, protocol, target_contract)
- Current timestamp
- Nonce

**Private Inputs (Witness):**
- Agent private key (to sign delegation)
- Full delegation message

**Circuit Constraints:**
```
1. signature_valid(delegation_hash, agent_pubkey, agent_signature)
2. intent_type ∈ delegation.allowed_intents
3. amount ≤ delegation.max_amounts[intent_type]
4. protocol ∈ delegation.allowed_protocols
5. target_contract == protocol_registry[protocol]
6. timestamp < delegation.expiry
7. nonce == delegation.nonce
```

**Server-side proving** for speed (~1-3s). Proof verification happens on-chain.

### 4.3 StrategyVault (Solidity)
- Accepts deposits (ETH/ERC20)
- Stores delegations (user → agent → limits)
- Verifies ZKP before executing any action
- Executes via protocol adapters (Aave, Compound, Uniswap)
- Emits events: `Deposited`, `DelegationCreated`, `ActionExecuted`, `MEVRebated`

### 4.4 MEV Capture Module
- Agent submits transactions via MEV-protected channels (Flashbots Protect, MEV-Blocker)
- Captures arbitrage/backrun value from execution flow
- MEV split proven in ZK:
  - 50% to user
  - 40% to agent
  - 10% to protocol
- Users see MEV rebate in real-time status

### 4.5 Proof-of-Solvency
- Vault periodically generates ZKP: `sum(user_deposits) ≤ vault_assets`
- Proves solvency without revealing individual balances or positions
- Published on-chain for anyone to verify
- UI shows: *"Vault solvency verified 2 hours ago"*

### 4.6 Social / Strategy Layer
- **Strategy Registry:** On-chain registry of published strategies
- **Copy-Trading:** Delegate to a published strategy with one click
- **Creator Fees:** Strategy creator earns 0.1% on copy volume
- **Leaderboard:** Agents ranked by proofs submitted, yield generated, MEV captured
- **Follow:** Subscribe to strategy updates via WebSocket

### 4.7 Multi-Chain Support
- **Primary:** Ethereum mainnet
- **L2:** Arbitrum (lower gas, faster execution)
- **Cross-chain intents:** *"Lend on Aave Arbitrum if yield > Mainnet"*
- Chain selection in delegation limits

---

## 5. Trust Model

| Component | Trust Assumption | Risk |
|-----------|-----------------|------|
| **Vault Contract** | Audited Solidity | Smart contract bug |
| **Noir Circuit** | Public, auditable | Circuit bug / missing constraint |
| **Server** | Liveness only | Downtime, censorship |
| **Wallet** | User holds keys | Key compromise |
| **Agent** | Economic bond (stake) | Slashing if proof invalid |

**What users do NOT trust:**
- The server with their funds
- The server to act honestly
- The server to keep data private

**What ZKP guarantees:**
- Agent cannot exceed delegation limits (math-enforced)
- Agent cannot forge proofs (cryptographically impossible)
- MEV split is correct (proven in circuit)
- Vault is solvent (periodic ZK attestation)

---

## 6. Revenue Model

| Stream | Rate | Mechanism |
|--------|------|-----------|
| **Protocol fee** | 0.1% of executed volume | Taken by vault before execution |
| **MEV share** | 10% of captured MEV | Protocol portion of MEV split |
| **Strategy fees** | 0.1% of copy volume | Creator fee on copied strategies |

**User economics:**
- Deposit 10,000 USDC
- Agent executes lend strategy
- MEV captured: 5 USDC
- User receives: 2.50 USDC (50%)
- Agent keeps: 2.00 USDC (40%)
- Protocol: 0.50 USDC (10%)
- Net cost to user: negative (they got paid)

---

## 7. Competitive Positioning

| | Gelato | Aperture | UniswapX | Otter |
|--|--------|----------|----------|-------|
| Natural Language | Non | Oui | Non | Oui |
| Automation | Oui | Non | | Oui |
| Delegation | Non | | Non | Oui |
| Cryptographic Enforcement | Non | | Oui (signed intents) | Oui (ZKP) |
| MEV Rebates | Non | | Non | Oui |
| Proof-of-Solvency | Non | | Non | Oui |
| Social / Copy | Non | | Non | Oui |
| Multi-Chain | Oui | Non | | Oui |

**Otter's moat:** The only platform combining user-defined natural language intents, persistent cryptographic delegation, MEV rebates, proof-of-solvency, and social strategy sharing.

---

## 8. What's In Scope (MVP)

### Vague 0 — Setup [FAIT]
- Workspace, CI/CD, hexagonal architecture

### Vague 1 — Intent Parsing
- Local LLM integration (llama.cpp)
- Regex fallback parser
- Strategy planner (Aave, Compound, Uniswap)
- Use cases: ParseIntent, PlanExecution, ValidateIntent, EvaluateCondition
- **IN:** CLI + Web UI intent input

### Vague 2 — ZKP Delegation
- Noir circuit: EdDSA signature + delegation constraints
- Rust ↔ Noir integration (NoirAdapter)
- DelegationVerifier Solidity contract
- StrategyVault with ZKP verification
- **IN:** Per-delegation revoke
- **IN:** Proof explorer in UI

### Vague 5 — Blockchain Integration
- Ethereum adapter (Alloy)
- Wallet management (keystore, signing)
- Aave / Compound / Uniswap adapters
- **IN:** Multi-chain (Ethereum + Arbitrum)
- **IN:** MEV capture module
- **IN:** Proof-of-solvency circuit

### Vague 6 — Orchestrator
- State machine (IDLE → MONITORING → PROVING → EXECUTING → CONFIRMED)
- Event bus (tokio channels)
- Monitoring loop (yield, price, gas)
- Error handling + retry logic
- **IN:** Agent marketplace + reputation
- **IN:** Economic bonding + slashing

### Vague 6.5 — Web UI (React + Vite)
- Wallet connection (RainbowKit + wagmi)
- Intent creation screen (live parsing)
- Delegation wizard
- Dashboard (portfolio, active intents, history)
- Agent marketplace (browse, stake, delegate)
- Real-time status (WebSocket)
- **IN:** Social layer (share, copy, leaderboard, follow)
- **IN:** Proof explorer / verification UI
- **IN:** Dark/light mode

### Vague 7 — Production
- REST API (Axum)
- WebSocket server
- Docker deployment
- Structured logging (JSON)
- Prometheus metrics

---

## 9. What's Explicitly Out (For Now)

| Feature | Why Out |
|---------|---------|
| **FHE encrypted calculations** | Too slow for real-time DeFi. May be added as Labs demo later. |
| **Encrypted mempool / threshold encryption** | Requires infrastructure equivalent to Shutter Network. Out of scope for MVP. |
| **Browser-side proving** | Too slow (~5-15s). Server-side proving is real ZKP; contract verifies. Browser proving = v2 advanced mode. |
| **Cross-chain bridges** | Stick to Ethereum + Arbitrum native. Bridges add security risk. |
| **Advanced protocols** (GMX, perps, options) | Complex, niche. Add after core is solid. |
| **Mobile app** | Responsive web is enough for MVP. |
| **Governance token** | Controversial, distracting. Focus on product first. |

---

## 10. Success Metrics

| Metric | Target |
|--------|--------|
| Testnet proofs generated | 1,000+ |
| Mainnet transactions executed | 100+ |
| Average proof generation time | < 3s |
| Average MEV rebate per action | > gas cost |
| Strategies published | 50+ |
| Vault solvency proofs | Daily |
| Agent uptime (primary) | > 99% |

---

*Locked in. Build the thing.*
