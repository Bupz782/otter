# Manuel de mise à jour — Projet Otter

Ce document décrit la politique de versionnement et les procédures de mise à
jour du projet Otter (backend Rust/Axum, frontend React, contrats
Solidity/Foundry, circuit Noir, déploiement Docker + GitHub Actions), pour
chaque environnement : développement local, serveur Docker, testnet Sepolia et
mainnet. Toutes les commandes et procédures décrites ici sont celles
effectivement présentes dans le dépôt.

---

## 1. Politique de versionnement

### 1.1 Tags semver

Les versions livrables du projet sont marquées par des **tags Git au format
`v*`** (par exemple `v1.0.0`, `v1.2.3`). C'est le push d'un tel tag qui
déclenche la chaîne de publication complète :

- `.github/workflows/docker.yml` se déclenche sur les pushes de `main` et
  `develop`, sur les tags `v*` et sur les pull requests. Il extrait les
  étiquettes d'image avec `docker/metadata-action@v5` selon trois règles :
  - `type=ref,event=branch` — étiquette portant le nom de la branche ;
  - `type=semver,pattern={{version}}` — étiquette portant la version semver
    extraite du tag ;
  - `type=sha,prefix=,suffix=,format=short` — étiquette portant le SHA court
    du commit.
- `.github/workflows/deploy-testnet.yml` se déclenche exclusivement sur les
  tags `v*` (ou manuellement via `workflow_dispatch`) et déploie ces images
  sur l'hôte testnet.

Les images sont publiées sur le GitHub Container Registry (GHCR) sous :

```
ghcr.io/<propriétaire>/otter/api:<étiquette>
ghcr.io/<propriétaire>/otter/frontend:<étiquette>
```

Chaque image est donc traçable par **trois étiquettes** : version semver
(`v1.2.3`), nom de branche (`main`, `develop`) et SHA court du commit. Cela
permet à tout moment de savoir exactement quel code tourne sur un
environnement.

Les versions des outils ZK embarqués dans l'image backend sont lues depuis
`.noir-version` et `.bb-version` à la racine du dépôt et passées en arguments
de build (`NOIR_VERSION`, `BB_VERSION`), ce qui garantit la reproductibilité
du vérificateur de preuves entre les versions.

### 1.2 Conventional Commits

L'historique du dépôt suit la convention **Conventional Commits**, comme le
montre `git log --oneline` :

```
f4ecb98 fix(debt): emit missing metrics and enable Prometheus alerts
0aa2439 feat(secrets): add AWS KMS and Vault secret providers
c54cab6 ci(testnet): add post-deploy smoke tests
2cb3222 chore(observability): extend Prometheus alerting rules
```

Préfixes utilisés : `feat`, `fix`, `ci`, `chore`, `merge`, avec une portée
entre parenthèses (`ci`, `deploy-mainnet`, `secrets`, `debt`…). Cette
convention facilite la rédaction des notes de version et le choix du numéro de
version (incrément *patch* pour `fix`, *minor* pour `feat`, *major* en cas de
rupture de compatibilité).

### 1.3 Création d'une version

```bash
git tag v1.2.3
git push origin v1.2.3
```

Le push du tag déclenche automatiquement le build/push des images et le
déploiement testnet (voir §2.3).

---

## 2. Procédure de mise à jour par environnement

### 2.1 Développement local

La mise à jour d'un poste de développement consiste à récupérer le code et à
relancer la pile locale orchestrée par `scripts/dev.sh` :

```bash
git pull

# Backend Rust (toolchain nightly, installée par dev-setup.sh)
cargo build

# Frontend React
cd frontend && npm ci && cd ..

# Lancement de la pile complète
DEPLOYER_PRIVATE_KEY=0x... ./scripts/dev.sh
```

`scripts/dev.sh` (fichier `scripts/dev.sh`) effectue dans l'ordre :

1. démarrage d'**Anvil** (chaîne locale, avec fork optionnel si
   `SEPOLIA_RPC_URL` et `FORK_BLOCK` sont définis) ;
2. démarrage de **PostgreSQL** via `docker compose up postgres -d` ;
3. **déploiement des contrats** sur Anvil avec
   `forge script contracts/script/DeployDelegationVault.s.sol --broadcast` et
   capture de l'adresse du vault ;
