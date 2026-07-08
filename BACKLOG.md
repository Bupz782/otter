# Otter Project Backlog

> Previously known as "Metis". A trustless DeFi automation protocol using ZKP delegation, MEV rebates, and proof-of-solvency.

---

## Progress Overview

| Wave | Title | Scope | Stories | Status | Progress |
|------|-------|-------|---------|--------|----------|
| 0 | Setup & Architecture Foundation | **MVP** | 14 | 🚧 In Progress | ~80% |
| 1 | Intent Parsing & LLM Integration | **MVP** | 41 | 🚧 In Progress | ~65% |
| 2 | ZKP - Delegation with Intent Verification | **MVP** | 55 | ⏳ Not Started | ~5% |
| 5 | Blockchain Integration & Protocol Adapters | **MVP** | 68 | ⏳ Not Started | ~2% |
| 6 | Orchestrator & Integrated Flow | **MVP** | 58 | ⏳ Not Started | ~3% |
| 6.5 | Web UI (React + Vite) | **MVP** | 64 | ⏳ Not Started | ~0% |
| 7 | Production-Ready (APIs, Monitoring, Deployment) | **MVP** | 84 | ⏳ Not Started | ~0% |
| 8 | Advanced Features & Polish | **FUTURE** | 42 | 🔮 Future | ~0% |
| 9 | Research & Whitepaper | **FUTURE** | 25 | 🔮 Future | ~0% |
| 10 | Open Source & Community | **FUTURE** | 22 | 🔮 Future | ~0% |
| **Total** | | | **481** | | **~18%** |
| **MVP Total** | | | **396** | | |

**Legend**: ✅ Done | 🚧 In Progress / Partial | ⏳ Pending | ⬛ Cut | 🔮 Future

---

## 🌊 Vague 0 : Setup & Architecture Foundation (2 semaines)

### Epic 0.1 : Environnement de développement

- ✅ **US-001** : En tant que dev, je veux installer Rust nightly avec tous les toolchains nécessaires
- ✅ **US-002** : En tant que dev, je veux installer Noir (nargo) pour développer des circuits ZK
- ✅ **US-003** : En tant que dev, je veux installer Foundry (forge, cast, anvil) pour les smart contracts
- ✅ **US-004** : En tant que dev, je veux un workspace Cargo avec tous les crates définis
- ✅ **US-005** : En tant que dev, je veux un dossier `lab/` pour expérimenter sans polluer l'architecture

### Epic 0.2 : Architecture squelette

- ✅ **US-006** : En tant que système, je veux la structure hexagonale (domain/application/infrastructure/interfaces)
- 🚧 **US-007** : En tant que système, je veux tous les ports définis (traits vides) : ZkpPort, FhePort, MempoolPort, BlockchainPort, IntentParserPort
  - *Note*: Ports exist but are mostly empty stubs.
- 🚧 **US-008** : En tant que système, je veux un `orchestrator.rs` squelette avec state machine enum
  - *Note*: `State` enum exists with only `Idle` variant. `Orchestrator` struct is empty.
- 🚧 **US-009** : En tant que système, je veux un système de logs structuré (tracing + tracing-subscriber)
  - *Note*: `observability/logging.rs` exists but `init_logging()` is empty.
- ✅ **US-010** : En tant que dev, je veux un `LEARNING.md` pour documenter ce que j'apprends chaque semaine

### Epic 0.3 : CI/CD basique

- ✅ **US-011** : En tant que dev, je veux un GitHub Actions workflow qui run `cargo test`
- ✅ **US-012** : En tant que dev, je veux un workflow qui run `cargo clippy` (lints)
- ✅ **US-013** : En tant que dev, je veux un workflow qui vérifie le formatting (`cargo fmt --check`)
- ⏳ **US-014** : En tant que dev, je veux un script `scripts/benchmarks.sh` pour mesurer les performances

---

## 🌊 Vague 1 : Intent Parsing & LLM Integration (6 semaines)

### Epic 1.1 : Domain - Intent Model

- ✅ **US-015** : En tant que système, je veux une enum `Intent` couvrant les actions DeFi de base (Lend, Borrow, Swap, Stake)
- ✅ **US-016** : En tant que système, je veux une struct `Condition` pour les intents conditionnels (yield > X%, price > Y)
- ✅ **US-017** : En tant que système, je veux une enum `Protocol` représentant les protocoles supportés (Aave, Compound, Uniswap, Curve)
- ✅ **US-018** : En tant que système, je veux une struct `ExecutionPlan` décomposant un intent en steps exécutables
- ✅ **US-019** : En tant que système, je veux valider qu'un `Intent` est bien formé (amounts > 0, protocols valides, etc.)

### Epic 1.2 : Rule-based Parser (v1)

- ✅ **US-020** : En tant que parser, je veux extraire un intent "lend X ASSET on PROTOCOL" via regex
- ✅ **US-021** : En tant que parser, je veux extraire un intent "swap X ASSET for Y ASSET"
- ✅ **US-022** : En tant que parser, je veux extraire un intent "borrow X ASSET with Y COLLATERAL"
- ✅ **US-023** : En tant que parser, je veux extraire un intent "stake X ASSET on PROTOCOL"
- ✅ **US-024** : En tant que parser, je veux extraire une condition "if METRIC > VALUE"
- ✅ **US-025** : En tant que parser, je veux combiner intent + condition : "lend X if yield > Y%"
- ✅ **US-026** : En tant que parser, je veux gérer les montants avec unités (1000 USDC, 1.5 ETH, 50%)
- ✅ **US-027** : En tant que parser, je veux retourner des erreurs précises (UnknownAsset, InvalidAmount, UnknownProtocol)

### Epic 1.3 : LLM Integration (Claude API)

- 🚧 **US-028** : En tant qu'infra, je veux un module `llm_client.rs` wrappant l'API Claude
  - *Note*: LLM module exists but wraps local llama.cpp, not Claude API.
- 🚧 **US-029** : En tant que parser LLM, je veux générer un prompt structuré avec schema JSON
  - *Note*: `PromptBuilder` exists but not tailored for Claude JSON schema.
- 🚧 **US-030** : En tant que parser LLM, je veux parser la réponse JSON de Claude
  - *Note*: `ResponseParser` exists but for local model tokens.
- ⏳ **US-031** : En tant que parser LLM, je veux valider que la sortie LLM respecte le schema Intent
- ⏳ **US-032** : En tant que parser LLM, je veux un fallback : si LLM échoue → rule-based parser
- ⏳ **US-033** : En tant que parser LLM, je veux logger les tokens consommés (coût monitoring)
- ⏳ **US-034** : En tant que parser LLM, je veux gérer les rate limits (exponential backoff)
- ⏳ **US-035** : En tant que parser LLM, je veux cacher les réponses pour éviter de re-parser le même texte

### Epic 1.4 : Strategy Planner

- ✅ **US-036** : En tant que planner, je veux décomposer `Intent::Lend` en ExecutionPlan (approve + supply)
- ✅ **US-037** : En tant que planner, je veux décomposer `Intent::Swap` en ExecutionPlan (approve + swap)
- ✅ **US-038** : En tant que planner, je veux décomposer `Intent::Borrow` en ExecutionPlan (approve collateral + borrow)
- ✅ **US-039** : En tant que planner, je veux gérer les multi-step : "lend then stake" → 2 plans séquentiels
- ✅ **US-040** : En tant que planner, je veux vérifier les conditions AVANT de créer le plan
- ✅ **US-041** : En tant que planner, je veux calculer le gas estimé total du plan
- ✅ **US-042** : En tant que planner, je veux détecter les impossibilités (ex: borrow sans collateral)

### Epic 1.5 : Application - Use Cases

- ✅ **US-043** : En tant qu'application, je veux un use case `ParseIntent(text)` → Intent
- ✅ **US-044** : En tant qu'application, je veux un use case `PlanExecution(intent)` → ExecutionPlan
- ✅ **US-045** : En tant qu'application, je veux un use case `ValidateIntent(intent)` → Result<(), ValidationError>
- ✅ **US-046** : En tant qu'application, je veux un use case `EvaluateCondition(condition)` → bool

