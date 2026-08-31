# Bloc 3 — Scénario de démonstration (compétence éliminatoire C3.4.2)

> 6 à 7 minutes maximum dans les 30 minutes de présentation. Zéro
> improvisation : ce scénario est écrit pour être relu tel quel la veille.
> Chaque étape répond à la liste de référence `01_FONCTIONNALITES_REFERENCE.md`.
> Temps mesurés sur la machine de démo le 2026-08-31.

## 0. Avant l'entrée du jury (T-15 min)

- [ ] Docker Desktop lancé, `docker compose up --build -d` exécuté, `docker compose ps` : 3 services up.
- [ ] `curl -s localhost:3001/health` → réponse OK.
- [ ] `curl -s -X POST localhost:8545 -H 'Content-Type: application/json' --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'` → `0x7a69`.
- [ ] Navigateur ouvert sur `http://localhost:3000`, wallet de démo déverrouillé (compte anvil #0 importé).
- [ ] Terminal 1 : repo racine, police grande. Terminal 2 : prêt sur `scripts/`.
- [ ] Vidéo de secours + captures horodatées sur le bureau (plan B).

## 1. Minute par minute (plan A — tout fonctionne)

| T | Durée | Action | Ce que je dis | Features |
|---|---|---|---|---|
| 0:00 | 0:30 | Montrer `docker compose ps` (3 services) | « Le logiciel tourne tel quel : `docker compose up --build`, 3 services — anvil (chaîne locale), API Rust, frontend. C'est le livrable de la CI. » | F12 |
| 0:30 | 1:00 | UI `localhost:3000` → connexion wallet, challenge SIWE signé | « Authentification sans mot de passe : Sign-In with Ethereum, JWT, et du RBAC derrière — rôles reader/writer/admin sur les endpoints sensibles. » | F1 |
| 1:30 | 1:00 | `/app/intents/new` : taper « lend 1000 USDC on Aave if yield > 3 », montrer le stepper Review avec l'intent structuré | « L'utilisateur écrit en langage naturel ; le parseur produit un intent structuré qu'il relit et valide — jamais d'exécution sans validation humaine. » | F2 |
| 2:30 | 1:30 | Terminal : `bash scripts/demo.sh` — commenter pendant l'exécution : déploiement DelegationVault, SolvencyRegistry, délégation signée, dépôt, **preuve ZK réelle bb vérifiée on-chain**, exécution | « C'est le cœur du protocole : l'agent génère une preuve Noir/Barretenberg que l'action respecte la délégation ; le contrat la vérifie AVANT d'exécuter. 27 secondes mesurées, cible < 3 minutes. » | F3 F4 F5 |
| 4:00 | 1:00 | UI `/app/solvency` (registry déployé par demo.sh) | « Preuve de solvabilité du vault, vérifiée on-chain — le statut lit le registre déployé il y a deux minutes sous vos yeux. » | F6 |
| 5:00 | 1:00 | UI `/app/bridge` puis `/app/mev` | « Extensions livrées cette semaine : bridge cross-chain lock/mint, et le searcher MEV V2 — bundles Flashbots, monitor mempool, rebates paramétrables à chaud. » | F7 F8 |
| 6:00 | 0:30 | UI `/app/solana` (ou slide DOC si non configuré) + phrase de clôture | « Et la brique multi-chaîne : attestations Solana, avec un scheduler qui ancre périodiquement la racine Merkle de solvabilité. » | F9 |
| 6:30 | — | Retour slides | « Ce que vous avez vu couvre la liste de référence F1–F9 — le tableau est dans le dossier. » | — |

**Si le chronomètre dépasse 6:30 à l'étape bridge : sauter MEV/Solana UI et
clore.** Les features F7–F9 sont aussi couvertes par le support (docs +
commits), la démo ne doit jamais mordre sur le temps de pilotage.

## 2. Plan B — un composant lâche (anvil, réseau, wallet)

Déclencheur : anvil ne répond pas, la preuve ZK échoue, ou le wallet ne
signe pas. Annoncer calmement : « je bascule sur le plan B, prévu ».

1. **Vidéo/captures horodatées** du run `demo.sh` (à enregistrer la veille,
   voir checklist J-1) — commenter exactement le même script.
2. Sortie réelle du run du 2026-08-31 (27 s, test e2e vert) imprimée dans ce
   dossier — `docs/bloc3/00_INVENTAIRE_PREUVES.md` §3.
3. `cd contracts && forge test` en direct : 28/28 en moins d'une seconde,
   dont 8 cas de revert (anti-rejeu, expiration, montant dépassé, preuve
   falsifiée) — **ça marche sans réseau ni Docker**, à garder comme réflexe.

## 3. Plan C — tout lâche (machine morte, projecteur KO)

Déroulé oral, sans écran :

1. « La chaîne critique est prouvée par 282 tests Rust, 28 tests Solidity,
   une couverture mesurée à 55,92 % le 20 juillet, et une démo scriptée
   `scripts/demo.sh` que voici imprimée — chaque étape y est commentée. »
2. Montrer le script imprimé + la sortie horodatée du run du matin même.
3. Expliquer la preuve ZK en 30 secondes avec le schéma de la slide
   architecture (chaîne de valeur) et passer au temps de pilotage : « la
   démo fait 6 minutes sur 30 ; le pilotage, lui, est entièrement consultable
   en ligne : issues, PRs, CI, tags. »
4. Proposer au jury de rejouer la démo eux-mêmes : repo public, une commande.

## 4. Mesures réelles (à rafraîchir la veille)

| Élément | Mesure | Date |
|---|---|---|
| `scripts/demo.sh` complet (anvil → preuve → exécution) | **27 s** (cache cargo chaud) | 2026-08-31 |
| dont test e2e preuve réelle | 1,5 s | 2026-08-31 |
| `forge test` (28 tests) | 0,055 s | 2026-08-31 |
| `cargo test --workspace` (282 tests) | 4,5 s d'exécution pure des suites | 2026-08-31 |
| `docker compose up --build -d` (build à froid) | **2 min 40** | 2026-08-31 |
| Stack Compose vérifiée | anvil `eth_chainId` → `0x7a69` depuis l'hôte, `/health` OK, frontend 200, nginx→API 200 | 2026-08-31 |

## 5. Journal de répétition

- 2026-08-31 : `demo.sh` rejoué avec succès (27 s). Stack Compose testée en
  conditions réelles — **trois bugs trouvés et corrigés le jour même** :
  ① build frontend cassé sur arm64/alpine (`utf-8-validate` sans prebuild →
  toolchain node-gyp ajoutée au builder stage) ; ② anvil injoignable depuis
  l'hôte (l'entrypoint `/bin/sh -c` de l'image foundry mangeait les flags —
  corrigé par `entrypoint: ["anvil"]` explicite) ; ③ `OTTER_NETWORKS` jamais
  lu par la config (`parse_networks_spec` sans appelant, `/api/v1/networks`
  vide) → override env ajouté + test. Chaque fix vérifié par re-mesure.
  **Cet épisode est réutilisable tel quel à la slide 12 / Q12** : un suivi
  (test de la démo avant le jour J) qui déclenche des actions datées.
- _À compléter : répétition chronométrée complète (slides + démo) le 14/09._
