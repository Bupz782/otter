# Plan de correction des bogues — Projet Otter

> Document de certification RNCP — critère « les bogues sont détectés, qualifiés et traités ».
> Toutes les entrées du registre (section 2) correspondent à des commits réels de l'historique git, vérifiables via `git show <sha>`.

---

## 1. Méthodologie

### 1.1 Détection des bogues

Les bogues sont détectés par quatre canaux complémentaires :

- **Intégration continue (`.github/workflows/ci.yml`)** — chaque push/PR déclenche :
  - `cargo clippy --workspace --all-targets -- -D warnings` (tout avertissement est bloquant) ;
  - `cargo test --workspace` (tests unitaires et d'intégration Rust) ;
  - `forge test` (contrats Solidity) ;
  - `nargo test` (circuit Noir) ;
  - `npm run test` (vitest, frontend) ;
  - un job `docker-smoke` qui construit les images et vérifie leur démarrage.
- **Smoke tests post-déploiement (`scripts/smoke-test.sh`)** — exécutés après chaque déploiement testnet/mainnet : sondage de `/ready` (30 s, 503 toléré pendant le chauffage), vérification stricte de `/health` et de `/intents/parse`.
- **Revues de code** — chaque passe de revue (notamment sur le plan CI/DevOps) a produit des findings classés *Critical* / *Important*, convertis en commits de correction (voir registre).
- **Recette manuelle** — parcours frontend (onboarding, mapping des intents) et exercice de l'API.

### 1.2 Grille de qualification

| Criticité | Définition | Exemple réel |
|---|---|---|
| **Bloquant** | Compile pas, faille de sécurité exploitable, ou perte/corruption de fonds possible. Arrêt immédiat de toute autre tâche. | `setProtocolRouter` sans contrôle d'accès (`feb03f5`) |
| **Majeur** | Fonctionnalité dégradée ou indisponible, CI/déploiement en échec, comportement erroné en production. Correction avant toute release. | Smoke tests en échec faute de route `/api/v1/health` (`30506dd`) |
| **Mineur** | Contournement simple disponible, impact limité à un environnement ou à un cas marginal. Planifié dans l'itération. | Script de déploiement sans `set -euo pipefail` (`51e9ae9`) |
| **Cosmétique** | Aucun impact fonctionnel : formatage, nommage, style. Corrigé à l'occasion. | Échec `cargo fmt` en CI (`c4afa6f`) |

La **priorité** est dérivée de la criticité et du contexte : un Majeur touchant la CI passe devant un Majeur touchant un script de laboratoire, car il bloque toute l'équipe.

### 1.3 Workflow de traitement

1. **Détection** — log CI, smoke test, revue ou recette (le lien vers le run ou le finding est conservé).
2. **Qualification** — criticité + priorité selon la grille ci-dessus.
3. **Reproduction** — commande ou scénario minimal qui déclenche le bogue (ex. `forge test`, `cargo test -p infrastructure`, appel curl sur la route fautive).
4. **Correction** — commit dédié au format `fix(<scope>): …`, atomique autant que possible.
5. **Test de non-régression** — chaque correction est accompagnée d'un test ou d'un contrôle automatisé qui aurait détecté le bogue (voir colonne dédiée du registre).
6. **Validation** — pipeline CI verte (clippy + tests des 4 stacks) puis merge sur `main`.

### 1.4 Prévention des régressions

- Règle d'équipe : **aucune correction sans test associé** — test unitaire ajouté (ex. `174c315` ajoute les tests KMS/Vault), contrat de route vérifié par smoke test (ex. `30506dd`), ou règle CI renforcée (ex. `881a418`).
- `clippy -D warnings` et `cargo fmt --check` rendent bloquantes les classes de bogues syntaxiques et de style, qui ne peuvent plus réapparaître silencieusement.
- Les smoke tests post-déploiement rejouent à chaque release le contrat HTTP minimal de l'API.

---

## 2. Registre des bogues traités

Bogues représentatifs extraits de l'historique git réel (`git log --oneline`, détails via `git show --stat <sha>`). Statut « Résolu » = correction mergée sur `main` et CI verte.

| ID | Description | Détection | Criticité | Correction (commit) | Test de non-régression | Statut |
|---|---|---|---|---|---|---|
| BUG-01 | `DelegationVault.setProtocolRouter` appelable par n'importe qui : absence de contrôle d'accès sur une fonction d'administration du vault | Revue de code (finding Critical) | Bloquant | `feb03f5` — ajout de `Ownable` (OpenZeppelin) et du modificateur `onlyOwner` | `forge test` sur `contracts/` en CI | Résolu |
| BUG-02 | Panics Tokio sur runtime imbriqué dans les providers de secrets ; `VAULT_TOKEN` vide accepté silencieusement ; erreurs providers retournant `None` sans log | Revue de code (finding Critical) | Majeur | `174c315` — helper `block_on_secret` sûr dans/hors runtime, `VAULT_TOKEN` manquant traité comme erreur, `tracing::error` sur tous les chemins d'erreur | Tests unitaires Env/File/KMS/Vault ajoutés dans le commit, scénarios « dans / hors runtime » | Résolu |
| BUG-03 | Smoke tests en échec : l'API n'expose pas `/api/v1/health` ; versions Noir/bb non passées en build-args au build Docker | Smoke tests CI (`docker.yml`) | Majeur | `30506dd` — alias de route `/api/v1/health` dans `otter_api.rs` + lecture de `NOIR_VERSION`/`BB_VERSION` dans `docker.yml` | `scripts/smoke-test.sh` rejoué à chaque déploiement | Résolu |
| BUG-04 | Push d'images GHCR invalide : nom de dépôt avec majuscules, et syntaxe `${{ github.repository \| lower }}` non supportée dans `env:` | Échec des workflows `docker.yml` / `deploy-testnet.yml` | Majeur | `881a418` — normalisation en minuscules via une étape `tr` et `GITHUB_OUTPUT` | Workflow de build/push d'images vert en CI | Résolu |
| BUG-05 | Dette CI/DevOps : migration `user_address` dupliquée (`0003` SQL + `20250708000002`), avertissements clippy, tests ZKP instables, version Noir non épinglée | CI en échec (clippy, tests) | Majeur | `de28a51` — consolidation des migrations, corrections clippy, stabilisation `zkp_noir.rs`, ajout de `.noir-version` | `cargo clippy -D warnings`, `cargo test`, `nargo test` en CI | Résolu |
| BUG-06 | Alertes Prometheus référençant des métriques jamais émises par l'API (alertes muettes) | Revue de code | Majeur | `f4ecb98` — émission des métriques manquantes dans `otter_api.rs` et alignement d'`alerting.yml` | Job `docker-smoke` + vérification `/metrics` lors des smoke tests | Résolu |
| BUG-07 | Onboarding frontend : voile d'opacité assombrissant toute l'application, spotlight masquant le contenu cible, tooltip rendu sans cible valide, hook `useParseIntent` non mappé sur la forme réelle du backend | Recette manuelle + revue | Majeur | `23a778c` — suppression du voile, spotlight simplifié (anneau + halo), rendu conditionné à une cible valide, mapping via `mapBackendConditionalIntent` | `frontend/src/lib/api.test.ts` (vitest, 3 cas : Lend+condition, Swap sans condition, fallback Composite) | Résolu |
| BUG-08 | Clé privée Anvil codée en dur dans `scripts/dev.sh` | Revue de code (finding Critical) | Majeur | `03db1e3` — suppression de la clé, exigence de `DEPLOYER_PRIVATE_KEY` avec erreur explicite si absente | Revue + grep de secrets avant merge ; `set -euo pipefail` ajouté | Résolu |
| BUG-09 | Job `docker-smoke` non protégé contre l'échec du job `changes` (paths-filter) ; migration `0003` non idempotente côté PostgreSQL | Revue de code (finding Important) | Majeur | `9ca4724` — garde `needs.changes.result == 'success'`, `ALTER TABLE … ADD COLUMN IF NOT EXISTS` pour Postgres | Pipeline CI verte ; migrations rejouées au démarrage du conteneur | Résolu |
| BUG-10 | Erreur de compilation : déclaration redondante de modules dans `orchestrator/mod.rs` | `cargo build` / CI | Bloquant | `0af6335` — suppression de la double déclaration `pub mod` / `pub use` | `cargo clippy --workspace --all-targets -- -D warnings` | Résolu |
| BUG-11 | Script de déploiement mainnet sans `set -euo pipefail` (erreurs silencieuses) et message TODO laissé en production | Revue de code | Mineur | `51e9ae9` — ajout de `set -euo pipefail`, reformulation professionnelle du message | Exécution du workflow `deploy-mainnet.yml` avec smoke test final | Résolu |
| BUG-12 | Échec du contrôle de formatage en CI | CI (`cargo fmt --check`) | Cosmétique | `c4afa6f` — application de `cargo fmt` | Étape fmt ajoutée à la CI (`94bd629`) | Résolu |

---

## 3. Analyse des points d'amélioration

Points issus des tests en échec et des faiblesses constatées en recette. Pour chacun : constat, risque, action corrective, statut.

### 3.1 Couverture de tests non mesurée historiquement

- **Constat** — la couverture de code n'était pas mesurée : aucun outil ni rapport n'existait dans le dépôt. `cargo-tarpaulin` 0.34.1 a depuis été mis en place : la première mesure (`cargo tarpaulin --workspace --lib`) donne **41,14 % de lignes couvertes (1360/3306)** sur les bibliothèques Rust, avec un rapport HTML généré dans `coverage/tarpaulin-report.html` (artefact non versionné) et une configuration reproductible versionnée (`tarpaulin.toml`). La mesure exclut les tests d'intégration (qui requièrent Anvil/Postgres) : plusieurs adaptateurs d'infrastructure (Postgres, service d'exécution) y sont couverts et apparaissent donc sous-évalués dans ce chiffre.
- **Risque** — des régressions sur du code non couvert passent inaperçues ; impossible de prioriser les efforts de test.
- **Action corrective** — intégrer `cargo-tarpaulin` à la CI avec publication du rapport et définition d'un seuil minimal, en commençant par les crates `domain` et `application` (logique métier) ; objectif ≥ 70 % sur la logique métier.
- **Statut** — en cours (outil installé, première mesure réalisée : 41,14 % ; intégration CI à finaliser).

### 3.2 Tests frontend légers

- **Constat** — le frontend ne comptait qu'un seul fichier de tests, `frontend/src/lib/api.test.ts`, avec 3 cas portant uniquement sur `mapBackendConditionalIntent` (intent Lend avec condition, Swap sans condition, fallback Composite).
- **Risque** — les composants, hooks (`useParseIntent`, `useOnboarding`) et le parsing NLU→API n'étaient pas couverts ; le bogue BUG-07 (onboarding/mapping) a justement été détecté en recette manuelle, pas par un test.
- **Action corrective** — le harnais a été renforcé : **28 cas vitest** (contre 3) couvrant désormais le mapping complet des réponses API (`api.test.ts` : cas limites, erreurs HTTP, 204), la normalisation des statuts (`status.test.ts`), la logique pure de délégation (`delegation.test.ts` : split de signature, construction du message, packing des protocoles) et le composant `Stepper` avec ses attributs d'accessibilité (`Stepper.test.tsx` : `aria-current="step"`). À poursuivre avec des tests de hooks (React Testing Library).
- **Statut** — traité pour les utilitaires et le stepper (28/28 verts, `npm test`) ; tests de hooks prévus à la prochaine itération.

### 3.3 TLS géré hors dépôt

- **Constat** — aucune configuration TLS n'est versionnée : `frontend/nginx.conf` écoute en HTTP clair sur le port 3000, et `docker-compose.yml` n'expose pas de terminaison TLS. La mise en HTTPS repose sur l'infrastructure d'accueil (reverse proxy externe).
- **Risque** — comportement de production non reproductible ni auditable depuis le dépôt ; risque de déploiement en clair si le proxy externe est absent ou mal configuré.
- **Action corrective** — documenter l'exigence de terminaison TLS dans le guide de déploiement et fournir une configuration de reverse proxy de référence (ex. Traefik/Caddy) versionnée.
- **Statut** — traité pour la partie dépôt : une configuration TLS de référence est versionnée dans `deploy/Caddyfile` (Caddy, certificats automatiques via Let's Encrypt, frontend sur le port 3000, routage de `/api/*` vers l'API sur le port 3001, headers de sécurité de base ; `/health`, `/ready` et `/metrics` non exposés) et documentée dans `DEPLOYMENT.md` (section « Terminaison TLS »). La terminaison TLS reste volontairement hors `docker-compose.yml` : le déploiement effectif du proxy relève de l'infrastructure d'accueil.

### 3.4 Secret JWT aléatoire en développement

- **Constat** — lorsque `OTTER_JWT_SECRET` est absent, l'API génère un secret aléatoire à chaque démarrage (`crates/interfaces/src/bin/otter_api.rs`, avec avertissement « auth enabled but no OTTER_JWT_SECRET set; generating a random dev secret »).
- **Risque** — en développement, tous les tokens sont invalidés à chaque redémarrage (sessions perdues, faux échecs en recette) ; en production, un oubli de configuration passerait pour un système « qui marche » tout en étant non déterministe.
- **Action corrective** — conserver le comportement aléatoire uniquement en dev (déjà le cas, avec warning), et faire échouer le démarrage si le secret est absent lorsque `OTTER_NETWORK` vaut `mainnet`/`sepolia` ; documenter la variable dans `.env.example`.
- **Statut** — traité. La fonction `resolve_jwt_secret` (`crates/interfaces/src/bin/otter_api.rs`) refuse le démarrage (`std::process::exit(1)` avec message d'erreur explicite) si l'auth est activée sans `OTTER_JWT_SECRET` lorsque `OTTER_NETWORK` vaut `mainnet` ou `sepolia` ; le secret aléatoire + warning reste valable en local/dev. La variable est documentée dans `.env.example`. Preuve automatisée : tests `jwt_secret_required_on_public_networks` et `jwt_secret_random_allowed_locally`.

---

## 4. Synthèse

### 4.1 Volume et criticité (registre de la section 2)

| Criticité | Nombre | Résolus |
|---|---|---|
| Bloquant | 2 | 2 |
| Majeur | 8 | 8 |
| Mineur | 1 | 1 |
| Cosmétique | 1 | 1 |
| **Total** | **12** | **12** |

- **Taux de résolution : 100 %** des bogues qualifiés dans ce registre (12/12 mergés sur `main`, CI verte).
- L'historique complet compte une vingtaine de commits `fix(*)` ; le registre retient les 12 plus représentatifs par criticité et par stack (Solidity, Rust, Noir, frontend, CI/CD).

### 4.2 Enseignements et engagement qualité

- La majorité des bogues Majeur provient de la chaîne CI/DevOps et de la sécurité (contrôle d'accès, secrets) : les passes de revue structurées (findings Critical/Important) ont été le canal de détection le plus productif, avant même la mise en production.
- Chaque correction est adossée à un contrôle automatisé rejoué en continu (clippy, `cargo test`, `forge test`, `nargo test`, vitest, smoke tests), ce qui transforme chaque bogue traité en barrière de non-régression durable.
- Les points d'amélioration de la section 3 sont tracés avec un statut explicite ; leur traitement est intégré à la feuille de route (couverture tarpaulin en CI, tests de hooks frontend). Le durcissement du secret JWT en production (§3.4) est traité. Le point TLS (§3.3) est traité pour la partie dépôt : configuration de référence versionnée dans `deploy/Caddyfile`.

---

## 5. Anomalies issues de la recette

Le cahier de recettes (`docs/CAHIER_DE_RECETTES.md` §6.3) a consigné 7 anomalies détectées pendant la recette. Elles sont reprises ici pour qualification et suivi, conformément au workflow de la section 1.

| Réf. | Description (abrégée) | Criticité | Traitement / analyse du point d'amélioration | Statut |
|---|---|---|---|---|
| A1 | `POST /api/v1/intents/parse` n'applique pas `validate_intent_text` (texte > 2000 caractères accepté, contrairement à `POST /api/v1/intents`) | Mineur | `validate_intent_text` (trim, rejet vide, limite 2000 caractères) appelée au début de `parse_intent`, comme dans `create_intent` ; tests `parse_intent_rejects_long_text` et `parse_intent_rejects_empty_text` (HTTP 400, même code que sur `/intents`) | **Résolu** |
| A2 | `/agents`, `/strategies`, `/leaderboard`, `/proofs` retournent des données de démonstration embarquées (`default_agents()`, `default_strategies()`) | Mineur | Étiquetage explicite ajouté : header `X-Demo-Data: true` (middleware `demo_data_header` appliqué au groupe de routes démo) et champ `demo: true` dans les réponses JSON ; badge « Demo data » affiché sur les pages Agents/Strategies/Proofs quand l'API signale la donnée de démonstration (hooks `useAgents`/`useStrategies`/`useProofs`/`useLeaderboard`, composant `DemoDataNotice`). Tests : `demo_endpoints_set_x_demo_data_header_and_flag` et `non_demo_endpoints_do_not_set_x_demo_data_header` (`crates/interfaces/src/bin/otter_api.rs`). Le remplacement par des données on-chain/persistées reste tracé ici — retrait obligatoire avant mainnet | Étiquetage traité ; remplacement tracé (retrait obligatoire avant mainnet) — test `demo_endpoints_set_x_demo_data_header_and_flag` |
| A3 | Parsing LLM : modèle local llama.cpp/GGUF au lieu de l'API Claude visée par US-028/029/030 ; `HybridParser` non branché sur l'API (qui instancie `RegexParser` seul) | Mineur | Choix assumé (LLM local = confidentialité, hors-ligne) ; `build_orchestrator` instancie désormais le `HybridParser` (fallback LLM → regex) via `build_intent_parser` lorsqu'un modèle GGUF est présent et chargeable, sinon le `RegexParser` seul — le démarrage sans modèle reste fonctionnel (test `build_intent_parser_falls_back_to_regex_without_model`). Adaptations : `LlmIntentParser` passe de `RefCell` à `Mutex` (Send + Sync requis par l'état Axum) et le port `IntentParserPort` est implémenté pour `Arc<T>` | **Résolu** |
| A4 | Rate limiting **par adresse IP** alors que US-422 spécifie « 100 req/min **par user** » | Mineur | Clé de comptage étendue : `sub` du JWT quand un token valide est présent (header `Authorization: Bearer`), IP sinon ; le chemin non authentifié est inchangé ; test `rate_limit_scoped_per_user` (comptage différencié par utilisateur depuis la même IP) | **Résolu** |
| A5 | `_execute` ETH natif débite le solde interne sans transfert sortant (compatibilité ascendante) ; seul le flux ERC-20 transfère vers le routeur | Majeur | Documenté comme limitation du flux ETH natif ; implémenter le transfert sortant ETH ou restreindre l'exécution aux ERC-20 en production | À traiter avant mainnet |
| A6 | `docs/PLAN_CORRECTION_BOGUES.md` référencé par le cahier n'existait pas à la date de la recette | Mineur | Document créé (le présent fichier), qui trace A1–A7 | **Résolu** |
| A7 | Vérification SIWE de bout en bout (signature réelle d'un wallet) non couverte par un test automatisé | Mineur | Test d'intégration `siwe_end_to_end_real_signature` : challenge `POST /auth/challenge`, signature EIP-191 réelle avec la clé de test Anvil #0 via `k256`, `POST /auth/verify`, JWT émis dont le `sub` est l'adresse du signataire, appel protégé accepté ; test unitaire `verify_signature_accepts_real_eip191_signature` dans `auth.rs`. Corrections rendues nécessaires par le test : vérification de signature synchrone (`verify_eip191` au lieu d'un `block_on` imbriqué qui paniquait dans le runtime async) et `sub` = adresse EIP-55 au lieu du debug du tableau d'octets | **Résolu** |

Aucune anomalie Bloquante n'a été détectée pendant la recette ; la chaîne critique (authentification, délégation signée, preuve ZK, anti-rejeu) reste intégralement vérifiée par les tests automatisés.

---

*Document rédigé à partir de l'historique git du dépôt (`git log`, `git show`) et de la configuration CI effective — aucune entrée n'est fictive.*