### Epic 1.6 : CLI - Intent Testing

- ⏳ **US-047** : En tant que user, je veux exécuter `metis parse "lend 1000 USDC on Aave"` et voir l'intent parsé
- ⏳ **US-048** : En tant que user, je veux exécuter `metis plan <intent>` et voir l'ExecutionPlan
- ⏳ **US-049** : En tant que user, je veux un mode `--llm` vs `--rules` pour comparer les parsers
- ⏳ **US-050** : En tant que user, je veux voir les erreurs de parsing avec suggestions de correction

### Epic 1.7 : Tests Intent Layer

- ✅ **US-051** : En tant que dev, je veux des tests unitaires pour 20+ intents valides
- ✅ **US-052** : En tant que dev, je veux des tests pour intents invalides (error handling)
- ✅ **US-053** : En tant que dev, je veux des tests pour intents complexes multi-conditions
- ⏳ **US-054** : En tant que dev, je veux mocker Claude API pour tester sans coût
- ⏳ **US-055** : En tant que dev, je veux benchmarker : temps de parsing rule-based vs LLM

---

## 🌊 Vague 2 : ZKP - Delegation with Intent Verification (8 semaines)

### Epic 2.1 : Noir Fundamentals

- ⏳ **US-056** : En tant que learner, je veux lire ZK Learning cours 1-2 (circuits arithmétiques)
- ⏳ **US-057** : En tant que dev, je veux écrire un circuit "hello world" (prouver connaissance d'un secret)
- ⏳ **US-058** : En tant que dev, je veux compiler avec nargo et générer une preuve
- ⏳ **US-059** : En tant que dev, je veux vérifier la preuve avec `nargo verify`
- ⏳ **US-060** : En tant que dev, je veux mesurer : # contraintes, temps de preuve, taille de la preuve

### Epic 2.2 : EdDSA Signature Verification

- ⏳ **US-061** : En tant que learner, je veux comprendre EdDSA Baby JubJub (courbe ZK-friendly)
- ⏳ **US-062** : En tant que circuit, je veux vérifier une signature EdDSA sur un message fixe
- ⏳ **US-063** : En tant que circuit, je veux paramétrer le message (input public)
- ⏳ **US-064** : En tant que circuit, je veux tester avec 10 paires (valid/invalid signatures)
- ⏳ **US-065** : En tant que dev, je veux benchmarker la vérification EdDSA en circuit

### Epic 2.3 : Delegation Message Structure

- ⏳ **US-066** : En tant que circuit, je veux définir `DelegationMessage` struct (agent_pubkey, allowed_intents, max_amounts, allowed_protocols, expiry, nonce)
- ⏳ **US-067** : En tant que circuit, je veux hasher le `DelegationMessage` pour obtenir un digest
- ⏳ **US-068** : En tant que circuit, je veux vérifier qu'une signature EdDSA couvre ce digest
- ⏳ **US-069** : En tant que circuit, je veux supporter jusqu'à 10 intent types différents
- ⏳ **US-070** : En tant que circuit, je veux supporter jusqu'à 5 protocols whitelistés

### Epic 2.4 : Intent Authorization Circuit

- ⏳ **US-071** : En tant que circuit, je veux une struct `ProposedIntent` (intent_type, amount, protocol, target_contract)
- ⏳ **US-072** : En tant que circuit, je veux vérifier que intent_type est dans allowed_intents (bitfield check)
- ⏳ **US-073** : En tant que circuit, je veux vérifier que amount <= max_amounts[intent_type]
- ⏳ **US-074** : En tant que circuit, je veux vérifier que protocol est dans allowed_protocols (array membership)
- ⏳ **US-075** : En tant que circuit, je veux vérifier que target_contract correspond au protocol
- ⏳ **US-076** : En tant que circuit, je veux vérifier que current_timestamp < expiry
- ⏳ **US-077** : En tant que circuit, je veux vérifier que nonce_provided == nonce_expected

### Epic 2.5 : Circuit Optimization

- ⏳ **US-078** : En tant que dev, je veux profiler le circuit (quelles opérations coûtent le plus de contraintes)
- ⏳ **US-079** : En tant que dev, je veux réduire les contraintes de 20% (optimizations)
- ⏳ **US-080** : En tant que dev, je veux tester le circuit sur 100+ combinaisons d'inputs
- ⏳ **US-081** : En tant que dev, je veux vérifier qu'aucun cas edge ne passe (security audit)

### Epic 2.6 : Rust ↔ Noir Integration

- ⏳ **US-082** : En tant qu'infra, je veux un `NoirAdapter` implémentant `ZkpPort`
- ⏳ **US-083** : En tant qu'infra, je veux générer une preuve depuis Rust (serialize inputs → call nargo → parse proof bytes)
- ⏳ **US-084** : En tant qu'infra, je veux vérifier une preuve off-chain depuis Rust (Barretenberg verifier)
- ⏳ **US-085** : En tant qu'infra, je veux gérer les erreurs (ProofGenerationFailed, InvalidWitness, Timeout)
- ⏳ **US-086** : En tant qu'infra, je veux cacher les proving keys en mémoire (éviter recompilation)
- ⏳ **US-087** : En tant qu'infra, je veux logger les temps : witness generation, proving, verification

### Epic 2.7 : Domain Integration

- ⏳ **US-088** : En tant que domain, je veux une struct `Delegation` avec tous les champs typés
- ⏳ **US-089** : En tant que domain, je veux une méthode `delegation.can_execute(intent)` → bool (business logic)
- ⏳ **US-090** : En tant que domain, je veux une struct `DelegationProof` wrappant les proof bytes
- ⏳ **US-091** : En tant que domain, je veux `ZkpPort` trait complet (`prove_delegation`, `verify_delegation_offchain`)

### Epic 2.8 : Application - Delegation Use Cases

- ⏳ **US-092** : En tant qu'application, je veux un use case `CreateDelegation(params)` → Delegation
- ⏳ **US-093** : En tant qu'application, je veux un use case `SignDelegation(delegation, privkey)` → Signature
- ⏳ **US-094** : En tant qu'application, je veux un use case `ProveIntent(delegation, intent)` → DelegationProof
- ⏳ **US-095** : En tant qu'application, je veux un use case `VerifyProofOffchain(proof)` → bool

### Epic 2.9 : Smart Contracts - Verifier

- ⏳ **US-096** : En tant que dev, je veux exporter le verifier Solidity depuis Noir
- ⏳ **US-097** : En tant que dev, je veux créer `DelegationVerifier.sol` wrappant le verifier Noir
- ⏳ **US-098** : En tant que contract, je veux une fonction `verifyDelegation(bytes proof, bytes32[] publicInputs)` → bool
- ⏳ **US-099** : En tant que dev, je veux déployer sur testnet Sepolia
- ⏳ **US-100** : En tant que dev, je veux tester on-chain : 10 valid proofs → true, 10 invalid → false

### Epic 2.10 : CLI - Delegation Flow

- ⏳ **US-101** : En tant que user, je veux `metis keygen` pour générer une keypair EdDSA
- ⏳ **US-102** : En tant que user, je veux `metis delegate --agent <pubkey> --max-lend 5000 --protocols aave,compound --expiry 2025-12-31`
- ⏳ **US-103** : En tant que user, je veux voir la délégation créée avec hash et signature
- ⏳ **US-104** : En tant que user, je veux `metis prove --delegation <file> --intent "lend 1000 USDC on Aave"`
- ⏳ **US-105** : En tant que user, je veux voir la preuve générée (temps, taille, hash)
- ⏳ **US-106** : En tant que user, je veux `metis verify-onchain --proof <file>` pour tester le contract

### Epic 2.11 : Tests ZKP Layer

- ⏳ **US-107** : En tant que dev, je veux des tests unitaires du circuit (20+ cas valid/invalid)
- ⏳ **US-108** : En tant que dev, je veux des tests de génération de preuve (benchmarks)
- ⏳ **US-109** : En tant que dev, je veux des tests E2E : delegation → parse intent → prove → verify onchain
- ⏳ **US-110** : En tant que dev, je veux tester les cas limites (expiry à la seconde près, nonce edge cases)


## 🌊 Vague 5 : Blockchain Integration & Protocol Adapters (8 semaines)

### Epic 5.1 : Domain - Blockchain Abstractions

- ⏳ **US-217** : En tant que domain, je veux une struct `Transaction` (from, to, value, data, gas)
- ⏳ **US-218** : En tant que domain, je veux une struct `TransactionReceipt` (hash, block, status, gas_used)
- ⏳ **US-219** : En tant que domain, je veux `BlockchainPort` trait (`send_tx`, `get_balance`, `call_contract`, `estimate_gas`)
- ⏳ **US-220** : En tant que domain, je veux `WalletPort` trait (`sign_tx`, `get_address`, `get_nonce`)

### Epic 5.2 : Infrastructure - Ethereum Adapter

- ⏳ **US-221** : En tant qu'infra, je veux un `EthereumAdapter` utilisant ethers-rs ou alloy
- ⏳ **US-222** : En tant qu'infra, je veux me connecter à un RPC (Infura/Alchemy/Ankr)
- ⏳ **US-223** : En tant qu'infra, je veux lire le balance d'une adresse
- ⏳ **US-224** : En tant qu'infra, je veux estimer le gas d'une transaction
- ⏳ **US-225** : En tant qu'infra, je veux envoyer une transaction signée
- ⏳ **US-226** : En tant qu'infra, je veux attendre la confirmation (polling ou WebSocket)
- ⏳ **US-227** : En tant qu'infra, je veux gérer les erreurs (revert, out of gas, nonce too low)
- ⏳ **US-228** : En tant qu'infra, je veux gérer les nonces automatiquement (anticiper les pending txs)
- ⏳ **US-229** : En tant qu'infra, je veux retry avec plus de gas si la tx échoue pour cette raison
- ⏳ **US-230** : En tant qu'infra, je veux supporter les réseaux testnet (Sepolia, Holesky)

### Epic 5.3 : Infrastructure - Wallet Management

- ⏳ **US-231** : En tant qu'infra, je veux générer une keypair secp256k1 pour l'agent
- ⏳ **US-232** : En tant qu'infra, je veux stocker la clé privée dans un keystore chiffré (scrypt/pbkdf2)
- ⏳ **US-233** : En tant qu'infra, je veux charger la clé depuis le keystore avec password
- ⏳ **US-234** : En tant qu'infra, je veux signer une transaction avec la clé privée
- ⏳ **US-235** : En tant qu'infra, je veux dériver plusieurs adresses depuis une seed (HD wallet optionnel)

### Epic 5.4 : Protocol Adapters - Aave

- ⏳ **US-236** : En tant qu'infra, je veux un `AaveAdapter` wrappant les contracts Aave v3
- ⏳ **US-237** : En tant qu'adapter Aave, je veux une méthode `get_apy(asset)` → f64
- ⏳ **US-238** : En tant qu'adapter Aave, je veux une méthode `supply(asset, amount)` → Transaction
- ⏳ **US-239** : En tant qu'adapter Aave, je veux une méthode `withdraw(asset, amount)` → Transaction
- ⏳ **US-240** : En tant qu'adapter Aave, je veux une méthode `borrow(asset, amount, collateral)` → Transaction
- ⏳ **US-241** : En tant qu'adapter Aave, je veux une méthode `repay(asset, amount)` → Transaction
- ⏳ **US-242** : En tant qu'adapter Aave, je veux gérer l'approve préalable (ERC20)

### Epic 5.5 : Protocol Adapters - Compound

- ⏳ **US-243** : En tant qu'infra, je veux un `CompoundAdapter` wrappant Compound v3
- ⏳ **US-244** : En tant qu'adapter Compound, je veux les mêmes méthodes que Aave (interface unifiée)
- ⏳ **US-245** : En tant qu'adapter Compound, je veux gérer les cTokens (mint/redeem)

### Epic 5.6 : Protocol Adapters - Uniswap

- ⏳ **US-246** : En tant qu'infra, je veux un `UniswapAdapter` wrappant SwapRouter v3
- ⏳ **US-247** : En tant qu'adapter Uniswap, je veux une méthode `get_quote(from, to, amount)` → expected_output
- ⏳ **US-248** : En tant qu'adapter Uniswap, je veux une méthode `swap(from, to, amount, slippage)` → Transaction
- ⏳ **US-249** : En tant qu'adapter Uniswap, je veux calculer le path optimal (direct ou via WETH)
- ⏳ **US-250** : En tant qu'adapter Uniswap, je veux gérer le wrapping ETH → WETH si nécessaire

### Epic 5.7 : Protocol Adapters - Curve (staking)

- ⏳ **US-251** : En tant qu'infra, je veux un `CurveAdapter` pour staker des LP tokens
- ⏳ **US-252** : En tant qu'adapter Curve, je veux une méthode `stake(lp_token, amount)` → Transaction
- ⏳ **US-253** : En tant qu'adapter Curve, je veux une méthode `unstake(lp_token, amount)` → Transaction
- ⏳ **US-254** : En tant qu'adapter Curve, je veux une méthode `claim_rewards()` → Transaction

### Epic 5.8 : Domain - Protocol Abstraction

- ⏳ **US-255** : En tant que domain, je veux un trait `LendingProtocol` unifié (supply, withdraw, borrow, repay, get_apy)
- ⏳ **US-256** : En tant que domain, je veux un trait `DexProtocol` unifié (swap, get_quote)
- ⏳ **US-257** : En tant que domain, je veux un trait `StakingProtocol` unifié (stake, unstake, claim)
- ⏳ **US-258** : En tant que domain, je veux un `ProtocolRegistry` pour mapper protocol name → adapter

### Epic 5.9 : Application - Execution Use Cases

- ⏳ **US-259** : En tant qu'application, je veux un use case `ExecuteIntent(intent)` → TransactionReceipt
- ⏳ **US-260** : En tant qu'application, je veux un use case `SimulateExecution(intent)` → GasEstimate
- ⏳ **US-261** : En tant qu'application, je veux un use case `ApproveToken(token, spender, amount)` → Transaction
- ⏳ **US-262** : En tant qu'application, je veux gérer les multi-step executions (approve then execute)

### Epic 5.10 : Smart Contracts - Strategy Vault

- ⏳ **US-263** : En tant que dev, je veux un contrat `StrategyVault.sol` qui détient les fonds users
- ⏳ **US-264** : En tant que vault, je veux accepter des dépôts (deposit ETH/ERC20)
- ⏳ **US-265** : En tant que vault, je veux permettre les retraits (withdraw)
- ⏳ **US-266** : En tant que vault, je veux autoriser l'agent à exécuter des actions (via delegation proof)
- ⏳ **US-267** : En tant que vault, je veux vérifier la preuve ZKP avant chaque action
- ⏳ **US-268** : En tant que vault, je veux intégrer le `DelegationVerifier`
- ⏳ **US-269** : En tant que vault, je veux gérer les nonces (anti-replay)
- ⏳ **US-270** : En tant que vault, je veux émettre des events (Deposited, Withdrawn, ActionExecuted)

### Epic 5.11 : Smart Contracts - Deployment & Testing

- ⏳ **US-271** : En tant que dev, je veux déployer tous les contracts sur Sepolia
- ⏳ **US-272** : En tant que dev, je veux un script de setup (deploy + configure)
- ⏳ **US-273** : En tant que dev, je veux tester le vault avec Foundry (unit tests)
- ⏳ **US-274** : En tant que dev, je veux tester l'intégration vault + verifier (valid proof → success, invalid → revert)

### Epic 5.12 : CLI - Blockchain Operations

- ⏳ **US-275** : En tant que user, je veux `metis wallet create` pour générer un wallet agent
- ⏳ **US-276** : En tant que user, je veux `metis wallet balance` pour voir le solde
- ⏳ **US-277** : En tant que user, je veux `metis execute --intent "lend 1000 USDC on Aave"` en réel
- ⏳ **US-278** : En tant que user, je veux un mode `--dry-run` qui simule sans envoyer
- ⏳ **US-279** : En tant que user, je veux voir les logs de transaction (hash, block, gas, status)
- ⏳ **US-280** : En tant que user, je veux `metis vault deposit --amount 5000 --asset USDC`

### Epic 5.13 : Tests Blockchain Layer

- ⏳ **US-281** : En tant que dev, je veux des tests avec Anvil (fork testnet local)
- ⏳ **US-282** : En tant que dev, je veux tester chaque adapter de protocol (mock contracts ou fork)
- ⏳ **US-283** : En tant que dev, je veux un test E2E : deposit → delegate → execute intent → withdraw
- ⏳ **US-284** : En tant que dev, je veux tester les error cases (insufficient balance, revert, etc.)

### Epic 5.14 : MEV Capture & Rebate

- ⏳ **US-581** : En tant qu'infra, je veux intégrer Flashbots Protect / MEV-Blocker pour soumettre les transactions
- ⏳ **US-582** : En tant qu'infra, je veux capturer le MEV (arbitrage, backrunning) du flow d'exécution
- ⏳ **US-583** : En tant que circuit, je veux prouver le split MEV : 50% user, 40% agent, 10% protocol
- ⏳ **US-584** : En tant que vault, je veux distribuer les rebates MEV aux users après chaque exécution
- ⏳ **US-585** : En tant que user, je veux voir le MEV rebâté par action dans le dashboard
- ⏳ **US-586** : En tant que dev, je veux benchmarker le MEV capturé vs le gas dépensé

### Epic 5.15 : Proof-of-Solvency

- ⏳ **US-587** : En tant que circuit, je veux prouver `sum(deposits) ≤ vault_assets` sans révéler les balances
- ⏳ **US-588** : En tant que vault, je veux générer une preuve de solvabilité périodiquement (daily)
- ⏳ **US-589** : En tant que vault, je veux publier la preuve de solvabilité on-chain
- ⏳ **US-590** : En tant que user, je veux voir le statut de solvabilité du vault ("Vérifié il y a 2h")
- ⏳ **US-591** : En tant que user, je veux télécharger / vérifier indépendamment la preuve de solvabilité
- ⏳ **US-592** : En tant que dev, je veux tester la preuve de solvabilité sur 100+ scénarios de deposits/withdrawals


## 🌊 Vague 6 : Orchestrator & Integrated Flow (10 semaines)

### Epic 6.1 : State Machine Design

- ⏳ **US-285** : En tant que système, je veux une FSM (Finite State Machine) avec états : IDLE, MONITORING, ANALYZING, DECIDING, PROVING, ENCRYPTING, SUBMITTING, CONFIRMING, ERROR
- ⏳ **US-286** : En tant que système, je veux définir les transitions valides entre états
- ⏳ **US-287** : En tant que système, je veux logger chaque transition d'état
- ⏳ **US-288** : En tant que système, je veux gérer les timeouts (si bloqué dans un état trop longtemps)
- ⏳ **US-289** : En tant que système, je veux rollback sur erreur (revenir à IDLE ou MONITORING)

### Epic 6.2 : Event Bus Architecture

- ⏳ **US-290** : En tant que système, je veux un event bus basé sur tokio channels (mpsc)
- ⏳ **US-291** : En tant que système, je veux définir les events : PriceUpdated, ConditionMet, IntentParsed, ProofGenerated, TransactionSubmitted, TransactionConfirmed, Error
- ⏳ **US-292** : En tant que système, je veux un dispatcher qui route les events vers les handlers
- ⏳ **US-293** : En tant que système, je veux que les modules publient des events (découplage)
- ⏳ **US-294** : En tant que système, je veux logger tous les events (audit trail)

### Epic 6.3 : Orchestrator Core

- ⏳ **US-295** : En tant qu'orchestrator, je veux une boucle principale qui réagit aux events
- ⏳ **US-296** : En tant qu'orchestrator, je veux maintenir l'état global (current_state, active_intents, delegations)
- ⏳ **US-297** : En tant qu'orchestrator, je veux coordonner les appels aux différents ports (ZKP, Blockchain)
- ⏳ **US-298** : En tant qu'orchestrator, je veux gérer les dépendances entre étapes (PROVING doit finir avant ENCRYPTING)

### Epic 6.4 : Monitoring Loop

- ⏳ **US-299** : En tant qu'orchestrator, je veux un loop qui check les conditions périodiquement (ex: toutes les 60s)
- ⏳ **US-300** : En tant qu'orchestrator, je veux fetcher les prix via PriceOraclePort
- ⏳ **US-301** : En tant qu'orchestrator, je veux évaluer les conditions des intents actifs
- ⏳ **US-302** : En tant qu'orchestrator, je veux publier ConditionMet event si une condition est vraie

### Epic 6.5 : Decision Making

- ⏳ **US-303** : En tant qu'orchestrator, je veux recevoir ConditionMet event → transition vers ANALYZING
- ⏳ **US-304** : En tant qu'orchestrator, je veux appeler le strategy planner pour créer ExecutionPlan
- ⏳ **US-305** : En tant qu'orchestrator, je veux vérifier que le plan respecte la délégation (business rules)
- ⏳ **US-306** : En tant qu'orchestrator, je veux transition vers DECIDING → choisir d'exécuter ou attendre

### Epic 6.6 : Proof Generation Flow

- ⏳ **US-307** : En tant qu'orchestrator, je veux transition vers PROVING
- ⏳ **US-308** : En tant qu'orchestrator, je veux appeler `ZkpPort.prove_delegation(intent)`
- ⏳ **US-309** : En tant qu'orchestrator, je veux gérer les erreurs de proof generation (retry ou abort)
- ⏳ **US-310** : En tant qu'orchestrator, je veux publier ProofGenerated event avec la preuve

### Epic 6.7 : Transaction Encryption Flow

- ⏳ **US-311** : En tant qu'orchestrator, je veux transition vers ENCRYPTING
- ⏳ **US-312** : En tant qu'orchestrator, je veux construire la transaction (via protocol adapter)
- ⏳ **US-313** : En tant qu'orchestrator, je veux attacher la preuve ZKP à la transaction
- ⏳ **US-314** : En tant qu'orchestrator, je veux préparer la transaction chiffrée pour soumission
- ⏳ **US-315** : En tant qu'orchestrator, je veux publier TransactionEncrypted event

### Epic 6.8 : Submission & Confirmation Flow

- ⏳ **US-316** : En tant qu'orchestrator, je veux transition vers SUBMITTING
- ⏳ **US-317** : En tant qu'orchestrator, je veux soumettre la transaction à la blockchain
- ⏳ **US-318** : En tant qu'orchestrator, je veux publier TransactionSubmitted event avec tx_hash
- ⏳ **US-319** : En tant qu'orchestrator, je veux transition vers CONFIRMING
- ⏳ **US-320** : En tant qu'orchestrator, je veux attendre la confirmation on-chain (polling ou events)
- ⏳ **US-321** : En tant qu'orchestrator, je veux publier TransactionConfirmed event avec receipt
- ⏳ **US-322** : En tant qu'orchestrator, je veux transition vers IDLE (prêt pour next iteration)

### Epic 6.9 : Error Handling

- ⏳ **US-323** : En tant qu'orchestrator, je veux catcher toutes les erreurs des ports
- ⏳ **US-324** : En tant qu'orchestrator, je veux transition vers ERROR state avec contexte
- ⏳ **US-325** : En tant qu'orchestrator, je veux publier Error event avec détails
- ⏳ **US-326** : En tant qu'orchestrator, je veux retry automatiquement selon le type d'erreur
- ⏳ **US-327** : En tant qu'orchestrator, je veux notifier le user si erreur non récupérable

### Epic 6.10 : Full Flow Integration

- ⏳ **US-328** : En tant que système, je veux un test E2E du flow complet :
  - User écrit intent dans dapp
  - Intent parsé (LLM)
  - Delegated validée (business rules)
  - Condition monitorée
  - Condition met → Decision
  - Proof generated (ZKP)
  - Transaction encrypted
  - Submitted to blockchain
  - Decrypted & executed on-chain
  - Confirmation reçue
- ⏳ **US-329** : En tant que système, je veux logger chaque étape du flow avec timestamps
- ⏳ **US-330** : En tant que système, je veux mesurer le temps total du flow (SLA)

### Epic 6.11 : Multi-Intent Management

- ⏳ **US-331** : En tant qu'orchestrator, je veux gérer plusieurs intents actifs simultanément
- ⏳ **US-332** : En tant qu'orchestrator, je veux prioriser les intents (ordre d'exécution)
- ⏳ **US-333** : En tant qu'orchestrator, je veux éviter les conflits (2 intents modifiant même asset)
- ⏳ **US-334** : En tant qu'orchestrator, je veux supporter les intents récurrents ("rebalance every week")