4. génération de `.env.local` (racine) et `frontend/.env.local` avec toutes
   les variables `OTTER_*` locales ;
5. démarrage du frontend Vite (`npm run dev`) ;
6. démarrage de l'API avec `cargo watch -x "run --bin otter_api"` (rechargement
   automatique ; repli sur `cargo run --bin otter_api` si `cargo-watch` est
   absent).

> **Prérequis outillage** : si les outils ne sont pas installés ou si les
> versions ont changé (Rust nightly, Foundry, Noir, Barretenberg, Node ≥ 20,
> `cargo-watch`), relancer `scripts/dev-setup.sh`. Ce script lit les versions
> attendues dans `.noir-version` et `.bb-version` et installe les dépendances
> frontend (`npm install`).

### 2.2 Docker / serveur

Sur un serveur exploité avec Docker Compose (fichier `docker-compose.yml`),
les images sont désignées par les variables `OTTER_API_IMAGE` et
`OTTER_FRONTEND_IMAGE`. La mise à jour consiste à pointer vers la nouvelle
étiquette puis à tirer et recréer les conteneurs :

```bash
export OTTER_API_IMAGE=ghcr.io/<propriétaire>/otter/api:v1.2.3
export OTTER_FRONTEND_IMAGE=ghcr.io/<propriétaire>/otter/frontend:v1.2.3
docker compose pull
docker compose up -d --remove-orphans
docker compose ps
```

La pile comprend trois services : `postgres` (PostgreSQL 15, volume
`otter-postgres-data`), `api` (port 3001, healthcheck sur
`curl -f http://localhost:3001/ready`, volume `otter-data` pour le nonce
persisté) et `frontend` (port 3000). Le service `api` ne démarre que lorsque
`postgres` est *healthy*, et le `frontend` que lorsque l'`api` l'est. Cette
procédure est celle décrite dans `DEPLOYMENT.md` (section 7) et utilisée par
les workflows de déploiement.

### 2.3 Testnet (automatisé)

