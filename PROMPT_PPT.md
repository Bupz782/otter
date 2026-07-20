# Prompt — Génération du PowerPoint de Présentation Projet

## Métadonnées
- **Projet** : Otter (anciennement Otter) — Trustless DeFi Automation Protocol
- **Objectif** : Réaliser un PowerPoint professionnel répondant aux critères d'évaluation C1.1.1 à C1.6
- **Public visé** : Jury d'évaluation professionnelle (formateur, examinateur, client)
- **Ton** : Professionnel, technique mais vulgarisé, argumenté, visuel
- **Format attendu** : Présentation structurée en diapositives, avec schémas, tableaux, graphiques, et légendes explicites.
- **Longueur cible** : ~20-25 slides max

---

## Contexte Projet (à intégrer dans l'intro)

Otter est un protocole d'automatisation DeFi **trustless**. L'utilisateur décrit sa stratégie en langage naturel (ex: *"Lend 1000 USDC on Aave if yield > 3%"*), délègue l'exécution à un agent avec des limites cryptographiques signées, et l'agent exécute sur Ethereum/Arbitrum/Solana. Chaque action requiert une preuve ZKP (Zero-Knowledge Proof) prouvant qu'elle respecte la délégation. L'agent capture du MEV (Maximum Extractable Value) et le redistribue à l'utilisateur. Le vault prouve périodiquement sa solvabilité sans révéler les balances individuelles.

**Stack technique réel** :
- Frontend : React + Vite + Tailwind CSS + RainbowKit + wagmi/viem
- Backend API : Rust (Axum) + tokio + llama.cpp (GGUF local)
- ZKP : Noir circuits + Barretenberg verifier
- Smart Contracts : Solidity + Foundry
- Blockchain : Ethereum mainnet + Arbitrum + Solana

**Architecture** : Hexagonale (domain / application / infrastructure / interfaces)
**Backlog** : 481 user stories, ~18% complété, MVP cible ~384 stories

---

## Structure du PowerPoint par Critère

### Slide 1 — Page de Garde
- Titre : **Otter 🦦 — Protocole d'Automatisation DeFi Trustless**
- Sous-titre : Projet de développement d'application logicielle — Présentation C1.1 à C1.6
- Auteur, date, contexte professionnel
- Visuel : **Logo loutre (🦦)** en grand format, style flat design moderne, couleurs douces bleu océan + sable blanc

---

### Partie C1.1.1 — Cartographie des Parties Prenantes
**Titre slide** : Cartographie des Acteurs et Parties Prenantes

**Contenu attendu** :
1. **Tableau de cartographie** avec 4 colonnes : Acteur | Rôle | Niveau d'implication | Périmètre d'influence
   - **Développeurs** (Rust/Solidity/Noir/TypeScript) : Développement backend, smart contracts, circuits ZK, frontend. Implication élevée, opérationnelle.
   - **Architecte logiciel** : Conception architecture hexagonale, choix tech, intégration ZK/blockchain. Implication élevée, stratégique.
   - **Administrateurs système / Ops** : Déploiement infrastructure (nodes RPC, Docker, K8s), monitoring (Prometheus/Grafana), logs structurés. Implication moyenne, opérationnelle.
   - **Clients / Utilisateurs finaux DeFi** : Déposent des fonds, créent des intents, signent des délégations. Implication très élevée, utilisateurs finaux.
   - **Agents exécutants** : Opèrent le backend, génèrent les preuves ZK, capturent MEV, stakent un bond économique. Implication élevée, opérationnelle.
   - **Acteurs externes** :
     - Protocoles DeFi (Aave, Compound, Uniswap, Curve) — partenaires d'intégration
     - Fournisseurs RPC (Infura, Alchemy, Ankr) — infrastructure blockchain
     - Flashbots / MEV-Blocker — infrastructure MEV
     - Auditeurs sécurité (smart contracts + circuits ZK) — validation externe
     - Régulateurs (MiCA EU) — conformité

2. **Profils des futurs utilisateurs** (slide annexe ou sous-section) :
   - **Utilisateur DeFi non-technique** : veut automatiser sans coder, utilise le langage naturel. Besoin : simplicité, sécurité, transparence.
   - **Trader avancé** : veut des stratégies complexes, conditions personnalisées. Besoin : flexibilité, preuves vérifiables, rebates MEV.
   - **Opérateur Agent** : fait tourner le logiciel agent, capture MEV. Besoin : documentation technique, rentabilité, réputation.
   - **Créateur de stratégie** : publie des stratégies gagnantes, gagne des frais. Besoin : viralité, leaderboard, analytics.

---

### Partie C1.1.2 — Analyse de la Demande et des Objectifs
**Titre slide** : Analyse du Besoin, Objectifs et Enjeux

**Contenu attendu** :
1. **Entretien d'explicitation du besoin** (simulé, basé sur l'étude du marché) :
   - **Besoin exprimé** : Les utilisateurs DeFi veulent automatiser des stratégies sans donner la garde de leurs fonds à un tiers.
   - **Attentes** : Langage naturel (pas de code), délégation persistante avec limites, preuve de chaque action, capture de valeur MEV, transparence solvabilité.
   - **Exigences** : Preuve ZKP < 3s, exécution multi-chain (ETH + Arbitrum + Solana), local LLM pour privacy, trust model minimal.

2. **État des lieux de l'existant** :
   - **Gelato / Keep3r** : Automation mais nécessite confiance dans les smart contracts de l'opérateur.
   - **Aperture** : Langage naturel mais one-shot, pas de délégation persistante.
   - **UniswapX** : Signed intents mais pas d'automation ni de délégation avec contraintes.
   - **Aucune solution** ne combine NL + ZKP persistant + MEV + proof-of-solvency + social sharing.

3. **Objectifs du projet par partie prenante** (tableau) :
   | Partie prenante | Objectif | Enjeu |
   |----------------|----------|-------|
   | Utilisateur | Automatiser sans confiance | Sécurité des fonds, simplicité |
   | Agent | Générer du revenu MEV | Rentabilité, réputation, slashing |
   | Protocole Otter | Prouver un modèle économique viable | Adoption, frais protocol, audits |
   | Développeur | Démontrer une stack ZK-Rust moderne | Compétences, opensource, grants |

4. **Problématique client** :
   > *Comment permettre à un utilisateur DeFi d'automatiser ses stratégies de manière entièrement trustless, avec une délégation cryptographique persistante et vérifiable, tout en capturant et redistribuant la valeur MEV générée par son exécution ?*

5. **Pistes de solutions émergentes** :
   - Vault smart contract avec vérification ZKP on-chain
   - Circuit Noir pour contraintes de délégation (amount, protocol, expiry, nonce)
   - LLM local pour parser les intents sans fuite de données
   - Intégration Flashbots Protect pour MEV capture
   - Preuve périodique de solvabilité par ZKP

---

### Partie C1.2.1 — Cartographie Opportunités et Menaces (SWOT)
**Titre slide** : Analyse SWOT et Impact Environnemental

