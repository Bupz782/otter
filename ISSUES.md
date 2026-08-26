# Otter — All Issues

> Auto-generated from BACKLOG.md. Cut issues (FHE, Encrypted Mempool) excluded.


## Vague 0 : Setup & Architectureoundation (2 semaines)

### Epic 0.1 : Environnement de développement

- [FAIT] **US-001** : En tant que dev, je veux installer Rust nightly avec tous les toolchains nécessaires
- [FAIT] **US-002** : En tant que dev, je veux installer Noir (nargo) pour développer des circuits ZK
- [FAIT] **US-003** : En tant que dev, je veux installer Foundry (forge, cast, anvil) pour les smart contracts
- [FAIT] **US-004** : En tant que dev, je veux un workspace Cargo avec tous les crates définis
- [FAIT] **US-005** : En tant que dev, je veux un dossier `lab/` pour expérimenter sans polluer l'architecture
### Epic 0.2 : Architecture squelette

- [FAIT] **US-006** : En tant que système, je veux la structure hexagonale (domain/application/infrastructure/interfaces)
- [EN COURS] **US-007** : En tant que système, je veux tous les ports définis (traits vides) : ZkpPort, FhePort, MempoolPort, BlockchainPort, IntentParserPort
- [EN COURS] **US-008** : En tant que système, je veux un `orchestrator.rs` squelette avec state machine enum
- [EN COURS] **US-009** : En tant que système, je veux un système de logs structuré (tracing + tracing-subscriber)
- [FAIT] **US-010** : En tant que dev, je veux un `LEARNING.md` pour documenter ce que j'apprends chaque semaine
### Epic 0.3 : CI/CD basique

- [FAIT] **US-011** : En tant que dev, je veux un GitHub Actions workflow qui run `cargo test`
- [FAIT] **US-012** : En tant que dev, je veux un workflow qui run `cargo clippy` (lints)
- [FAIT] **US-013** : En tant que dev, je veux un workflow qui vérifie le formatting (`cargo fmt --check`)
- [EN ATTENTE] **US-014** : En tant que dev, je veux un script `scripts/benchmarks.sh` pour mesurer les performances

## Vague 1 : Intentarsing & LLM Integration (6 semaines)

### Epic 1.1 : Domain - Intent Model

- [FAIT] **US-015** : En tant que système, je veux une enum `Intent` couvrant les actions DeFi de base (Lend, Borrow, Swap, Stake)
- [FAIT] **US-016** : En tant que système, je veux une struct `Condition` pour les intents conditionnels (yield > X%, price > Y)
- [FAIT] **US-017** : En tant que système, je veux une enum `Protocol` représentant les protocoles supportés (Aave, Compound, Uniswap, Curve)
- [FAIT] **US-018** : En tant que système, je veux une struct `ExecutionPlan` décomposant un intent en steps exécutables
- [FAIT] **US-019** : En tant que système, je veux valider qu'un `Intent` est bien formé (amounts > 0, protocols valides, etc.)
### Epic 1.2 : Rule-based Parser (v1)

- [FAIT] **US-020** : En tant que parser, je veux extraire un intent "lend X ASSET on PROTOCOL" via regex
- [FAIT] **US-021** : En tant que parser, je veux extraire un intent "swap X ASSET for Y ASSET"
- [FAIT] **US-022** : En tant que parser, je veux extraire un intent "borrow X ASSET with Y COLLATERAL"
- [FAIT] **US-023** : En tant que parser, je veux extraire un intent "stake X ASSET on PROTOCOL"
- [FAIT] **US-024** : En tant que parser, je veux extraire une condition "if METRIC > VALUE"
- [FAIT] **US-025** : En tant que parser, je veux combiner intent + condition : "lend X if yield > Y%"
- [FAIT] **US-026** : En tant que parser, je veux gérer les montants avec unités (1000 USDC, 1.5 ETH, 50%)
- [FAIT] **US-027** : En tant que parser, je veux retourner des erreurs précises (UnknownAsset, InvalidAmount, UnknownProtocol)
### Epic 1.3 : LLM Integration (Claude API)

- [EN COURS] **US-028** : En tant qu'infra, je veux un module `llm_client.rs` wrappant l'API Claude
- [EN COURS] **US-029** : En tant que parser LLM, je veux générer un prompt structuré avec schema JSON
- [EN COURS] **US-030** : En tant que parser LLM, je veux parser la réponse JSON de Claude
- [EN ATTENTE] **US-031** : En tant que parser LLM, je veux valider que la sortie LLM respecte le schema Intent
- [EN ATTENTE] **US-032** : En tant que parser LLM, je veux un fallback : si LLM échoue → rule-based parser
- [EN ATTENTE] **US-033** : En tant que parser LLM, je veux logger les tokens consommés (coût monitoring)
- [EN ATTENTE] **US-034** : En tant que parser LLM, je veux gérer les rate limits (exponential backoff)
- [EN ATTENTE] **US-035** : En tant que parser LLM, je veux cacher les réponses pour éviter de re-parser le même texte
### Epic 1.4 : Strategy Planner

- [FAIT] **US-036** : En tant que planner, je veux décomposer `Intent::Lend` en ExecutionPlan (approve + supply)
- [FAIT] **US-037** : En tant que planner, je veux décomposer `Intent::Swap` en ExecutionPlan (approve + swap)
- [FAIT] **US-038** : En tant que planner, je veux décomposer `Intent::Borrow` en ExecutionPlan (approve collateral + borrow)
- [FAIT] **US-039** : En tant que planner, je veux gérer les multi-step : "lend then stake" → 2 plans séquentiels
- [FAIT] **US-040** : En tant que planner, je veux vérifier les conditions AVANT de créer le plan
- [FAIT] **US-041** : En tant que planner, je veux calculer le gas estimé total du plan
- [FAIT] **US-042** : En tant que planner, je veux détecter les impossibilités (ex: borrow sans collateral)
### Epic 1.5 : Application - Use Cases

- [FAIT] **US-043** : En tant qu'application, je veux un use case `ParseIntent(text)` → Intent
- [FAIT] **US-044** : En tant qu'application, je veux un use case `PlanExecution(intent)` → ExecutionPlan
- [FAIT] **US-045** : En tant qu'application, je veux un use case `ValidateIntent(intent)` → Result<(), ValidationError>
- [FAIT] **US-046** : En tant qu'application, je veux un use case `EvaluateCondition(condition)` → bool
### Epic 1.6 : CLI - Intent Testing

- [EN ATTENTE] **US-047** : En tant que user, je veux exécuter `otter parse "lend 1000 USDC on Aave"` et voir l'intent parsé
- [EN ATTENTE] **US-048** : En tant que user, je veux exécuter `otter plan <intent>` et voir l'ExecutionPlan
- [EN ATTENTE] **US-049** : En tant que user, je veux un mode `--llm` vs `--rules` pour comparer les parsers
- [EN ATTENTE] **US-050** : En tant que user, je veux voir les erreurs de parsing avec suggestions de correction
### Epic 1.7 : Tests Intent Layer

- [FAIT] **US-051** : En tant que dev, je veux des tests unitaires pour 20+ intents valides
- [FAIT] **US-052** : En tant que dev, je veux des tests pour intents invalides (error handling)
- [FAIT] **US-053** : En tant que dev, je veux des tests pour intents complexes multi-conditions
- [EN ATTENTE] **US-054** : En tant que dev, je veux mocker Claude API pour tester sans coût
- [EN ATTENTE] **US-055** : En tant que dev, je veux benchmarker : temps de parsing rule-based vs LLM

## Vague 2 : ZKP - Delegation with Intenterification (8 semaines)

### Epic 2.1 : Noir Fundamentals