### Epic 6.12 : CLI - Orchestrator Control

- ⏳ **US-335** : En tant que user, je veux `metis start` pour lancer l'orchestrator en daemon
- ⏳ **US-336** : En tant que user, je veux `metis stop` pour arrêter proprement
- ⏳ **US-337** : En tant que user, je veux `metis status` pour voir l'état actuel (current_state, active_intents)
- ⏳ **US-338** : En tant que user, je veux `metis logs --follow` pour voir les events en temps réel

### Epic 6.13 : Tests Orchestrator

- ⏳ **US-339** : En tant que dev, je veux mocker tous les ports pour tester l'orchestrator isolément
- ⏳ **US-340** : En tant que dev, je veux tester chaque transition de state machine
- ⏳ **US-341** : En tant que dev, je veux tester les scénarios d'erreur (proof fails, tx reverts, timeout)
- ⏳ **US-342** : En tant que dev, je veux un test E2E avec tous les composants réels

---

## 🌊 Vague 6.5 : DAPP Frontend (8 semaines)

### Epic 6.5.1 : Frontend Setup

- ⏳ **US-343** : En tant que dev, je veux initialiser un projet Next.js 14 (App Router)
- ⏳ **US-344** : En tant que dev, je veux configurer Tailwind CSS + shadcn/ui
- ⏳ **US-345** : En tant que dev, je veux configurer RainbowKit pour wallet connection
- ⏳ **US-346** : En tant que dev, je veux configurer wagmi/viem pour interactions blockchain
- ⏳ **US-347** : En tant que dev, je veux configurer TypeScript strict mode

