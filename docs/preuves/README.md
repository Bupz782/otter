# Preuves d'execution — bloc 4 (exploitation, maintenance, supervision)

Sorties de commande reelles, horodatees, produites le 2026-08-19. Chaque
preuve est posterieure au correctif qu'elle demontre (chronologie
verifiable via les timestamps des fichiers et les runs CI).

## 1. Audit des dependances (C4.1.1)

- `cargo-audit-20260819.log` — `cargo audit` brut sur `Cargo.lock` (597
  dependances) : 8 occurrences / 7 identifiants uniques. Traitement le jour
  meme : `h2` 0.4.15 -> 0.4.16, `ruint` 1.18.0 -> 1.20.0 (corriges),
  `h2` 0.3.27 ignore (feature `aws-kms` jamais activee, pas de correctif
  0.3.x). Apres traitement : 0 vulnerabilite hors ignores justifies.
  Detail dans `docs/SECURITE.md`, section A06.
- Le job `security-audit` de `.github/workflows/ci.yml` est bloquant et
  vert (voir run CI ci-dessous).

## 2. Supervision et signalement (C4.1.2)

- `alerte-test-20260819-094726.json` — payload webhook recu lors du test
  de bout en bout `scripts/test-alert.sh` : alerte `OtterAgentDown`
  injectee dans Alertmanager (API v2), groupee (group_wait 10s), POSTee au
  webhook et horodatee a la reception (2026-08-19T07:47:37Z). Demontre que
  le canal de signalement est configure et fonctionnel, pas seulement
  decrit.
- Regles validees par `promtool check rules` (8 regles, succes).
- Stack : profil `monitoring` de `docker-compose.yml` (Prometheus scrape
  `api:3001/metrics` et evalue `alerting.yml`, Alertmanager notifie via
  `ALERT_WEBHOOK_URL`). Mise en route : `DEPLOYMENT.md`, section 5.

## 3. Correctif deploye via CI/CD (C4.2.2)

- PR #260 (https://github.com/Bupz782/otter/pull/260) : CI complete verte
  (`rust-check`, `security-audit`, `docker-smoke`, `build-and-push`),
  fusionnee le 2026-08-19.
- Tag `v0.1.1` pose le 2026-08-19 sur le commit de fusion
  (`git tag -l` : v0.1.0, v0.1.1).
- Pipeline `Deploy Testnet` sur le tag : build + publication des images
  `ghcr.io/bupz782/otter/api:v0.1.1` et `frontend:v0.1.1` ; le job deploy
  SSH est saute explicitement (notice) tant que les secrets `TESTNET_*`
  ne sont pas provisionnes — voir `deploy-tag-v0.1.1.txt`.
- Le job `docker-smoke` de la CI reconstruit les images et execute le
  smoke test HTTP (`/health`, `/ready`) contre la version taggee : voir
  `ci-pr-260.txt`.

## 4. Journal des versions (C4.3.2)

- `CHANGELOG.md` a la racine, regenere par `scripts/generate-changelog.sh`
  depuis les Conventional Commits : sections `[v0.1.1]` (2026-08-19) et
  `[v0.1.0]` (2026-07-20).