Le déploiement testnet est entièrement automatisé par
`.github/workflows/deploy-testnet.yml`. Il suffit de **créer et pousser un tag
`v*`** (ou de lancer le workflow manuellement depuis l'onglet Actions) :

```bash
git tag v1.2.3
git push origin v1.2.3
```

Le workflow exécute alors la séquence suivante :

1. **Job `build-and-push`** :
   - checkout du tag, conversion du nom du dépôt en minuscules ;
   - connexion à GHCR avec le `GITHUB_TOKEN` ;
   - lecture de `.noir-version` et `.bb-version` (passées en build args) ;
   - build et push de `ghcr.io/<repo>/api:<tag>` (Dockerfile racine) et
     `ghcr.io/<repo>/frontend:<tag>` (`frontend/Dockerfile`), avec cache
     `type=gha`.
2. **Job `deploy`** (environnement GitHub `testnet`, dépend du build) :
   - connexion SSH à l'hôte testnet via `appleboy/ssh-action@v1.0.3`
     (secrets `TESTNET_HOST`, `TESTNET_USER`, `TESTNET_SSH_KEY`) ;
   - dans `/opt/otter`, export de `OTTER_API_IMAGE` et
     `OTTER_FRONTEND_IMAGE` pointant vers le nouveau tag, puis
     `docker compose pull`, `docker compose up -d --remove-orphans` et
     `docker compose ps` ;
   - exécution des **smoke tests** sur l'hôte avec
     `OTTER_API_URL=http://localhost:3001 ./scripts/smoke-test.sh`.

Si une étape échoue (build, SSH ou smoke test), le pipeline s'arrête en
erreur et la version n'est pas considérée comme déployée.

### 2.4 Mainnet (manuel, avec validation multisig)

Le déploiement mainnet est volontairement **manuel** : le workflow
`.github/workflows/deploy-mainnet.yml` ne se déclenche que par
`workflow_dispatch` (bouton « Run workflow » dans GitHub Actions) et exige
deux paramètres :

- **`tag`** : le tag Git à déployer ;
- **`multisig_tx_hash`** : le hash de la transaction de déploiement
  **multisig** des contrats, qui atteste que le déploiement on-chain a été
  approuvé par les signataires requis.

Séquence du workflow (environnement GitHub `mainnet`) :

1. **Job `check-mainnet-readiness`** : checkout du tag demandé et affichage de
   la checklist de préparation (vérifier le déploiement testnet, les rapports
   d'audit et la simulation sur fork avant de continuer).
2. **Job `deploy`** : connexion SSH à l'hôte mainnet
   (`MAINNET_HOST`/`MAINNET_USER`/`MAINNET_SSH_KEY`), puis dans
   `/opt/otter-mainnet` :

   ```bash
   export OTTER_TAG=<tag>
   export OTTER_API_IMAGE=ghcr.io/<repo>/api:<tag>
   export OTTER_FRONTEND_IMAGE=ghcr.io/<repo>/frontend:<tag>
   docker compose pull
   docker compose up -d
   ./scripts/smoke-test.sh
   ```

**Checklist préalable au déploiement mainnet :**

- [ ] Le tag a été déployé et validé sur testnet (smoke tests verts).
- [ ] Les contrats ont été redéployés sur mainnet via transaction multisig et
      le hash est disponible.
- [ ] Les audits de sécurité sont à jour.
- [ ] Une simulation sur fork a été exécutée avec la nouvelle version.
- [ ] Les adresses des contrats mainnet sont renseignées dans la
      configuration de l'hôte (`OTTER_VAULT_ADDRESS`).

---

## 3. Migrations de base de données

Le schéma PostgreSQL est versionné par des fichiers SQL numérotés dans
`crates/infrastructure/migrations/` :

```
0001_init.sql
0002_indexes.sql
0003_add_user_address.sql
```

**Application automatique au démarrage.** Lors de la création du stockage
PostgreSQL (`PgStorage::new`, fichier
`crates/infrastructure/src/storage/postgres.rs`), la fonction
`run_migrations` s'exécute dans une transaction unique :

1. création de la table de suivi `schema_migrations (version INTEGER PRIMARY
   KEY, applied_at INTEGER NOT NULL)` si elle n'existe pas ;
2. énumération des fichiers `.sql` du répertoire de migrations, triés
   lexicographiquement ;
3. pour chaque fichier dont le numéro de version (préfixe numérique du nom,
   par exemple `0001`) n'est pas encore enregistré dans `schema_migrations` :
   exécution du SQL puis insertion de la version avec le timestamp Unix
   courant ;
4. commit de la transaction.

Le répertoire des migrations est résolu par `migrations_dir()` dans
`crates/infrastructure/src/storage/migrations.rs` : la variable
d'environnement **`OTTER_MIGRATIONS_DIR`** est prioritaire (utile dans les
conteneurs), sinon le chemin est dérivé de `CARGO_MANIFEST_DIR` à la
compilation.

**Règles pour ajouter une migration lors d'une mise à jour :**

- créer un nouveau fichier `NNNN_descriptif.sql` avec le numéro suivant
  (`0004_...sql`), sans jamais modifier un fichier déjà appliqué ;
- la migration s'appliquera automatiquement au prochain démarrage de l'API,
  sans action manuelle ;
- les migrations doivent être idempotentes autant que possible et écrites
  pour préserver les données existantes (les volumes `otter-postgres-data`
  persistent entre les versions).

**Vérification de l'état des migrations :**

```bash
docker compose exec postgres \
  psql -U otter -d otter -c "SELECT version, applied_at FROM schema_migrations ORDER BY version;"
```

---

## 4. Mise à jour des contrats

### 4.1 Redéploiement

Le script `contracts/script/DeployDelegationVault.s.sol` déploie les deux
contrats dans l'ordre : `DelegationVerifier`, puis `DelegationVault` en lui
passant l'adresse du vérificateur. Il affiche les deux adresses déployées
(`DelegationVerifier deployed at: ...` / `DelegationVault deployed at: ...`).

Sur Sepolia (procédure de `DEPLOYMENT.md`) :

```bash
cd contracts
forge script script/DeployDelegationVault.s.sol \
  --rpc-url $SEPOLIA_RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --verify \
  --etherscan-api-key $ETHERSCAN_API_KEY
```

Sur mainnet, le déploiement passe par une **transaction multisig** dont le
hash est ensuite fourni au workflow de déploiement mainnet (§2.4).

### 4.2 Mise à jour de la configuration

Après chaque redéploiement, reporter la nouvelle adresse du vault :

- `OTTER_VAULT_ADDRESS=0x...` dans le `.env` de l'hôte (modèle :
  `.env.example`) — c'est cette variable que `docker-compose.yml` injecte
  dans le service `api` ;
