# Changelog

Journal des versions d'Otter, genere depuis l'historique git
(Conventional Commits) par `scripts/generate-changelog.sh`.

## [Unreleased]

### Features

- feat(supervision): stack prometheus + alertmanager avec canal de notification eprouve (9878a40)

### Fixes

- fix(ci): installer nargo dans rust-check, corriger h2 0.4.16 et ruint 1.20.0 (0782a3d)
- fix: durcissement securite, accessibilite et etiquetage demo avant rendu (bae724d)
- fix: traiter les points residuels du registre (R1, R3-R6) (f2858b7)

### Chores

- chore: retirer les notes de travail hors perimetre du rendu (PROMPT_PPT, CHANGELOG de notes infra) (2dda8d3)

## [v0.1.0] - 2026-07-20

### Features

- feat: SocialFi strategies — user-created, forkable intent templates (cfbf746)
- feat(secrets): add AWS KMS and Vault secret providers (0aa2439)
- feat(docker): self-contained backend image with nargo and bb (330840d)
- feat(docker): add frontend Dockerfile and nginx config (b2be0de)
- feat(orchestrator/api): async tasks, finer-grained locking, validation, and user scoping (#237) (300fa64)
- feat(zkp-verifier-sync): unblock ERC-20 end-to-end execution (#236) (18cbdb1)
- feat: frontend tooling, CI expansion and Docker tooling docs (#235) (cf73c3f)
- feat(frontend): onboarding fixes + parseIntent API mapping (#234) (23a778c)
- feat: wire frontend hooks to real backend API and implement signed delegation E2E (21e1755)
- feat(US-41 & US-42): calculate gas & detect imposibble plan (#230) (1d52bf7)
- feat(014): add benchmark.sh (62a89db)
- feat(013): add fmt to ci (94bd629)
- feat(012): add clippy to ci (17402c9)
- feat(010): add learning.md (c79f0fa)
- feat(009): add loggers (#221) (a78b7cf)
- feat(008): add orchestrator (#219) (0c876fc)
- feat(007): add ports (22893fc)
- feat(007): add ports (594b3c4)
- feat(007): add ports (9a320c3)
- feat(007): add ports (1bfad26)
- feat(007): add ports (0cab64a)
- feat(006): add folders for archi hexa (4c78d03)
- feat(005): add labs folders (5d7fe5f)
- feat(004): add cargo base & archi hexa (887cbe6)
- feat(003): add foundry dep (2a49a73)
- feat(002): add nargo dep (d243cc8)
- feat(001): add rust tool chain + setup rust (1b93b8a)

### Fixes

- fix(debt): emit missing metrics and enable Prometheus alerts (f4ecb98)
- fix(debt): pin Rust toolchain, Debian tool stages, versioned install scripts, wire KMS/Vault config (b9b8d83)
- fix(debt): stabilize CI/DevOps debt — migrations, clippy, zkp tests, .noir-version (de28a51)
- fix(ci): lowercase GHCR repo name via step output (881a418)
- fix(ci-devops): Important review findings pass 7 (114feb9)
- fix: final critical/important CI/devops review findings (feb03f5)
- fix(ci): add /api/v1/health alias and pass Noir/bb versions in docker.yml (30506dd)
- fix(ci-devops): address Important review findings (9ca4724)
- fix(ci/devops): address final review Critical and Important issues (c67af35)
- fix(ci/devops): address critical and important review findings (03db1e3)
- fix(ci/devops): address final review findings for CI/DevOps plan (d2fbc78)
- fix(deploy-mainnet): add set -euo pipefail and remove TODO placeholder (51e9ae9)
- fix(secrets): runtime safety, error logging, tests for KMS/Vault providers (174c315)
- fix(lint): fmt check (c4afa6f)
- fix(syntax): redundance declaration (0af6335)

### Chores

- chore(observability): extend Prometheus alerting rules (2cb3222)
- chore(dev): add setup and dev launch scripts (03e616c)
- chore(db): consolidate migrations and add schema_migrations table (bd3588b)
- chore(tooling): pin bb version and add justfile (4bc1747)
- chore(git): ignore isolated worktrees directory (d24f2a4)

### Other

- refactor: renommer metis en otter partout, retirer les emojis des docs (1ea66b7)
- docs(bc02): dossier de certification, manuels, recettes, securite, accessibilite, couverture (382e826)
- ci(mainnet): add manual gated mainnet deployment workflow (85e23b2)
- ci(testnet): add post-deploy smoke tests (c54cab6)
- ci: unified pipeline with rust, forge, nargo, frontend and docker smoke (2d3d445)
- Merge develop into main (#238) (bf9f58f)
- docs: add implementation plan for CI/dev env/testnet/mainnet (5a84ac1)
- docs: add CI/dev env/testnet/mainnet strategy design spec (b28e932)
- Epic 1.6/strategy planner (#229) (202e912)
- all ticket in one commit bad epic (#228) (bc909d8)
- Epic 0.2 - Epic 1.1 - Epic 1.2 - Epic 1.3 (#227) (58e9c50)
- Vague 1/epic 1.3 amount error handling (#226) (45892bb)
- Vague-1/epic 1.2 parser (#225) (91db547)
- epic1.1 (#224) (965a695)
- Epic 0.4 & Epic 0.5 (992deec)
- epic 0.2 & epic 0.3 (#220) (b821244)
- Initial commit (c7605d6)