- [EN ATTENTE] **US-056** : En tant que learner, je veux lire ZK Learning cours 1-2 (circuits arithmétiques)
- [EN ATTENTE] **US-057** : En tant que dev, je veux écrire un circuit "hello world" (prouver connaissance d'un secret)
- [EN ATTENTE] **US-058** : En tant que dev, je veux compiler avec nargo et générer une preuve
- [EN ATTENTE] **US-059** : En tant que dev, je veux vérifier la preuve avec `nargo verify`
- [EN ATTENTE] **US-060** : En tant que dev, je veux mesurer : # contraintes, temps de preuve, taille de la preuve
### Epic 2.2 : EdDSA Signature Verification

- [EN ATTENTE] **US-061** : En tant que learner, je veux comprendre EdDSA Baby JubJub (courbe ZK-friendly)
- [EN ATTENTE] **US-062** : En tant que circuit, je veux vérifier une signature EdDSA sur un message fixe
- [EN ATTENTE] **US-063** : En tant que circuit, je veux paramétrer le message (input public)
- [EN ATTENTE] **US-064** : En tant que circuit, je veux tester avec 10 paires (valid/invalid signatures)
- [EN ATTENTE] **US-065** : En tant que dev, je veux benchmarker la vérification EdDSA en circuit
### Epic 2.3 : Delegation Message Structure

- [EN ATTENTE] **US-066** : En tant que circuit, je veux définir `DelegationMessage` struct (agent_pubkey, allowed_intents, max_amounts, allowed_protocols, expiry, nonce)
- [EN ATTENTE] **US-067** : En tant que circuit, je veux hasher le `DelegationMessage` pour obtenir un digest
- [EN ATTENTE] **US-068** : En tant que circuit, je veux vérifier qu'une signature EdDSA couvre ce digest
- [EN ATTENTE] **US-069** : En tant que circuit, je veux supporter jusqu'à 10 intent types différents
- [EN ATTENTE] **US-070** : En tant que circuit, je veux supporter jusqu'à 5 protocols whitelistés
### Epic 2.4 : Intent Authorization Circuit

- [EN ATTENTE] **US-071** : En tant que circuit, je veux une struct `ProposedIntent` (intent_type, amount, protocol, target_contract)
- [EN ATTENTE] **US-072** : En tant que circuit, je veux vérifier que intent_type est dans allowed_intents (bitfield check)
- [FAIT] **US-073** : En tant que circuit, je veux vérifier que amount <= max_amounts[intent_type] — prouvé par `delegation_circuit/src/main.nr:153`
- [EN ATTENTE] **US-074** : En tant que circuit, je veux vérifier que protocol est dans allowed_protocols (array membership)
- [EN ATTENTE] **US-075** : En tant que circuit, je veux vérifier que target_contract correspond au protocol
- [EN ATTENTE] **US-076** : En tant que circuit, je veux vérifier que current_timestamp < expiry
- [EN ATTENTE] **US-077** : En tant que circuit, je veux vérifier que nonce_provided == nonce_expected
### Epic 2.5 : Circuit Optimization

- [EN ATTENTE] **US-078** : En tant que dev, je veux profiler le circuit (quelles opérations coûtent le plus de contraintes)
- [EN ATTENTE] **US-079** : En tant que dev, je veux réduire les contraintes de 20% (optimizations)
- [EN ATTENTE] **US-080** : En tant que dev, je veux tester le circuit sur 100+ combinaisons d'inputs
- [EN ATTENTE] **US-081** : En tant que dev, je veux vérifier qu'aucun cas edge ne passe (security audit)
### Epic 2.6 : Rust ↔ Noir Integration

- [EN ATTENTE] **US-082** : En tant qu'infra, je veux un `NoirAdapter` implémentant `ZkpPort`
- [EN ATTENTE] **US-083** : En tant qu'infra, je veux générer une preuve depuis Rust (serialize inputs → call nargo → parse proof bytes)
- [EN ATTENTE] **US-084** : En tant qu'infra, je veux vérifier une preuve off-chain depuis Rust (Barretenberg verifier)
- [EN ATTENTE] **US-085** : En tant qu'infra, je veux gérer les erreurs (ProofGenerationFailed, InvalidWitness, Timeout)
- [EN ATTENTE] **US-086** : En tant qu'infra, je veux cacher les proving keys en mémoire (éviter recompilation)
- [EN ATTENTE] **US-087** : En tant qu'infra, je veux logger les temps : witness generation, proving, verification
### Epic 2.7 : Domain Integration

- [EN ATTENTE] **US-088** : En tant que domain, je veux une struct `Delegation` avec tous les champs typés
- [EN ATTENTE] **US-089** : En tant que domain, je veux une méthode `delegation.can_execute(intent)` → bool (business logic)
- [EN ATTENTE] **US-090** : En tant que domain, je veux une struct `DelegationProof` wrappant les proof bytes
- [EN ATTENTE] **US-091** : En tant que domain, je veux `ZkpPort` trait complet (`prove_delegation`, `verify_delegation_offchain`)
### Epic 2.8 : Application - Delegation Use Cases

- [EN ATTENTE] **US-092** : En tant qu'application, je veux un use case `CreateDelegation(params)` → Delegation
- [EN ATTENTE] **US-093** : En tant qu'application, je veux un use case `SignDelegation(delegation, privkey)` → Signature
- [EN ATTENTE] **US-094** : En tant qu'application, je veux un use case `ProveIntent(delegation, intent)` → DelegationProof
- [EN ATTENTE] **US-095** : En tant qu'application, je veux un use case `VerifyProofOffchain(proof)` → bool
### Epic 2.9 : Smart Contracts - Verifier

- [EN ATTENTE] **US-096** : En tant que dev, je veux exporter le verifier Solidity depuis Noir
- [EN ATTENTE] **US-097** : En tant que dev, je veux créer `DelegationVerifier.sol` wrappant le verifier Noir
- [FAIT] **US-098** : En tant que contract, je veux une fonction `verifyDelegation(bytes proof, bytes32[] publicInputs)` → bool — implémenté sous le nom `executeWithProof` dans `contracts/src/DelegationVault.sol:176` (écart de nommage assumé)
- [EN ATTENTE] **US-099** : En tant que dev, je veux déployer sur testnet Sepolia
- [EN ATTENTE] **US-100** : En tant que dev, je veux tester on-chain : 10 valid proofs → true, 10 invalid → false
### Epic 2.10 : CLI - Delegation Flow

- [EN ATTENTE] **US-101** : En tant que user, je veux `otter keygen` pour générer une keypair EdDSA
- [EN ATTENTE] **US-102** : En tant que user, je veux `otter delegate --agent <pubkey> --max-lend 5000 --protocols aave,compound --expiry 2025-12-31`
- [EN ATTENTE] **US-103** : En tant que user, je veux voir la délégation créée avec hash et signature
- [EN ATTENTE] **US-104** : En tant que user, je veux `otter prove --delegation <file> --intent "lend 1000 USDC on Aave"`
- [EN ATTENTE] **US-105** : En tant que user, je veux voir la preuve générée (temps, taille, hash)
- [FAIT] **US-106** : En tant que user, je veux `otter verify-onchain --proof <file>` pour tester le contract — implémenté dans `crates/interfaces/src/bin/otter_cli.rs:37`
### Epic 2.11 : Tests ZKP Layer

- [EN ATTENTE] **US-107** : En tant que dev, je veux des tests unitaires du circuit (20+ cas valid/invalid)
- [EN ATTENTE] **US-108** : En tant que dev, je veux des tests de génération de preuve (benchmarks)
- [EN ATTENTE] **US-109** : En tant que dev, je veux des tests E2E : delegation → parse intent → prove → verify onchain
- [EN ATTENTE] **US-110** : En tant que dev, je veux tester les cas limites (expiry à la seconde près, nonce edge cases)

## Vague 5 : Blockchain Integration &rotocol Adapters (8 semaines)

### Epic 5.1 : Domain - Blockchain Abstractions

- [EN ATTENTE] **US-217** : En tant que domain, je veux une struct `Transaction` (from, to, value, data, gas)
- [EN ATTENTE] **US-218** : En tant que domain, je veux une struct `TransactionReceipt` (hash, block, status, gas_used)
- [EN ATTENTE] **US-219** : En tant que domain, je veux `BlockchainPort` trait (`send_tx`, `get_balance`, `call_contract`, `estimate_gas`)
- [EN ATTENTE] **US-220** : En tant que domain, je veux `WalletPort` trait (`sign_tx`, `get_address`, `get_nonce`)
### Epic 5.2 : Infrastructure - Ethereum Adapter

- [EN ATTENTE] **US-221** : En tant qu'infra, je veux un `EthereumAdapter` utilisant ethers-rs ou alloy
- [EN ATTENTE] **US-222** : En tant qu'infra, je veux me connecter à un RPC (Infura/Alchemy/Ankr)
- [EN ATTENTE] **US-223** : En tant qu'infra, je veux lire le balance d'une adresse
- [EN ATTENTE] **US-224** : En tant qu'infra, je veux estimer le gas d'une transaction
- [EN ATTENTE] **US-225** : En tant qu'infra, je veux envoyer une transaction signée
- [EN ATTENTE] **US-226** : En tant qu'infra, je veux attendre la confirmation (polling ou WebSocket)
- [EN ATTENTE] **US-227** : En tant qu'infra, je veux gérer les erreurs (revert, out of gas, nonce too low)
- [EN ATTENTE] **US-228** : En tant qu'infra, je veux gérer les nonces automatiquement (anticiper les pending txs)
- [EN ATTENTE] **US-229** : En tant qu'infra, je veux retry avec plus de gas si la tx échoue pour cette raison
- [EN ATTENTE] **US-230** : En tant qu'infra, je veux supporter les réseaux testnet (Sepolia, Holesky)
### Epic 5.3 : Infrastructure - Wallet Management

- [EN ATTENTE] **US-231** : En tant qu'infra, je veux générer une keypair secp256k1 pour l'agent
- [EN ATTENTE] **US-232** : En tant qu'infra, je veux stocker la clé privée dans un keystore chiffré (scrypt/pbkdf2)
- [EN ATTENTE] **US-233** : En tant qu'infra, je veux charger la clé depuis le keystore avec password
- [EN ATTENTE] **US-234** : En tant qu'infra, je veux signer une transaction avec la clé privée
- [EN ATTENTE] **US-235** : En tant qu'infra, je veux dériver plusieurs adresses depuis une seed (HD wallet optionnel)
### Epic 5.4 : Protocol Adapters - Aave

- [EN ATTENTE] **US-236** : En tant qu'infra, je veux un `AaveAdapter` wrappant les contracts Aave v3
- [EN ATTENTE] **US-237** : En tant qu'adapter Aave, je veux une méthode `get_apy(asset)` → f64
- [EN ATTENTE] **US-238** : En tant qu'adapter Aave, je veux une méthode `supply(asset, amount)` → Transaction
- [EN ATTENTE] **US-239** : En tant qu'adapter Aave, je veux une méthode `withdraw(asset, amount)` → Transaction
- [EN ATTENTE] **US-240** : En tant qu'adapter Aave, je veux une méthode `borrow(asset, amount, collateral)` → Transaction
- [EN ATTENTE] **US-241** : En tant qu'adapter Aave, je veux une méthode `repay(asset, amount)` → Transaction
- [EN ATTENTE] **US-242** : En tant qu'adapter Aave, je veux gérer l'approve préalable (ERC20)
### Epic 5.5 : Protocol Adapters - Compound

- [EN ATTENTE] **US-243** : En tant qu'infra, je veux un `CompoundAdapter` wrappant Compound v3
- [EN ATTENTE] **US-244** : En tant qu'adapter Compound, je veux les mêmes méthodes que Aave (interface unifiée)
- [EN ATTENTE] **US-245** : En tant qu'adapter Compound, je veux gérer les cTokens (mint/redeem)
### Epic 5.6 : Protocol Adapters - Uniswap

- [EN ATTENTE] **US-246** : En tant qu'infra, je veux un `UniswapAdapter` wrappant SwapRouter v3
- [EN ATTENTE] **US-247** : En tant qu'adapter Uniswap, je veux une méthode `get_quote(from, to, amount)` → expected_output
- [EN ATTENTE] **US-248** : En tant qu'adapter Uniswap, je veux une méthode `swap(from, to, amount, slippage)` → Transaction
- [EN ATTENTE] **US-249** : En tant qu'adapter Uniswap, je veux calculer le path optimal (direct ou via WETH)
- [EN ATTENTE] **US-250** : En tant qu'adapter Uniswap, je veux gérer le wrapping ETH → WETH si nécessaire
### Epic 5.7 : Protocol Adapters - Curve (staking)

- [EN ATTENTE] **US-251** : En tant qu'infra, je veux un `CurveAdapter` pour staker des LP tokens
- [EN ATTENTE] **US-252** : En tant qu'adapter Curve, je veux une méthode `stake(lp_token, amount)` → Transaction
- [EN ATTENTE] **US-253** : En tant qu'adapter Curve, je veux une méthode `unstake(lp_token, amount)` → Transaction
- [EN ATTENTE] **US-254** : En tant qu'adapter Curve, je veux une méthode `claim_rewards()` → Transaction
### Epic 5.8 : Domain - Protocol Abstraction

- [EN ATTENTE] **US-255** : En tant que domain, je veux un trait `LendingProtocol` unifié (supply, withdraw, borrow, repay, get_apy)
- [EN ATTENTE] **US-256** : En tant que domain, je veux un trait `DexProtocol` unifié (swap, get_quote)
- [EN ATTENTE] **US-257** : En tant que domain, je veux un trait `StakingProtocol` unifié (stake, unstake, claim)
- [EN ATTENTE] **US-258** : En tant que domain, je veux un `ProtocolRegistry` pour mapper protocol name → adapter
### Epic 5.9 : Application - Execution Use Cases

- [EN ATTENTE] **US-259** : En tant qu'application, je veux un use case `ExecuteIntent(intent)` → TransactionReceipt
- [EN ATTENTE] **US-260** : En tant qu'application, je veux un use case `SimulateExecution(intent)` → GasEstimate
- [EN ATTENTE] **US-261** : En tant qu'application, je veux un use case `ApproveToken(token, spender, amount)` → Transaction
- [EN ATTENTE] **US-262** : En tant qu'application, je veux gérer les multi-step executions (approve then execute)
### Epic 5.10 : Smart Contracts - Strategy Vault

- [EN ATTENTE] **US-263** : En tant que dev, je veux un contrat `StrategyVault.sol` qui détient les fonds users
- [EN ATTENTE] **US-264** : En tant que vault, je veux accepter des dépôts (deposit ETH/ERC20)
- [EN ATTENTE] **US-265** : En tant que vault, je veux permettre les retraits (withdraw)
- [EN ATTENTE] **US-266** : En tant que vault, je veux autoriser l'agent à exécuter des actions (via delegation proof)
- [EN ATTENTE] **US-267** : En tant que vault, je veux vérifier la preuve ZKP avant chaque action
- [EN ATTENTE] **US-268** : En tant que vault, je veux intégrer le `DelegationVerifier`
- [EN ATTENTE] **US-269** : En tant que vault, je veux gérer les nonces (anti-replay)
- [EN ATTENTE] **US-270** : En tant que vault, je veux émettre des events (Deposited, Withdrawn, ActionExecuted)
### Epic 5.11 : Smart Contracts - Deployment & Testing

- [EN ATTENTE] **US-271** : En tant que dev, je veux déployer tous les contracts sur Sepolia
- [EN ATTENTE] **US-272** : En tant que dev, je veux un script de setup (deploy + configure)
- [EN ATTENTE] **US-273** : En tant que dev, je veux tester le vault avec Foundry (unit tests)
- [EN ATTENTE] **US-274** : En tant que dev, je veux tester l'intégration vault + verifier (valid proof → success, invalid → revert)
### Epic 5.12 : CLI - Blockchain Operations

- [EN ATTENTE] **US-275** : En tant que user, je veux `otter wallet create` pour générer un wallet agent
- [EN ATTENTE] **US-276** : En tant que user, je veux `otter wallet balance` pour voir le solde
- [EN ATTENTE] **US-277** : En tant que user, je veux `otter execute --intent "lend 1000 USDC on Aave"` en réel
- [EN ATTENTE] **US-278** : En tant que user, je veux un mode `--dry-run` qui simule sans envoyer
- [EN ATTENTE] **US-279** : En tant que user, je veux voir les logs de transaction (hash, block, gas, status)
- [EN ATTENTE] **US-280** : En tant que user, je veux `otter vault deposit --amount 5000 --asset USDC`
### Epic 5.13 : Tests Blockchain Layer

- [EN ATTENTE] **US-281** : En tant que dev, je veux des tests avec Anvil (fork testnet local)
- [EN ATTENTE] **US-282** : En tant que dev, je veux tester chaque adapter de protocol (mock contracts ou fork)
- [EN ATTENTE] **US-283** : En tant que dev, je veux un test E2E : deposit → delegate → execute intent → withdraw
- [EN ATTENTE] **US-284** : En tant que dev, je veux tester les error cases (insufficient balance, revert, etc.)
### Epic 5.14 : MEV Capture & Rebate

- [EN ATTENTE] **US-581** : En tant qu'infra, je veux intégrer Flashbots Protect / MEV-Blocker pour soumettre les transactions
- [EN ATTENTE] **US-582** : En tant qu'infra, je veux capturer le MEV (arbitrage, backrunning) du flow d'exécution
- [EN ATTENTE] **US-583** : En tant que circuit, je veux prouver le split MEV : 50% user, 40% agent, 10% protocol
- [EN ATTENTE] **US-584** : En tant que vault, je veux distribuer les rebates MEV aux users après chaque exécution
- [EN ATTENTE] **US-585** : En tant que user, je veux voir le MEV rebâté par action dans le dashboard
- [EN ATTENTE] **US-586** : En tant que dev, je veux benchmarker le MEV capturé vs le gas dépensé
### Epic 5.15 : Proof-of-Solvency

- [EN ATTENTE] **US-587** : En tant que circuit, je veux prouver `sum(deposits) ≤ vault_assets` sans révéler les balances
- [EN ATTENTE] **US-588** : En tant que vault, je veux générer une preuve de solvabilité périodiquement (daily)
- [EN ATTENTE] **US-589** : En tant que vault, je veux publier la preuve de solvabilité on-chain
- [EN ATTENTE] **US-590** : En tant que user, je veux voir le statut de solvabilité du vault ("Vérifié il y a 2h")
- [EN ATTENTE] **US-591** : En tant que user, je veux télécharger / vérifier indépendamment la preuve de solvabilité
- [EN ATTENTE] **US-592** : En tant que dev, je veux tester la preuve de solvabilité sur 100+ scénarios de deposits/withdrawals

## Vague 6 : Orchestrator & Integratedlow (10 semaines)

### Epic 6.1 : State Machine Design

- [EN ATTENTE] **US-285** : En tant que système, je veux une FSM (Finite State Machine) avec états : IDLE, MONITORING, ANALYZING, DECIDING, PROVING, ENCRYPTING, SUBMITTING, CONFIRMING, ERROR
- [EN ATTENTE] **US-286** : En tant que système, je veux définir les transitions valides entre états
- [EN ATTENTE] **US-287** : En tant que système, je veux logger chaque transition d'état
- [EN ATTENTE] **US-288** : En tant que système, je veux gérer les timeouts (si bloqué dans un état trop longtemps)
- [EN ATTENTE] **US-289** : En tant que système, je veux rollback sur erreur (revenir à IDLE ou MONITORING)
### Epic 6.2 : Event Bus Architecture

- [EN ATTENTE] **US-290** : En tant que système, je veux un event bus basé sur tokio channels (mpsc)
- [EN ATTENTE] **US-291** : En tant que système, je veux définir les events : PriceUpdated, ConditionMet, IntentParsed, ProofGenerated, TransactionSubmitted, TransactionConfirmed, Error
- [EN ATTENTE] **US-292** : En tant que système, je veux un dispatcher qui route les events vers les handlers
- [EN ATTENTE] **US-293** : En tant que système, je veux que les modules publient des events (découplage)
- [EN ATTENTE] **US-294** : En tant que système, je veux logger tous les events (audit trail)
### Epic 6.3 : Orchestrator Core

- [EN ATTENTE] **US-295** : En tant qu'orchestrator, je veux une boucle principale qui réagit aux events
- [EN ATTENTE] **US-296** : En tant qu'orchestrator, je veux maintenir l'état global (current_state, active_intents, delegations)
- [EN ATTENTE] **US-297** : En tant qu'orchestrator, je veux coordonner les appels aux différents ports (ZKP, Blockchain)
- [EN ATTENTE] **US-298** : En tant qu'orchestrator, je veux gérer les dépendances entre étapes (PROVING doit finir avant ENCRYPTING)
### Epic 6.4 : Monitoring Loop

- [EN ATTENTE] **US-299** : En tant qu'orchestrator, je veux un loop qui check les conditions périodiquement (ex: toutes les 60s)
- [EN ATTENTE] **US-300** : En tant qu'orchestrator, je veux fetcher les prix via PriceOraclePort
- [EN ATTENTE] **US-301** : En tant qu'orchestrator, je veux évaluer les conditions des intents actifs
- [EN ATTENTE] **US-302** : En tant qu'orchestrator, je veux publier ConditionMet event si une condition est vraie
### Epic 6.5 : Decision Making

- [EN ATTENTE] **US-303** : En tant qu'orchestrator, je veux recevoir ConditionMet event → transition vers ANALYZING
- [EN ATTENTE] **US-304** : En tant qu'orchestrator, je veux appeler le strategy planner pour créer ExecutionPlan
- [EN ATTENTE] **US-305** : En tant qu'orchestrator, je veux vérifier que le plan respecte la délégation (business rules)
- [EN ATTENTE] **US-306** : En tant qu'orchestrator, je veux transition vers DECIDING → choisir d'exécuter ou attendre
### Epic 6.6 : Proof Generation Flow

- [EN ATTENTE] **US-307** : En tant qu'orchestrator, je veux transition vers PROVING
- [EN ATTENTE] **US-308** : En tant qu'orchestrator, je veux appeler `ZkpPort.prove_delegation(intent)`
- [EN ATTENTE] **US-309** : En tant qu'orchestrator, je veux gérer les erreurs de proof generation (retry ou abort)
- [EN ATTENTE] **US-310** : En tant qu'orchestrator, je veux publier ProofGenerated event avec la preuve
### Epic 6.7 : Transaction Encryption Flow

- [EN ATTENTE] **US-311** : En tant qu'orchestrator, je veux transition vers ENCRYPTING
- [EN ATTENTE] **US-312** : En tant qu'orchestrator, je veux construire la transaction (via protocol adapter)
- [EN ATTENTE] **US-313** : En tant qu'orchestrator, je veux attacher la preuve ZKP à la transaction
- [EN ATTENTE] **US-314** : En tant qu'orchestrator, je veux préparer la transaction chiffrée pour soumission
- [EN ATTENTE] **US-315** : En tant qu'orchestrator, je veux publier TransactionEncrypted event
### Epic 6.8 : Submission & Confirmation Flow

- [EN ATTENTE] **US-316** : En tant qu'orchestrator, je veux transition vers SUBMITTING
- [EN ATTENTE] **US-317** : En tant qu'orchestrator, je veux soumettre la transaction à la blockchain
- [EN ATTENTE] **US-318** : En tant qu'orchestrator, je veux publier TransactionSubmitted event avec tx_hash
- [EN ATTENTE] **US-319** : En tant qu'orchestrator, je veux transition vers CONFIRMING
- [EN ATTENTE] **US-320** : En tant qu'orchestrator, je veux attendre la confirmation on-chain (polling ou events)
- [EN ATTENTE] **US-321** : En tant qu'orchestrator, je veux publier TransactionConfirmed event avec receipt
- [EN ATTENTE] **US-322** : En tant qu'orchestrator, je veux transition vers IDLE (prêt pour next iteration)
### Epic 6.9 : Error Handling

- [EN ATTENTE] **US-323** : En tant qu'orchestrator, je veux catcher toutes les erreurs des ports
- [EN ATTENTE] **US-324** : En tant qu'orchestrator, je veux transition vers ERROR state avec contexte
- [EN ATTENTE] **US-325** : En tant qu'orchestrator, je veux publier Error event avec détails
- [EN ATTENTE] **US-326** : En tant qu'orchestrator, je veux retry automatiquement selon le type d'erreur
- [EN ATTENTE] **US-327** : En tant qu'orchestrator, je veux notifier le user si erreur non récupérable
### Epic 6.10 : Full Flow Integration

- [EN ATTENTE] **US-328** : En tant que système, je veux un test E2E du flow complet :
- [EN ATTENTE] **US-329** : En tant que système, je veux logger chaque étape du flow avec timestamps
- [EN ATTENTE] **US-330** : En tant que système, je veux mesurer le temps total du flow (SLA)
### Epic 6.11 : Multi-Intent Management

- [EN ATTENTE] **US-331** : En tant qu'orchestrator, je veux gérer plusieurs intents actifs simultanément
- [EN ATTENTE] **US-332** : En tant qu'orchestrator, je veux prioriser les intents (ordre d'exécution)
- [EN ATTENTE] **US-333** : En tant qu'orchestrator, je veux éviter les conflits (2 intents modifiant même asset)
- [EN ATTENTE] **US-334** : En tant qu'orchestrator, je veux supporter les intents récurrents ("rebalance every week")
### Epic 6.12 : CLI - Orchestrator Control