- `vault_address = "0x..."` dans `config.toml` pour une exécution hors Docker
  (modèle : `config.example.toml`).

### 4.3 Régénération du vérificateur après changement de circuit

Toute modification du circuit Noir change la clé de vérification et donc le
contrat vérificateur : **les adresses précédentes deviennent obsolètes** (cas
réel documenté dans `DEPLOYMENT.md` après l'ajout de `target_contract` au
circuit). Dans ce cas la mise à jour impose :

1. de recompiler le circuit avec la version de `nargo` fixée dans
   `.noir-version` ;
2. de redéployer `DelegationVerifier` **et** `DelegationVault` via le script
   Foundry (§4.1) ;
3. de mettre à jour `OTTER_VAULT_ADDRESS` partout où elle est configurée ;
4. de vérifier que les versions `nargo`/`bb` de l'image Docker (build args
   `NOIR_VERSION`/`BB_VERSION`) correspondent à celles ayant servi à compiler
   le circuit, sans quoi la génération de preuve échouera.

Le script `lab/zkp_e2e.sh` permet de valider la chaîne complète en local
(déploiement → dépôt → génération de preuve → exécution on-chain →
vérification) avant tout redéploiement sur un réseau public.

---

## 5. Rollback

### 5.1 Retour à l'image précédente

Chaque image étant étiquetée par version semver, le retour arrière consiste à
repointer les variables d'environnement vers le tag précédent et à recréer
les conteneurs :

```bash
export OTTER_API_IMAGE=ghcr.io/<propriétaire>/otter/api:v1.2.2
export OTTER_FRONTEND_IMAGE=ghcr.io/<propriétaire>/otter/frontend:v1.2.2
docker compose pull
docker compose up -d --remove-orphans
```

Sur testnet, le rollback s'effectue en rejouant manuellement les commandes
SSH ci-dessus sur l'hôte (`/opt/otter`) avec le tag précédent, puis en
relançant `OTTER_API_URL=http://localhost:3001 ./scripts/smoke-test.sh`.

### 5.2 Réversibilité des migrations

Le mécanisme de migrations (§3) est **à sens unique** : il n'existe pas de
migration descendante (`down`) dans le dépôt. Les migrations appliquées
restent donc en place après un rollback applicatif. Pour garantir la
réversibilité :

- écrire les migrations de façon **additive** (`CREATE TABLE IF NOT EXISTS`,
  nouvelles colonnes nullable) pour que l'ancienne version de l'API continue
  de fonctionner sur le schéma nouveau — c'est le cas des migrations
  existantes (`0002_indexes.sql` ne fait qu'ajouter des index,
  `0003_add_user_address.sql` ajoute une colonne) ;
- avant une migration **destructive**, prendre un dump de la base :

  ```bash
  docker compose exec postgres pg_dump -U otter otter > backup-$(date +%Y%m%d).sql
  ```

  et, en cas de retour arrière nécessitant l'ancien schéma, restaurer :

  ```bash
  cat backup-YYYYMMDD.sql | docker compose exec -T postgres psql -U otter -d otter
  ```

- supprimer la ligne correspondante de `schema_migrations` uniquement si la
  migration a été annulée manuellement et doit être rejouée.

Les données applicatives hors base (fichier de nonce `OTTER_NONCE_STORE_PATH`)
sont persistées dans le volume `otter-data` et survivent au rollback.

### 5.3 Contrats

Les contrats déployés ne sont pas mutables : un « rollback » de contrat
consiste à **remettre l'ancienne adresse** dans `OTTER_VAULT_ADDRESS` (les
anciens contrats restent déployés on-chain), après avoir vérifié que la clé
de vérification du circuit courant est compatible avec ce vérificateur.

