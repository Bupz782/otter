# Bloc 3 — Checklist J-1 (à exécuter le dimanche 14 septembre 2026)

> À exécuter la veille, dans l'ordre. Chaque case coche une action réelle.

## 1. Versions épinglées

- [ ] `git fetch && git status` : `main` propre, à jour avec `origin/main`.
- [ ] Noter le SHA de référence : `git rev-parse --short HEAD` → le citer si le jury demande la version.
- [ ] Tags présents : `git tag -l` → `v0.1.0`, `v0.1.1`.
- [ ] Toolchains : `anvil --version`, `forge --version`, `nargo --version`, `~/.bb/bb --version`, `cargo --version` (noter les versions sur une fiche).
- [ ] Fixtures du circuit présentes : `ls delegation_circuit/target/*.json` (sinon relancer la génération — `scripts/demo.sh` refuse de démarrer sans elles, c'est voulu).

## 2. Re-mesurer tous les chiffres du support (arithmétique exacte)

- [ ] `git rev-list --count HEAD` → slide 4 et 9.
- [ ] Tableau hebdo : `git rev-list HEAD --format='%cd' --date=format:'%Y-W%V' --no-commit-header | sort | uniq -c` → vérifier que la somme = total.
- [ ] `gh api "search/issues?q=repo:Bupz782/otter+type:issue" --jq .total_count` (+ `state:closed`).
- [ ] `gh api "search/issues?q=repo:Bupz782/otter+type:pr+is:merged" --jq .total_count`.
- [ ] `cargo test --workspace` → nombre de tests (282 au 31/08).
- [ ] `cd contracts && forge test` → 28/28.
- [ ] `cd frontend && npm run typecheck`.
- [ ] Mettre à jour `00_INVENTAIRE_PREUVES.md` si un chiffre a bougé, puis les slides qui les citent.

## 3. Répétition chronométrée

- [ ] `time bash scripts/demo.sh` → noter le temps (27 s au 31/08, cache chaud ; prévoir la marge machine froide).
- [ ] **Enregistrer le plan B** : capture vidéo du run `demo.sh` + captures horodatées des pages UI (dashboard, intents, solvency, bridge, mev, solana) + sortie `forge test`.
- [ ] Répéter la présentation complète une fois, minuteur en main : 23 min slides + 7 min démo.
- [ ] Relire `04_QUESTIONS_JURY.md` à voix haute (Q1–Q5 au minimum).

## 4. Matériel et ordre des fenêtres

- [ ] Charger le laptop à 100 % + chargeur. Couper notifications, fermer apps inutiles.
- [ ] Ordre des fenêtres : ① slides plein écran · ② navigateur `localhost:3000` (wallet déverrouillé) · ③ onglet GitHub issues/PRs (plan Q10) · ④ terminal repo racine · ⑤ terminal `scripts/`.
- [ ] Imprimés en secours : scénario démo, sortie `demo.sh` horodatée, grille de compétences (slide 13), tableau de l'inventaire des preuves.
- [ ] Clé USB : export PDF du support + vidéo plan B + repo cloné (`.git` inclus).
- [ ] Adapter HDMI/USBC testé sur place si possible.

## 5. Matin J (T-30)

- [ ] Docker Desktop lancé ; `docker compose up --build -d` ; `docker compose ps` → 3 services up.
- [ ] `curl -s localhost:3001/health` OK ; `curl -s -X POST localhost:8545 … eth_chainId` → `0x7a69`.
- [ ] Navigateur sur `localhost:3000`, wallet connecté (compte anvil #0).
- [ ] Vidéo plan B lisible (tester la lecture 10 s).
- [ ] Respirer. Le pilotage est le sujet ; la démo est la preuve.

## 6. Interdits (retours BC04)

- [ ] Aucun chiffre cité de mémoire : chacun vient de l'inventaire re-mesuré la veille.
- [ ] Aucune équipe fictive : dev solo assumé, testeurs cités par rôle réel (validation produit / recette), jamais comme co-développeurs.
- [ ] Aucune improvisation dans la démo : le scénario est suivi à la lettre, plans B/C assumés fièrement s'ils servent (« un plan de repli, c'est aussi du pilotage »).