- [EN ATTENTE] **US-335** : En tant que user, je veux `otter start` pour lancer l'orchestrator en daemon
- [EN ATTENTE] **US-336** : En tant que user, je veux `otter stop` pour arrêter proprement
- [EN ATTENTE] **US-337** : En tant que user, je veux `otter status` pour voir l'état actuel (current_state, active_intents)
- [EN ATTENTE] **US-338** : En tant que user, je veux `otter logs --follow` pour voir les events en temps réel
### Epic 6.13 : Tests Orchestrator

- [EN ATTENTE] **US-339** : En tant que dev, je veux mocker tous les ports pour tester l'orchestrator isolément
- [EN ATTENTE] **US-340** : En tant que dev, je veux tester chaque transition de state machine
- [EN ATTENTE] **US-341** : En tant que dev, je veux tester les scénarios d'erreur (proof fails, tx reverts, timeout)
- [EN ATTENTE] **US-342** : En tant que dev, je veux un test E2E avec tous les composants réels

## Vague 6.5 : DAPProntend (8 semaines)

### Epic 6.5.1 : Frontend Setup

- [EN ATTENTE] **US-343** : En tant que dev, je veux initialiser un projet Next.js 14 (App Router)
- [EN ATTENTE] **US-344** : En tant que dev, je veux configurer Tailwind CSS + shadcn/ui
- [EN ATTENTE] **US-345** : En tant que dev, je veux configurer RainbowKit pour wallet connection
- [EN ATTENTE] **US-346** : En tant que dev, je veux configurer wagmi/viem pour interactions blockchain
- [EN ATTENTE] **US-347** : En tant que dev, je veux configurer TypeScript strict mode
### Epic 6.5.2 : Wallet Connection

