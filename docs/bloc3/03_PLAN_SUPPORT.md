# Bloc 3 — Plan du support (≈17 slides, 30 min)

> Règles : 30 min = 23 min de slides + 7 min de démo (éliminatoire, non
> raccourcissable). Chaque compétence éliminatoire a un bloc identifiable.
> Chaque slide cite sa preuve (voir `00_INVENTAIRE_PREUVES.md`).
>
> **Angle figé (réponses du 2026-08-31)** : développement **100 % solo
> assumé**. Les acteurs extérieurs réels — mentor (commu crypto), dev ZK,
> dev Solidity, dev opensource crypto, communautés Discord/IA — sont
> intervenus comme **testeurs et validateurs de la demande produit** (retours
> d'usage, confirmation du besoin), jamais comme co-développeurs. Pas de
> vécu de pilotage externe mobilisable. La délégation à des sous-agents IA
> est **assumée** comme délégation outillée (brief écrit, contrat de sortie,
> vérification CI). Évaluation le **15 septembre 2026**.

Minutage cible : slides 1–13 = 21:30 · démo = 7:00 · slides 15–17 = 1:30 → **30:00**.
Options de coupe si dérive (jamais dans C3.1 / C3.2.1 / démo) : slide 2 à 1:00, slide 11 à 1:00, slide 16 fusionnée avec 17.

| # | Titre | Min | Je montre | Je dis (trame) |
|---|---|---|---|---|
| 1 | Otter — pilotage d'un projet DeFi trustless | 0:30 | Titre, nom, date, repo | Une phrase : « intents en langage naturel → délégation ZKP → exécution on-chain ». Annonce du plan. |
| 2 | Le logiciel en 90 secondes | 1:30 | Schéma chaîne de valeur (DOSSIER_BC02 §1.2) + arborescence repo | Ce que fait le produit, pour qui ; périmètre MVP vs CUT. Prépare la démo. |
| **C3.1 — PLANIFIER (slides 3–7, 9:30)** | | | | |
| 3 | Cadre méthodologique | 2:00 | Tableau des 10 vagues (`BACKLOG.md`), extrait de statuts US | Itératif/incrémental en vagues, chaque vague livre un incrément testable ; pourquoi pas de cycle en V (incertitude ZK) ni de Scrum formel (solo, mais rituels adaptés : releases, revues de dette). DoD concret : tests + clippy + CI verte + PR. |
| 4 | Planning rétrospectif (Gantt réel) | 2:30 | Gantt reconstruit depuis git (§ ci-dessous) | Jalons : création repo 14/11/2025 → v0.1.0 20/07/2026 (BC02 rendu 23/07) → v0.1.1 19/08 (BC04) → vague avancée 24–31/08. **Creux fév.–juin assumé** et expliqué (montée en compétences), reprise planifiée en sprint. |
| 5 | Planning prévisionnel | 1:30 | Table des vagues restantes + jalons datés | Suite : fin vague 2/5/6 (API → mainnet), points de validation datés (item 12), charge estimée en semaines, marge. |
| 6 | Ressources nécessaires | 1:30 | Tableau ressources : humaines, matérielles, logicielles, coûts | 1 dev lead (moi) + contributeurs bénévoles ponctuels (mentor, dev ZK, dev Sol, testeur opensource — slide 7) ; machine de dev + CI GitHub, tooling 100 % open source (Rust/Noir/Foundry), coûts ≈ 0 € hors temps ; services externes : RPC, relay Flashbots, devnet Solana. |
| 7 | Acteurs, responsabilités, coordination | 2:00 | Matrice RACI réelle (ci-dessous) | Qui fait quoi : moi R/A sur tout, contributeurs C ou R ponctuels par domaine, communautés Discord/IA en I. Coordination : Discord (asynchrone, avis rapides), GitHub issues/PRs (traçable, mémoire), docs/ (transmission). Solo assumé, contributeurs nommés par rôle — pas d'équipe fictive. |
| **C3.2.1 — PILOTER (slides 8–10, 5:30)** | | | | |
| 8 | Outil de suivi — montré en vrai | 2:00 | **Écran live** : issues GitHub (207, 57 fermées), PRs (34 mergées), board, `ISSUES.md` | Backlog → issues synchronisées (`sync-issues.sh`) ; workflow PR + protection de branche (3 checks requis) ; dependabot. |
| 9 | Indicateurs de pilotage | 2:00 | Tableau chiffré (slide = photo du 31/08) | Délais : jalons vs dates réelles. Qualité : 282 tests Rust, 28 forge, clippy clean, couverture 55,92 % (20/07) — objectif 70 % non atteint, **assumé + plan**. Coûts : temps, CI. Vélocité : 114 commits, pics W28/W35. Lead time PR. |
| 10 | Suivi régulier & communication | 1:30 | `CHANGELOG.md`, releases v0.1.0/v0.1.1, `docs/preuves/` | Rituels : changelog à chaque release, annexe de preuves horodatées par bloc, dossiers BC02/BC04 comme comptes rendus formels (item 11 amorcé ici). |
| 11 | Cas d'arbitrage (item 5) | 1:30 | 2 arbitrages chiffrés | ① FHE + mempool chiffré **CUT** (périmètre vs délai — issues encore visibles) ; ② couverture 55,92 % : choix de livrer la vague fonctionnelle plutôt que de bloquer sur 70 %, avec dette tracée dans `PLAN_CORRECTION_BOGUES.md`. Ce que chaque arbitrage a **coûté**. |
| 12 | Management : missions & styles (items 6–8) | 2:00 | Tableau style → exemple réel (ci-dessous) | Affectation par compétence et disponibilité (bénévoles → on ne peut pas ordonner) ; 4 styles mobilisés avec exemples ; délégation IA assumée et encadrée (brief écrit, vérification CI — « je délègue l'exécution, jamais la validation »). |
| 13 | Compétences (items 9–10) | 1:30 | Grille d'évaluation remplie + plan de développement (ci-dessous) | Grille par acteur et par domaine ; plan de développement exécuté (mon creux fév.–juin = montée en compétences Rust/ZK, mesurable dans les commits) et en cours pour les contributeurs (docs comme support). |
| **C3.4.2 — DÉMO (slide 14, 7:00, non raccourcissable)** | | | | |
| 14 | **Démonstration live** | 7:00 | Scénario `02_SCENARIO_DEMO.md` | Annoncée comme répondant à la liste de référence F1–F9 (slide ouverte sur le tableau). Plan B/C prêts. |
| 15 | Points de validation & satisfaction (items 12–13) | 1:00 | Frise des points de validation passés/futurs | Points de validation : revues de vague, gates CI, releases, évals BC02/BC04, Bloc 3 le 15/09. Satisfaction : retours réels des testeurs communautaires (dev opensource + Discord) + proxies assumés (recette BC02 soldée, zéro régression CI, démo < 3 min) — pas d'utilisateurs en production, dit franchement. |
| 16 | Évolutions & améliorations (item 11) | 0:30 | 3 comptes rendus datés | Ex. remediation BC04 (#260, preuves horodatées) ; delta backlog/GitHub FHE identifié ; couverture < cible tracée. |
| 17 | Bilan & questions | 0:30 | Une phrase + repo QR/URL | Ce que je ferais différemment (amorce les échanges). |

## Gantt rétrospectif (matière slide 4 — données git exactes)

```
2025-W46 █████████████████████ 21  Lancement, archi hexa, CI          (repo créé 14/11/2025)
2025-W47 ████████████ 12            Fondations vagues 0–1
2025-W49 ██ 2                       Fondations
2026-W04 ███ 3                      Pointillés (plan de compétences)
2026-W06 █ 1
2026-W07 ██ 2
   …      ░░░░░░░░░░                Creux assumé fév.–juin 2026
2026-W28 █████████████████████████████████████ 37  Sprint BC02
2026-W30 ███████ 7                  v0.1.0 (20/07) · rendu BC02 (23/07)
2026-W34 ███ 3                      Remediation BC04 → v0.1.1 (19/08)
2026-W35 █████████████████████████ 25  Auth SIWE · solvency ZK · bridge · MEV V1/V2 · solana
2026-W36 █ 1                        Scheduler solana (31/08)
Total : 114 commits — régénérable : git rev-list --count HEAD
```

Jalons : ● 14/11/2025 création repo · ● 20/07/2026 v0.1.0 · ● 23/07/2026 BC02
· ● 19/08/2026 v0.1.1 + BC04 · ● 31/08/2026 vague avancée · ○ **15/09/2026
Bloc 3** · ○ mainnet (prévisionnel).

## Matrice RACI réelle (matière slide 7)

Développement : moi seul, sur tout. Acteurs extérieurs : testeurs et
validateurs de la demande produit. C'est la réalité, et elle se défend :
la coordination porte sur la **validation**, pas sur la production de code.

| Domaine | Moi (lead) | Mentor crypto | Dev ZK / Dev Sol | Testeur opensource | Communautés Discord/IA |
|---|---|---|---|---|---|
| Circuit Noir / preuves | R, A | I | | C (test) | I |
| Contrats (Vault, Bridge, Registry) | R, A | I | | C (test) | I |
| Backend Rust / API | R, A | I | | C (test) | I |
| Frontend | R, A | I | | C (test) | I |
| Recette / tests d'usage | R, A | C | C | R | C |
| Validation de la demande produit | A | C | C | C | C |

Lecture pour le jury : R = réalise, A = redevable, C = consulté, I = informé.
Les testeurs sont bénévoles : l'affectation des missions de validation se
fait par compétence (un dev ZK teste la partie ZK) ET disponibilité, jamais
par obligation — d'où la centralité du style persuasif (slide 12).

## Styles managériaux mobilisés (matière slide 12 — 1 exemple daté par style)

| Style | Où / quand | Exemple concret |
|---|---|---|
| Participatif | Définition et validation produit | Besoins et retours d'usage collectés auprès du mentor et des communautés (le produit répond à une demande qu'ils ont confirmée) ; leurs retours orientent le périmètre MVP. |
| Persuasif | Testeurs bénévoles | Pas de hiérarchie ni de budget : obtenir du temps de test suppose de convaincre de l'intérêt du projet (démo reproductible, docs, repo public comme arguments). |
| Délégatif | Recette + sous-agents IA | Exécution des tests d'usage confiée au testeur opensource avec accès total ; tâches outillées déléguées à des agents IA avec brief écrit et contrat de sortie — **vérification systématique par la CI** (un agent peut se tromper, les 282 tests tranchent). |
| Directif | Sécurité et périmètre | CUT du FHE sans discussion (délai) ; correctifs de sécurité du vault (expiration ancrée au block.timestamp, `8ed546b`) appliqués comme non négociables. |

Outils de communication (item 8) : **Discord** (communautés — avis rapides,
recrutement de relecteurs ; objectif : vitesse) · **GitHub issues/PRs**
(objectif : traçabilité et mémoire — toute décision aboutit en ticket ou PR)
· **`docs/` + dossiers de bloc** (objectif : transmission et preuves
horodatées).

## Grille d'évaluation des compétences (matière slide 13 — item 9)

Échelle : 1 initié · 2 autonome · 3 référent (peut en référer d'autres).
Auto-évaluation (moi, sur le développement) + niveaux des testeurs déduits
de leurs missions réelles ; **à valider en relisant — c'est toi qui les
connais**.

| Acteur | Rust/backend | Solidity | ZK/Noir | DevOps/CI | Recette | Produit/DeFi |
|---|---|---|---|---|---|---|
| Moi (lead, seul dev) | 2 | 2 | 1 → 2 (plan exécuté) | 2 | 2 | 2 |
| Dev ZK (testeur) | | | 3 | | 2 | 2 |
| Dev Solidity (testeur) | | 3 | | | 2 | 2 |
| Testeur opensource | 1 | 1 | 1 | | 2 | 2 |
| Mentor crypto | | 2 | 2 | | | 3 |

Écarts identifiés → plan de développement (item 10) :
- **Moi, ZK/Noir 1 → 2** : plan exécuté fév.–juin 2026 (montée en compétences
  visible dans le creux de commits), preuve : circuit réel livré et vérifié
  on-chain en août. Prochain échelon : solvency avancée, lecture d'audits.
- **Testeur opensource, ZK 1 → 2** : montée en compétence sur la recette ZK
  via `docs/MANUEL_UTILISATION.md` + scénarios de `CAHIER_DE_RECETTES.md`.
- **Tous les testeurs** : les dossiers de bloc (BC02/BC04/Bloc 3) servent de
  supports de transmission — documenter pour transmettre, pas pour décorer.

## Planning prévisionnel (matière slide 5 — jalons datés)

| Quand | Jalon |
|---|---|
| 01–14/09/2026 | Finition vague avancée, répétitions, gel du code (checklist J-1) |
| **15/09/2026** | **Évaluation Bloc 3** |
| Oct. 2026 | Vagues 5–6 restantes (adaptateurs protocoles, orchestrateur complet) |
| Nov. 2026 | Vague 7 (durcissement prod, couverture → 70 %, audit externe) |
| T1 2027 | Testnet public, premiers utilisateurs réels (indicateurs satisfaction réels) |
| (Futur) | Mainnet — gate : audit + couverture + recette |

## Notes de fabrication du support

- Support sobre (slide = 1 idée + 1 preuve), aucune capture non horodatée.
- Les chiffres de la slide 9 sont figés au 31/08 et régénérés la veille par
  les commandes de `00_INVENTAIRE_PREUVES.md` — si un chiffre change, le
  changer partout (retour BC04 : arithmétique exacte).
- Prévoir le Gantt en image (le bloc ASCII ci-dessus peut être refondu en
  diagramme propre) + QR code du repo en dernière slide.