### Epic 6.5.2 : Wallet Connection

- ⏳ **US-348** : En tant que user, je veux connecter mon wallet (MetaMask, WalletConnect, Coinbase)
- ⏳ **US-349** : En tant que user, je veux voir mon adresse et balance dans la navbar
- ⏳ **US-350** : En tant que user, je veux switcher de réseau (Mainnet ↔ Sepolia)
- ⏳ **US-351** : En tant que user, je veux déconnecter mon wallet

### Epic 6.5.3 : Intent Input Interface

- ⏳ **US-352** : En tant que user, je veux une page "Create Intent" avec un textarea
- ⏳ **US-353** : En tant que user, je veux des suggestions d'intents (autocomplete ou exemples)
- ⏳ **US-354** : En tant que user, je veux un bouton "Parse Intent" qui appelle le backend
- ⏳ **US-355** : En tant que user, je veux voir l'intent parsé (structure JSON formatée)
- ⏳ **US-356** : En tant que user, je veux voir les erreurs de parsing avec suggestions
- ⏳ **US-357** : En tant que user, je veux éditer manuellement l'intent parsé (JSON editor)

### Epic 6.5.4 : Intent Validation UI

- ⏳ **US-358** : En tant que user, je veux voir les validations de l'intent :
  - ✓ Protocol supporté
  - ✓ Asset existant
  - ✓ Amount valide
  - ⚠️ Yield actuel : 2.8% (condition : >3%, pas encore remplie)