- [EN ATTENTE] **US-348** : En tant que user, je veux connecter mon wallet (MetaMask, WalletConnect, Coinbase)
- [EN ATTENTE] **US-349** : En tant que user, je veux voir mon adresse et balance dans la navbar
- [EN ATTENTE] **US-350** : En tant que user, je veux switcher de réseau (Mainnet ↔ Sepolia)
- [EN ATTENTE] **US-351** : En tant que user, je veux déconnecter mon wallet
### Epic 6.5.3 : Intent Input Interface

- [EN ATTENTE] **US-352** : En tant que user, je veux une page "Create Intent" avec un textarea
- [EN ATTENTE] **US-353** : En tant que user, je veux des suggestions d'intents (autocomplete ou exemples)
- [EN ATTENTE] **US-354** : En tant que user, je veux un bouton "Parse Intent" qui appelle le backend
- [EN ATTENTE] **US-355** : En tant que user, je veux voir l'intent parsé (structure JSON formatée)
- [EN ATTENTE] **US-356** : En tant que user, je veux voir les erreurs de parsing avec suggestions
- [EN ATTENTE] **US-357** : En tant que user, je veux éditer manuellement l'intent parsé (JSON editor)
### Epic 6.5.4 : Intent Validation UI

- [EN ATTENTE] **US-358** : En tant que user, je veux voir les validations de l'intent :
- [EN ATTENTE] **US-359** : En tant que user, je veux voir les permissions requises :
- [EN ATTENTE] **US-360** : En tant que user, je veux voir le gas estimé
- [EN ATTENTE] **US-361** : En tant que user, je veux voir les risques (smart contract risk, impermanent loss, etc.)
### Epic 6.5.5 : Delegation Setup UI