**Contenu attendu** :
1. **Matrice SWOT** (tableau unique 2×2 ou liste condensée en un bloc visuel) :

   | FORCES (internes positives) | FAIBLESSES (internes négatives) |
   |-----------------------------|---------------------------------|
   | • Combinaison unique sur le marché (NL + ZKP + MEV + Solvency + Social) | • Complexité ZK (courbe d'apprentissage, audits coûteux) |
   | • Stack technique moderne et robuste (Rust, Noir, Foundry, Solana) | • Dépendance infrastructure blockchain (RPC, gas, outages) |
   | • Architecture hexagonale maintenable et testable | • Projet jeune, pas encore en production (charge 220-354 jh restante) |
   | • LLM local = privacy by design, pas de fuite de données financières | • Besoin de liquidité initiale pour démontrer la capture MEV |
   | • Multichain natif (ETH + Arbitrum + Solana) | • Double stack EVM + Solana = complexité d'interopérabilité |

   | OPPORTUNITÉS (externes positives) | MENACES (externes négatives) |
   |-----------------------------------|------------------------------|
   | • Marché DeFi en croissance, besoin d'automatisation trustless | • Risque de hack smart contract / bug circuit ZK (perte de fonds) |
   | • Réglementation MiCA (EU) pousse à la preuve de solvabilité | • Volatilité crypto et bear market (adoption ralentie) |
   | • L2 scaling (Arbitrum) + Solana réduisent les coûts gas | • Concurrence établie (Gelato, Keep3r avec plus de ressources) |
   | • Tendance SocialFi / copy-trading | • Risque réglementaire (statut juridique des agents, MEV, MiCA) |
   | • Rust natif sur Solana = synergie stack backend | • Dépendance fournisseurs RPC / single point of failure |

2. **Impact environnemental** :
   - **Positif** : Preuves ZKP réduisent le gas on-chain vs vérifications classiques (moins de calcul on-chain)
   - **Positif** : Utilisation d'Arbitrum (L2) consomme ~97% moins d'énergie que Ethereum L1 par transaction
   - **Positif** : LLM local (GGUF Q4_K_M) évite les appels API cloud (réduction empreinte serveur distant)
   - **Négatif** : Génération de preuves ZK consomme du CPU local (optimisation circuit cible < 1000 contraintes)
   - **Préconisation sécurité** : Audit circuit + smart contract obligatoire avant mainnet, bug bounty, slashing agent
   - **Points de vigilance** : dépendance RPC (single point of failure), compromission clé privée agent, reentrancy attacks

---

### Partie C1.2.2 — Faisabilité Technique (Audit & Diagnostic)
**Titre slide** : Audit Technique et Diagnostic des Infrastructures

**Contenu attendu** :
1. **Démarche d'audit** :
   - Analyse de l'existant (solutions concurrentes, technologie actuelle)
   - Évaluation de la stack technique disponible (Noir v0.19+, Foundry v0.2+, Axum stable)
   - Proof-of-concept : circuit "hello world" compilé, contrat Counter.sol déployable, parsing d'intent fonctionnel
   - Benchmarks : temps de parsing LLM local, estimation contraintes ZK

2. **État des infrastructures existantes** (tableau) :
   | Couche | Technologie | État | Caractéristiques |
   |--------|-------------|------|-----------------|
   | Langages | Rust, Solidity, Noir, TypeScript | Disponibles, documentés | Rust nightly, Foundry, nargo |
   | Base de données | PostgreSQL (planifiée), in-memory (actuel) | Schema en cours | Relations : intents, delegations, transactions |
   | Architecture | Hexagonale (Clean Architecture) | Implémentée à 80% | 4 crates : domain, application, infrastructure, interfaces |
   | Smart Contracts | Solidity + Foundry | Squelette (Counter.sol) | Besoin StrategyVault.sol + DelegationVerifier.sol |
   | ZKP | Noir + Barretenberg | Circuit hello-world OK | Besoin circuit EdDSA + delegation constraints |
   | LLM | llama.cpp (GGUF Q4_K_M) | Intégré, parsing fonctionnel | Modèles locaux LFM2.5-1.2B et Qwen3-8B |
   | CI/CD | GitHub Actions | Opérationnel | cargo test, clippy, fmt --check |

3. **Contraintes identifiées** :
   - **Techniques** : Temps de preuve ZK cible < 3s, compatibilité EVM (Noir→Solidity verifier), gestion nonces anti-replay
   - **Financières** : Coût gas testnet/mainnet, frais audit sécurité (est. 30-50k€), infrastructure serveur
   - **Humaines** : Compétences rares (Rust + ZK + Solidity), 1-2 développeurs fullstack
   - **Délais** : 481 user stories, ~18% complété, MVP estimé à 12-18 mois
   - **Données** : Volume on-chain (historique transactions), nombre d'utilisateurs (scalabilité L2)
   - **Hébergement** : Serveur agent dédié (preuve ZK CPU-intensive), nodes RPC payants

4. **Avis critique sur la faisabilité** :
   - **Faisable** : Tous les composants individuels existent et sont prouvés (Noir, Foundry, Axum, Flashbots)
   - **Risqué** : L'intégration ZKP + Solidity + Rust est complexe et peu documentée
   - **Recommandation** : Livraison par phases — MVP testnet (intents simples) → audit → mainnet restreint

---

### Partie C1.2.3 — Cartographie des Risques Techniques et Fonctionnels
**Titre slide** : Référentiel des Risques et Indicateurs de Contrôle

**Contenu attendu** :
1. **Matrice des risques** (tableau avec criticité : Impact x Probabilité = Rouge/Orange/Vert) :

   | ID | Risque | Type | Impact | Probabilité | Criticité | Mitigation |
   |----|--------|------|--------|-------------|-----------|------------|
   | R1 | Bug circuit ZK (missing constraint) | Technique | Critique (perte fonds) | Moyenne | 🔴 Rouge | Audit externe, tests 100+ cas edge, formal verification |
   | R2 | Bug smart contract (reentrancy, overflow) | Technique | Critique | Moyenne | 🔴 Rouge | Slither, Mythril, audit, tests Foundry |
   | R3 | Perte de données (DB corruption) | Technique | Élevé | Faible | 🟠 Orange | Backup automatique, immutabilité on-chain |
   | R4 | Interruption système (RPC down, agent offline) | Technique | Moyen | Élevée | 🟠 Orange | Multi-RPC failover, health checks, alerting |
   | R5 | Parsing LLM incorrect (mauvais montant) | Fonctionnel | Élevé | Moyenne | 🟠 Orange | Fallback regex, validation stricte, confirmation UI |
   | R6 | MEV négatif (gas > value capturée) | Fonctionnel | Moyen | Moyenne | 🟡 Jaune | Estimation pré-exécution, seuil minimum MEV |
   | R7 | Slashing agent injuste | Fonctionnel | Moyen | Faible | 🟡 Jaune | Période de contestation, multi-signature |
   | R8 | Dégradation performance (preuve > 3s) | Technique | Moyen | Moyenne | 🟡 Jaune | Optimisation circuit, cache proving keys, benchmarking |
   | R9 | Compromission clé privée agent | Sécurité | Critique | Faible | 🔴 Rouge | Keystore chiffré (scrypt), HSM optionnel, rotation |
   | R10 | Attaque front-running (mempool public) | Sécurité | Moyen | Élevée | 🟠 Orange | Flashbots Protect, MEV-Blocker, soumission privée |

2. **Référentiel de suivi des incidents** :
   - Fichier/incident : timestamp, gravité, description, impact, action corrective, responsable, statut (ouvert/résolu)
   - Revue hebdomadaire des incidents dans le backlog (ISSUES.md)

3. **Indicateurs de contrôle (KPIs)** :
   - **Performance** : Temps moyen de génération de preuve (cible < 3s)
   - **Sécurité** : Taux de preuves invalides / transactions revert (cible < 0.1%)
   - **Disponibilité** : Uptime agent (cible > 99%)
   - **Économique** : MEV capturé vs gas dépensé par action (cible : MEV > gas)
   - **Qualité** : Couverture de tests (cible > 80%), nombre de bugs critiques ouverts
   - **Environnemental** : Gas moyen par transaction (optimisation via L2), empreinte serveur (LLM local vs cloud)

---

### Partie C1.3.1 — Veille Technique, Technologique et Réglementaire
**Titre slide** : Veille Technologique et Réglementaire

**Contenu attendu** :
1. **Stratégie de veille et objectifs** :
   - Objectif : Anticiper les évolutions ZK, DeFi, Rust, réglementaires pour maintenir la compétitivité et la conformité du projet.
   - Périmètre : Zero-Knowledge proofs, L2 scaling, smart contract security, LLM open source, réglementation crypto EU.

2. **Outils de veille utilisés** :
   - **Automatisation** : Google Alerts ("Noir language", "ZK DeFi", "MEV rebates"), RSS (ZK research blogs)
   - **Communautés** : Discord officiel (Noir, Foundry, Rust), Twitter/X (ZK researchers, DeFi protocols)
   - **Salons & Events** : ETHGlobal hackathons, Devcon, ZK Summit, RustConf
   - **Réseaux professionnels** : Ethereum Magicians, ZK Hack (challenges), GitHub trending (Noir repos)
   - **Newsletters** : Week in Ethereum, ZK Hack newsletter, Paradigm blog

3. **Sources d'information principales** :
   | Domaine | Sources |
   |---------|---------|
   | ZK / Cryptographie | noir-lang.org, aztec.network, ZK Hack, academic papers (ePrint) |
   | Smart Contracts | Foundry book, OpenZeppelin, Solidity blog, audit reports (Trail of Bits, OpenZeppelin) |
   | DeFi / Protocoles | Aave docs, Compound docs, Uniswap v3 whitepaper, Flashbots docs |
   | Rust / Backend | Rust blog, Axum docs, Tokio ecosystem, crates.io |
   | Réglementation | MiCA EU texts, ESMA guidelines, FATF crypto recommendations |
   | LLM / AI | HuggingFace, llama.cpp releases, GGUF quantization papers |

4. **Évolutions identifiées et justifications** :
   | Évolution | Type | Impact Métier | Impact Environnemental | Action |
   |-----------|------|---------------|------------------------|--------|
   | EIP-4337 (Account Abstraction) | Technique | Simplifie signature délégation, meilleure UX | Neutre | Évaluer intégration v2 |
   | Arbitrum Stylus (Rust smart contracts) | Technique | Possible réécriture vault en Rust/WASM | Potentiellement positif (efficacité) | Veille active |
   | MiCA (réglementation EU 2024) | Réglementaire | Obligation de preuve de solvabilité = avantage compétitif | Neutre | Anticiper conformité |
   | ZK-SNARKs proving plus rapides (Barretenberg upgrades) | Technique | Réduction temps preuve < 1s possible | Positif (moins de CPU/serveur) | Benchmarker nouvelles versions |
   | Modèles LLM quantifiés plus légers (Q4_K_M → Q2_K) | Technique | Réduction empreinte mémoire, edge deployment | Positif (moins de consommation) | Tester modèles plus petits |
   | Preuves de solvabilité mainstream (post-FTX) | Marché | Demande croissante de transparence | Neutre | Positionner le produit |

---

### Partie C1.3.2 — Sélection de l'Architecture Technique (Étude Comparative)
**Titre slide** : Étude Comparative des Solutions Techniques

**Contenu attendu** :
1. **Analyse comparative par couche** (tableaux avec avantages/inconvénients) :

   **Backend** :
   | Solution | Avantages | Inconvénients | Verdict |
   |----------|-----------|---------------|---------|
   | **Rust (Axum)** | Sécurité mémoire, performance ZK, async natif (tokio), typage fort | Courbe d'apprentissage, écosystème plus jeune que Node | Retenu |
   | Node.js/Express | Écosystème mature, rapide à développer | Moins performant, typage faible (JS), pas adapté ZK | Rejeté |
   | Go/Gin | Performant, simple | Moins riche pour crypto/ZK, gestion mémoire moins sûre | Rejeté |

   **Zero-Knowledge Proofs** :
   | Solution | Avantages | Inconvénients | Verdict |
   |----------|-----------|---------------|---------|
   | **Noir** | Syntaxe proche Rust, verifier Solidity exportable facilement, tooling Aztec mature | Moins optimisé que Circom, écosystème plus jeune | Retenu |
   | Circom | Très optimisé, grand écosystème | Syntaxe complexe, courbe d'apprentissage abrupte, audit plus difficile | Rejeté |
   | Cairo (Starknet) | Puissant, scaling natif | Non-EVM, refonte complète des smart contracts, écosystème séparé | Rejeté |

   **Smart Contract Framework** :
   | Solution | Avantages | Inconvénients | Verdict |
   |----------|-----------|---------------|---------|
   | **Foundry** | Tests en Solidity natif, très rapide, forge script pour deploy, fuzzing natif | Moins de plugins que Hardhat | Retenu |
   | Hardhat | Écosystème riche, TypeScript tasks | Plus lent, moins intégré au testing natif Solidity | Rejeté |

   **LLM / Intent Parsing** :
   | Solution | Avantages | Inconvénients | Verdict |
   |----------|-----------|---------------|---------|
   | **Local llama.cpp (GGUF)** | Privacy totale, pas de fuite données, pas de coût API, offline possible | Qualité inférieure aux grands modèles API, requiert RAM/GPU locale | Retenu |
   | API Claude/OpenAI | Excellente qualité parsing, simple | Coût récurrent, fuite données financières sensibles, dépendance fournisseur | Rejeté |

   **Frontend** :
   | Solution | Avantages | Inconvénients | Verdict |
   |----------|-----------|---------------|---------|
   | **React + Vite** | Simplicité, pas de SSR nécessaire, hot reload rapide, tailwind intégré | Pas de SEO natif (pas critique pour dApp) | Retenu |
   | Next.js | SEO, SSR, routing intégré | Complexité inutile pour dApp, hydration | Rejeté |

   **Blockchain** :
   | Solution | Avantages | Inconvénients | Verdict |
   |----------|-----------|---------------|---------|
   | **Ethereum + Arbitrum** | Écosystème DeFi mature, L2 réduit gas/consommation, EVM compatible | Congestion L1, coûts L1 élevés | Retenu (principal) |
   | **Solana** | Très rapide, faibles coûts, écosystème Rust natif (cohérence stack), haut débit | Écosystème DeFi différent (programs Anchor), historique d'outages, courbe d'apprentissage | Retenu (secondaire) |

2. **Synthèse des choix retenus et justifications** :
   - **Rust** : choisi pour la sécurité mémoire critique dans un projet financier, et la performance nécessaire à la génération de preuves.
   - **Noir** : choisi pour l'intégration transparente avec Solidity (export verifier), réduisant la complexité d'interopérabilité.
   - **LLM local** : choisi pour la confidentialité des données financières (pas de données envoyées à des tiers).
   - **Ethereum + Arbitrum** : choisi pour l'écosystème DeFi mature et la sécurité L1, avec L2 pour réduire les coûts.
   - **Solana** : choisi pour sa complémentarité (haute vitesse, faibles coûts, Rust natif qui correspond à notre stack backend), permettant des intents rapides et peu coûteux.

3. **Analyse sécurité, réseau, accessibilité, impact environnemental** :
   - **Sécurité** : Rust élimine les vulnérabilités mémoire (buffer overflow, use-after-free). Noir permet des preuves vérifiables on-chain.
   - **Réseau** : Multi-RPC avec failover. WebSocket pour temps réel.
   - **Accessibilité** : Langage naturel abaisse la barrière d'entrée. RainbowKit supporte 100+ wallets.
   - **Impact environnemental** : L2 Arbitrum = ~97% moins d'énergie que L1. LLM local Q4_K_M = modèle 1.2B-8B quantifié, consommation CPU raisonnable. Preuve ZK = réduction du gas on-chain vs logique classique.

---

### Partie C1.4.1 — Évaluation de la Charge de Travail
**Titre slide** : Analyse Fonctionnelle et Charge de Travail

**Contenu attendu** :
1. **Diagramme de fonctionnalités** (ou tableau hiérarchisé) :
   - **Fonctions Principales** (cœur de métier, indispensables) :
     - FP1 : Parser un intent en langage naturel → structuration JSON (LLM + regex fallback)
     - FP2 : Générer une preuve ZKP de respect de délégation (Noir circuit)
     - FP3 : Vérifier la preuve on-chain et exécuter l'action (StrategyVault Solidity)
     - FP4 : Monitorer les conditions et déclencher l'exécution (Orchestrator Rust)
     - FP5 : Capturer et redistribuer le MEV (Flashbots integration, Jito sur Solana)
   - **Fonctions Secondaires** (importantes, mais pas bloquantes pour le MVP) :
     - FS1 : Proof-of-solvency périodique (circuit ZK dédié)
     - FS2 : Agent marketplace et réputation (staking, bonding, slashing)
     - FS3 : Social sharing et copy-trading (Strategy Registry)
     - FS4 : Multi-chain support (Arbitrum adapter, Solana adapter)
   - **Fonctions Complémentaires** (valeur ajoutée, v2 ou future) :
     - FC1 : Mode simulation (paper trading)
     - FC2 : Export fiscal et rapports compliance
     - FC3 : Gamification (badges, levels)
     - FC4 : Mobile responsive / React Native

2. **Estimation de charge** (exprimée en jours-homme) :
   - Basée sur le backlog de 481 user stories, méthode agile par vagues.
   - MVP cible : ~396 stories (hors features cut/future)
   - Estimation moyenne : 1 story = 0.5 à 1.5 jours-homme selon complexité
   - Charge totale estimée MVP : **~220-354 jours-homme**
   - Répartition par vague (graphique en barres ou camembert) :
     - Vague 0 (Setup) : ~12 jh
     - Vague 1 (Intent Parsing) : ~35 jh
     - Vague 2 (ZKP Delegation) : ~45 jh
     - Vague 5 (Blockchain + MEV) : ~50 jh
     - Vague 6 (Orchestrator) : ~55 jh
     - Vague 6.5 (Web UI) : ~45 jh
     - Vague 7 (Production) : ~38 jh

3. **Outil d'analyse fonctionnelle** :
   - Méthode : User Stories (format "En tant que..., je veux..., afin de...")
   - Organisation : Backlog structuré par vagues (waves) et epics (BACKLOG.md)
   - Suivi : Fichier markdown versionné avec statuts ([FAIT] / [EN COURS] / [EN ATTENTE])

4. **Couverture technique des besoins fonctionnels** :
   - Toute fonction principale est couverte par une technologie validée et un POC réalisé.
   - Le parsing NL → LLM local (POC validé). La preuve ZK → circuit hello-world compilé. Le vault → squelette Foundry. L'orchestration → state machine définie.
   - Les fonctions secondaires ont des preuves de concept ou des solutions identifiées.

5. **Expérience utilisateur (UX)** :
   - Feedback live lors du parsing (affichage structuré en temps réel)
   - Wizard de délégation pas à pas avec récapitulatif avant signature
   - Dashboard avec portfolio, intents actifs, historique, graphiques
   - Status temps réel via WebSocket (stepper visuel IDLE → MONITORING → ... → CONFIRMED)
   - Notifications push et toast pour événements clés

---

### Partie C1.4.2 — Estimation des Coûts et Budget Prévisionnel
**Titre slide** : Budget Prévisionnel du Projet

**Contenu attendu** :
1. **Budget prévisionnel** (tableau avec postes et montants estimés) :
   | Poste de coût | Description | Estimation (€) | % du budget |
   |--------------|-------------|----------------|-------------|
   | **Développement** | Charge 220-354 jh × 500€/jh (freelance senior blockchain) | 110 000 - 177 000 | ~55% |
   | **Audit Sécurité** | Smart contracts (2 audits) + Circuit ZK (1 audit) | 40 000 - 60 000 | ~20% |
   | **Infrastructures** | Serveurs dédiés (agent + preuve ZK), nodes RPC (Infura/Alchemy), PostgreSQL hébergé | 6 000 - 12 000/an | ~8% |
   | **Licences / Outils** | Aucune licence propriétaire (open source). Outils dev gratuits (GitHub, Foundry, Noir) | 0 | 0% |
   | **Testnet / Mainnet Gas** | Déploiements, tests, exécutions (ETH + Arbitrum + Solana) | 6 000 - 12 000 | ~5% |
   | **Bug Bounty** | Programme post-launch (Immunefi ou similaire) | 10 000 - 20 000 | ~6% |
   | **Marketing / Community** | Site web, documentation, Discord, réseaux | 5 000 - 10 000 | ~4% |
   | **Juridique / Compliance** | Conseil MiCA, terms of service | 5 000 - 8 000 | ~3% |
   | **Réserve imprévu** | ~10% du total | 20 000 - 30 000 | ~7% |
   | **TOTAL** | | **~211 000 - 342 000 €** | 100% |

2. **Cohérence avec la charge de travail** :
   - Le poste développement est calibré sur 450-600 jours-homme, cohérent avec l'analyse fonctionnelle.
   - Les postes infrastructure et gas sont alignés sur la volumétrie cible (100+ transactions mainnet, 1000+ preuves testnet).

3. **Visualisation** : Graphique en camembert ou en barres montrant la répartition des coûts.

---

### Partie C1.5 — Modélisation de l'Architecture Logicielle
**Titre slide** : Architecture Logicielle Proposée

**Contenu attendu** :

#### Slide 1 — Vue d'ensemble 3 couches (C4 Model Niveau 2 : Container Diagram)

**Légende obligatoire** :
- Rectangle arrondi = Service / Module / Container
- Flèche pleine = Flux synchrone (HTTP/gRPC/RPC)
- Flèche pointillée = Flux asynchrone (WebSocket / Event Bus / tokio channels)
- Cylindre = Base de données / Stockage persistant
- Losange = Décision / Vérification on-chain
- Hexagone = Port (interface) — formalisme Clean Architecture
- Couleur bleue = Frontend (User-facing)
- Couleur verte = Backend Rust (Business logic)
- Couleur orange = Blockchain (On-chain)
- Couleur violette = ZKP / Cryptographie
- Couleur grise = Infrastructure externe

**Architecture Container (vue haut niveau)** :
```
┌─────────────────────────────────────────────────────────────────────┐
│  [BLEU] CLIENT LAYER — React + Vite + Tailwind + RainbowKit       │
│                                                                     │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌───────────┐  │
│  │ IntentInput  │ │ Delegation   │ │ Dashboard    │ │ Social    │  │
│  │ (textarea NL)│ │ Wizard       │ │ (portfolio)  │ │ Feed      │  │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘ └─────┬─────┘  │
│         └─────────────────┴────────────────┴───────────────┘        │
│                           │                                         │
│                    HTTP / WebSocket (Axum)                          │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────────┐
│  [VERT] OTTER NODE — Rust Workspace (Axum + tokio)                  │
│  Architecture Hexagonale : domain / application / infrastructure    │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  INTERFACES CRATE  (adapters entrants)                       │   │
│  │  - REST API (Axum handlers)                                  │   │
│  │  - WebSocket server (real-time events)                       │   │
│  │  - gRPC server (optionnel, streaming)                        │   │
│  └──────────────────────┬──────────────────────────────────────┘   │
│                         │ traits (Ports)                           │
│  ┌──────────────────────▼──────────────────────────────────────┐   │
│  │  APPLICATION CRATE  (use cases / orchestration)              │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │   │
│  │  │ ParseIntent  │ │ CreateDeleg. │ │ ExecuteIntent        │  │   │
│  │  │ PlanExecution│ │ ProveIntent  │ │ CaptureMev           │  │   │
│  │  └──────────────┘ └──────────────┘ └──────────────────────┘  │   │
│  │  ┌──────────────────────────────────────────────────────────┐│   │
│  │  │ Orchestrator (Finite State Machine)                      ││   │
│  │  │ IDLE → MONITORING → ANALYZING → DECIDING → PROVING →    ││   │
│  │  │ SUBMITTING → CONFIRMING → ERROR → IDLE                   ││   │
│  │  └──────────────────────────────────────────────────────────┘│   │
│  └──────────────────────┬──────────────────────────────────────┘   │
│                         │ traits (Ports)                           │
│  ┌──────────────────────▼──────────────────────────────────────┐   │
│  │  DOMAIN CRATE  (entités / règles métier — pur, sans deps)   │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │   │
│  │  │ Intent       │ │ Delegation   │ │ ExecutionPlan        │  │   │
│  │  │ Condition    │ │ Proof        │ │ Transaction          │  │   │
│  │  └──────────────┘ └──────────────┘ └──────────────────────┘  │   │
│  └──────────────────────┬──────────────────────────────────────┘   │
│                         │ traits (Ports)                           │
│  ┌──────────────────────▼──────────────────────────────────────┐   │
│  │  INFRASTRUCTURE CRATE  (adapters sortants — technologie)    │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │   │
│  │  │[VIOLET] ZKP  │ │ LLM Parser   │ │ PostgreSQL Adapter   │  │   │
│  │  │Prover (Noir) │ │ (llama.cpp)  │ │ (StoragePort)        │  │   │
│  │  └──────────────┘ └──────────────┘ └──────────────────────┘  │   │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │   │
│  │  │ Ethereum     │ │ Solana       │ │ MEV Searcher         │  │   │
│  │  │Adapter(Alloy)│ │Adapter(Anchor│ │ (Flashbots/Jito)     │  │   │
│  │  └──────────────┘ └──────────────┘ └──────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────────┘   │
└───────────────────────────┬─────────────────────────────────────────┘
                            │ RPC / JSON-RPC / gRPC
┌───────────────────────────▼─────────────────────────────────────────┐
│  [ORANGE] CHAIN LAYER — Smart Contracts & Programs                  │
│                                                                     │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │ EVM CHAINS       │  │ EVM CHAINS (L2)  │  │ SOLANA           │  │
│  │ Ethereum Mainnet │  │ Arbitrum One     │  │ Solana Mainnet   │  │
│  ├──────────────────┤  ├──────────────────┤  ├──────────────────┤  │
│  │ StrategyVault.sol│  │ StrategyVault.sol│  │ strategy.so      │  │
│  │ DelegationVerif. │  │ (bridged)        │  │ delegation.so    │  │
│  │ Aave/Compound/   │  │ Aave/Uni (L2)    │  │ Jupiter/Solend/  │  │
│  │ Uniswap adapters │  │                  │  │ Marinade         │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

#### Slide 2 — Architecture Hexagonale Détaillée (C4 Model Niveau 3 : Component)

**Schéma du découpage en crates Rust** :
```
                         ┌──────────────────┐
                         │   FRONTEND       │
                         │  React + Vite    │
                         └────────┬─────────┘
                                  │ HTTP / WS
┌─────────────────────────────────┼──────────────────────────────────┐
│         INTERFACES CRATE        │                                  │
│  ┌─────────────┐  ┌─────────────┴─────────────┐  ┌──────────────┐ │
│  │ REST Router │  │ WebSocket Handler         │  │ gRPC Service │ │
│  │ (Axum)      │  │ (tokio-tungstenite)       │  │ (Tonic)      │ │
│  └──────┬──────┘  └─────────────┬─────────────┘  └──────┬───────┘ │
└─────────┼───────────────────────┼───────────────────────┼─────────┘
          │                       │                       │
          └───────────────────────┼───────────────────────┘
                                  │ appelle les Use Cases
┌─────────────────────────────────▼──────────────────────────────────┐
│       APPLICATION CRATE (Services / Use Cases)                     │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────────────┐  │
│  │ IntentService│ │ Delegation   │ │ ExecutionService           │  │
│  │ - parse()    │ │ Service      │ │ - plan()                   │  │
│  │ - validate() │ │ - create()   │ │ - simulate()               │  │
│  │ - plan()     │ │ - prove()    │ │ - submit()                 │  │
│  └──────┬───────┘ └──────┬───────┘ └────────────┬───────────────┘  │
│         └─────────────────┴──────────────────────┘                  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ OrchestratorService (FSM + Event Bus)                        │  │
│  │  - state_machine: State → Event → State                      │  │
│  │  - event_bus: tokio::sync::mpsc (events internes)            │  │
│  │  - scheduler: tokio::time::interval (monitoring 60s)         │  │
│  └──────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────┬─────────────────────────────────┘
                                   │ dépend des traits (Ports)
┌──────────────────────────────────▼─────────────────────────────────┐
│          DOMAIN CRATE (Entités et Ports — ZERO dépendance externe) │
│                                                                    │
│  ENTITÉS                    PORTS (traits)                         │
│  ┌────────────┐            ┌──────────────────────────────┐        │
│  │ Intent     │◄──────────►│ IntentParserPort             │        │
│  │ Condition  │            │   parse(text) → Intent       │        │
│  │ Amount     │            │   validate(intent) → Result  │        │
│  └────────────┘            ├──────────────────────────────┤        │
│  ┌────────────┐            │ ZkpPort                      │        │
│  │ Delegation │◄──────────►│   prove(delegation, intent)  │        │
│  │ Limits     │            │   verify_offchain(proof)     │        │
│  │ Expiry     │            ├──────────────────────────────┤        │
│  └────────────┘            │ BlockchainPort               │        │
│  ┌────────────┐            │   send_tx(tx) → Receipt      │        │
│  │ Execution  │◄──────────►│   get_balance(addr)          │        │
│  │ Plan       │            │   estimate_gas(tx)           │        │
│  │ Step       │            ├──────────────────────────────┤        │
│  └────────────┘            │ StoragePort                  │        │
│  ┌────────────┐            │   save_intent(), get_intent()│        │
│  │ Proof      │◄──────────►│   list_delegations()         │        │
│  │ PublicInputs│           ├──────────────────────────────┤        │
│  └────────────┘            │ MevPort                      │        │
│                            │   capture_bundle(intent)     │        │
│                            │   submit_bundle()            │        │
│                            └──────────────────────────────┘        │
└────────────────────────────────────────────────────────────────────┘
                                   ▲
                                   │ implémenté par
┌──────────────────────────────────┴─────────────────────────────────┐
│      INFRASTRUCTURE CRATE (Adapters — dépendances technologiques)  │
│                                                                    │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────────────┐  │
│  │ LlmAdapter   │ │ NoirAdapter  │ │ PostgresAdapter            │  │
│  │ (llama.cpp)  │ │ (nargo CLI + │ │ (sqlx)                     │  │
│  │              │ │  barretenberg)│ │                            │  │
│  └──────────────┘ └──────────────┘ └────────────────────────────┘  │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────────────┐  │
│  │ EthereumAdapter│ │ SolanaAdapter│ │ MevAdapter               │  │
│  │ (Alloy)      │ │ (solana-sdk +│ │ (flashbots + jito-rs)      │  │
│  │              │ │  anchor-client)│ │                           │  │
│  └──────────────┘ └──────────────┘ └────────────────────────────┘  │
│  ┌──────────────┐ ┌──────────────┐ ┌────────────────────────────┐  │
│  │ PriceOracle  │ │ WalletAdapter│ │ EventBusAdapter            │  │
│  │ (Chainlink/  │ │ (keystore    │ │ (tokio channels +          │  │
│  │  protocol)   │ │  scrypt)     │ │  broadcast)                │  │
│  └──────────────┘ └──────────────┘ └────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

#### Slide 3 — Flux de Données : Intent → Proof → Execution (Data Flow Diagram)

```
PHASE 1 : INTENTION (Off-chain, User side)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
User ┌──────────┐     ┌─────────────┐     ┌──────────────┐
text │ Frontend │────►│ LLM Adapter │────►│ Intent Entity│
     └──────────┘     │ (local)     │     │ (JSON struct)│
                      └─────────────┘     └──────┬───────┘
                                                 │
PHASE 2 : DÉLÉGATION (Off-chain, Signing)        │
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━│
User ┌──────────────┐     ┌─────────────────────┐ │
sign │ Delegation   │────►│ Delegation Entity   │◄┘
     │ Wizard (UI)  │     │ (limits, protocols) │
     └──────────────┘     └──────────┬──────────┘
                                     │ hash + sign
                                     ▼
                              ┌──────────────┐
                              │ EIP-712 Sign │
                              │ (wallet)     │
                              └──────┬───────┘
                                     │ delegation_hash
                                     ▼
PHASE 3 : ORCHESTRATION (Off-chain, Server side)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                              ┌──────────────────┐
                              │ Orchestrator FSM │
                              │ MONITORING (60s) │
                              └────────┬─────────┘
                                       │ condition met ?
                                       ▼
                              ┌──────────────────┐
                              │ ProposedIntent   │
                              │ + DelegationHash │
                              └────────┬─────────┘
                                       │
PHASE 4 : PREUVE ZKP (Off-chain, Proving)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                                       ▼
                              ┌──────────────────┐
                              │ NoirAdapter      │
                              │ (nargo prove)    │
                              │                  │
  Private Inputs (Witness)    │ ┌──────────────┐ │
  ├─ agent_privkey            │ │ Circuit Noir │ │
  ├─ full_delegation_msg      │ │ ├─ sig check │ │
  │                           │ │ ├─ amount ≤  │ │
  Public Inputs               │ │ ├─ protocol ∈│ │
  ├─ delegation_hash          │ │ ├─ expiry OK │ │
  ├─ proposed_intent          │ │ └────────────┘ │
  ├─ timestamp                │ └───────┬────────┘
  └─ nonce                    └─────────┘
                                       │ proof_bytes
                                       ▼
PHASE 5 : SOUMISSION (On-chain, Execution)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                              ┌──────────────────┐
                              │ Transaction      │
                              │ ├─ to: Vault     │
                              │ ├─ data: proof   │
                              │ └─ value: 0      │
                              └────────┬─────────┘
                                       │ RPC (Alloy)
                                       ▼
                    ┌─────────────────────────────────────┐
                    │  [EVM] StrategyVault.sol            │
                    │  ┌─────────────────────────────┐    │
                    │  │ verify(delegation_proof)    │    │
                    │  │ ├─ DelegationVerifier.call()│    │
                    │  │ ├─ nonce check (anti-replay)│    │
                    │  │ └─ expiry check             │    │
                    │  └─────────────────────────────┘    │
                    │              │ valid ?               │
                    │              ▼                       │
                    │  ┌─────────────────────────────┐    │
                    │  │ execute(protocol_adapter)   │    │
                    │  │ ├─ Aave.supply()            │    │
                    │  │ ├─ split_mev()              │    │
                    │  │ └─ emit ActionExecuted      │    │
                    │  └─────────────────────────────┘    │
                    └─────────────────────────────────────┘
                                       │
                    ┌──────────────────┴───────────────────┐
                    │ [SOLANA] strategy.so                 │
                    │  ├─ verify_proof()                   │
                    │  ├─ execute_jupiter_swap()           │
                    │  └─ distribute_fees()                │
                    └──────────────────────────────────────┘
```

#### Slide 4 — Modèle de Données et Persistence

```
┌─────────────────────────────────────────────────────────────────────┐
│                         PERSISTENCE LAYER                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ PostgreSQL   │  │ PostgreSQL   │  │ Local Files  │              │
│  │ (intents)    │  │ (delegations)│  │ (keystore)   │              │
│  ├──────────────┤  ├──────────────┤  ├──────────────┤              │
│  │ id (UUID)    │  │ id (UUID)    │  │ agent_keypair│              │
│  │ user_addr    │  │ user_addr    │  │ (scrypt)     │              │
│  │ intent_json  │  │ agent_pubkey │  │              │              │
│  │ status (FSM) │  │ limits_json  │  │              │              │
│  │ created_at   │  │ expiry       │  │              │              │
│  └──────────────┘  │ nonce        │  │              │              │
│                    └──────────────┘  └──────────────┘              │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ EVENT BUS (tokio::sync::mpsc + broadcast) — in-memory       │  │
│  │  - PriceUpdated, ConditionMet, IntentParsed                │  │
│  │  - ProofGenerated, TransactionSubmitted, TransactionConfirmed│  │
│  │  - MevCaptured, Error                                      │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

#### Slide 5 — Justification et Caractéristiques

1. **Choix du formalisme** :
   - **C4 Model** (niveaux 2 et 3) : adapté aux architectures microservices et hexagonales. Clarifie la séparation des responsabilités.
   - **Data Flow Diagram** : montre le cycle de vie d'un intent de sa création à son exécution on-chain, essentiel pour comprendre le flux ZKP.
   - Pas d'UML complet (classes, séquences) car l'architecture est orientée événements et traits Rust, mieux représentée par des diagrammes de composants et de flux.

2. **Signification des formes et couleurs** :
   - **Hexagones** = Ports (interfaces) — formalisme obligatoire de la Clean Architecture. Le domaine ne dépend que d'abstractions.
   - **Rectangles** = Adapters (implémentations concrètes) — peuvent être remplacés sans toucher au métier.
   - **Flèches verticales entre crates** = Dépendance directionnelle : `infrastructure → domain` est interdit. Seul `infrastructure → domain` via trait est autorisé.
   - **Cylindres** = Persistence externe — PostgreSQL pour la durabilité, Event Bus in-memory pour la latence.

3. **Interactions détaillées avec systèmes externes** :
   | Système externe | Protocole | Type d'interaction | Données échangées |
   |----------------|-----------|-------------------|-------------------|
   | Infura/Alchemy | JSON-RPC HTTPS | Synchrone | Transactions, balances, gas estimates |
   | Flashbots Protect | JSON-RPC + bundle API | Synchrone | Private transaction bundles, MEV extraction |
   | Jito (Solana) | gRPC + bundle API | Synchrone | Bundles de transactions Solana, tips |
   | Chainlink / Protocol Oracles | `call` on-chain | Synchrone | Prix assets, APY lending pools |
   | Aave/Compound/Uniswap | `delegatecall` | On-chain | Supply, borrow, swap, liquidity |
   | Solana Programs (Jupiter, Solend) | CPI / instruction | On-chain | Swap, stake, lend via Anchor |
   | llama.cpp server | HTTP local (localhost:8080) | Synchrone | Prompt texte → JSON structuré |
   | nargo (Noir CLI) | Process spawn + files | Synchrone | Inputs → proof_bytes + public_inputs |

4. **Caractéristiques architecturales** :
   - **Maintenable** : Découplage total via les traits (Ports). Changer de base de données, de blockchain, ou de LLM ne nécessite de modifier que l'Adapter correspondant.
   - **Sécurisée** :
     - ZKP vérifiée on-chain avant exécution (DelegationVerifier.sol / program)
     - Keystore agent chiffré (scrypt/pbkdf2) — pas de clés en clair
     - Nonces incrémentaux on-chain pour anti-replay
     - Event bus interne = audit trail immuable en mémoire
   - **Extensible** :
     - `BlockchainPort` permet d'ajouter Solana, Optimism, Base sans toucher l'orchestrator
     - `ProtocolRegistry` mappe dynamiquement les protocoles (Aave, Compound, Solend...)
     - `IntentParserPort` permet de basculer de llama.cpp à Claude API si besoin
   - **Impact environnemental** :
     - Preuve ZK = vérification on-chain moins coûteuse en gas que la logique classique (moins d'opérations EVM)
     - Arbitrum (L2) = ~97% moins d'énergie que L1 par transaction
     - Solana = consensus PoH (Proof of History) à haut rendement énergétique, faible coût par tx
     - LLM local Q4_K_M = modèle quantifié léger, pas de data center distant
     - Architecture event-driven = traitement asynchrone, pas de polling intensif

---

### Partie C1.6 — Préconisation des Axes de Solutions et Argumentaire Client
**Titre slide** : Préconisation et Argumentaire Client

**Contenu attendu** :
1. **Résumé de la proposition de valeur** :
   > Otter est la seule plateforme qui combine : langage naturel + délégation cryptographique persistante + preuve ZKP + capture MEV + preuve de solvabilité + partage social de stratégies.

2. **Axes de solutions préconisés** :
   - **Axe 1 — Trustless by Design** : Grâce aux preuves ZK, l'utilisateur ne fait confiance à personne. Les limites sont mathématiques, pas contractuelles.
   - **Axe 2 — Simplicité UX** : Exprimer une stratégie en anglais, pas en code. Le LLM local traduit immédiatement.
   - **Axe 3 — Rentabilité** : L'automatisation ne coûte pas cher — elle paie l'utilisateur via les rebates MEV.
   - **Axe 4 — Transparence** : Preuve de solvabilité périodique, proof explorer public, tout est vérifiable.

3. **Argumentaire structuré par objection** (tableau ou slides) :

   | Objection client | Réponse argumentée | Preuve / Donnée |
   |------------------|--------------------|-----------------|
   | "Pourquoi pas Gelato ou Keep3r ?" | Ces solutions requièrent confiance dans le code de l'opérateur. Otter est trustless grâce aux ZKPs. | Matrice comparative (PRODUCT.md section 7) |
   | "ZK, c'est lent et coûteux ?" | Cible < 3s par preuve, circuit optimisé (< 1000 contraintes), L2 Arbitrum et Solana réduisent les coûts. | Benchmarks cibles (PRODUCT.md section 10) |
   | "C'est sûr ? Mon argent est en danger ?" | Vault audité, circuit audité, slashing agent, preuve de solvabilité, keystore chiffré. | Trust Model (PRODUCT.md section 5) |
   | "Pourquoi un LLM local et pas ChatGPT ?" | Privacy : vos données financières ne quittent jamais votre machine. Pas de coût API récurrent. | Architecture LLM (README section Tech Stack) |
   | "Pourquoi Rust et pas Node.js ?" | Sécurité mémoire critique pour un projet financier. Performance pour la génération ZK. | Étude comparative C1.3.2 |

4. **Vulgarisation du discours** :
   - Utiliser des analogies accessibles :
     - *"La délégation ZKP, c'est comme donner une procuration à un agent immobilier avec des limites gravées dans la pierre — il ne peut pas les dépasser, mathématiquement."*
     - *"Le MEV, c'est comme le 'reste' d'une transaction boursière. Au lieu que les traders haute fréquence le gardent, Otter le redistribue à vous."*
     - *"La preuve de solvabilité, c'est comme un avocat qui certifie 'le coffre contient assez d'argent' sans révéler combien chaque client a déposé."*

5. **Supports de communication cohérents** :
   - Schéma d'architecture avec légendes (C1.5)
   - Matrice comparative concurrentielle (PRODUCT.md)
   - Diagrammes de flux Mermaid (FLOW.md) — main flow, ZKP delegation, MEV capture
   - Tableau de backlog avec progression visuelle (BACKLOG.md)
   - Graphique de budget prévisionnel

---

## Instructions de Style et de Format

### Conception Visuelle
- **Palette** : Fond **clair et aéré** (light mode) — blanc cassé (#F8FAFC) ou très léger bleu océan (#F0F9FF) pour les slides, texte foncé (#1E293B). Accents pastel :
  - Bleu océan doux (#0EA5E9) pour le frontend
  - Vert menthe (#34D399) pour le backend
  - Orange corail (#FB923C) pour la blockchain
  - Violet lavande (#A78BFA) pour ZKP
  - Sable beige (#FDE68A) pour les highlights et annotations
  - **Logo loutre (🦦)** utilisé comme filigrane discret ou icône de section
- **Typographie** : Sans-serif moderne et lisible (Inter, Roboto, ou système). Titres 32-40pt gras, corps 18-24pt, lignes larges (1.5) pour aérer.
- **Icinographie** : Icônes cohérentes style "line" (Heroicons, Phosphor Icons). Une petite loutre stylisée 🦦 comme pictogramme de transition entre les grandes parties.
- **Schémas** : Tous les schémas doivent être légendés (formes, flèches, couleurs). Utiliser des diagrammes en couche (layer cake) pour l'architecture. Fonds des boîtes légèrement colorés (pastel 10-20% opacité) sur fond blanc.

### Structure par Slide
- **Titre** : Clair, en haut, taille 36pt
- **Contenu** : Listes à puces courtes, tableaux lisibles, graphiques simples
- **Footer** : Numéro de slide + "Otter 🦦 — Projet DeFi ZKP" + référence du critère (ex: C1.1.1)
- **Transitions** : Slide de transition entre chaque grande partie (C1.1, C1.2, C1.3, C1.4, C1.5, C1.6)

### Langage
- **Professionnel** : vocabulaire technique approprié (ZKP, MEV, DeFi, EVM, preuve de solvabilité)
- **Vulgarisé** : expliquer chaque acronyme à sa première occurrence, utiliser des analogies pour les concepts complexes
- **Concis** : pas de phrases longues, privilégier les tableaux et les listes
- **Argumenté** : chaque choix technique doit être justifié par un avantage métier ou une contrainte projet

### Vérification Finale
- Chaque critère C1.1.1 à C1.6 doit être clairement identifiable (titre de slide ou section)
- Tous les éléments demandés dans le référentiel d'évaluation doivent être présents
- Les données chiffrées doivent être cohérentes entre les slides (budget, charge, backlog)
- Le document doit pouvoir être présenté en **20 minutes de présentation + 10 minutes de questions** devant un jury

---

## Données Chiffrées Clés à Inclure (cohérence obligatoire)

| Indicateur | Valeur | Source |
|------------|--------|--------|
| Stories backlog total | 481 | BACKLOG.md |
| Stories MVP | ~396 | BACKLOG.md |
| Temps preuve ZKP cible | < 3s | PRODUCT.md section 10 |
| Transactions testnet cible | 1 000+ | PRODUCT.md section 10 |
| Transactions mainnet cible | 100+ | PRODUCT.md section 10 |
| Split MEV | 50% user / 40% agent / 10% protocol | PRODUCT.md section 4.4 |
| Frais protocol | 0.1% volume + 10% MEV | PRODUCT.md section 6 |
| Charge estimée MVP | 220-354 jours-homme | Dérivé du backlog |
| Budget total estimé | 326 000 - 465 000 € | Calculé sur la charge |
| Uptime agent cible | > 99% | PRODUCT.md section 10 |
| Modèles LLM | LFM2.5-1.2B-Q4_K_M, Qwen3-8B-Q4_K_M | Répertoire `models/` |
| Réseaux | Ethereum mainnet + Arbitrum + Solana | README.md |
| Circuits ZK | ~847 contraintes (estimation) | PRODUCT.md section 2.5 |

---

## Livrable Attendu
Un fichier de présentation (format .pptx, .pdf, ou lien vers présentation web) contenant :
1. Page de garde
2. Sommaire
3. C1.1.1 — Cartographie parties prenantes (1-2 slides : tableau acteurs + profils users)
4. C1.1.2 — Analyse demande (2 slides : besoin/objectifs + problématique/pistes)
5. C1.2.1 — SWOT (1 slide : tableau 2×2 + impact environnemental en annotations)
6. C1.2.2 — Audit technique (2 slides : démarche + infrastructures/contraintes/faisabilité)
7. C1.2.3 — Risques (1-2 slides : matrice risques + indicateurs de contrôle)
8. C1.3.1 — Veille (1 slide : sources/outils + évolutions classées)
9. C1.3.2 — Étude comparative (1-2 slides : tableaux comparatifs + choix retenus)
10. C1.4.1 — Charge de travail (1 slide : fonctionnalités hiérarchisées + estimation jh)
11. C1.4.2 — Budget (1 slide : tableau prévisionnel par poste)
12. C1.5 — Architecture (3-4 slides max : vue globale + hexagonale + flux de données + modèle de données)
13. C1.6 — Préconisation (2 slides : proposition valeur/axes + argumentaire/objections)
14. Conclusion / Questions

**Total : ~20-25 slides max**
