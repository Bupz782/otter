# Bloc 3 — Inventaire des preuves (collecte du 2026-08-31)

> Chaque affirmation du support de présentation doit pointer vers une ligne de
> ce tableau. Rien d'inventé : chaque preuve est un artefact réel, daté, et
> régénérable avec la commande indiquée. Retour BC04 appliqué : preuves
> horodatées partout, arithmétique exacte, échecs assumés.

## 1. Dépôt et activité

| Fait | Valeur vérifiée | Commande / source |
|---|---|---|
| Dépôt | https://github.com/Bupz782/otter (public) | `gh repo view` |
| Création du dépôt GitHub | 2025-11-14 | `gh api repos/Bupz782/otter --jq .created_at` |
| Dernier push | 2026-08-31 | idem, `.pushed_at` |
| Commits sur `main` | **114** | `git rev-list --count HEAD` |
| Auteurs | 5 identités git, **une seule personne** (Arthur Bertier — emails/machine différents) | `git shortlog -sne HEAD` |
| Tags / releases | `v0.1.0` (2026-07-20), `v0.1.1` (2026-08-19) | `git tag -l --format='%(refname:short) %(creatordate:short)'` |
| Protection de branche | `main` protégée : PR obligatoire + 3 status checks requis (contourné une fois par l'admin le 2026-08-31, visible dans la sortie de push) | sortie `git push` du 2026-08-31 |

### Activité hebdomadaire (commits par semaine, dates de commit)

| Semaine | Commits | Phase |
|---|---|---|
| 2025-W46 | 21 | Lancement (repo créé le 14/11) |
| 2025-W47 | 12 | Fondations |
| 2025-W49 | 2 | Fondations |
| 2026-W04 | 3 | Pointillés (période études) |
| 2026-W06 | 1 | Pointillés |
| 2026-W07 | 2 | Pointillés |
| 2026-W28 | 37 | Sprint BC02 |
| 2026-W30 | 7 | Release v0.1.0 + rendu BC02 (23/07) |
| 2026-W34 | 3 | BC04 remediation → v0.1.1 (19/08) |
| 2026-W35 | 25 | Vague avancée (auth, solvency, bridge, MEV, solana) |
| 2026-W36 | 1 | Scheduler solana (31/08) |
| **Total** | **114** | cohérent avec `git rev-list --count HEAD` |

Commande : `git rev-list HEAD --format='%cd' --date=format:'%Y-W%V' --no-commit-header | sort | uniq -c`

Lecture honnête pour le jury : démarrage intense en nov. 2025, **creux
février–juin 2026 assumé** (montée en compétences en parallèle), reprise en
sprint en juillet pour le BC02, puis vague de fonctionnalités avancées fin
août. Le Gantt rétrospectif du support reprend ces phases.

## 2. Suivi de projet (C3.2.1)

| Fait | Valeur vérifiée | Source |
|---|---|---|
| Backlog produit | `BACKLOG.md` — **481 user stories** (lignes contenant `US-`), 10 vagues, statuts FAIT/EN COURS/EN ATTENTE/CUT/FUTUR, tags MVP/Future/Cut | `grep -c 'US-' BACKLOG.md` |
| Suivi des tickets | **207 issues GitHub** (57 fermées, 150 ouvertes) au 2026-08-31, nommage `[EPIC-x.y]` / `[US-nnn]` | `gh api "search/issues?q=repo:Bupz782/otter+type:issue"` |
| Synchronisation backlog ↔ issues | `ISSUES.md` auto-généré depuis `BACKLOG.md` par `scripts/sync-issues.sh` | en-tête de `ISSUES.md` |
| Pull requests | **56 PRs**, dont **34 mergées** (une partie = dependabot) ; PRs de fond : #219–#238 (vagues 0–1), #251 (durcissement), #260 (remediation BC04), #261 (fix vault), #262 (cleanup), #263 (debt) | `gh api "search/issues?q=...+type:pr+is:merged"` |
| CI/CD | Workflows GitHub Actions : CI (rust/forge/nargo/frontend), Docker Build & Push, déploiement testnet/mainnet ; **échecs CI réels** visibles dans l'historique des runs (ex. run 33072617190 FAIL) puis corrigés | `gh run list` |
| Changelog | `CHANGELOG.md` généré depuis Conventional Commits (`scripts/generate-changelog.sh`) | repo |
| Versions | tags signés par date (v0.1.0, v0.1.1) + release notes dans CHANGELOG | repo |

**Delta assumé (matière pour "comptes rendus sur évolutions et
améliorations")** : les issues GitHub contiennent encore le périmètre FHE
(US-139…US-168) marqué CUT dans `ISSUES.md` ; les pourcentages de progression
de `BACKLOG.md` sont conservateurs par rapport au code réellement livré (la
délégation ZKP fonctionne alors que la vague 2 y est notée ~5 %). La source de
vérité opérationnelle est l'historique git + CI, pas les pourcentages du
backlog. C'est un point d'amélioration identifié, pas un fait caché.

## 3. Qualité logicielle (mesures du 2026-08-31, sauf mention contraire)

| Indicateur | Valeur | Commande de reproduction |
|---|---|---|
| Tests Rust | **282 passed** (18 suites) | `cargo test --workspace` |
| Lints | clippy clean | `cargo clippy --workspace --all-targets -- -D warnings` |
| Tests contrats | **28/28** (6 suites) | `cd contracts && forge test` |
| Typage frontend | vert | `cd frontend && npm run typecheck` |
| Couverture Rust | **55,92 %** (1951/3489 lignes, `--workspace --lib`), logique critique 91–100 % — mesure du **2026-07-20**, tarpaulin 0.34.1 | `DOSSIER_BC02.md` §couverture, `tarpaulin.toml` |
| Démo end-to-end | **27 s** mesurées le 2026-08-31 (machine chaude, cible affichée < 180 s) | `time bash scripts/demo.sh` |
| Preuves horodatées BC04 | `docs/preuves/` : alerte Alertmanager (2026-08-19T07:47:37Z), `cargo-audit` (2026-08-19), checks CI PR #260 (2026-08-19T09:13:07Z), déploiement tag v0.1.1 (2026-08-19T09:22:15Z) | fichiers dans `docs/preuves/` |

## 4. Documentation existante (réutilisable dans le support)

- `DOSSIER_BC02.md` — dossier de certification BC02 (rendu 2026-07-23) : structure critère ↔ preuve à réutiliser.
- `docs/preuves/README.md` — annexe de preuves d'exécution horodatées (bloc 4, 2026-08-19).
- `BACKLOG.md` / `ISSUES.md` — planification et suivi.
- `CHANGELOG.md` — historique de versions.
- `docs/` : SECURITE, ACCESSIBILITE, CAHIER_DE_RECETTES, MANUEL_UTILISATION, MANUEL_MISE_A_JOUR, OPS, BRIDGE, MEV_SEARCHER, SOLANA, PLAN_CORRECTION_BOGUES.
- `DEPLOYMENT.md`, `docker-compose.yml`, `Dockerfile`, `alerting.yml`, `deploy/prometheus.yml`.

## 5. Retours BC04 (août 2026) — règles appliquées à ce dossier

1. Preuves horodatées partout (chaque mesure ci-dessus porte sa date).
2. Arithmétique exacte des chiffres cités (114 commits = somme du tableau
   hebdomadaire ; 481 US ; 207/57/150 issues ; 56/34 PRs — tous régénérables).
3. Assumer les échecs plutôt que les cacher (creux d'activité fév.–juin,
   échecs CI, delta backlog/GitHub, couverture 55,92 % < objectif 70 % sur le
   périmètre global).