- ⏳ **US-359** : En tant que user, je veux voir les permissions requises :
  - Approve 1000 USDC pour Aave Pool
  - Execute sur Aave Pool contract
- ⏳ **US-360** : En tant que user, je veux voir le gas estimé
- ⏳ **US-361** : En tant que user, je veux voir les risques (smart contract risk, impermanent loss, etc.)

### Epic 6.5.5 : Delegation Setup UI

- ⏳ **US-362** : En tant que user, je veux configurer les limites de délégation (wizard step-by-step)
- ⏳ **US-363** : En tant que user, je veux sélectionner les protocols autorisés (checkboxes : Aave, Compound, Uniswap, Curve)
- ⏳ **US-364** : En tant que user, je veux définir les max amounts par intent type (sliders)
- ⏳ **US-365** : En tant que user, je veux définir une date d'expiration (date picker)
- ⏳ **US-366** : En tant que user, je veux voir un résumé de la délégation avant signature
- ⏳ **US-367** : En tant que user, je veux signer la délégation avec MetaMask (EdDSA key derivation via EIP-712 ou custom)
- ⏳ **US-368** : En tant que user, je veux télécharger le fichier de délégation (.json)

### Epic 6.5.6 : Agent Dashboard

- ⏳ **US-369** : En tant que user, je veux voir une page "Dashboard" avec vue d'ensemble
- ⏳ **US-370** : En tant que user, je veux voir mon portfolio actuel (assets + balances)
- ⏳ **US-371** : En tant que user, je veux voir les intents actifs (table : intent, status, condition, actions)
- ⏳ **US-372** : En tant que user, je veux voir l'historique des actions (timeline : actions executées avec tx hash)
- ⏳ **US-373** : En tant que user, je veux voir les métriques :
  - Total value locked
  - Gas spent
  - Proofs generated
  - Success rate
- ⏳ **US-374** : En tant que user, je veux filtrer l'historique (par date, par intent, par status)

### Epic 6.5.7 : Intent Status & Monitoring

- ⏳ **US-375** : En tant que user, je veux voir le statut détaillé d'un intent :
  - Intent: Lend 1000 USDC on Aave (conditional)
  - Status: ⏳ Monitoring
  - Condition: Yield > 3%
  - Current yield: 2.8%
  - Last check: 2 minutes ago
  - Next check: in 58 seconds
- ⏳ **US-376** : En tant que user, je veux voir le statut d'une transaction en cours :
  - Transaction: Lend 1000 USDC
  - Stage: PROVING (2/8)
  - Proof generation: 3.2s ✓
  - Encryption: in progress...
- ⏳ **US-377** : En tant que user, je veux un indicateur visuel du flow (stepper UI : IDLE → MONITORING → ... → CONFIRMED)

### Epic 6.5.8 : Real-time Updates

- ⏳ **US-378** : En tant que système, je veux un WebSocket server dans le backend
- ⏳ **US-379** : En tant que frontend, je veux me connecter au WebSocket pour recevoir les events
- ⏳ **US-380** : En tant que user, je veux recevoir des notifications push :
  - 🔔 Condition met: Yield is now 3.2%
  - 🔔 Intent executing: Generating proof...
  - 🔔 Transaction submitted: 0xabc...def
  - ✅ Transaction confirmed: 1000 USDC lent on Aave
- ⏳ **US-381** : En tant que user, je veux voir les notifications dans une sidebar (toast + history)


- ⏳ **US-387** : En tant que user, je veux voir toutes mes délégations actives (table)
- ⏳ **US-388** : En tant que user, je veux voir les détails d'une délégation (protocols, limits, expiry, nonce)
- ⏳ **US-389** : En tant que user, je veux révoquer une délégation (increments nonce on-chain)
- ⏳ **US-390** : En tant que user, je veux créer une nouvelle délégation (wizard)
- ⏳ **US-391** : En tant que user, je veux voir l'historique des preuves générées pour chaque délégation

### Epic 6.5.11 : Analytics & Charts

- ⏳ **US-392** : En tant que user, je veux un graphique de la valeur du portfolio dans le temps (line chart)
- ⏳ **US-393** : En tant que user, je veux un graphique de l'allocation (pie chart : % ETH, % DAI, etc.)
- ⏳ **US-394** : En tant que user, je veux un graphique des yields générés (bar chart par protocol)
- ⏳ **US-395** : En tant que user, je veux un graphique du gas spent (timeline)

### Epic 6.5.12 : Settings & Configuration

