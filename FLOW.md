# Otter — User Flows & Diagrams

> Complete visual guide to every interaction in the Otter protocol.

---

## 1. Main Flow — End to End

The full journey from connecting a wallet to receiving a confirmed execution with MEV rebate.

```mermaid
flowchart TD
    A[User connects wallet] --> B[Deposits USDC into Vault]
    B --> C[Types intent: "Lend 1000 USDC if yield > 3%"]
    C --> D[LLM parses intent]
    D --> E[User reviews parsed intent]
    E --> F[User sets delegation limits]
    F --> G[User signs delegation with wallet]
    G --> H[Delegation stored on-chain]
    H --> I[Agent monitors conditions]
    I --> J{Yield > 3%?}
    J -->|No| I
    J -->|Yes| K[Agent generates ZKP]
    K --> L[Agent captures MEV]
    L --> M[Agent submits tx to Vault]
    M --> N[Vault verifies ZKP]
    N -->|Valid| O[Vault executes on Aave]
    N -->|Invalid| P[Transaction reverts]
    O --> Q[MEV split: 50% user / 40% agent / 10% protocol]
    Q --> R[User sees confirmation in UI]
    R --> S[Permanent proof recorded on-chain]
```

---

## 2. Flow by Role

### 2.1 User Flow

```mermaid
flowchart LR
    subgraph User
        A[Connect Wallet] --> B[Dashboard]
        B --> C[Create Intent]
        B --> D[Manage Delegations]
        B --> E[Browse Marketplace]
        B --> F[View History]
        
        C --> C1[Type natural language]
        C1 --> C2[Review parsed intent]
        C2 --> C3[Set limits & expiry]
        C3 --> C4[Sign delegation]
        C4 --> C5[See live status]
        
        D --> D1[Revoke delegation]
        D --> D2[Edit limits]
        
        E --> E1[View agent reputation]
        E1 --> E2[Delegate to agent]
        
        F --> F1[Download proofs]
        F --> F2[Verify solvency]
    end
```

### 2.2 Agent Flow

```mermaid
flowchart LR
    subgraph Agent
        A[Register on Marketplace] --> B[Stake ETH bond]
        B --> C[Receive delegations]
        C --> D[Monitor conditions]
        D --> E{Condition met?}
        E -->|No| D
        E -->|Yes| F[Generate ZKP]
        F --> G[Capture MEV]
        G --> H[Submit to Vault]
        H --> I{Proof valid?}
        I -->|Yes| J[Earn 40% MEV fee]
        I -->|No| K[Slashed! Bond lost]
    end
```

### 2.3 Vault Contract Flow

```mermaid
flowchart TD
    A[Receive deposit] --> B[Store user balance]
    B --> C[Receive delegation]
    C --> D[Store delegation params]
    D --> E[Receive execution tx]
    E --> F[Verify ZKP]
    F --> G{Valid?}
    G -->|No| H[Revert]
    G -->|Yes| I[Check nonce]
    I --> J{Nonce OK?}
    J -->|No| H
    J -->|Yes| K[Check expiry]
    K --> L{Not expired?}
    L -->|No| H
    L -->|Yes| M[Execute protocol call]
    M --> N[Split MEV]
    N --> O[Emit events]
```

---

## 3. Feature Flows

### 3.1 ZKP Delegation Proof

```mermaid
sequenceDiagram
    participant Agent
    participant Prover as Noir Prover
    participant Vault
    participant Verifier as On-chain Verifier

    Agent->>Prover: intent_type, amount, protocol, timestamp, nonce
    Agent->>Prover: delegation_hash, agent_pubkey, agent_signature
    Prover->>Prover: Check intent_type ∈ allowed_intents
    Prover->>Prover: Check amount ≤ max_amount
    Prover->>Prover: Check protocol ∈ allowed_protocols
    Prover->>Prover: Check timestamp < expiry
    Prover->>Prover: Check nonce == expected_nonce
    Prover->>Prover: Verify EdDSA signature
    Prover->>Agent: proof_bytes
    Agent->>Vault: submit(proof_bytes, public_inputs)
    Vault->>Verifier: verify(proof_bytes, public_inputs)
    Verifier->>Vault: true / false
```

### 3.2 MEV Capture & Rebate

```mermaid
flowchart TD
    A[Agent detects arbitrage opportunity] --> B[Build bundle via Flashbots]
    B --> C[Include user intent + backrun]
    C --> D[Submit to block builder]
    D --> E[Block mined]
    E --> F[MEV extracted]
    F --> G[Calculate split]
    G --> H[50% to User Vault balance]
    G --> I[40% to Agent reward]
    G --> J[10% to Protocol treasury]
    H --> K[Update user claimable amount]
    K --> L[User claims rebate]
```

### 3.3 Proof-of-Solvency

```mermaid
flowchart TD
    A[Daily cron triggers] --> B[Read all user deposits]
    B --> C[Read vault asset balances]
    C --> D[Generate ZKP: sum(deposits) ≤ assets]
    D --> E[Publish proof on-chain]
    E --> F[Update UI timestamp]
    F --> G[User verifies independently]
```

### 3.4 Social — Strategy Sharing

```mermaid
flowchart TD
    A[User creates winning strategy] --> B[Click "Publish Strategy"]
    B --> C[Set creator fee: 0.1%]
    C --> D[Mint Strategy NFT]
    D --> E[Strategy appears on Marketplace]
    E --> F[Other users see it]
    F --> G[User clicks "Copy Strategy"]
    G --> H[Auto-fill delegation params]
    H --> I[User signs new delegation]
    I --> J[Creator earns fee on every copy execution]
```