---

## 6. Vérification post-mise-à-jour

### 6.1 Smoke tests

Le script `scripts/smoke-test.sh` est la vérification de référence après
toute mise à jour (il est exécuté automatiquement par les workflows testnet
et mainnet) :

```bash
export OTTER_API_URL=http://localhost:3001   # valeur par défaut
./scripts/smoke-test.sh
```

Il vérifie successivement :

1. **`GET /ready`** — interrogé jusqu'à 30 s ; un `503` est retenté pendant
   le préchauffage, tout autre code inattendu fait échouer le test ;
2. **`GET /api/v1/health`** — liveness de l'API (alias de `/health`) ;
3. **`POST /api/v1/intents/parse`** — parsing fonctionnel d'une intention de
   bout en bout (`{"text":"lend 100 USDC on Aave if yield > 1"}`).

Le script se termine par `Smoke tests passed` et un code de sortie nul en cas
de succès.

### 6.2 Endpoints de santé et métriques

L'API (`crates/interfaces/src/bin/otter_api.rs`) expose :

| Endpoint | Rôle |
|---|---|
| `/health` et `/api/v1/health` | liveness |
| `/health/live` | liveness détaillée |
| `/ready` | readiness (utilisé par le healthcheck Docker) |
| `/metrics` | métriques Prometheus (si `OTTER_METRICS_ENABLED=true`) |

```bash
curl -fsS http://localhost:3001/ready
curl -fsS http://localhost:3001/api/v1/health
curl -fsS http://localhost:3001/metrics
```

### 6.3 Supervision Prometheus

Le fichier `alerting.yml` contient les règles d'alerte à charger dans
Prometheus (`rule_files`). Après une mise à jour, surveiller en priorité :

- `OtterAgentDown` — API injoignable plus d'une minute (critique) ;
- `OtterHighErrorRate` — taux d'erreur > 0,1/s sur 5 min ;
- `OtterExecutionStalled` — conditions remplies mais aucune exécution on-chain ;
- `OtterNoPriceUpdates` — supervision de prix arrêtée ;
- `OtterProofVerificationFailing` — échecs de vérification de preuve on-chain
  (critique, typiquement le symptôme d'un vérificateur non redéployé après
  changement de circuit) ;
- `OtterRpcUnhealthy` — endpoint RPC défaillant ;
- `OtterAgentLowBalance` et `OtterProofPipelineSlow` — alertes de seuil.

La complétion de `docker compose ps` (services *healthy*) et l'absence
d'alerte critique pendant les minutes suivant la mise à jour valident le
déploiement.

---

## 7. Checklist finale de mise à jour

- [ ] Numéro de version choisi selon Conventional Commits (`fix` → patch,
      `feat` → minor, rupture → major).
- [ ] Tag `vX.Y.Z` créé et poussé (`git tag vX.Y.Z && git push origin vX.Y.Z`).
- [ ] Pipeline `docker.yml` vert : images semver/branche/sha publiées sur GHCR.
- [ ] Pipeline `deploy-testnet.yml` vert : déploiement SSH effectué et
      `scripts/smoke-test.sh` passé sur l'hôte testnet.
- [ ] Nouvelles migrations SQL (si présentes) numérotées à la suite, additives
      et vérifiées dans `schema_migrations` après redémarrage.
- [ ] Contrats redéployés si le circuit ou les contrats ont changé, adresse
      `OTTER_VAULT_ADDRESS` mise à jour dans `.env` / `config.toml`.
- [ ] Endpoints `/ready`, `/api/v1/health` et `/metrics` vérifiés manuellement.
- [ ] Aucune alerte critique (`OtterAgentDown`,
      `OtterProofVerificationFailing`, `OtterExecutionStalled`) après la
      fenêtre de supervision.
- [ ] Pour mainnet : workflow `deploy-mainnet.yml` lancé manuellement avec le
      tag et le hash de transaction multisig, checklist de readiness validée.
- [ ] Plan de rollback prêt : tag d'image précédent identifié, dump SQL pris
      avant toute migration destructive.
