# Bloc 3 — Préparation aux échanges (15 min de Q&A)

> Éléments de réponse **honnêtes**, alignés sur l'angle figé le 2026-08-31 :
> dev 100 % solo assumé ; acteurs extérieurs = testeurs et validateurs de la
> demande produit (mentor crypto, dev ZK, dev Solidity, dev opensource,
> communautés Discord/IA) ; délégation IA assumée comme délégation outillée.
> Ne jamais bluffer : un jury de professionnels détecte une équipe fictive
> en deux questions.

## Questions pièges (à travailler en priorité)

**Q1. « Votre projet est solo. Où est le pilotage d'équipe ? »**
- Ne pas nier : « Le développement est entièrement solo, je l'assume et je le
  dis dans le support. Mais le projet n'est pas une tour d'ivoire : des
  testeurs réels — un dev ZK, un dev Solidity, un dev opensource, un mentor
  crypto et des communautés Discord — ont validé la demande produit et
  exécuté de la recette d'usage. Mon pilotage d'acteurs porte sur la
  validation : affectation des tests par compétence, animation de bénévoles,
  collecte et traitement des retours. »
- « Et le pilotage de projet existe : 481 user stories priorisées, 207 issues
  suivies, 34 PRs mergées sous protection de branche, deux releases datées,
  des arbitrages de périmètre documentés. »
- Délégation IA, si questionnée : « de la délégation outillée — brief écrit,
  contrat de sortie, vérification systématique par la CI. Je délègue
  l'exécution, jamais la validation. »

**Q2. « Indicateurs de satisfaction… avec zéro utilisateur ? »**
- « Pas d'utilisateurs en production — dit franchement. Mais la demande
  produit a été validée par de vraies personnes : testeurs des communautés
  crypto/IA, dont des devs ZK et Solidity, qui ont testé et confirmé le
  besoin. Mes indicateurs : leurs retours d'usage, la recette BC02 soldée,
  zéro régression CI, une démo reproductible < 3 min. »
- Ne jamais inventer un taux de satisfaction.

**Q3. « Montrez la grille d'évaluation des compétences remplie. »**
- L'avoir **imprimée et dans le support** (slide 13). Répondre en la
  montrant, pas en la décrivant. Préciser qui a été évalué (moi en
  auto-évaluation + testeurs sur leurs missions) et ce que le plan de
  développement a changé : le creux fév.–juin = montée en compétences
  Rust/ZK, mesurable dans les commits de juillet.

**Q4. « Un arbitrage qui a coûté quelque chose ? »**
- FHE + mempool chiffré : ~40 user stories coupées (visibles dans les
  issues), soit plusieurs semaines de cadrage perdues — mais le MVP est
  sorti à date. Coût : périmètre différenciant abandonné, dette de doc
  (issues encore ouvertes, assumé slide 16).
- Ou : livrer la vague du 24–31/08 avec une couverture à 55,92 % sous
  l'objectif de 70 % — coût : dette de tests tracée dans
  `PLAN_CORRECTION_BOGUES.md`.

**Q5. « Que feriez-vous différemment ? »**
- Trois réponses courtes : ① synchroniser le backlog markdown et les issues
  GitHub dès le cut FHE (le delta est encore visible) ; ② intégrer tarpaulin
  à la CI plus tôt (la couverture n'a été mesurée qu'en juillet) ; ③ impliquer
  les testeurs plus tôt dans le cycle — ils sont arrivés sur un produit déjà
  avancé, leurs retours auraient plus pesé en amont.

## Questions probables — méthodologie & planning (C3.1)

**Q6. Pourquoi itératif en vagues plutôt que Scrum / cycle en V ?**
- Incertitude technique forte (ZKP) → incréments testables ; solo → pas de
  cérémonies d'équipe, mais rituels adaptés (releases datées, revues de
  dette, dossiers de bloc comme points de validation formels).

**Q7. Comment le planning a-t-il été construit puis tenu ?**
- Découpage en 10 vagues estimées en semaines (`BACKLOG.md`), ordonnancement
  par dépendances (archi → parsing → ZK → on-chain → UI → prod), jalons =
  releases. Tenu ? Montrer le Gantt réel vs prévisionnel : creux fév.–juin
  assumé, re-planification en sprint juillet.

**Q8. Quelles ressources, quel budget ?**
- 1 développeur, 1 machine, CI GitHub (gratuite en repo public), tooling
  100 % open source, services externes gratuits (anvil, devnet Solana,
  relay Flashbots testnet). Coût ≈ temps seul. Dire le nombre d'heures si
  demandé — estimé, présenté comme tel.