### 3.5 Agent Marketplace

```mermaid
flowchart TD
    A[Developer runs agent software] --> B[Stake 10 ETH bond]
    B --> C[Register on marketplace]
    C --> D[Start executing intents]
    D --> E[Accumulate proof count]
    E --> F[Generate yield for users]
    F --> G[Reputation score increases]
    G --> H[More users delegate]
    H --> I[Agent earns more MEV fees]
    
    J[Agent submits invalid proof] --> K[Vault detects]
    K --> L[Slash bond: 10 ETH burned]
    L --> M[Agent removed from marketplace]
```

---

## 4. Error & Edge Case Flows

### 4.1 Invalid Proof

```mermaid
flowchart TD
    A[Agent generates bad proof] --> B[Submits to Vault]
    B --> C[Vault calls verifier]
    C --> D[Verifier returns false]
    D --> E[Transaction reverts]
    E --> F[Agent reputation -1]
    F --> G{Repeat offenses?}
    G -->|Yes| H[Slash agent bond]
    G -->|No| I[Continue monitoring]
```

### 4.2 Delegation Expired

```mermaid
flowchart TD
    A[Agent tries to execute] --> B[Proof generation starts]
    B --> C[Circuit checks timestamp]
    C --> D{timestamp < expiry?}
    D -->|No| E[Proof generation fails]
    E --> F[Agent stops monitoring]
    F --> G[Notify user: delegation expired]
```

### 4.3 Revoke Delegation

```mermaid
flowchart TD
    A[User clicks "Revoke"] --> B[Sign revoke transaction]
    B --> C[Vault increments nonce]
    C --> D[Old delegation invalidated]
    D --> E[Agent's next proof fails nonce check]
    E --> F[Agent stops executing for this user]
```

### 4.4 Server Downtime (Liveness Failure)

```mermaid
flowchart TD
    A[Agent server goes offline] --> B[Conditions not monitored]
    B --> C[Intent never executes]
    C --> D[User sees "Agent offline" status]
    D --> E{Agent offline > 24h?}
    E -->|Yes| F[User revokes delegation]
    E -->|No| G[Wait for agent recovery]
```

---

## 5. Data Flow

### 5.1 Intent → Proof → Execution

```mermaid
flowchart LR
    subgraph OffChain["Off-Chain"]
        A[Natural Language] --> B[LLM Parser]
        B --> C[Structured Intent]
        C --> D[Strategy Planner]
        D --> E[ExecutionPlan]
        E --> F[Noir Prover]
        F --> G[Proof Bytes]
    end

    subgraph OnChain["On-Chain"]
        G --> H[Vault Contract]
        H --> I[Verifier Contract]
        I --> J{Valid?}
        J -->|Yes| K[Aave/Compound/Uniswap]
        J -->|No| L[Revert]
    end

    subgraph UserSide["User Side"]
        M[Wallet Signature] --> N[Delegation Message]
        N --> F
    end
```

### 5.2 State Transitions

```mermaid
stateDiagram-v2
    [*] --> IDLE
    IDLE --> MONITORING: User creates intent
    MONITORING --> ANALYZING: Condition met
    ANALYZING --> DECIDING: Plan validated
    DECIDING --> PROVING: Execute approved
    PROVING --> SUBMITTING: Proof generated
    SUBMITTING --> CONFIRMING: Tx submitted
    CONFIRMING --> IDLE: Tx confirmed
    CONFIRMING --> ERROR: Tx failed
    ERROR --> IDLE: Retry / Abort
    MONITORING --> ERROR: Timeout
```

---

## 6. Multi-Chain Flow

```mermaid
flowchart TD
    A[User on Ethereum] --> B[Create cross-chain intent]
    B --> C["Lend on Aave Arbitrum if yield > Mainnet"]
    C --> D[Agent monitors both chains]
    D --> E{Arbitrum yield > Mainnet?}
    E -->|No| D
    E -->|Yes| F[Generate ZKP]
    F --> G[Submit to Arbitrum Vault]
    G --> H[Execute on Arbitrum Aave]
    H --> I[User receives aTokens on Arbitrum]
```

---

## 7. Quick Reference — All Flows

| Flow | Trigger | Actor | Output |
|------|---------|-------|--------|
| **Intent Creation** | User types text | User | Parsed intent + delegation |
| **Condition Monitoring** | Every 60s | Agent | ConditionMet event |
| **ZKP Generation** | Condition met | Agent | proof_bytes |
| **MEV Capture** | During execution | Agent | Extracted value |
| **Vault Execution** | ZKP verified | Vault | Protocol tx + events |
| **MEV Rebate** | Post-execution | Vault | User claimable amount |
| **Solvency Proof** | Daily cron | Vault | On-chain attestation |
| **Strategy Publish** | User clicks | User | Strategy NFT |
| **Copy Strategy** | User clicks | User | New delegation |
| **Agent Register** | Developer stakes | Agent | Marketplace listing |
| **Agent Slash** | Invalid proof | Vault | Bond burned |
| **Revoke** | User signs | User | Nonce incremented |

---

*See [`PRODUCT.md`](PRODUCT.md) for detailed product specification and [`BACKLOG.md`](BACKLOG.md) for implementation stories.*