- [EN ATTENTE] **US-362** : En tant que user, je veux configurer les limites de délégation (wizard step-by-step)
- [EN ATTENTE] **US-363** : En tant que user, je veux sélectionner les protocols autorisés (checkboxes : Aave, Compound, Uniswap, Curve)
- [EN ATTENTE] **US-364** : En tant que user, je veux définir les max amounts par intent type (sliders)
- [EN ATTENTE] **US-365** : En tant que user, je veux définir une date d'expiration (date picker)
- [EN ATTENTE] **US-366** : En tant que user, je veux voir un résumé de la délégation avant signature
- [EN ATTENTE] **US-367** : En tant que user, je veux signer la délégation avec MetaMask (EdDSA key derivation via EIP-712 ou custom)
- [EN ATTENTE] **US-368** : En tant que user, je veux télécharger le fichier de délégation (.json)
### Epic 6.5.6 : Agent Dashboard

- [EN ATTENTE] **US-369** : En tant que user, je veux voir une page "Dashboard" avec vue d'ensemble
- [EN ATTENTE] **US-370** : En tant que user, je veux voir mon portfolio actuel (assets + balances)
- [EN ATTENTE] **US-371** : En tant que user, je veux voir les intents actifs (table : intent, status, condition, actions)
- [EN ATTENTE] **US-372** : En tant que user, je veux voir l'historique des actions (timeline : actions executées avec tx hash)
- [EN ATTENTE] **US-373** : En tant que user, je veux voir les métriques :
- [EN ATTENTE] **US-374** : En tant que user, je veux filtrer l'historique (par date, par intent, par status)
### Epic 6.5.7 : Intent Status & Monitoring

- [EN ATTENTE] **US-375** : En tant que user, je veux voir le statut détaillé d'un intent :
- [EN ATTENTE] **US-376** : En tant que user, je veux voir le statut d'une transaction en cours :
- [EN ATTENTE] **US-377** : En tant que user, je veux un indicateur visuel du flow (stepper UI : IDLE → MONITORING → ... → CONFIRMED)
### Epic 6.5.8 : Real-time Updates

- [EN ATTENTE] **US-378** : En tant que système, je veux un WebSocket server dans le backend
- [EN ATTENTE] **US-379** : En tant que frontend, je veux me connecter au WebSocket pour recevoir les events
- [EN ATTENTE] **US-380** : En tant que user, je veux recevoir des notifications push :
- [EN ATTENTE] **US-381** : En tant que user, je veux voir les notifications dans une sidebar (toast + history)
- [EN ATTENTE] **US-387** : En tant que user, je veux voir toutes mes délégations actives (table)
- [EN ATTENTE] **US-388** : En tant que user, je veux voir les détails d'une délégation (protocols, limits, expiry, nonce)
- [EN ATTENTE] **US-389** : En tant que user, je veux révoquer une délégation (increments nonce on-chain)
- [EN ATTENTE] **US-390** : En tant que user, je veux créer une nouvelle délégation (wizard)
- [EN ATTENTE] **US-391** : En tant que user, je veux voir l'historique des preuves générées pour chaque délégation
### Epic 6.5.11 : Analytics & Charts

- [EN ATTENTE] **US-392** : En tant que user, je veux un graphique de la valeur du portfolio dans le temps (line chart)
- [EN ATTENTE] **US-393** : En tant que user, je veux un graphique de l'allocation (pie chart : % ETH, % DAI, etc.)
- [EN ATTENTE] **US-394** : En tant que user, je veux un graphique des yields générés (bar chart par protocol)
- [EN ATTENTE] **US-395** : En tant que user, je veux un graphique du gas spent (timeline)
### Epic 6.5.12 : Settings & Configuration