- ⏳ **US-396** : En tant que user, je veux une page Settings pour configurer :
  - RPC endpoint (custom node)
  - Gas price strategy (slow/medium/fast)
  - Monitoring interval (30s, 60s, 5min)
  - Notification preferences (email, push, none)
- ⏳ **US-397** : En tant que user, je veux exporter mes données (intents, history, delegations) en JSON
- ⏳ **US-398** : En tant que user, je veux un dark mode / light mode toggle

### Epic 6.5.13 : Mobile Responsive

- ⏳ **US-399** : En tant que user mobile, je veux que toutes les pages soient responsive
- ⏳ **US-400** : En tant que user mobile, je veux une navigation simplifiée (bottom nav ou hamburger)
- ⏳ **US-401** : En tant que user mobile, je veux pouvoir créer un intent (textarea adapté)
- ⏳ **US-402** : En tant que user mobile, je veux recevoir les notifications (push notifications via service worker)

### Epic 6.5.14 : Tests Frontend

- ⏳ **US-403** : En tant que dev, je veux des tests unitaires pour les composants (Vitest + React Testing Library)
- ⏳ **US-404** : En tant que dev, je veux des tests E2E (Playwright : connect wallet → create intent → delegate)
- ⏳ **US-405** : En tant que dev, je veux tester le WebSocket (mock server)
- ⏳ **US-406** : En tant que dev, je veux tester le WebSocket en temps réel (events flow)

---

## 🌊 Vague 7 : Production-Ready (APIs, Monitoring, Deployment) (6 semaines)

### Epic 7.1 : REST API

- ✅ **US-407** : En tant que backend, je veux un serveur HTTP Axum
- ✅ **US-408** : En tant qu'API, je veux exposer `POST /api/v1/intents/parse` (body: text → response: Intent)
- ✅ **US-409** : En tant qu'API, je veux exposer `POST /api/v1/intents` (create new intent)
- ✅ **US-410** : En tant qu'API, je veux exposer `GET /api/v1/intents` (list active intents)
- ✅ **US-411** : En tant qu'API, je veux exposer `GET /api/v1/intents/:id` (get intent details)
- ✅ **US-412** : En tant qu'API, je veux exposer `DELETE /api/v1/intents/:id` (cancel intent)
- ✅ **US-413** : En tant qu'API, je veux exposer `POST /api/v1/delegations` (create delegation)
- ✅ **US-414** : En tant qu'API, je veux exposer `GET /api/v1/delegations` (list delegations)
- ⏳ **US-415** : En tant qu'API, je veux exposer `POST /api/v1/delegations/:id/revoke` (revoke delegation)
- ⏳ **US-416** : En tant qu'API, je veux exposer `GET /api/v1/portfolio` (get portfolio state)
- ⏳ **US-417** : En tant qu'API, je veux exposer `GET /api/v1/history` (execution history)
- ✅ **US-418** : En tant qu'API, je veux exposer `GET /api/v1/metrics` (stats & analytics)

### Epic 7.2 : Authentication & Security

- ✅ **US-419** : En tant qu'API, je veux authentifier via signature de message (EIP-4361 Sign-In with Ethereum)
- ✅ **US-420** : En tant qu'API, je veux générer un JWT après authentification
- ✅ **US-421** : En tant qu'API, je veux vérifier le JWT sur chaque requête protégée
- ✅ **US-422** : En tant qu'API, je veux rate limiting (100 req/min par user)
- ✅ **US-423** : En tant qu'API, je veux CORS configuré correctement (whitelist domains)

### Epic 7.3 : gRPC API (optionnel)

- ⏳ **US-424** : En tant que backend, je veux un serveur gRPC (Tonic)
- ⏳ **US-425** : En tant qu'API gRPC, je veux définir le protobuf schema (Intent, Delegation, Transaction, etc.)
- ⏳ **US-426** : En tant qu'API gRPC, je veux exposer les mêmes méthodes que REST
- ⏳ **US-427** : En tant qu'API gRPC, je veux supporter le streaming (stream des events en temps réel)

### Epic 7.4 : WebSocket Server

- ✅ **US-428** : En tant que backend, je veux un WebSocket server (Axum WS)
- ✅ **US-429** : En tant que WS server, je veux accepter les connexions des clients
- ⏳ **US-430** : En tant que WS server, je veux authentifier les connexions (JWT dans handshake)
- ✅ **US-431** : En tant que WS server, je veux broadcaster les events aux clients connectés
- 🚧 **US-432** : En tant que WS server, je veux gérer les disconnections et reconnections

### Epic 7.5 : Monitoring & Observability

- ✅ **US-433** : En tant que système, je veux exposer des métriques Prometheus (`/metrics` endpoint)
- 🚧 **US-434** : En tant que système, je veux tracker les métriques :
  - Nombre d'intents actifs
  - Nombre de transactions soumises
  - Temps moyen de génération de preuve
  - Temps moyen de génération de preuve
  - Gas total dépensé
  - Success rate (% txs confirmées)
- ⏳ **US-435** : En tant que ops, je veux configurer Grafana pour visualiser les métriques
- ⏳ **US-436** : En tant que ops, je veux des dashboards :
  - Agent health (uptime, memory, CPU)
  - Transaction flow (funnel : intent → proof → submit → confirm)
  - Performance (latencies, throughput)

### Epic 7.6 : Logging Structured

- ✅ **US-437** : En tant que système, je veux logger en JSON (structured logging avec tracing-subscriber)
- 🚧 **US-438** : En tant que système, je veux inclure des contextes dans les logs (request_id, user_id, intent_id)
- ✅ **US-439** : En tant que système, je veux différents niveaux (ERROR, WARN, INFO, DEBUG, TRACE)
- ⏳ **US-440** : En tant que ops, je veux envoyer les logs à un aggregator (Loki, ElasticSearch, CloudWatch)

### Epic 7.7 : Configuration Management

- ✅ **US-441** : En tant que système, je veux charger la config depuis un fichier TOML
- ✅ **US-442** : En tant que système, je veux override la config avec des env vars (12-factor app)
- ✅ **US-443** : En tant que système, je veux valider la config au démarrage (fail fast si invalid)
- 🚧 **US-444** : En tant que système, je veux supporter différents environnements (dev, staging, prod)

### Epic 7.8 : Database (persistence)

- ✅ **US-445** : En tant que système, je veux persister les intents dans une DB (PostgreSQL/SQLite)
- ✅ **US-446** : En tant que système, je veux persister les delegations
- ✅ **US-447** : En tant que système, je veux persister l'historique des transactions
- ⏳ **US-448** : En tant que système, je veux persister les events de l'orchestrator (audit trail)
- ✅ **US-449** : En tant que système, je veux un `StoragePort` trait (`save_intent`, `get_intent`, `list_intents`, etc.)
- ✅ **US-450** : En tant qu'infra, je veux un `PostgresAdapter` et `SqliteAdapter` implémentant `StoragePort`
- ✅ **US-451** : En tant que système, je veux des migrations DB (sqlx ou diesel migrations)
- ⏳ **US-452** : En tant que système, je veux indexer les queries fréquentes (performance)

### Epic 7.9 : Backup & Recovery

- ⏳ **US-453** : En tant qu'ops, je veux backup automatique de la DB (cron job)
- ⏳ **US-454** : En tant qu'ops, je veux exporter les keystores (encrypted backups)
- ⏳ **US-455** : En tant que système, je veux un script de recovery (restore depuis backup)
- ⏳ **US-456** : En tant que système, je veux tester le recovery (disaster recovery drills)

### Epic 7.10 : Health Checks

- ✅ **US-457** : En tant que système, je veux un endpoint `GET /health` (retourne status: UP/DOWN)
- ✅ **US-458** : En tant que système, je veux checker les dépendances (DB, RPC node, Oracle, etc.)
- ✅ **US-459** : En tant que système, je veux un endpoint `GET /ready` (readiness probe pour Kubernetes)
- ⏳ **US-460** : En tant qu'ops, je veux des alertes si health check fail (PagerDuty, Slack)

### Epic 7.11 : Deployment - Docker