**Q9. Qui est responsable de quoi ? (RACI)**
- Matrice réelle (slide 7) : moi R/A sur tout le développement — assumé ;
  testeurs R sur la recette d'usage, C sur la validation produit ; mentor et
  communautés en C/I. La montrer, ne pas la réciter.

## Questions probables — pilotage (C3.2.1)

**Q10. Montrez votre outil de suivi.**
- Ouvrir GitHub en live (onglet déjà prêt) : issues `[EPIC-x.y]`/`[US-nnn]`,
  PRs mergées, checks CI, protection de branche. Fallback : captures
  horodatées.

**Q11. Quels indicateurs suivez-vous, à quelle fréquence ?**
- Délais (jalons vs réel), qualité (tests, clippy, couverture datée),
  coûts (temps, minutes CI), vélocité (commits/semaine — montrer le tableau
  114 commits). Fréquence : à chaque release et à chaque vague ; les
  mesures du support datent du 31/08.

**Q12. Exemple de suivi qui a déclenché une action ?**
- Échecs CI visibles dans l'historique des runs → fixes `de28a51`,
  `b9b8d83` (juillet). Retour BC04 → remediation `fb12f58` + annexe de
  preuves horodatées (19/08). Couverture 41,14 % → campagnes de tests →
  55,92 % (mesures datées dans le dossier BC02).

**Q13. Votre définition de "terminé" ?**
- Tests + clippy `-D warnings` + CI verte + PR (protection de branche : 3
  checks requis) + doc quand la feature est exposée (`docs/`).

## Questions probables — management & compétences (items 6–10)

**Q14. Comment affectez-vous les missions ?**
- « Par compétence et disponibilité — mes testeurs sont bénévoles, on ne
  peut pas ordonner. Le dev ZK teste la chaîne de preuve, le dev Solidity
  les contrats, le testeur opensource la recette d'usage de bout en bout.
  Chaque mission = un objectif écrit et un livrable attendu (retour
  structuré). » Si angle IA questionné : même discipline avec les agents —
  brief écrit, contrat de sortie, vérification par les tests.

**Q15. Quel(s) style(s) managérial(aux) ? Donnez un exemple par style.**
- Tableau slide 12 : participatif (besoins collectés auprès du mentor et des
  communautés, périmètre MVP orienté par leurs retours) ; persuasif
  (convaincre des bénévoles de donner du temps — démo, docs, repo public) ;
  délégatif (recette confiée au testeur avec accès total ; agents IA sous
  contrôle CI) ; directif (CUT FHE, correctifs sécurité non négociables).

**Q16. Quels outils de communication, avec quels objectifs ?**
- Discord (communautés — objectif vitesse : avis rapides, recrutement de
  testeurs) ; GitHub issues/PRs (objectif traçabilité — toute décision
  aboutit en ticket ou PR) ; docs/ et dossiers de bloc (objectif
  transmission et preuves horodatées).

**Q17. Comment évaluez-vous puis développez les compétences ?**
- Grille par compétence et par acteur (slide 13) + plan daté ; preuve
  d'exécution : ma montée en compétences ZK/Rust du premier semestre 2026 se
  lit dans la reprise de vélocité de juillet (37 commits W28) et le circuit
  réel vérifié on-chain en août.

## Questions techniques probables (hors compétences évaluées, mais crédibilité)

**Q18. La preuve ZK est-elle réelle ou simulée ?**
- Réelle : circuit Noir compilé, preuve Barretenberg/UltraHonk générée et
  vérifiée on-chain dans `scripts/demo.sh` (rejoué le 31/08, 27 s). Le
  contrat vérifie avant d'exécuter ; les 8 reverts forge prouvent les
  garde-fous. Les seules données simulées sont marquées `X-Demo-Data` dans
  l'API — choix assumé et tracé (anomalie A2).

**Q19. Pourquoi Solana et un bridge dans un projet Ethereum ?**
- Multi-chaîne = cœur du produit (attestations cross-chain de la
  solvabilité, liquidité fragmentée). V1 volontairement minimale, feature-
  gated, documentée.

**Q20. Quels risques et comment les gérez-vous ?**
- Techniques : temps de génération de preuve (mesuré, alerte > 30 s),
  sécurité (docs/SECURITE.md, cargo-audit daté, RBAC), dépendances
  (dependabot actif). Projet : risque solo (bus factor = 1, assumé) → doc
  exhaustive + repo public + scripts reproductibles.
