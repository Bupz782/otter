# Bloc 3 — Liste de fonctionnalités de référence (C3.4.2)

> La démonstration répond **point par point** à cette liste. Chaque
> fonctionnalité est adossée à une preuve vérifiable (artefact, test,
> commit) et à un moment précis du scénario de démo
> (`02_SCENARIO_DEMO.md`). Colonne "Démo" : LIVE = montré en direct ;
> SCRIPT = couvert par `scripts/demo.sh` ; PAGE = montré dans l'UI ;
> DOC = présenté sur preuve/document si le direct n'est pas possible.

| # | Fonctionnalité | Preuve vérifiable | Démo |
|---|---|---|---|
| F1 | Authentification Sign-In with Ethereum (SIWE) + JWT + RBAC (rôles reader/writer/admin) | commits `b5cf1a5` (roles/refresh/RBAC), `9bc10af` (endpoint rôles) 2026-08-26 ; routes `/api/v1/auth/*` ; tests auth dans les 282 tests Rust | PAGE (connexion wallet à l'ouverture) |
| F2 | Saisie d'intent en langage naturel → `ConditionalIntent` structuré (parseur hybride LLM + regex déterministe) | `crates/infrastructure/src/parsers/` ; page `/app/intents/new` (stepper Review) | PAGE (création d'un intent en direct) |
| F3 | Délégation signée à limites explicites (montants, protocoles, expiration) + hash enregistré on-chain | `DelegationVault.delegate()` ; page `/app/delegations/new` ; `scripts/demo.sh` étape 5 | SCRIPT |
| F4 | Preuve ZK réelle (circuit Noir + Barretenberg/UltraHonk) vérifiée on-chain avant exécution | `delegation_circuit/` ; test e2e `e2e_anvil_flow` (proof bb réelle) ; test forge `test_verifiesValidDelegationProof` | SCRIPT (moment fort, ~27 s au total) |
| F5 | Exécution on-chain via `DelegationVault.executeWithProof` avec garde-fous (anti-rejeu, expiration, intent non autorisé, montant dépassé, preuve falsifiée) | 28 tests forge dont 8 cas de revert ; `scripts/demo-negative.sh` (revert `AmountExceedsMax` avec preuve réelle) | SCRIPT + DOC (plan B/C) |
| F6 | Preuve de solvabilité ZK on-chain : `SolvencyRegistry` + vérifier réel, API `/api/v1/solvency/status` | commit `563c065` (fixtures bb réelles + verifier) ; page `/app/solvency` | PAGE (après déploiement du registry par demo.sh) |
| F7 | Bridge cross-chain EVM V1 (lock/mint, `OtterBridge`/`BridgeToken`) + API + historique des transferts | commits `ec8bcc4`, `a36b74f` ; page `/app/bridge` ; `docs/BRIDGE.md` | PAGE ou DOC selon config |
| F8 | MEV searcher V1 (tx privées) et V2 (bundles Flashbots, mempool monitor, backrun handler) + rebate_bps modifiable à chaud | commits `bd43b58`, `3785a50`, `bf3e3c7` ; page `/app/mev` ; `docs/MEV_SEARCHER.md` | PAGE ou DOC selon config |
| F9 | Attestations Solana : programme Anchor `attestation_registry`, adapter Rust feature-gated, **scheduler d'attestation du merkle root solvency** | commits `cee90fc`, `68542ff` (2026-08-31) ; page `/app/solana` ; `docs/SOLANA.md` | PAGE (état configuré ou non) ou DOC |
| F10 | Marketplace d'agents + stratégies SocialFi forkables | commit `cfbf746` ; pages `/app/agents`, `/app/strategies` | PAGE (données marquées démo dans l'API — header X-Demo-Data, à assumer) |
| F11 | Observabilité : métriques `/metrics`, alertes Prometheus (`alerting.yml`), événements temps réel WebSocket | `alerting.yml`, `deploy/prometheus.yml`, preuve alerte horodatée `docs/preuves/alerte-test-20260819-094726.json` | DOC |
| F12 | Packaging reproductible : stack Docker Compose (anvil + api + frontend), images CI | `docker-compose.yml`, `Dockerfile`, workflow Docker vert | LIVE (lancement de la stack en ouverture de démo) |

## Correspondance avec le périmètre annoncé

- Source de référence : `BACKLOG.md` (481 US, vagues MVP) et `PRODUCT.md`.
- Fonctionnalités marquées démo dans l'API (agents, leaderboard, proofs
  synthétiques) : signalées par le header `X-Demo-Data` et le champ `demo:
  true` des réponses — **dire au jury que le marquage est volontaire**
  (anomalie A2 tracée, retrait obligatoire avant mainnet).
- Fonctionnalités CUT assumées : FHE, mempool chiffré (visibles dans les
  issues GitHub ouvertes, marquées CUT dans `ISSUES.md`) — arbitrage de
  périmètre documenté, matière pour la question "cas d'arbitrage".

## Parcours de démo retenu (couverture maximale en 6–7 min)

1. Stack lancée (F12) → ouverture UI, connexion wallet (F1).
2. Création d'un intent en langage naturel (F2).
3. Bascule terminal : `scripts/demo.sh` → déploiement vault + registry,
   délégation, dépôt, **preuve ZK réelle vérifiée on-chain** (F3, F4, F5, F6
   côté contrat) — 27 s mesurées.
4. Retour UI : page solvency (F6), puis selon configuration bridge (F7),
   MEV (F8), solana (F9).
5. Clôture : `forge test` (28/28 en < 1 s) ou scroll des pages
   agents/strategies (F10) si le temps reste.