- ⏳ **US-461** : En tant que dev, je veux un Dockerfile multi-stage (build + runtime optimisé)
- ⏳ **US-462** : En tant que dev, je veux un docker-compose.yml pour dev local (agent + DB + monitoring)
- ⏳ **US-463** : En tant que dev, je veux builder les images pour différentes arches (amd64, arm64)
- ⏳ **US-464** : En tant que ops, je veux publier les images sur un registry (Docker Hub, GHCR)

### Epic 7.12 : Deployment - Kubernetes (optionnel)

- ⏳ **US-465** : En tant qu'ops, je veux des manifests Kubernetes (Deployment, Service, ConfigMap, Secret)
- ⏳ **US-466** : En tant qu'ops, je veux un Helm chart pour simplifier le déploiement
- ⏳ **US-467** : En tant qu'ops, je veux configurer l'autoscaling (HPA)
- ⏳ **US-468** : En tant qu'ops, je veux configurer le monitoring (Prometheus Operator)

### Epic 7.13 : CI/CD Pipeline

- ⏳ **US-469** : En tant que dev, je veux un workflow GitHub Actions pour build & test
- ⏳ **US-470** : En tant que dev, je veux un workflow pour publish les images Docker
- ⏳ **US-471** : En tant que dev, je veux un workflow pour déployer sur staging (auto)
- ⏳ **US-472** : En tant que dev, je veux un workflow pour déployer sur prod (manual approval)
- ⏳ **US-473** : En tant que dev, je veux des checks de qualité (coverage, clippy, audit)

### Epic 7.14 : Documentation Technique

- ⏳ **US-474** : En tant que dev, je veux un README.md complet (installation, usage, architecture)
- ⏳ **US-475** : En tant que dev, je veux générer la doc API avec OpenAPI/Swagger
- ⏳ **US-476** : En tant que dev, je veux documenter les circuits Noir (inputs, outputs, contraintes)
- ⏳ **US-477** : En tant que dev, je veux documenter les smart contracts (NatSpec)
- ⏳ **US-478** : En tant que dev, je veux un ARCHITECTURE.md avec diagrammes (C4 model)
- ⏳ **US-479** : En tant que dev, je veux un CONTRIBUTING.md pour les contributeurs
- ⏳ **US-480** : En tant que dev, je veux générer la rustdoc (`cargo doc`)

### Epic 7.15 : Security Audit

- ⏳ **US-481** : En tant que dev, je veux run `cargo audit` (check des vulnérabilités dans les deps)
- ⏳ **US-482** : En tant que dev, je veux scanner les images Docker (Trivy, Snyk)
- ⏳ **US-483** : En tant que dev, je veux faire un audit du circuit Noir (peer review ou professionnel)
- ⏳ **US-484** : En tant que dev, je veux faire un audit des smart contracts (Slither, Mythril)
- ⏳ **US-485** : En tant que dev, je veux un bug bounty program (post-launch)

### Epic 7.16 : Performance Optimization

- ⏳ **US-486** : En tant que dev, je veux profiler l'application (flamegraph, perf)
- ⏳ **US-487** : En tant que dev, je veux optimiser les hot paths (circuit compilation, proof generation)
- ⏳ **US-488** : En tant que dev, je veux cacher les résultats coûteux (proving keys, verification keys)
- ⏳ **US-489** : En tant que dev, je veux paralléliser les opérations indépendantes (rayon, tokio)
- ⏳ **US-490** : En tant que dev, je veux benchmarker et comparer (avant/après optimizations)


## 🌊 Vague 8 : Advanced Features & Polish (optionnel - 6 semaines) [FUTURE]

> **Post-MVP.** Features like cross-chain bridges, advanced protocols (GMX, perps), mobile app, and simulation mode.

### Epic 8.1 : Multi-User Support

- ⏳ **US-491** : En tant que système, je veux supporter plusieurs users simultanément
- ⏳ **US-492** : En tant que système, je veux isoler les données par user (row-level security)
- ⏳ **US-493** : En tant que système, je veux un user registry (mapping address → user_id)
- ⏳ **US-494** : En tant que système, je veux des quotas par user (rate limiting, max intents)

### Epic 8.2 : Social Features

- ⏳ **US-495** : En tant que user, je veux partager une stratégie publiquement (share link)
- ⏳ **US-496** : En tant que user, je veux copier la stratégie d'un autre user (template)
- ⏳ **US-497** : En tant que user, je veux voir un leaderboard (top performers)
- ⏳ **US-498** : En tant que user, je veux follow d'autres users (notifications de leurs actions)

### Epic 8.3 : Advanced Intent Features

- ⏳ **US-499** : En tant que user, je veux des intents récurrents : "rebalance every Monday at 9am"
- ⏳ **US-500** : En tant que user, je veux des intents avec stop-loss : "sell if price drops 10%"
- ⏳ **US-501** : En tant que user, je veux des intents composés complexes : "if X then Y else Z, repeat weekly"
- ⏳ **US-502** : En tant que user, je veux des intents avec priorités (high/medium/low)

### Epic 8.4 : Portfolio Insights (AI)

- ⏳ **US-503** : En tant que user, je veux des suggestions d'optimisation : "You could earn 0.5% more by..."
- ⏳ **US-504** : En tant que user, je veux une analyse de risque : "Your portfolio has 75% in stablecoins, low risk"
- ⏳ **US-505** : En tant que user, je veux des alertes proactives : "Yield on Aave dropped below 3%"
- ⏳ **US-506** : En tant que user, je veux un AI assistant conversationnel : "Ask Metis anything about your portfolio"

### Epic 8.5 : Cross-Chain Support

- ⏳ **US-507** : En tant que système, je veux supporter Arbitrum (L2)
- ⏳ **US-508** : En tant que système, je veux supporter Optimism (L2)
- ⏳ **US-509** : En tant que système, je veux supporter Polygon (sidechain)
- ⏳ **US-510** : En tant que système, je veux un bridge adapter pour cross-chain transfers
- ⏳ **US-511** : En tant que user, je veux des intents cross-chain : "Lend on Aave Arbitrum if yield > Mainnet"

### Epic 8.6 : Advanced Protocol Integrations

- ⏳ **US-512** : En tant que système, je veux supporter Balancer (AMM)
- ⏳ **US-513** : En tant que système, je veux supporter Yearn (vaults)
- ⏳ **US-514** : En tant que système, je veux supporter Lido (liquid staking)
- ⏳ **US-515** : En tant que système, je veux supporter GMX (perpetuals)
- ⏳ **US-516** : En tant que système, je veux un plugin system pour ajouter facilement de nouveaux protocols

### Epic 8.7 : Simulation Mode

- ⏳ **US-517** : En tant que user, je veux un mode simulation (paper trading)
- ⏳ **US-518** : En tant que user en simulation, je veux un portfolio virtuel avec fake tokens
- ⏳ **US-519** : En tant que user en simulation, je veux tester mes stratégies sans risque
- ⏳ **US-520** : En tant que user en simulation, je veux voir les performances projetées

### Epic 8.8 : Mobile App (React Native)

- ⏳ **US-521** : En tant que dev, je veux une app mobile React Native
- ⏳ **US-522** : En tant que user mobile, je veux me connecter avec WalletConnect
- ⏳ **US-523** : En tant que user mobile, je veux créer des intents (voice input optionnel)
- ⏳ **US-524** : En tant que user mobile, je veux recevoir des push notifications natives
- ⏳ **US-525** : En tant que user mobile, je veux voir mon dashboard (responsive native)

### Epic 8.9 : Compliance & Reporting

- ⏳ **US-526** : En tant que user, je veux exporter un rapport fiscal (CSV des gains/pertes)
- ⏳ **US-527** : En tant que user, je veux un rapport de compliance (toutes les actions avec timestamps)
- ⏳ **US-528** : En tant que système, je veux logger toutes les actions pour audit (immutable log)
- ⏳ **US-529** : En tant que système, je veux supporter des juridictions différentes (KYC optionnel)

### Epic 8.10 : Gamification