- [EN ATTENTE] **US-396** : En tant que user, je veux une page Settings pour configurer :
- [EN ATTENTE] **US-397** : En tant que user, je veux exporter mes données (intents, history, delegations) en JSON
- [EN ATTENTE] **US-398** : En tant que user, je veux un dark mode / light mode toggle
### Epic 6.5.13 : Mobile Responsive

- [EN ATTENTE] **US-399** : En tant que user mobile, je veux que toutes les pages soient responsive
- [EN ATTENTE] **US-400** : En tant que user mobile, je veux une navigation simplifiée (bottom nav ou hamburger)
- [EN ATTENTE] **US-401** : En tant que user mobile, je veux pouvoir créer un intent (textarea adapté)
- [EN ATTENTE] **US-402** : En tant que user mobile, je veux recevoir les notifications (push notifications via service worker)
### Epic 6.5.14 : Tests Frontend

- [EN ATTENTE] **US-403** : En tant que dev, je veux des tests unitaires pour les composants (Vitest + React Testing Library)
- [EN ATTENTE] **US-404** : En tant que dev, je veux des tests E2E (Playwright : connect wallet → create intent → delegate)
- [EN ATTENTE] **US-405** : En tant que dev, je veux tester le WebSocket (mock server)
- [EN ATTENTE] **US-406** : En tant que dev, je veux tester le WebSocket en temps réel (events flow)

## Vague 7 : Production-Ready (APIs,onitoring, Deployment) (6 semaines)

### Epic 7.1 : REST API

- [EN ATTENTE] **US-407** : En tant que backend, je veux un serveur HTTP Axum ou Actix-Web
- [EN ATTENTE] **US-408** : En tant qu'API, je veux exposer `POST /api/v1/intents/parse` (body: text → response: Intent)
- [EN ATTENTE] **US-409** : En tant qu'API, je veux exposer `POST /api/v1/intents` (create new intent)
- [EN ATTENTE] **US-410** : En tant qu'API, je veux exposer `GET /api/v1/intents` (list active intents)
- [EN ATTENTE] **US-411** : En tant qu'API, je veux exposer `GET /api/v1/intents/:id` (get intent details)
- [EN ATTENTE] **US-412** : En tant qu'API, je veux exposer `DELETE /api/v1/intents/:id` (cancel intent)
- [EN ATTENTE] **US-413** : En tant qu'API, je veux exposer `POST /api/v1/delegations` (create delegation)
- [EN ATTENTE] **US-414** : En tant qu'API, je veux exposer `GET /api/v1/delegations` (list delegations)
- [EN ATTENTE] **US-415** : En tant qu'API, je veux exposer `POST /api/v1/delegations/:id/revoke` (revoke delegation)
- [EN ATTENTE] **US-416** : En tant qu'API, je veux exposer `GET /api/v1/portfolio` (get portfolio state)
- [EN ATTENTE] **US-417** : En tant qu'API, je veux exposer `GET /api/v1/history` (execution history)
- [EN ATTENTE] **US-418** : En tant qu'API, je veux exposer `GET /api/v1/metrics` (stats & analytics)
### Epic 7.2 : Authentication & Security

- [EN ATTENTE] **US-419** : En tant qu'API, je veux authentifier via signature de message (EIP-4361 Sign-In with Ethereum)
- [EN ATTENTE] **US-420** : En tant qu'API, je veux générer un JWT après authentification
- [EN ATTENTE] **US-421** : En tant qu'API, je veux vérifier le JWT sur chaque requête protégée
- [EN ATTENTE] **US-422** : En tant qu'API, je veux rate limiting (100 req/min par user)
- [EN ATTENTE] **US-423** : En tant qu'API, je veux CORS configuré correctement (whitelist domains)
### Epic 7.3 : gRPC API (optionnel)

- [EN ATTENTE] **US-424** : En tant que backend, je veux un serveur gRPC (Tonic)
- [EN ATTENTE] **US-425** : En tant qu'API gRPC, je veux définir le protobuf schema (Intent, Delegation, Transaction, etc.)
- [EN ATTENTE] **US-426** : En tant qu'API gRPC, je veux exposer les mêmes méthodes que REST
- [EN ATTENTE] **US-427** : En tant qu'API gRPC, je veux supporter le streaming (stream des events en temps réel)
### Epic 7.4 : WebSocket Server

- [EN ATTENTE] **US-428** : En tant que backend, je veux un WebSocket server (tokio-tungstenite ou Axum WS)
- [EN ATTENTE] **US-429** : En tant que WS server, je veux accepter les connexions des clients
- [EN ATTENTE] **US-430** : En tant que WS server, je veux authentifier les connexions (JWT dans handshake)
- [EN ATTENTE] **US-431** : En tant que WS server, je veux broadcaster les events aux clients connectés
- [EN ATTENTE] **US-432** : En tant que WS server, je veux gérer les disconnections et reconnections
### Epic 7.5 : Monitoring & Observability

- [EN ATTENTE] **US-433** : En tant que système, je veux exposer des métriques Prometheus (`/metrics` endpoint)
- [EN ATTENTE] **US-434** : En tant que système, je veux tracker les métriques :
- [EN ATTENTE] **US-435** : En tant que ops, je veux configurer Grafana pour visualiser les métriques
- [EN ATTENTE] **US-436** : En tant que ops, je veux des dashboards :
### Epic 7.6 : Logging Structured

- [EN ATTENTE] **US-437** : En tant que système, je veux logger en JSON (structured logging avec tracing-subscriber)
- [EN ATTENTE] **US-438** : En tant que système, je veux inclure des contextes dans les logs (request_id, user_id, intent_id)
- [EN ATTENTE] **US-439** : En tant que système, je veux différents niveaux (ERROR, WARN, INFO, DEBUG, TRACE)
- [EN ATTENTE] **US-440** : En tant que ops, je veux envoyer les logs à un aggregator (Loki, ElasticSearch, CloudWatch)
### Epic 7.7 : Configuration Management

- [EN ATTENTE] **US-441** : En tant que système, je veux charger la config depuis un fichier TOML
- [EN ATTENTE] **US-442** : En tant que système, je veux override la config avec des env vars (12-factor app)
- [EN ATTENTE] **US-443** : En tant que système, je veux valider la config au démarrage (fail fast si invalid)
- [EN ATTENTE] **US-444** : En tant que système, je veux supporter différents environnements (dev, staging, prod)
### Epic 7.8 : Database (persistence)

- [EN ATTENTE] **US-445** : En tant que système, je veux persister les intents dans une DB (PostgreSQL)
- [EN ATTENTE] **US-446** : En tant que système, je veux persister les delegations
- [EN ATTENTE] **US-447** : En tant que système, je veux persister l'historique des transactions
- [EN ATTENTE] **US-448** : En tant que système, je veux persister les events de l'orchestrator (audit trail)
- [EN ATTENTE] **US-449** : En tant que système, je veux un `StoragePort` trait (`save_intent`, `get_intent`, `list_intents`, etc.)
- [EN ATTENTE] **US-450** : En tant qu'infra, je veux un `PostgresAdapter` implémentant `StoragePort`
- [EN ATTENTE] **US-451** : En tant que système, je veux des migrations DB (sqlx ou diesel migrations)
- [EN ATTENTE] **US-452** : En tant que système, je veux indexer les queries fréquentes (performance)
### Epic 7.9 : Backup & Recovery

- [EN ATTENTE] **US-453** : En tant qu'ops, je veux backup automatique de la DB (cron job)
- [EN ATTENTE] **US-454** : En tant qu'ops, je veux exporter les keystores (encrypted backups)
- [EN ATTENTE] **US-455** : En tant que système, je veux un script de recovery (restore depuis backup)
- [EN ATTENTE] **US-456** : En tant que système, je veux tester le recovery (disaster recovery drills)
### Epic 7.10 : Health Checks

