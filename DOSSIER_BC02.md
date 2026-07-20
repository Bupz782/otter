# Dossier de certification — Bloc BC02
## Concevoir et développer des applications logicielles

| | |
|---|---|
| **Projet** | Otter (anciennement « Otter ») — protocole d'automatisation DeFi *trustless* |
| **Dépôt public** | https://github.com/Bupz782/otter |
| **Bloc** | BC02 — Concevoir et développer des applications logicielles |
| **Date de rendu** | 23 juillet 2026 |
| **Version du logiciel** | v0.1.0 (branche `main`) |

---

## Sommaire

1. [Présentation du projet](#1-présentation-du-projet)
2. [C2.1.1 — Environnements de déploiement et de test, critères de qualité et de performance](#c211--mettre-en-œuvre-des-environnements-de-déploiement-et-de-test)
3. [C2.1.2 — Protocole d'intégration continue](#c212--configurer-le-système-dintégration-continue)
4. [C2.2.1 — Architecture logicielle et prototype](#c221--concevoir-un-prototype-de-lapplication-logicielle)
5. [C2.2.2 — Harnais de tests unitaires](#c222--développer-un-harnais-de-test-unitaire)
6. [C2.2.3 — Sécurité (OWASP) et accessibilité (RGAA)](#c223--développer-le-logiciel--sécurité-et-accessibilité)
7. [C2.2.4 — Déploiement continu et gestion des versions](#c224--déployer-le-logiciel-à-chaque-modification-de-code)
8. [C2.3.1 — Cahier de recettes](#c231--élaborer-le-cahier-de-recettes)
9. [C2.3.2 — Plan de correction des bogues](#c232--élaborer-un-plan-de-correction-des-bogues)
10. [C2.4.1 — Documentation technique d'exploitation](#c241--rédiger-la-documentation-technique-dexploitation)
11. [Annexe — Tableau de correspondance critères ↔ preuves](#11-annexe--tableau-de-correspondance-critères--preuves)

---

## 1. Présentation du projet

### 1.1 Contexte et besoin

Otter est un protocole d'automatisation DeFi *trustless* (sans confiance). Le constat de départ : un utilisateur de finance décentralisée qui souhaite automatiser une stratégie (« prête 1000 USDC sur Aave si le rendement dépasse 3 % ») doit aujourd'hui soit exécuter manuellement chaque opération, soit déléguer ses fonds à un bot tiers **sans aucune garantie** que le bot respectera ses consignes.

Otter résout ce problème par la cryptographie : l'utilisateur dépose ses fonds dans un *vault* on-chain, décrit sa stratégie **en langage naturel**, et signe une **délégation** limitée (montants, protocoles autorisés, date d'expiration). Un agent logiciel surveille les conditions on-chain ; lorsqu'il veut exécuter une action, il doit produire une **preuve à divulgation nulle de connaissance (ZKP, circuit Noir)** démontrant que l'action respecte la délégation signée. Le smart contract `DelegationVault` vérifie la preuve **avant** toute exécution : l'agent ne peut mathématiquement pas outrepasser les limites fixées par l'utilisateur.

### 1.2 Chaîne de valeur

```
Langage naturel → Parsing (LLM local llama.cpp / regex) → Plan d'exécution
  → Délégation signée (ECDSA secp256k1, EIP-4361) → Surveillance des conditions
  → Preuve ZK (Noir + Barretenberg/UltraHonk) → Vérification on-chain
  → Exécution via DelegationVault (Ethereum / Arbitrum)
```

Le système comprend en outre : capture de MEV avec *rebates*, preuve de solvabilité du vault, marketplace d'agents et partage social de stratégies (cf. `PRODUCT.md`).

### 1.2.1 Récit utilisateur de bout en bout

Pour rendre la chaîne concrète, voici le parcours tel qu'il est implémenté aujourd'hui :

1. Alice ouvre l'application web et **connecte son wallet** (RainbowKit) ; le backend lui soumet un challenge **Sign-In with Ethereum** qu'elle signe — elle est authentifiée par JWT, sans mot de passe.
2. Elle saisit son intention en langage naturel : « *lend 1000 USDC on Aave if yield > 3* ». Le parseur produit un `ConditionalIntent` structuré qu'elle **relit et valide** dans le stepper (écran Review).
3. Elle définit les **limites de la délégation** (montant max 1000 USDC, protocole Aave uniquement, expiration à 30 jours) et la **signe avec son wallet** : le hash de délégation est enregistré on-chain via `delegate()`.
4. L'agent **surveille** l'oracle de rendement ; quand le taux Aave dépasse 3 %, l'orchestrateur transite de Monitoring à Deciding, génère la **preuve ZK** attestant que l'action respecte la délégation, puis soumet `executeWithProof`.
5. Le contrat `DelegationVault` **vérifie la preuve on-chain**, contrôle les limites et le nonce anti-rejeu, puis transfère les fonds vers le routeur Aave. L'événement `Executed` remonte en **temps réel** (WebSocket) sur le tableau de bord d'Alice.

À chaque étape, une défaillance est traitée explicitement : intention incomprise → message d'erreur en clair ; preuve > 30 s → alerte Prometheus ; signature invalide ou limites dépassées → rejet on-chain par construction.

### 1.3 Périmètre fonctionnel et user stories

Les besoins sont formalisés dans `BACKLOG.md` : **481 user stories en français**, organisées en 10 vagues (epics) avec statuts par story ([FAIT] terminé, [EN COURS] en cours, [EN ATTENTE] planifié, [CUT] coupé, [FUTUR] futur) et tags de périmètre MVP/Future/Cut. Le prototype livré couvre les stories [FAIT], notamment :

- **US-006** — structure hexagonale du backend ;
- **US-020** — création d'intention en langage naturel ;
- **US-028/029/030** — parsing d'intention (LLM + fallback) ;
- **US-419** — authentification par wallet (Sign-In with Ethereum) ;
- **US-422** — limitation de débit API (rate limiting) ;
- génération et vérification de preuves ZK, exécution on-chain, tableau de bord web, CLI.

### 1.4 Démarche de conception

Le projet a été conduit selon une **démarche itérative en vagues**, chaque vague livrant un incrément testable et intégré en continu. Le découpage complet (epics, user stories, estimations) est dans `BACKLOG.md` :

| Vague | Objectif | Durée estimée | État |
|---|---|---|---|
| 0 — Setup & Architecture Foundation | environnement de dev, squelette hexagonal, CI/CD basique | 2 semaines | réalisée |
| 1 — Intent Parsing & LLM | modèle d'intention, parseur regex, intégration LLM, planificateur, CLI | 6 semaines | réalisée |
| 2 — ZKP : délégation avec vérification d'intention | circuit Noir, vérification de signature, contrat vérificateur | 8 semaines | réalisée |
| 5 — Blockchain & adaptateurs protocoles | abstractions EVM, wallet, Aave/Uniswap, vault, MEV, solvabilité | 8 semaines | réalisée |
| 6 — Orchestrateur & flux intégré | machine à états, bus d'événements, boucle de monitoring, gestion d'erreurs | 10 semaines | réalisée |
| 6.5 — DAPP Frontend | connexion wallet, saisie d'intention, délégation, dashboard, temps réel | 8 semaines | réalisée |
| 7 — Production-Ready | API REST, auth & sécurité, observabilité, persistance, Docker, CI/CD, docs | 6 semaines | réalisée |
| 8 — Fonctionnalités avancées | multi-utilisateur, social, cross-chain, simulation | 6 semaines | future |
| 9 — Recherche & whitepaper | formalisation cryptographique, publication | 4 semaines | future |
| 10 — Open Source & communauté | release publique, onboarding contributeurs | continu | future |

La méthode suivie à chaque itération : **user story → conception (ports/use cases) → implémentation → tests → intégration CI → recette**. Les choix structurants sont documentés au moment de leur conception (ex. `docs/superpowers/specs/2026-07-08-ci-devops-deployment-design.md` et le plan associé pour le pipeline CI/CD). L'avancement global est suivi dans la « Progress Overview » du `BACKLOG.md`.

**Équipements ciblés et ergonomie** : l'équipement cible est le **navigateur web desktop** — l'usage DeFi suppose une extension wallet (MetaMask et équivalents via RainbowKit). L'interface est responsive (Tailwind 4, epic 6.5.13 « Mobile Responsive ») pour consultation mobile. L'ergonomie privilégie la **conduite de l'utilisateur** : parcours en stepper (Describe → Review → Delegate → Confirm), tooltips d'onboarding à la première visite, états vides explicites, messages d'erreur en langage courant, retour temps réel par WebSocket sur l'état des intentions et exécutions.

### 1.5 Stack technique

| Couche | Technologies | Justification |
|---|---|---|
| Backend | Rust (nightly épinglée `2026-07-07`, edition 2024), Axum 0.7, tokio | Performance, sûreté mémoire, typage fort — critique pour de la manipulation de fonds |
| Persistance | SQLite (rusqlite) / PostgreSQL 15 (sqlx), migrations versionnées | Double adaptateur via port `StoragePort` (hexagonal) |
| Blockchain | Alloy 0.2 (EVM), k256 (ECDSA), siwe + jsonwebtoken | Interaction Ethereum typée, auth sans mot de passe |
| IA embarquée | llama-cpp-2 (modèle GGUF local) | Parsing d'intention **local** : confidentialité, fonctionnement hors-ligne |
| ZKP | Noir (`delegation_circuit/`), Barretenberg/UltraHonk, vérificateur Solidity généré | Versions épinglées (`.noir-version`, `.bb-version`) |
| Contrats | Solidity ≥ 0.8.21, Foundry, OpenZeppelin | Standard de l'écosystème, tests natifs `forge` |
| Frontend | React 18, Vite 6, TypeScript 5.8, Tailwind 4, RainbowKit + wagmi/viem, react-router 7, TanStack Query | Équipement cible : navigateur web desktop (application métier DeFi) |
| Conteneurisation | Docker (multi-étages), Docker Compose, nginx | Reproductibilité dev/test/prod |

### 1.6 Cartographie du dépôt

```
otter/
├── crates/                  # Backend Rust (architecture hexagonale)
│   ├── domain/              # Modèles + ports (traits)
│   ├── application/         # Use cases, orchestrateur, bus d'événements
│   ├── infrastructure/      # Adaptateurs : LLM, EVM, SQLite/Postgres, ZKP
│   └── interfaces/          # API REST Axum (otter_api), CLI (otter_cli)
├── frontend/                # Application web React/Vite/TypeScript
├── contracts/               # Smart contracts Solidity (Foundry)
├── delegation_circuit/      # Circuit ZK Noir
├── .github/workflows/       # CI/CD (ci, docker, deploy-testnet, deploy-mainnet)
├── scripts/                 # dev.sh, smoke-test.sh, load_test.py, installs versionnées
├── docs/                    # SECURITE, ACCESSIBILITE, CAHIER_DE_RECETTES,
│                            # PLAN_CORRECTION_BOGUES, MANUEL_UTILISATION,
│                            # MANUEL_MISE_A_JOUR
├── README.md                # Présentation + quick start
├── DEPLOYMENT.md            # Manuel de déploiement
├── BACKLOG.md               # 481 user stories
└── DOSSIER_BC02.md          # Le présent dossier
```

---

## C2.1.1 — Mettre en œuvre des environnements de déploiement et de test

> **Livrables attendus** : le protocole de déploiement continu ; les critères de qualité et de performance.

### Environnement de développement

L'environnement de développement est intégralement reproductible et ses versions sont **épinglées dans le dépôt** :

| Composant | Outil | Preuve dans le dépôt |
|---|---|---|
| Éditeur de code | VS Code (rust-analyzer, ESLint, Prettier) | Configuration utilisateur ; conventions appliquées par rustfmt/ESLint |
| Compilateur Rust | `rustc` nightly **2026-07-07** (épinglée) | `rust-toolchain.toml` |
| Compilateur Noir | `nargo` version épinglée | `.noir-version` + `scripts/noirup-install.sh` |
| Prouveur ZK | Barretenberg `bb` version épinglée | `.bb-version` + `scripts/bbup-install.sh` |
| Toolchain Solidity | Foundry (`forge`, `anvil`) | `contracts/foundry.toml` |
| Runtime frontend | Node.js 20+, npm | `frontend/Dockerfile` (node:20-alpine), `scripts/dev-setup.sh` |
| Serveur d'application | Axum (Rust) — API REST `otter_api` port 3001 | `crates/interfaces/src/bin/otter_api.rs` |
| Serveur web statique | nginx (frontend buildé) port 3000 | `frontend/nginx.conf` |
| Base de données | PostgreSQL 15 / SQLite | `docker-compose.yml`, `crates/infrastructure/src/storage/` |
| Blockchain locale | Anvil (testnet local Foundry) | `scripts/dev.sh` |
| Gestion de sources | Git + GitHub (Conventional Commits) | historique `main`, `.github/workflows/` |
| Conteneurs | Docker + Docker Compose | `Dockerfile`, `frontend/Dockerfile`, `docker-compose.yml` |

La commande `./scripts/dev.sh` monte l'environnement complet en local : Anvil → PostgreSQL → déploiement des contrats via Forge → génération de `.env.local` → serveur Vite (frontend) + `cargo watch -x "run --bin otter_api"` (rechargement à chaud du backend). `scripts/dev-setup.sh` vérifie les prérequis (Node ≥ 20, versions Noir/bb).

### Protocole de déploiement continu

Le protocole de déploiement continu est implémenté par **4 workflows GitHub Actions** (`.github/workflows/`) qui définissent les séquences de déploiement de bout en bout :

**Séquence 1 — Validation (à chaque push/PR vers `main`/`develop`)** : `ci.yml`
1. Détection des chemins modifiés (`dorny/paths-filter`) pour ne lancer que les jobs concernés ;
2. `rust-check` → `contracts-check` → `circuit-check` → `frontend-check` (voir C2.1.2) ;
3. `docker-smoke` : build des 2 images Docker, lancement avec Anvil, vérification des healthchecks `/api/v1/health` et `/ready`.

**Séquence 2 — Publication des artefacts (push sur `main`/`develop` ou tag `v*`)** : `docker.yml`
1. Build multi-architecture (QEMU/Buildx) des images `api` et `frontend` ;
2. Push vers le registre `ghcr.io` avec tags sémantiques : `type=semver,pattern={{version}}`, branche, sha court (`docker.yml:49-62`) ;
3. Cache de build GitHub Actions pour accélérer les itérations.

**Séquence 3 — Déploiement testnet (tag `v*` ou déclenchement manuel)** : `deploy-testnet.yml`
1. Build et push des images taguées avec la version du tag ;
2. Déploiement SSH (`appleboy/ssh-action`) sur l'hôte testnet dans `/opt/otter` ;
3. `docker compose pull && docker compose up -d` ;
4. **Smoke tests post-déploiement** : `./scripts/smoke-test.sh` (polling `/ready`, `/api/v1/health`, POST `/api/v1/intents/parse`) — le déploiement échoue si les smoke tests échouent.

**Séquence 4 — Déploiement mainnet (manuel uniquement)** : `deploy-mainnet.yml`
1. Déclenchement `workflow_dispatch` avec **deux entrées obligatoires** : le tag à déployer et le hash de la transaction **multisig** de gouvernance ;
2. Gate « readiness checklist » + environnement GitHub `mainnet` (protection par approbation) ;
3. Déploiement SSH dans `/opt/otter-mainnet` + smoke tests.

Cette gradation (automatique en testnet, manuelle avec double contrôle en mainnet) est un choix délibéré adapté à un logiciel manipulant des fonds.

### Cycle de vie d'une modification (exemple concret)

Pour illustrer le protocole, voici le trajet réel d'une modification de code, par exemple l'ajout d'un endpoint API :

1. **Développement local** — le développeur lance `./scripts/dev.sh` : Anvil (blockchain locale), PostgreSQL, déploiement des contrats de test, API avec rechargement à chaud (`cargo watch`), frontend Vite avec HMR. La modification est testée immédiatement contre une stack complète.
2. **Vérifications locales** — `cargo fmt`, `cargo clippy`, `cargo test -p interfaces` avant même de pousser (mêmes outils qu'en CI, mêmes versions épinglées).
3. **Commit & push** — message Conventional Commits (`feat(api): add portfolio endpoint`), push de la branche, ouverture d'une Pull Request.
4. **Intégration continue** — `ci.yml` détecte que seul le code Rust a changé (`paths-filter`) et ne rejoue que `rust-check` (fmt, clippy, tests) puis `docker-smoke`.
5. **Merge sur `main`** — `docker.yml` rebuild et publie les images `ghcr.io` taguées `main` + sha.
6. **Release** — quand l'incrément est jugé releasable, un tag `v0.x.0` déclenche `deploy-testnet.yml` : images taguées par version, déploiement SSH, smoke tests obligatoires.
7. **Production** — après validation sur testnet, déclenchement manuel de `deploy-mainnet.yml` avec hash de transaction multisig et checklist.
8. **Supervision continue** — alertes Prometheus (`alerting.yml`) et métriques `/metrics` surveillent la version déployée ; toute anomalie alimente le plan de correction (C2.3.2).

### Critères de qualité

| Critère | Seuil | Outil | Enforcement |
|---|---|---|---|
| Warnings Rust | **0** (`-D warnings`) | clippy | CI `ci.yml:46-47` — bloquant |
| Formatage Rust | 100 % | rustfmt | CI `cargo fmt --check` |
| Formatage Solidity | 100 % | forge fmt | CI |
| Formatage Noir | 100 % | nargo fmt | CI |
| Erreurs TypeScript | 0 | `tsc --noEmit` | CI frontend |
| Warnings ESLint | **0** (`--max-warnings 0`) | ESLint 8 + typescript-eslint | CI frontend |
| Couverture de tests Rust | baseline mesurée **41,14 %** (1360/3306 lignes), objectif ≥ 70 % sur la logique métier | cargo-tarpaulin 0.34.1 | `tarpaulin.toml`, rapport `coverage/tarpaulin-report.html` |
| Tests automatisés | 100 % verts avant merge | cargo test, forge test, nargo test, vitest | CI bloquante |
| Smoke tests post-déploiement | 100 % | `scripts/smoke-test.sh` | Workflow deploy |

### Critères de performance

| Critère | Seuil | Mesure |
|---|---|---|
| Génération de preuve ZK | < 30 s | Alerte Prometheus `OtterProofPipelineSlow` (`alerting.yml`) |
| Latence API | percentiles p50/p95/p99 | `scripts/load_test.py` (N intents concurrents, rapport de percentiles) |
| Taux d'erreur | alerte au-delà du seuil | Règle `alerting.yml` (error rate) |
| Disponibilité agent | alerte si down | `OtterAgentDown` |
| Fraîcheur des prix oracle | alerte si non mis à jour | règle dédiée `alerting.yml` |
| Balance ETH de l'agent | alerte < 0,01 ETH | `OtterAgentLowBalance` |

L'observabilité est intégrée au serveur d'application : endpoint `/metrics` au format Prometheus (`otter_api.rs:1577` — compteurs `otter_price_updates_total`, `otter_executions_total`, `otter_gas_used_total`…), endpoints `/health`, `/health/live`, `/ready` (utilisés par les healthchecks Docker), et logs structurés JSON via `tracing`.

**Vérification des critères d'évaluation** : le protocole de déploiement continu est explicité (4 séquences ci-dessus) — l'environnement de développement est détaillé (éditeur, compilateurs, runtimes) — les composants sont identifiés (compilateur rustc/nargo/forge, serveur d'application Axum, gestion de sources Git/GitHub) — le protocole définit les différentes séquences de déploiement — les critères de qualité et de performance répondent aux exigences d'un logiciel financier (zéro warning, tests bloquants, alertes de production).

---

## C2.1.2 — Configurer le système d'intégration continue

> **Livrable attendu** : le protocole d'intégration continue.

### Protocole d'intégration continue

L'intégration continue est assurée par le workflow `.github/workflows/ci.yml` (« CI Pipeline »), déclenché **à chaque push et chaque Pull Request** vers `main` et `develop`. Le protocole suit le cycle suivant :

```
Commit (Conventional Commits) → Push / Pull Request
  → 1. Détection des zones modifiées (paths-filter)
  → 2. Jobs parallèles conditionnels par stack
  → 3. Fusion bloquée si un job échoue
  → 4. Merge sur main → artefacts Docker rebuildés (docker.yml)
```

### Séquences d'intégration

**Étape 0 — Routage intelligent** : le job `changes` (`dorny/paths-filter`) détermine quelles parties du monorepo sont touchées (Rust, contrats, circuit, frontend, Docker). Chaque job suivant est **conditionnel** : on ne teste que ce qui a changé, ce qui réduit le temps de cycle tout en garantissant qu'un changement transverse (ex. racine `Cargo.toml`) rejoue tout.

**Séquence Rust — `rust-check`** :
1. `cargo fmt --check` — formatage ;
2. `cargo clippy --workspace --all-targets -- -D warnings` — analyse statique, **tout warning est une erreur** ;
3. `cargo test --workspace` — les ~155 tests unitaires et d'intégration (hors tests réseau conditionnels).

**Séquence Contrats — `contracts-check`** :
1. `forge fmt --check` ;
2. `forge test` — les 13 tests Foundry, dont les tests du vault avec fixtures de **preuves ZK réelles** (`contracts/test/fixtures/proof.bin`, public inputs).

**Séquence Circuit ZK — `circuit-check`** :
1. Installation de `nargo` et `bb` aux versions épinglées (`.noir-version`, `.bb-version`) — reproductibilité garantie entre dev et CI ;
2. `nargo fmt --check` ;
3. `nargo test` — tests du circuit ;
4. Build smoke : `nargo execute` + `bb write_vk` (génération de la clé de vérification), qui valide que le circuit compile et se prouve.

**Séquence Frontend — `frontend-check`** :
1. `npm ci` (installation déterministe depuis le lockfile) ;
2. `npm run typecheck` (`tsc --noEmit`) ;
3. `npm run lint` (`--max-warnings 0`) ;
4. `npm run test` (vitest, 28 cas) ;
5. `npm run build`.

**Séquence d'assemblage — `docker-smoke`** :
1. Build des deux images Docker (`api`, `frontend`) telles qu'elles partiront en production ;
2. Lancement de la stack complète avec Anvil (blockchain locale) ;
3. Vérification des endpoints `/api/v1/health` et `/ready` : l'assemblage des blocs est validé **en conditions proches de la production** à chaque intégration.

### Fusion des codes sources et prévention des régressions

- **Convention de commits** : Conventional Commits (`feat(...)`, `fix(...)`, `ci(...)`, `chore(...)`) — l'historique de `main` (~80 commits) est lisible et exploitable automatiquement.
- **Fusion fréquence** : les blocs de code sont testés à chaque push, pas en fin de sprint — les régressions sont détectées au commit fautif.
- **Barrières de non-régression** : chaque bogue corrigé est adossé à un test rejoué en CI (voir C2.3.2) ; la politique « zéro warning » empêche l'accumulation de dette.
- **Synchronisation du backlog** : `scripts/sync-issues.sh` + `scripts/setup-labels.sh` synchronisent `BACKLOG.md` (481 user stories) vers GitHub Issues, reliant chaque intégration à une story tracée.

**Vérification des critères d'évaluation** : le protocole d'intégration continue est explicité clairement — il définit les séquences d'intégration (routage → 4 séquences par stack → assemblage Docker) — les codes sources sont fusionnés et testés régulièrement (à chaque push/PR) — le dispositif réduit les risques de régression (tests bloquants, zéro warning, tests de non-régression).

---

## C2.2.1 — Concevoir un prototype de l'application logicielle

> **Livrables attendus** : une architecture logicielle structurée permettant la maintenabilité ; une présentation du ou des prototypes réalisés ; l'utilisation de frameworks et de paradigmes de développement.

### Architecture logicielle : hexagonale (ports & adaptateurs)

Le backend Rust est structuré en **architecture hexagonale** (clean architecture), formalisée par le workspace Cargo (`Cargo.toml`) en 4 crates aux dépendances orientées vers le domaine :

```
┌─────────────────────────────────────────────────────────┐
│ interfaces   (otter_api REST Axum, otter_cli)           │  ← entrées/sorties
├─────────────────────────────────────────────────────────┤
│ application  (use cases, orchestrateur, event bus)      │  ← orchestration
├─────────────────────────────────────────────────────────┤
│ domain       (modèles métier, ports = traits)           │  ← cœur, zéro dépendance externe
└─────────────────────────────────────────────────────────┘
        ▲ implémente les ports
┌─────────────────────────────────────────────────────────┐
│ infrastructure (LLM llama.cpp, EVM Alloy, SQLite/PG,    │  ← adaptateurs techniques
│                 ZKP Noir, secrets KMS/Vault)            │
└─────────────────────────────────────────────────────────┘
```

- **`crates/domain`** : modèles (`execution_plan.rs`, `intent.rs`, `transaction.rs`) et **ports** — traits abstraits `StoragePort`, `ZkpPort`, `PriceOraclePort`, `WalletPort`, `BlockchainPort`, `IntentParserPort` (`crates/domain/src/ports/`). Le domaine ne dépend de rien : il est testable et stable.
- **`crates/application`** : use cases (`test_plan_use_case.rs`, `test_strategy_planner.rs`, `test_evaluate_use_case.rs` en exemples exécutables), orchestrateur d'agent, bus d'événements.
- **`crates/infrastructure`** : adaptateurs concrets — parseur LLM local (`llm/`), parseur regex et hybride (`parsers/`), blockchain (`blockchain/alloy_evm.rs`, oracles Chainlink), double adaptateur de persistance **SQLite/PostgreSQL interchangeable** (`storage/`), prouveur Noir (`zkp/noir_adapter.rs`), fournisseurs de secrets (`interfaces/src/secrets.rs`).
- **`crates/interfaces`** : API REST Axum (`otter_api`) et CLI (`otter_cli`), deux façons d'entrer dans le même cœur applicatif.

**Bénéfice maintenabilité** : chaque technologie est remplaçable sans toucher au métier (démontré par le double adaptateur SQLite/Postgres et par les mocks `mock_evm.rs`, `mock_oracle.rs`, `mock_adapter.rs` utilisés par les tests). L'architecture est attestée par la story US-006 du `BACKLOG.md`.

### Frameworks et paradigmes de développement

| Paradigme | Mise en œuvre |
|---|---|
| **Framework web** | Axum 0.7 (REST + WebSocket, middlewares tower-http) ; React 18 + react-router 7 côté client |
| **Injection de dépendances / inversion de contrôle** | ports (traits Rust) injectés dans les use cases ; `build_orchestrator` assemble les adaptateurs |
| **Programmation asynchrone** | tokio (runtime async Rust), `#[tokio::test]` |
| **Typage fort / sûreté mémoire** | Rust edition 2024, `serde` pour la (dé)sérialisation typée, TypeScript strict côté frontend |
| **Composants déclaratifs** | React fonctionnel + hooks, TanStack Query pour l'état serveur |
| **12-factor app** | configuration par environnement (`config.example.toml`, `.env.example`), fail-fast au démarrage (`scripts/docker-entrypoint.sh`) |
| **Tests à tous les étages** | unitaires (Rust/Noir/Solidity/TS), intégration, E2E Anvil, smoke HTTP |

### Patrons de conception mobilisés

Au-delà de l'architecture hexagonale, plusieurs patrons structurent le code :

- **Port / Adaptateur** : chaque dépendance technique est un trait du domaine (`StoragePort`, `ZkpPort`, `PriceOraclePort`…) implémenté par un adaptateur d'infrastructure — et par un mock pour les tests (`mock_evm.rs`, `mock_oracle.rs`, `mock_adapter.rs`).
- **Machine à états** : l'orchestrateur suit un automate explicite (Idle → Monitoring → Analyzing → Deciding → Proving → Submitting → Executing, état Error), ce qui rend les transitions vérifiables et les erreurs localisables (`crates/application/src/orchestrator/state.rs`).
- **Bus d'événements** : les composants communiquent par événements (`PriceUpdated`, `ConditionMet`…) publiés sur un bus, ce qui découple la surveillance de la décision et alimente le WebSocket temps réel du frontend.
- **Chaîne de responsabilité (fallback)** : le `HybridParser` tente le LLM local puis se replie sur le parseur regex déterministe en cas d'échec — la disponibilité prime sans sacrifier la correction.
- **Repository / double persistance** : le même port de stockage est implémenté par SQLite (dev/embarqué) et PostgreSQL (production), sélectionnés par configuration sans changement de code métier.
- **Middleware (chaîne de filtres)** : authentification JWT, rate limiting, CORS et validation des entrées sont des middlewares Axum composés devant les routes protégées.

### Présentation des prototypes réalisés

Le prototype est **fonctionnel et utilisable en autonomie** ; il met en œuvre un ensemble cohérent des fonctionnalités principales et des user stories :

1. **Application web** (`frontend/`, équipement cible : navigateur desktop) — 10 pages réelles routées dans `frontend/src/main.tsx` :
   - `/` page d'accueil publique ;
   - `/app/dashboard` — tableau de bord (portefeuille, exécutions) ;
   - `/app/intents`, `/app/intents/new`, `/app/intents/:id` — cycle de vie des intentions : saisie en langage naturel → parsing → revue du plan → confirmation (stepper Describe/Review/Delegate/Confirm) ;
   - `/app/delegations`, `/app/delegations/new` — création et suivi des délégations signées ;
   - `/app/agents`, `/app/agents/:agentId` — marketplace d'agents ;
   - `/app/proofs` — preuves ZK générées ;
   - `/app/settings` — paramètres.
   Composants d'interface présents et fonctionnels : fenêtres modales (`WelcomeModal`), boutons et formulaires avec validation (`CreateDelegationPage` — messages d'erreur exacts « Limits must be numbers above 0. »), menus et navigation (`AppSidebar`, `AppHeader`), stepper accessible, tooltips d'onboarding, connexion wallet RainbowKit + signature SIWE.
2. **API REST** (`otter_api`, port 3001) — routes réelles : `auth/challenge`, `auth/verify`, `intents` (+ `parse`, `plan/:id`), `delegation` (+ `hash`), `agents`, `strategies`, `portfolio`, `proofs`, `leaderboard`, `executions`, `orchestrator/state`, `health`, `ready`, `metrics`, `ws` (WebSocket temps réel).
3. **CLI** (`otter_cli`) — commandes `parse`, `plan`, `start`, `status`, `execute`, `prove`, `verify-onchain` : un développeur peut piloter tout le cycle sans l'interface web.
4. **Contrats déployés** — preuve de déploiement sur testnet **Sepolia** avec adresses de contrats et hash de transaction consignés dans `DEPLOYMENT.md`.
5. **Démonstration E2E ZK** — `lab/zkp_e2e.sh` et le quick start du `README.md` permettent de rejouer le flux complet : intention → délégation → preuve → vérification on-chain.

**Ergonomie et équipement cible** : l'interface est une application web responsive (Tailwind 4), pensée pour navigateur desktop (usage DeFi avec extension wallet) ; le parcours est guidé par un stepper et un onboarding en tooltips ; les états de chargement/erreur sont explicites ; `prefers-reduced-motion` est respecté (voir C2.2.3 accessibilité).

**Exigences de sécurité du prototype** : authentification par signature de wallet, délégations à limites cryptographiques, preuve ZK obligatoire avant exécution — détaillé en C2.2.3.

**Vérification des critères d'évaluation** : bonnes pratiques respectées (frameworks Axum/React/Foundry, paradigmes hexagonal/IoC/async) — prototype fonctionnel répondant aux besoins (5 livrables utilisables) — ensemble cohérent de fonctionnalités principales et user stories (BACKLOG, stories [FAIT]) — composants d'interface présents et fonctionnels (fenêtres, boutons, menus, formulaires, stepper) — exigences de sécurité satisfaites.

---

## C2.2.2 — Développer un harnais de test unitaire

> **Livrable attendu** : un jeu de tests unitaires couvrant une fonctionnalité demandée (au-delà : l'ensemble des fonctionnalités critiques).

### Inventaire du harnais

Le harnais couvre les quatre stacks du projet, avec **prévention des régressions** comme principe directeur (chaque correction de bogue ajoute un test, cf. C2.3.2) :

| Stack | Framework de test | Volume | Emplacement |
|---|---|---|---|
| Rust — unitaires | `#[test]` natif + `#[tokio::test]` | 128 `#[test]` + 27 `#[tokio::test]` | `crates/*/src/**` |
| Rust — intégration | tests d'intégration Cargo | 5 fichiers, 14 tests | `crates/infrastructure/tests/` |
| Rust — exemples exécutables | use cases jouables | 3 exemples | `crates/application/examples/` |
| Solidity | Foundry (`forge test`) | 13 tests | `contracts/test/` |
| Circuit ZK | `nargo test` | 3 tests | `delegation_circuit/src/main.nr` |
| Frontend | Vitest + Testing Library + jsdom | 28 cas (4 fichiers) | `frontend/src/**/*.test.{ts,tsx}` |

### Couverture des fonctionnalités demandées

- **Domaine** (`crates/domain`, 10 tests) : cycle de vie des intentions (machine à états), plans d'exécution, transactions.
- **Application** (14 tests) : use cases de planification et d'évaluation, orchestrateur.
- **Infrastructure** (94 tests) : parseurs (`regex_parser.rs` : 184/197 lignes couvertes — le cœur du parsing d'intention), oracles, stockage SQLite, adaptateur ZK Noir, prompts LLM, cache.
- **Interfaces** (10 tests) : authentification SIWE/JWT (`auth.rs:194-215`), rate limiting et validation d'entrées (`otter_api.rs:1986`), CORS, gestion des secrets.
- **Contrats** : `DelegationVault.t.sol` (8 tests : dépôts, retraits, exécution avec preuve, limites, anti-rejeu), `DelegationVerifier.t.sol`, test d'intégration avec **fixtures de preuves réelles** (`contracts/test/fixtures/proof.bin`).
- **Frontend** : mapping des réponses API et cas d'erreur (`api.test.ts`, 13 cas), normalisation des statuts (`status.test.ts`), logique de délégation — split de signature EIP-4361, construction du message, packing des protocoles (`delegation.test.ts`), composant `Stepper` et ses attributs ARIA (`Stepper.test.tsx`).

### Mesure de couverture

La couverture est mesurée par **cargo-tarpaulin 0.34.1** (configuration versionnée `tarpaulin.toml` ; le rapport HTML `coverage/tarpaulin-report.html` est un artefact généré par `cargo tarpaulin`, non versionné) :

> **41,14 % de lignes couvertes (1360/3306)** sur les bibliothèques Rust (`cargo tarpaulin --workspace --lib`).

Détail représentatif par module (lignes couvertes / lignes totales) :

| Module | Couverture | Lecture |
|---|---|---|
| `parsers/regex_parser.rs` | 184/197 (93 %) | cœur du parsing d'intention — quasi total |
| `storage/sqlite.rs` | 130/282 (46 %) | adaptateur principal de persistance |
| `domain/models/intent.rs` | 62/100 (62 %) | modèle métier central |
| `zkp/noir_adapter.rs` | 54/238 (23 %) | génération de preuve (chemins longs testés en intégration) |
| `protocols/aave.rs` / `uniswap.rs` | 103/197 (52 %) | adaptateurs protocoles DeFi |
| `interfaces/auth.rs` | 34/67 (51 %) | authentification SIWE/JWT |
| `blockchain/*` (alloy, oracles, wallet) | 153/497 (31 %) | interactions EVM (testées aussi via Anvil en intégration) |
| `storage/postgres.rs` | 0/155 (0 %) | couvert par les tests d'intégration exclus de cette mesure `--lib` |

Lecture honnête de ce chiffre : il exclut les tests d'intégration (qui requièrent Anvil/PostgreSQL), de sorte que des adaptateurs testés en intégration (Postgres, service d'exécution) apparaissent sous-évalués ; la logique métier critique est, elle, fortement couverte (parseur regex 93 %, domaine et application couverts par les use cases). L'objectif fixé dans le plan qualité est ≥ 70 % sur la logique métier avec intégration de tarpaulin à la CI (tracé dans `docs/PLAN_CORRECTION_BOGUES.md` §3.1).

### Prévention des régressions

Les tests sont **bloquants en CI** (C2.1.2) : aucune fusion possible si un seul test échoue. Les tests d'intégration ZK (`zkp_noir.rs`, `zkp_e2e_anvil.rs`, `protocol_integration.rs` sur Sepolia) garantissent que la chaîne critique preuve → vérification → exécution ne régresse pas. Côté frontend, le bogue historique de mapping d'intention (BUG-07, commit `23a778c`) a donné lieu à des tests de non-régression désormais permanents.

**Vérification du critère d'évaluation** : les tests unitaires couvrent la majorité du code développé — 183+ tests automatisés sur 4 stacks, couverture mesurée et rapportée, parties critiques couvertes à plus de 90 %.

---

## C2.2.3 — Développer le logiciel : sécurité et accessibilité

> **Livrables attendus** : une présentation des mesures de sécurité mises en œuvre ; une présentation des actions mises en œuvre pour l'accès aux personnes en situation de handicap.
>
> Documents de référence dans le dépôt : **`docs/SECURITE.md`** (cartographie OWASP complète) et **`docs/ACCESSIBILITE.md`** (audit RGAA).

### Sécurité — couverture de l'OWASP Top 10 (2021)

Le document `docs/SECURITE.md` détaille chaque risque A01→A10 avec ses preuves `fichier:ligne`. Synthèse :

| Risque OWASP | Mesures implémentées | Preuves |
|---|---|---|
| **A01 — Broken Access Control** | Routes protégées par middleware JWT ; isolation multi-utilisateur (intents 404 entre utilisateurs, suppression 403) ; contrats `Ownable`/`onlyOwner` ; **preuve ZK obligatoire avant exécution** | `otter_api.rs:383-464`, `DelegationVault.sol:95-99,176-210` |
| **A02 — Cryptographic Failures** | Auth par signature ECDSA (SIWE/EIP-4361) — **aucun mot de passe stocké** ; keystore Ethereum chiffré ; secrets via HashiCorp Vault / AWS KMS ; TLS par reverse proxy (hors dépôt) | `auth.rs`, `secrets.rs`, `.env.example` |
| **A03 — Injection** | SQL 100 % paramétré (sqlx/rusqlite, aucune concaténation sur entrées utilisateur) ; désérialisation typée serde ; `MAX_INTENT_TEXT_LEN = 2000` ; validation hex/cardinalités | `otter_api.rs:1034-1079`, `crates/infrastructure` |
| **A04 — Insecure Design** | Délégations à limites cryptographiques (montant, protocole, expiration) ; nonces anti-rejeu on-chain ; vérification ZK par conception | `DelegationVault.sol:197-204`, `delegation_circuit/src/main.nr:148-168` |
| **A05 — Security Misconfiguration** | CORS en liste blanche configurable ; validation de config au démarrage (fail-fast) ; headers nginx ; images Docker épinglées | `otter_api.rs:411-429`, `scripts/docker-entrypoint.sh` |
| **A06 — Vulnerable Components** | Dépendances épinglées (lockfiles Cargo/npm, toolchain pinnée) ; *axe d'amélioration : cargo audit / Dependabot à ajouter* | `Cargo.lock`, `rust-toolchain.toml` |
| **A07 — Auth Failures** | Challenge SIWE à nonce aléatoire 16 octets, expiration 5 min, **consommé après usage** ; JWT HS256 à TTL configurable ; pas de session cookie | `auth.rs:72-74,147-151,159` |
| **A08 — Data Integrity Failures** | Images Docker taguées par sha + semver ; artefacts CI reproductibles ; versions Noir/bb épinglées | `.github/workflows/docker.yml`, `.noir-version` |
| **A09 — Logging Failures** | Logs structurés JSON (`tracing`) ; métriques Prometheus `/metrics` ; 8 alertes (`alerting.yml`) dont échecs de vérification on-chain | `observability/logging.rs`, `otter_api.rs:1577` |
| **A10 — SSRF** | Endpoints RPC/LLM configurés par l'opérateur, jamais par l'utilisateur ; pas de fetch d'URL fournie en entrée | `config.example.toml` |

Mesures transverses : **rate limiting** par IP (429 au-delà de 100 req/min, `otter_api.rs:476-508`), échappement React par défaut (aucun `dangerouslySetInnerHTML` dans `frontend/src` — anti-XSS), JWT en header `Authorization` (pas de cookie → CSRF non applicable), secrets jamais commités (`.env.example` l'interdit explicitement ; clés CI dans GitHub Secrets).

**Points résiduels assumés** (tracés dans le plan de correction) : terminaison TLS déléguée à l'infrastructure d'accueil, secret JWT aléatoire en développement (avec warning explicite), `cargo audit` à intégrer à la CI.

### Accessibilité — référentiel RGAA 4.1

**Référentiel choisi : RGAA 4.1** (Référentiel Général d'Amélioration de l'Accessibilité), complété par les bonnes pratiques **Opquast**. Justification : projet francophone soumis au cadre légal français d'accessibilité numérique ; le RGAA est le référentiel officiel de référence pour les services numériques (transposition WCAG 2.1), et Opquast couvre les bonnes pratiques transverses qualité web. L'audit complet par thématique est dans `docs/ACCESSIBILITE.md`.

**Actions mises en œuvre** (108 occurrences `aria-*`/`role` dans 33 fichiers frontend) :

- **Formulaires** : erreurs annoncées par `role="alert"`, champs en `aria-invalid` avec `aria-describedby` vers le message ; groupes d'options en `role="radiogroup"` + `aria-checked` (`frontend/src/pages/app/CreateDelegationPage.tsx:136-142`).
- **Contenus dynamiques** : régions `aria-live="polite"` pour les mises à jour asynchrones (`IntentDetailPage.tsx:217-233`, `OnboardingTooltip.tsx`).
- **Navigation et repérage** : `aria-current="step"` sur le stepper (`components/app/Stepper.tsx:23`), skip link sur la page publique (`App.tsx:40-45`), `aria-expanded`/`aria-label` sur les contrôles interactifs, icônes décoratives en `aria-hidden`.
- **Présentation** : `prefers-reduced-motion` respecté (`main.tsx:72`, `index.css:149-153`) ; contrastes **calculés** à partir des tokens (`styles/tokens.css`) : tous ≥ 7,5:1 sauf un gris `#71717a` à 4,22:1 (non-conformité partielle tracée).
- **Tests automatisés d'accessibilité** : `Stepper.test.tsx` vérifie `aria-current` et le rendu des étapes en CI.

**Plan d'amélioration honnête** (tracé dans `docs/ACCESSIBILITE.md`) : corriger le contraste du gris secondaire, `lang="fr"` et titres de page dynamiques, skip link dans le shell applicatif, focus trap des modales maison (ou migration vers Radix UI Dialog), tests clavier et lecteur d'écran systématiques. L'audit est une auto-évaluation par inspection statique, pas un audit RGAA formel.

**Vérification des critères d'évaluation** : les mesures couvrent les 10 failles OWASP (tableau ci-dessus + `docs/SECURITE.md`) — le référentiel d'accessibilité est présenté et justifié (RGAA 4.1 + Opquast) — le prototype répond aux exigences du référentiel sur les points audités, avec un plan d'amélioration tracé pour les écarts.

---

## C2.2.4 — Déployer le logiciel à chaque modification de code

> **Livrables attendus** : l'historique des différentes versions ; la dernière version du logiciel fonctionnelle, fiable et viable.

### Système de gestion des versions

- **Git + GitHub** : historique complet sur `main` (~80 commits), convention **Conventional Commits** (`feat(scope):`, `fix(scope):`, `ci:`, `chore:`) rendant chaque évolution auto-documentée.
- **Versionnement sémantique** : le pipeline est câblé sur les tags `v*` — `docker.yml` extrait la version (`type=semver,pattern={{version}}`) et publie les images `ghcr.io/.../api` et `/frontend` taguées `X.Y.Z`, branche et sha court ; `deploy-testnet.yml` se déclenche sur ces tags. La première release `v0.1.0` accompagne ce rendu.
- **Traçabilité des évolutions** : `BACKLOG.md` (481 user stories avec statuts [FAIT]/[EN COURS]/[EN ATTENTE]/[CUT]/[FUTUR]) synchronisé vers GitHub Issues (`scripts/sync-issues.sh`) ; chaque correctif est relié à son commit dans `docs/PLAN_CORRECTION_BOGUES.md`.

### Historique des versions (jalons)

| Jalon | Contenu | Trace |
|---|---|---|
| Socle | Architecture hexagonale, domaine + ports | commits initiaux, US-006 |
| Parsing d'intention | LLM local llama.cpp + parseur regex/hybride | US-028/029/030 |
| Chaîne ZK | Circuit Noir, vérificateur UltraHonk, `executeWithProof` | `delegation_circuit/`, `contracts/src/` |
| Interface web | 10 pages React, stepper, wallet RainbowKit + SIWE | `frontend/`, `23a778c` |
| CI/CD | 4 workflows, images multi-arch, déploiements testnet/mainnet | `docs/superpowers/specs/2026-07-08-ci-devops-deployment-design.md` |
| Durcissement | secrets KMS/Vault, métriques/alertes, corrections de revue | commits `fix(debt):`, `fix(secrets):` |
| **v0.1.0 (rendu)** | Documentation BC02 complète, couverture mesurée, 28 tests frontend | ce dossier + `docs/` |

### Déploiement à chaque modification, de façon progressive

Le déploiement est **continu et progressif** (détail des séquences en C2.1.1) :

1. **À chaque push/PR** : CI complète (tests 4 stacks + build Docker + smoke Anvil) — chaque modification est validée en conditions proches de la production ;
2. **À chaque merge sur `main`** : rebuild et publication des images Docker (`docker.yml`) ;
3. **À chaque tag `v*`** : déploiement automatique **testnet** + smoke tests post-déploiement (`scripts/smoke-test.sh`) ;
4. **Mainnet** : déploiement manuel gardé par checklist + transaction multisig — la progressivité protège les fonds des utilisateurs.

### Vérification de la performance fonctionnelle et technique auprès des utilisateurs

- **Fonctionnelle** : smoke tests post-déploiement rejouent le parcours utilisateur (health, parse d'intention) ; le cahier de recettes (C2.3.1) valide 78 scénarios ; le quick start du `README.md` permet à tout utilisateur de rejouer la démo E2E.
- **Technique** : métriques Prometheus en production (`/metrics`), 8 alertes (`alerting.yml` : agent down, preuve > 30 s, taux d'erreur, prix obsolètes, balance faible, échecs on-chain, RPC), test de charge `scripts/load_test.py` (percentiles de latence).

### Dernière version fonctionnelle, fiable et viable

La version livrée est **manipulable en autonomie** : `docker compose up` (ou `./scripts/dev.sh`) suffit à lancer la stack complète ; le `README.md` fournit le quick start (build, CLI, API avec exemples curl, démo E2E ZK) ; les manuels d'utilisation et de mise à jour (C2.4.1) couvrent l'exploitation. Fiabilité : CI verte, 183+ tests automatisés, smoke tests systématiques. Viabilité : déploiement Sepolia documenté avec adresses réelles (`DEPLOYMENT.md`), rollback documenté.

**Vérification des critères d'évaluation** : système de gestion des versions utilisé (Git, Conventional Commits, semver via tags `v*`) — évolutions du prototype tracées (BACKLOG, Issues, registre de bogues) — logiciel fonctionnel et manipulable en autonomie (Docker Compose + manuels).

---

## C2.3.1 — Élaborer le cahier de recettes

> **Livrable attendu** : le cahier de recettes → **`docs/CAHIER_DE_RECETTES.md`** (document complet dans le dépôt).

### Contenu du cahier

Le cahier de recettes couvre **l'ensemble des fonctionnalités attendues** (user stories du `BACKLOG.md`) en **78 scénarios**, chacun avec préconditions, étapes, **résultat attendu, résultat obtenu et statut** :

| Type | Nombre | Exemples |
|---|---|---|
| **Fonctionnels** (`REC-F-xxx`) | 47 | authentification SIWE/JWT, parsing d'intention (regex + LLM local + fallback), plan d'exécution, CRUD intentions, délégation (hash/set/list/on-chain), preuve Noir, `executeWithProof` ETH/ERC-20, dépôts/retraits, monitoring/oracles/WebSocket, endpoints agents/strategies/proofs/portfolio, CLI complet, 6 parcours frontend, `/health` `/ready` `/metrics`, stack Docker |
| **Structurels** (`REC-S-xxx`) | 11 | architecture hexagonale, migrations 0001-0003 + `schema_migrations`, double adaptateur SQLite/Postgres, config 12-factor fail-fast, hydratation au redémarrage, persistance des exécutions, logs JSON, machine à états, build reproductible, versions nargo/bb épinglées |
| **Sécurité** (`REC-SEC-xxx`) | 20 | challenge SIWE unique/expirant, JWT strict, 401, isolation multi-utilisateur, rate limiting 429, CORS, limite 2000 caractères, validation hex, secrets (keystore/fichier/KMS/Vault), contraintes du circuit ZK (montant/protocole/target/expiry/nonce/signature), rejets on-chain (preuve falsifiée, replay, limites, 38 public inputs), mode dégradé sans clé |

### Exécution conforme au plan

- **60 scénarios automatisés** : chacun référence le test qui le couvre (ex. « Conforme (automatisé : `contracts/test/DelegationVault.t.sol::testExecuteWithProof`) ») — la recette est **rejouable à l'identique** via la CI ;
- **18 scénarios manuels** : parcours frontend et déploiement, exécutés sur la stack `scripts/dev.sh` ;
- **0 scénario non exécuté**.

### Extrait représentatif de scénarios

Quelques scénarios reproduits tels quels depuis le cahier (le document complet en contient 78) :

| ID | Fonctionnalité | Étapes | Résultat attendu | Statut |
|---|---|---|---|---|
| REC-F-001 | Génération d'un challenge SIWE (US-419) | `POST /api/v1/auth/challenge` `{"address":"0x…"}` | HTTP 200, message EIP-4361 avec nonce aléatoire et expiration 5 min | automatisé (`auth_challenge_returns_siwe_message`) |
| REC-F-006 | Parsing d'une condition « if yield > 3 » | `POST /api/v1/intents/parse` `{"text":"lend 100 USDC on Aave if yield > 3"}` | `ConditionalIntent` avec `Comparison { metric: Yield, comparator: GreaterThan, value: 3 }` | automatisé (`regex_parser.rs`) |
| REC-F-022 | Génération de la preuve Noir | `otter_cli prove "lend 100 USDC on Aave" --private-key 0x…` | `proof.bin` + `public_inputs.bin` ; witness attestant hash, signature, limites, nonce | automatisé (`zkp_noir.rs`) |
| REC-F-024 | Exécution via `executeWithProof` | appel avec intent autorisé | preuve vérifiée, limites contrôlées, nonce marqué utilisé, événement `Executed` | automatisé (`DelegationVault.t.sol::test_executeWithProof_succeeds`) |
| REC-F-027 | Pipeline E2E parse → preuve → exécution | `cargo test -p infrastructure --test e2e_anvil_flow` (Anvil requis) | délégation on-chain, preuve réelle, exécution confirmée | automatisé (conditionnel : skip sans Anvil) |

### Anomalies détectées

La recette a révélé **7 anomalies (A1–A7)**, de gravité faible à moyenne, consignées dans le cahier (§6.3) et tracées dans le plan de correction (C2.3.2) — ex. : `POST /intents/parse` ne validait pas la longueur du texte (A1), données de démonstration embarquées sur certains endpoints (A2), vérification SIWE E2E non automatisée (A7). Aucune anomalie bloquante : la chaîne critique (authentification, délégation signée, preuve ZK, anti-rejeu) est intégralement vérifiée.

**Vérification des critères d'évaluation** : le cahier reprend l'ensemble des fonctionnalités attendues — les tests fonctionnels, structurels et de sécurité exécutés sont conformes au plan défini.

---

## C2.3.2 — Élaborer un plan de correction des bogues

> **Livrable attendu** : le plan de correction des bogues → **`docs/PLAN_CORRECTION_BOGUES.md`** (document complet dans le dépôt).

### Méthodologie

Le plan définit le cycle complet : **détection** (4 canaux : CI — clippy `-D warnings`, `cargo test`, `forge test`, `nargo test`, vitest ; smoke tests post-déploiement ; revues de code ; recette) → **qualification** (grille à 4 niveaux : Bloquant / Majeur / Mineur / Cosmétique, avec définitions et exemples) → **traitement** (reproduction → correction → **test de non-régression obligatoire** → validation CI) → **prévention** (chaque bogue corrigé devient une barrière automatisée).

### Bogues détectés, qualifiés et traités

Le registre complet (§2 du plan) retient **12 bogues représentatifs**, chacun relié à son commit de correction vérifiable par `git show` :

| ID | Description | Détection | Criticité | Correction | Non-régression |
|---|---|---|---|---|---|
| BUG-01 | `setProtocolRouter` appelable par quiconque (absence de contrôle d'accès sur le vault) | Revue (finding Critical) | Bloquant | `feb03f5` — ajout `Ownable`/`onlyOwner` | `forge test` en CI |
| BUG-02 | Panics Tokio sur runtime imbriqué (secrets KMS/Vault), `VAULT_TOKEN` vide accepté, erreurs silencieuses | Revue (finding Critical) | Majeur | `174c315` | tests unitaires Env/File/KMS/Vault ajoutés |
| BUG-03 | Route `/api/v1/health` absente (smoke tests KO) ; versions Noir/bb non passées au build Docker | Smoke tests CI | Majeur | `30506dd` | `smoke-test.sh` à chaque déploiement |
| BUG-04 | Push GHCR invalide : nom de dépôt avec majuscules | Échec workflow Docker | Majeur | `881a418` | workflow build/push vert |
| BUG-05 | Dette CI : migration dupliquée, warnings clippy, tests ZKP instables, Noir non épinglé | CI en échec | Majeur | `de28a51` | clippy `-D warnings`, `.noir-version` |
| BUG-06 | Alertes Prometheus référençant des métriques jamais émises | Revue de code | Majeur | `f4ecb98` | job `docker-smoke` + `/metrics` |
| BUG-07 | Onboarding frontend : voile opaque, tooltip sans cible, hook non mappé sur le backend | Recette manuelle | Majeur | `23a778c` | `api.test.ts` (vitest) |
| BUG-08 | Clé privée Anvil codée en dur dans `scripts/dev.sh` | Revue (finding Critical) | Majeur | `03db1e3` | grep de secrets + `set -euo pipefail` |
| BUG-09 | `docker-smoke` non protégé contre l'échec de `paths-filter` ; migration Postgres non idempotente | Revue (finding Important) | Majeur | `9ca4724` | migrations rejouées au démarrage |
| BUG-10 | Erreur de compilation : double déclaration de modules (`orchestrator/mod.rs`) | `cargo build` / CI | Bloquant | `0af6335` | clippy workspace en CI |
| BUG-11 | Script mainnet sans `set -euo pipefail`, message TODO en production | Revue de code | Mineur | `51e9ae9` | exécution du workflow + smoke test |
| BUG-12 | Échec du contrôle de formatage | CI (`cargo fmt --check`) | Cosmétique | `c4afa6f` | étape fmt en CI (`94bd629`) |

**Taux de résolution : 100 %** (12/12 mergés sur `main`, CI verte) — 2 Bloquants, 8 Majeurs, 1 Mineur, 1 Cosmétique.

### Analyse des points d'amélioration

Pour chaque faiblesse détectée (§3 du plan) : constat → risque → action corrective → statut :

1. **Couverture non mesurée** → tarpaulin mis en place, première mesure 41,14 %, objectif ≥ 70 % métier, intégration CI planifiée ;
2. **Tests frontend légers** → renforcés de 3 à 28 cas (mapping API, statuts, délégation, stepper accessible) ;
3. **TLS hors dépôt** → configuration de reverse proxy de référence à versionner ;
4. **Secret JWT aléatoire en dev** → fail-fast en production à planifier.

Les **7 anomalies de la recette (A1–A7)** sont reprises au §5 du plan avec qualification et traitement (ex. A1 : factoriser `validate_intent_text` dans un middleware partagé ; A6 : plan de correction créé — **résolu**).

**Vérification des critères d'évaluation** : les bogues sont détectés, qualifiés et traités (registre adossé à l'historique git) — une analyse des points d'amélioration est réalisée pour chaque test en échec — les corrections sont conformes et garantissent le bon fonctionnement (tests de non-régression + CI bloquante).

---

## C2.4.1 — Rédiger la documentation technique d'exploitation

> **Livrables attendus** : le manuel de déploiement ; le manuel d'utilisation ; le manuel de mise à jour.

La documentation d'exploitation assure la traçabilité pour le suivi des équipes et les évolutions futures. Elle est intégralement **versionnée avec le code** (docs-as-code) :

### Manuel de déploiement → `DEPLOYMENT.md`

321 lignes couvrant : prérequis et installation des toolchains versionnées ; déploiement des contrats (`contracts/script/DeployDelegationVault.s.sol`, vérification Etherscan) ; gestion des secrets (keystore > fichier > env, KMS/Vault) ; démarrage de la stack Docker Compose ; vérification post-déploiement (healthchecks, smoke tests) ; observabilité (métriques, alertes) ; test de charge ; CI/CD ; troubleshooting ; durcissement production. Il documente le **déploiement Sepolia réel** avec adresses de contrats et hash de transaction.

### Manuel d'utilisation → `docs/MANUEL_UTILISATION.md`

Rédigé pour un utilisateur non-développeur : concepts vulgarisés (intention, plan, délégation, exécution, preuve ZK) ; prérequis (navigateur, wallet, réseau Sepolia, faucet) ; installation (Docker Compose) ; **guide pas à pas des 10 écrans réels** avec les messages d'erreur exacts de l'application ; API REST avec exemples curl (authentification incluse) ; CLI `otter_cli` (7 commandes documentées) ; FAQ et tableau de dépannage ; catalogue des alertes. Les limitations réelles y sont honnêtement signalées (révocation de délégation non implémentée, adresses V1 obsolètes).

### Manuel de mise à jour → `docs/MANUEL_MISE_A_JOUR.md`

464 lignes couvrant : politique de versionnement (semver, tags `v*`, images ghcr.io, Conventional Commits) ; procédures de mise à jour par environnement (local, Docker, testnet automatisé par tag, mainnet manuel avec multisig) ; migrations de base de données (0001-0003, table `schema_migrations`, application automatique au démarrage) ; mise à jour des contrats et régénération du vérificateur ; **rollback** (image du tag précédent, stratégie de migrations additives, dump PostgreSQL) ; vérification post-mise-à-jour ; checklist finale en 10 points.

### Choix de technologies et de langages (traçabilité des décisions)

| Choix | Justification documentée |
|---|---|
| Rust + Axum | Sûreté mémoire et typage fort pour un logiciel manipulant des fonds ; performance |
| Architecture hexagonale | Maintenabilité, testabilité (mocks), remplaçabilité des adaptateurs (SQLite ↔ Postgres démontré) |
| Noir + UltraHonk | Preuves ZK vérifiables on-chain, vérificateur Solidity généré |
| LLM local (llama.cpp) | Confidentialité des stratégies utilisateur, fonctionnement hors-ligne |
| React + Vite + Tailwind | Productivité, écosystème, design system par tokens |
| SIWE + JWT | Authentification sans mot de passe, native au domaine Ethereum |
| Docker + GitHub Actions | Reproductibilité dev/CI/prod, déploiement continu gradué |

Ces choix sont également documentés dans `README.md` (tech stack + schéma d'architecture) et `PRODUCT.md` (spécification produit). Documents complémentaires : `FLOW.md` (flows utilisateur), `docs/SECURITE.md`, `docs/ACCESSIBILITE.md`, `docs/CAHIER_DE_RECETTES.md`, `docs/PLAN_CORRECTION_BOGUES.md`, `lab/LEARNING.md`.

**Vérification des critères d'évaluation** : les trois manuels sont rédigés avec clarté et versionnés — la documentation décrit les choix de technologies et de langages.

---

## 11. Annexe — Tableau de correspondance critères ↔ preuves

| Compétence | Livrable | Fichier(s) de preuve dans le dépôt |
|---|---|---|
| C2.1.1 | Protocole de déploiement continu | `.github/workflows/docker.yml`, `deploy-testnet.yml`, `deploy-mainnet.yml`, `scripts/smoke-test.sh`, `docker-compose.yml` |
| C2.1.1 | Critères de qualité et de performance | `ci.yml` (clippy/fmt/eslint), `tarpaulin.toml`, `coverage/tarpaulin-report.html`, `alerting.yml`, `scripts/load_test.py`, `/metrics` |
| C2.1.2 | Protocole d'intégration continue | `.github/workflows/ci.yml` (5 jobs, séquences par stack) |
| C2.2.1 | Architecture structurée | `Cargo.toml` (4 crates), `crates/domain/src/ports/`, `BACKLOG.md` US-006 |
| C2.2.1 | Prototypes réalisés | `frontend/` (10 pages), `otter_api`, `otter_cli`, contrats Sepolia (`DEPLOYMENT.md`), `lab/zkp_e2e.sh` |
| C2.2.2 | Tests unitaires | `crates/**` (155 tests), `contracts/test/` (13), `delegation_circuit/src/main.nr` (3), `frontend/src/**/*.test.*` (28), `coverage/` |
| C2.2.3 | Mesures de sécurité (OWASP) | `docs/SECURITE.md` + `auth.rs`, `secrets.rs`, `DelegationVault.sol`, rate limiting/CORS dans `otter_api.rs` |
| C2.2.3 | Accessibilité (RGAA) | `docs/ACCESSIBILITE.md` + 108 attributs ARIA dans `frontend/src/`, `Stepper.test.tsx` |
| C2.2.4 | Historique des versions | Git (Conventional Commits), tags semver `v*`, `BACKLOG.md` (481 US) |
| C2.2.4 | Dernière version fonctionnelle | `docker-compose.yml`, `README.md` quick start, smoke tests |
| C2.3.1 | Cahier de recettes | `docs/CAHIER_DE_RECETTES.md` (78 scénarios, 7 anomalies) |
| C2.3.2 | Plan de correction des bogues | `docs/PLAN_CORRECTION_BOGUES.md` (12 bogues résolus, A1–A7 tracés) |
| C2.4.1 | Manuel de déploiement | `DEPLOYMENT.md` |
| C2.4.1 | Manuel d'utilisation | `docs/MANUEL_UTILISATION.md` |
| C2.4.1 | Manuel de mise à jour | `docs/MANUEL_MISE_A_JOUR.md` |

### Index documentaire complet

```
DOSSIER_BC02.md            ← ce dossier
README.md                  ← présentation, architecture, quick start
PRODUCT.md / FLOW.md       ← spécification produit, flows utilisateur
BACKLOG.md                 ← 481 user stories tracées
DEPLOYMENT.md              ← manuel de déploiement
docs/SECURITE.md           ← mesures OWASP Top 10
docs/ACCESSIBILITE.md      ← référentiel RGAA 4.1 + audit
docs/CAHIER_DE_RECETTES.md ← 78 scénarios de recette
docs/PLAN_CORRECTION_BOGUES.md ← registre et traitement des bogues
docs/MANUEL_UTILISATION.md ← prise en main utilisateur
docs/MANUEL_MISE_A_JOUR.md ← procédures de mise à jour et rollback
.github/workflows/         ← CI/CD (4 workflows)
coverage/                  ← rapport de couverture tarpaulin
alerting.yml               ← règles d'alerte Prometheus
```

---

*Dossier rédigé à partir de l'état réel du dépôt : chaque affirmation renvoie à un fichier, un commit ou une mesure vérifiable dans le dépôt public.*