- ⏳ **US-530** : En tant que user, je veux des achievements : "First lend", "10 successful rebalances", etc.
- ⏳ **US-531** : En tant que user, je veux des badges visuels (NFTs optionnel)
- ⏳ **US-532** : En tant que user, je veux un level system (XP basé sur volume traité)
- ⏳ **US-533** : En tant que user, je veux des rewards (fee discounts pour high-level users)

---

## 🌊 Vague 9 : Research & Whitepaper (4 semaines) [FUTURE]

> **Post-MVP.** Academic publication and whitepaper after core product is live and battle-tested.

### Epic 9.1 : Whitepaper - Introduction

- ⏳ **US-534** : En tant qu'auteur, je veux écrire l'abstract (200 mots max)
- ⏳ **US-535** : En tant qu'auteur, je veux écrire l'introduction (problématique DeFi)
- ⏳ **US-536** : En tant qu'auteur, je veux expliquer les 3 problèmes (délégation, MEV, confidentialité)
- ⏳ **US-537** : En tant qu'auteur, je veux positionner Metis vs solutions existantes (related work)

### Epic 9.2 : Whitepaper - Architecture

- ⏳ **US-538** : En tant qu'auteur, je veux décrire l'architecture hexagonale
- ⏳ **US-539** : En tant qu'auteur, je veux expliquer le flow complet avec diagrammes
- ⏳ **US-540** : En tant qu'auteur, je veux détailler chaque composant (ZKP, MEV, Proof-of-Solvency)
- ⏳ **US-541** : En tant qu'auteur, je veux inclure des sequence diagrams (PlantUML)

### Epic 9.3 : Whitepaper - Cryptographie

- ⏳ **US-542** : En tant qu'auteur, je veux expliquer le circuit ZKP (EdDSA + delegation logic)
- ⏳ **US-543** : En tant qu'auteur, je veux expliquer le schéma MEV (capture, redistribution, preuve ZK)
- ⏳ **US-544** : En tant qu'auteur, je veux expliquer le threshold encryption (Shamir SSS)
- ⏳ **US-545** : En tant qu'auteur, je veux inclure les preuves de sécurité (modèle de menace)

### Epic 9.4 : Whitepaper - Évaluation

- ⏳ **US-546** : En tant qu'auteur, je veux présenter les benchmarks (temps de preuve, MEV, gas)
- ⏳ **US-547** : En tant qu'auteur, je veux comparer avec l'état de l'art (tableau comparatif)
- ⏳ **US-548** : En tant qu'auteur, je veux discuter les trade-offs (latency vs privacy)
- ⏳ **US-549** : En tant qu'auteur, je veux inclure des graphiques de performance

### Epic 9.5 : Whitepaper - Conclusion

- ⏳ **US-550** : En tant qu'auteur, je veux résumer les contributions
- ⏳ **US-551** : En tant qu'auteur, je veux discuter les limitations actuelles
- ⏳ **US-552** : En tant qu'auteur, je veux proposer des future works (extensions possibles)
- ⏳ **US-553** : En tant qu'auteur, je veux rédiger les remerciements et références

### Epic 9.6 : Publication & Diffusion

- ⏳ **US-554** : En tant qu'auteur, je veux publier sur arXiv (preprint)
- ⏳ **US-555** : En tant qu'auteur, je veux soumettre à une conférence (IEEE S&P, USENIX, CCS)
- ⏳ **US-556** : En tant qu'auteur, je veux créer un blog post vulgarisé (Medium, Mirror)
- ⏳ **US-557** : En tant qu'auteur, je veux créer une vidéo explicative (YouTube)
- ⏳ **US-558** : En tant qu'auteur, je veux présenter à des meetups (Ethereum, ZK, DeFi communities)

---

## 🌊 Vague 10 : Open Source & Community (ongoing) [FUTURE]

> **Post-MVP.** Community building, grants, and open-source release after product-market fit.

### Epic 10.1 : Open Source Release

- ⏳ **US-559** : En tant que maintainer, je veux choisir une licence (MIT, Apache 2.0, GPL)
- ⏳ **US-560** : En tant que maintainer, je veux nettoyer le code (remove secrets, TODOs)
- ⏳ **US-561** : En tant que maintainer, je veux créer un repo GitHub public
- ⏳ **US-562** : En tant que maintainer, je veux écrire un CHANGELOG.md
- ⏳ **US-563** : En tant que maintainer, je veux créer un CODE_OF_CONDUCT.md

### Epic 10.2 : Community Building

- ⏳ **US-564** : En tant que maintainer, je veux créer un Discord server
- ⏳ **US-565** : En tant que maintainer, je veux créer un forum de discussions (GitHub Discussions)
- ⏳ **US-566** : En tant que maintainer, je veux créer un Twitter account pour announcements
- ⏳ **US-567** : En tant que maintainer, je veux écrire des tutoriels (getting started, advanced usage)
- ⏳ **US-568** : En tant que maintainer, je veux organiser des AMAs (Ask Me Anything)

### Epic 10.3 : Contributor Onboarding

- ⏳ **US-569** : En tant que maintainer, je veux labeler les issues (good first issue, help wanted)
- ⏳ **US-570** : En tant que maintainer, je veux reviewer les PRs rapidement (SLA 48h)
- ⏳ **US-571** : En tant que maintainer, je veux remercier les contributors (CONTRIBUTORS.md)
- ⏳ **US-572** : En tant que maintainer, je veux créer une roadmap publique (GitHub Projects)

### Epic 10.4 : Grants & Funding

- ⏳ **US-573** : En tant que founder, je veux appliquer à des grants (Ethereum Foundation, Protocol Labs)
- ⏳ **US-574** : En tant que founder, je veux créer un Gitcoin grant (community funding)
- ⏳ **US-575** : En tant que founder, je veux explorer des VCs (si pivot vers startup)
- ⏳ **US-576** : En tant que founder, je veux lancer un token (governance, incentives) (optionnel, controversé)

### Epic 10.5 : Maintenance & Evolution

- ⏳ **US-577** : En tant que maintainer, je veux fixer les bugs reportés (issue triage)
- ⏳ **US-578** : En tant que maintainer, je veux mettre à jour les dépendances (Dependabot)
- ⏳ **US-579** : En tant que maintainer, je veux suivre les nouveaux protocoles DeFi (integrations)
- ⏳ **US-580** : En tant que maintainer, je veux suivre les avancées crypto (nouveaux schemes ZK)

---

## Notes & Recommendations

### Realistic Timeline for Solo Developer

Given the current progress (~15% complete), here is a realistic phased approach:

**Phase 1 — MVP Core (3–4 months)**
- Finish Vague 0 (tracing, benchmarks, orchestrator skeleton)
- Finish Vague 1 (Claude API integration, CLI commands, benchmarks)
- Vague 2 — ZKP delegation circuit + Rust integration + verifier contract + CLI
- Vague 5 lite — Basic Ethereum adapter + 1 protocol (Aave) + wallet
- Vague 6 lite — Orchestrator with state machine + event bus + basic flow

**Phase 2 — Privacy Layer (4–6 months)**

**Phase 3 — Production (3–4 months)**
- Vague 7 — REST API, monitoring, Docker, docs
- Vague 5 full — All protocol adapters + vault contract

**Phase 4 — Polish (optional, 4–6 months)**
- Vague 6.5 — DAPP frontend (if needed for demo/users)
- Vague 8 — Advanced features
- Vague 9 — Whitepaper

**Total realistic timeline: 12–18 months solo, full-time.**

### What to Cut if Time-Constrained

1. **Vague 6.5 (DAPP Frontend)** — Use CLI + API instead. A full Next.js app is a second product.
2. **Vague 8.8 (Mobile App)** — Unless you have mobile expertise, defer indefinitely.
3. **Browser-side proving** — Server-side proving is real ZKP; contract verifies. Browser proving = v2 advanced mode.
4. **Vague 8.5 (Cross-Chain)** — Stick to Ethereum L1 + one L2 (Arbitrum) max.
5. **Vague 9 (Whitepaper)** — Only if pursuing grants/academic credibility.

### Tracking Progress

Update the status emoji in this file as you complete stories:
- Change `⏳` → `🚧` when you start working on it
- Change `🚧` → `✅` when it's done and tested

Run `git diff BACKLOG.md` to see your progress over time.