- [EN ATTENTE] **US-457** : En tant que système, je veux un endpoint `GET /health` (retourne status: UP/DOWN)
- [EN ATTENTE] **US-458** : En tant que système, je veux checker les dépendances (DB, RPC node, Oracle, etc.)
- [EN ATTENTE] **US-459** : En tant que système, je veux un endpoint `GET /ready` (readiness probe pour Kubernetes)
- [EN ATTENTE] **US-460** : En tant qu'ops, je veux des alertes si health check fail (PagerDuty, Slack)
### Epic 7.11 : Deployment - Docker

- [EN ATTENTE] **US-461** : En tant que dev, je veux un Dockerfile multi-stage (build + runtime optimisé)
- [EN ATTENTE] **US-462** : En tant que dev, je veux un docker-compose.yml pour dev local (agent + DB + monitoring)
- [EN ATTENTE] **US-463** : En tant que dev, je veux builder les images pour différentes arches (amd64, arm64)
- [EN ATTENTE] **US-464** : En tant que ops, je veux publier les images sur un registry (Docker Hub, GHCR)
### Epic 7.12 : Deployment - Kubernetes (optionnel)

- [EN ATTENTE] **US-465** : En tant qu'ops, je veux des manifests Kubernetes (Deployment, Service, ConfigMap, Secret)
- [EN ATTENTE] **US-466** : En tant qu'ops, je veux un Helm chart pour simplifier le déploiement
- [EN ATTENTE] **US-467** : En tant qu'ops, je veux configurer l'autoscaling (HPA)
- [EN ATTENTE] **US-468** : En tant qu'ops, je veux configurer le monitoring (Prometheus Operator)
### Epic 7.13 : CI/CD Pipeline

- [EN ATTENTE] **US-469** : En tant que dev, je veux un workflow GitHub Actions pour build & test
- [EN ATTENTE] **US-470** : En tant que dev, je veux un workflow pour publish les images Docker
- [EN ATTENTE] **US-471** : En tant que dev, je veux un workflow pour déployer sur staging (auto)
- [EN ATTENTE] **US-472** : En tant que dev, je veux un workflow pour déployer sur prod (manual approval)
- [EN ATTENTE] **US-473** : En tant que dev, je veux des checks de qualité (coverage, clippy, audit)
### Epic 7.14 : Documentation Technique

- [EN ATTENTE] **US-474** : En tant que dev, je veux un README.md complet (installation, usage, architecture)
- [EN ATTENTE] **US-475** : En tant que dev, je veux générer la doc API avec OpenAPI/Swagger
- [EN ATTENTE] **US-476** : En tant que dev, je veux documenter les circuits Noir (inputs, outputs, contraintes)
- [EN ATTENTE] **US-477** : En tant que dev, je veux documenter les smart contracts (NatSpec)
- [EN ATTENTE] **US-478** : En tant que dev, je veux un ARCHITECTURE.md avec diagrammes (C4 model)
- [EN ATTENTE] **US-479** : En tant que dev, je veux un CONTRIBUTING.md pour les contributeurs
- [EN ATTENTE] **US-480** : En tant que dev, je veux générer la rustdoc (`cargo doc`)
### Epic 7.15 : Security Audit

- [EN ATTENTE] **US-481** : En tant que dev, je veux run `cargo audit` (check des vulnérabilités dans les deps)
- [EN ATTENTE] **US-482** : En tant que dev, je veux scanner les images Docker (Trivy, Snyk)
- [EN ATTENTE] **US-483** : En tant que dev, je veux faire un audit du circuit Noir (peer review ou professionnel)
- [EN ATTENTE] **US-484** : En tant que dev, je veux faire un audit des smart contracts (Slither, Mythril)
- [EN ATTENTE] **US-485** : En tant que dev, je veux un bug bounty program (post-launch)
### Epic 7.16 : Performance Optimization

- [EN ATTENTE] **US-486** : En tant que dev, je veux profiler l'application (flamegraph, perf)
- [EN ATTENTE] **US-487** : En tant que dev, je veux optimiser les hot paths (circuit compilation, proof generation)
- [EN ATTENTE] **US-488** : En tant que dev, je veux cacher les résultats coûteux (proving keys, verification keys)
- [EN ATTENTE] **US-489** : En tant que dev, je veux paralléliser les opérations indépendantes (rayon, tokio)
- [EN ATTENTE] **US-490** : En tant que dev, je veux benchmarker et comparer (avant/après optimizations)

## Vague 8 : Advancedeatures &olish (optionnel - 6 semaines) [FUTURE]

### Epic 8.1 : Multi-User Support

- [EN ATTENTE] **US-491** : En tant que système, je veux supporter plusieurs users simultanément
- [EN ATTENTE] **US-492** : En tant que système, je veux isoler les données par user (row-level security)
- [EN ATTENTE] **US-493** : En tant que système, je veux un user registry (mapping address → user_id)
- [EN ATTENTE] **US-494** : En tant que système, je veux des quotas par user (rate limiting, max intents)
### Epic 8.2 : Social Features

- [EN ATTENTE] **US-495** : En tant que user, je veux partager une stratégie publiquement (share link)
- [EN ATTENTE] **US-496** : En tant que user, je veux copier la stratégie d'un autre user (template)
- [EN ATTENTE] **US-497** : En tant que user, je veux voir un leaderboard (top performers)
- [EN ATTENTE] **US-498** : En tant que user, je veux follow d'autres users (notifications de leurs actions)
### Epic 8.3 : Advanced Intent Features

- [EN ATTENTE] **US-499** : En tant que user, je veux des intents récurrents : "rebalance every Monday at 9am"
- [EN ATTENTE] **US-500** : En tant que user, je veux des intents avec stop-loss : "sell if price drops 10%"
- [EN ATTENTE] **US-501** : En tant que user, je veux des intents composés complexes : "if X then Y else Z, repeat weekly"
- [EN ATTENTE] **US-502** : En tant que user, je veux des intents avec priorités (high/medium/low)
### Epic 8.4 : Portfolio Insights (AI)

- [EN ATTENTE] **US-503** : En tant que user, je veux des suggestions d'optimisation : "You could earn 0.5% more by..."
- [EN ATTENTE] **US-504** : En tant que user, je veux une analyse de risque : "Your portfolio has 75% in stablecoins, low risk"
- [EN ATTENTE] **US-505** : En tant que user, je veux des alertes proactives : "Yield on Aave dropped below 3%"
- [EN ATTENTE] **US-506** : En tant que user, je veux un AI assistant conversationnel : "Ask Otter anything about your portfolio"
### Epic 8.5 : Cross-Chain Support

- [EN ATTENTE] **US-507** : En tant que système, je veux supporter Arbitrum (L2)
- [EN ATTENTE] **US-508** : En tant que système, je veux supporter Optimism (L2)
- [EN ATTENTE] **US-509** : En tant que système, je veux supporter Polygon (sidechain)
- [EN ATTENTE] **US-510** : En tant que système, je veux un bridge adapter pour cross-chain transfers
- [EN ATTENTE] **US-511** : En tant que user, je veux des intents cross-chain : "Lend on Aave Arbitrum if yield > Mainnet"
### Epic 8.6 : Advanced Protocol Integrations

- [EN ATTENTE] **US-512** : En tant que système, je veux supporter Balancer (AMM)
- [EN ATTENTE] **US-513** : En tant que système, je veux supporter Yearn (vaults)
- [EN ATTENTE] **US-514** : En tant que système, je veux supporter Lido (liquid staking)
- [EN ATTENTE] **US-515** : En tant que système, je veux supporter GMX (perpetuals)
- [EN ATTENTE] **US-516** : En tant que système, je veux un plugin system pour ajouter facilement de nouveaux protocols
### Epic 8.7 : Simulation Mode

- [EN ATTENTE] **US-517** : En tant que user, je veux un mode simulation (paper trading)
- [EN ATTENTE] **US-518** : En tant que user en simulation, je veux un portfolio virtuel avec fake tokens
- [EN ATTENTE] **US-519** : En tant que user en simulation, je veux tester mes stratégies sans risque
- [EN ATTENTE] **US-520** : En tant que user en simulation, je veux voir les performances projetées
### Epic 8.8 : Mobile App (React Native)

- [EN ATTENTE] **US-521** : En tant que dev, je veux une app mobile React Native
- [EN ATTENTE] **US-522** : En tant que user mobile, je veux me connecter avec WalletConnect
- [EN ATTENTE] **US-523** : En tant que user mobile, je veux créer des intents (voice input optionnel)
- [EN ATTENTE] **US-524** : En tant que user mobile, je veux recevoir des push notifications natives
- [EN ATTENTE] **US-525** : En tant que user mobile, je veux voir mon dashboard (responsive native)
### Epic 8.9 : Compliance & Reporting

- [EN ATTENTE] **US-526** : En tant que user, je veux exporter un rapport fiscal (CSV des gains/pertes)
- [EN ATTENTE] **US-527** : En tant que user, je veux un rapport de compliance (toutes les actions avec timestamps)
- [EN ATTENTE] **US-528** : En tant que système, je veux logger toutes les actions pour audit (immutable log)
- [EN ATTENTE] **US-529** : En tant que système, je veux supporter des juridictions différentes (KYC optionnel)
### Epic 8.10 : Gamification

- [EN ATTENTE] **US-530** : En tant que user, je veux des achievements : "First lend", "10 successful rebalances", etc.
- [EN ATTENTE] **US-531** : En tant que user, je veux des badges visuels (NFTs optionnel)
- [EN ATTENTE] **US-532** : En tant que user, je veux un level system (XP basé sur volume traité)
- [EN ATTENTE] **US-533** : En tant que user, je veux des rewards (fee discounts pour high-level users)

## Vague 9 : Research & Whitepaper (4 semaines) [FUTURE]

### Epic 9.1 : Whitepaper - Introduction

- [EN ATTENTE] **US-534** : En tant qu'auteur, je veux écrire l'abstract (200 mots max)
- [EN ATTENTE] **US-535** : En tant qu'auteur, je veux écrire l'introduction (problématique DeFi)
- [EN ATTENTE] **US-536** : En tant qu'auteur, je veux expliquer les 3 problèmes (délégation, MEV, confidentialité)
- [EN ATTENTE] **US-537** : En tant qu'auteur, je veux positionner Otter vs solutions existantes (related work)
### Epic 9.2 : Whitepaper - Architecture

- [EN ATTENTE] **US-538** : En tant qu'auteur, je veux décrire l'architecture hexagonale
- [EN ATTENTE] **US-539** : En tant qu'auteur, je veux expliquer le flow complet avec diagrammes
- [EN ATTENTE] **US-540** : En tant qu'auteur, je veux détailler chaque composant (ZKP, MEV, Proof-of-Solvency)
- [EN ATTENTE] **US-541** : En tant qu'auteur, je veux inclure des sequence diagrams (PlantUML)
### Epic 9.3 : Whitepaper - Cryptographie

- [EN ATTENTE] **US-542** : En tant qu'auteur, je veux expliquer le circuit ZKP (EdDSA + delegation logic)
- [EN ATTENTE] **US-543** : En tant qu'auteur, je veux expliquer le schéma MEV (capture, redistribution, preuve ZK)
- [EN ATTENTE] **US-544** : En tant qu'auteur, je veux expliquer le threshold encryption (Shamir SSS)
- [EN ATTENTE] **US-545** : En tant qu'auteur, je veux inclure les preuves de sécurité (modèle de menace)
### Epic 9.4 : Whitepaper - Évaluation

- [EN ATTENTE] **US-546** : En tant qu'auteur, je veux présenter les benchmarks (temps de preuve, MEV, gas)
- [EN ATTENTE] **US-547** : En tant qu'auteur, je veux comparer avec l'état de l'art (tableau comparatif)
- [EN ATTENTE] **US-548** : En tant qu'auteur, je veux discuter les trade-offs (latency vs privacy)
- [EN ATTENTE] **US-549** : En tant qu'auteur, je veux inclure des graphiques de performance
### Epic 9.5 : Whitepaper - Conclusion

- [EN ATTENTE] **US-550** : En tant qu'auteur, je veux résumer les contributions
- [EN ATTENTE] **US-551** : En tant qu'auteur, je veux discuter les limitations actuelles
- [EN ATTENTE] **US-552** : En tant qu'auteur, je veux proposer des future works (extensions possibles)
- [EN ATTENTE] **US-553** : En tant qu'auteur, je veux rédiger les remerciements et références
### Epic 9.6 : Publication & Diffusion

- [EN ATTENTE] **US-554** : En tant qu'auteur, je veux publier sur arXiv (preprint)
- [EN ATTENTE] **US-555** : En tant qu'auteur, je veux soumettre à une conférence (IEEE S&P, USENIX, CCS)
- [EN ATTENTE] **US-556** : En tant qu'auteur, je veux créer un blog post vulgarisé (Medium, Mirror)
- [EN ATTENTE] **US-557** : En tant qu'auteur, je veux créer une vidéo explicative (YouTube)
- [EN ATTENTE] **US-558** : En tant qu'auteur, je veux présenter à des meetups (Ethereum, ZK, DeFi communities)

## Vague 10 : Open Source & Community (ongoing) [FUTURE]

### Epic 10.1 : Open Source Release

- [EN ATTENTE] **US-559** : En tant que maintainer, je veux choisir une licence (MIT, Apache 2.0, GPL)
- [EN ATTENTE] **US-560** : En tant que maintainer, je veux nettoyer le code (remove secrets, TODOs)
- [EN ATTENTE] **US-561** : En tant que maintainer, je veux créer un repo GitHub public
- [FAIT] **US-562** : En tant que maintainer, je veux écrire un CHANGELOG.md
- [EN ATTENTE] **US-563** : En tant que maintainer, je veux créer un CODE_OF_CONDUCT.md
### Epic 10.2 : Community Building

- [EN ATTENTE] **US-564** : En tant que maintainer, je veux créer un Discord server
- [EN ATTENTE] **US-565** : En tant que maintainer, je veux créer un forum de discussions (GitHub Discussions)
- [EN ATTENTE] **US-566** : En tant que maintainer, je veux créer un Twitter account pour announcements
- [EN ATTENTE] **US-567** : En tant que maintainer, je veux écrire des tutoriels (getting started, advanced usage)
- [EN ATTENTE] **US-568** : En tant que maintainer, je veux organiser des AMAs (Ask Me Anything)
### Epic 10.3 : Contributor Onboarding

- [EN ATTENTE] **US-569** : En tant que maintainer, je veux labeler les issues (good first issue, help wanted)
- [EN ATTENTE] **US-570** : En tant que maintainer, je veux reviewer les PRs rapidement (SLA 48h)
- [EN ATTENTE] **US-571** : En tant que maintainer, je veux remercier les contributors (CONTRIBUTORS.md)
- [EN ATTENTE] **US-572** : En tant que maintainer, je veux créer une roadmap publique (GitHub Projects)
### Epic 10.4 : Grants & Funding

- [EN ATTENTE] **US-573** : En tant que founder, je veux appliquer à des grants (Ethereum Foundation, Protocol Labs)
- [EN ATTENTE] **US-574** : En tant que founder, je veux créer un Gitcoin grant (community funding)
- [EN ATTENTE] **US-575** : En tant que founder, je veux explorer des VCs (si pivot vers startup)
- [EN ATTENTE] **US-576** : En tant que founder, je veux lancer un token (governance, incentives) (optionnel, controversé)
### Epic 10.5 : Maintenance & Evolution

- [EN ATTENTE] **US-577** : En tant que maintainer, je veux fixer les bugs reportés (issue triage)
- [EN ATTENTE] **US-578** : En tant que maintainer, je veux mettre à jour les dépendances (Dependabot)
- [EN ATTENTE] **US-579** : En tant que maintainer, je veux suivre les nouveaux protocoles DeFi (integrations)
- [EN ATTENTE] **US-580** : En tant que maintainer, je veux suivre les avancées crypto (nouveaux schemes ZK)

---

**Total stories: 481**
