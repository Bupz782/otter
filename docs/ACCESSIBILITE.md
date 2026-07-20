# Accessibilité — Projet Otter

Ce document présente le référentiel d'accessibilité retenu pour le prototype Otter
(frontend React 18 + Vite + TypeScript + Tailwind CSS 4, dossier `frontend/`), justifie
ce choix, puis audite l'existant thématique par thématique avec des preuves issues du
code source. Il se termine par un plan d'amélioration honnête : ce qui n'est pas encore
couvert est explicitement listé, sans survente.

---

## 1. Choix du référentiel

### 1.1 Référentiel principal : RGAA 4.1

Le **RGAA (Référentiel Général d'Amélioration de l'Accessibilité), version 4.1**, a été
retenu comme référentiel de conformité. Justification :

- **Contexte légal français.** Le RGAA est le référentiel opposable en France, adossé à
  l'article 47 de la loi du 11 février 2005 et au décret n° 2019-768 du 24 juillet 2019,
  qui transpose la directive (UE) 2016/2102. Otter étant un service numérique développé
  dans un cadre français, c'est le référentiel de droit applicable — et le seul qui donne
  une grille d'audit normalisée utilisable en France (106 critères, méthodologie de test
  documentée).
- **Applicable aux services numériques.** Le RGAA s'applique aux sites web, applications
  web et applications mobiles. Une interface DeFi comme Otter — application web monopage
  avec formulaires, navigation et contenu dynamique — entre pleinement dans son
  périmètre technique.
- **Adossement international.** Le RGAA 4.1 reprend les **WCAG 2.1 niveaux A et AA**,
  ce qui garantit la compatibilité avec les pratiques internationales et facilite
  l'utilisation des outils d'audit du marché (axe DevTools, WAVE, NVDA, VoiceOver).

### 1.2 Complément : Opquast

En complément, les **bonnes pratiques Opquast (checklist « Qualité Web », 240 règles)**
sont utilisées comme référentiel transverse non opposable : elles couvrent ce que le
RGAA ne couvre pas ou peu (écoconception, contenus, sécurité, expérience utilisateur
générale, internationalisation). Opquast ne se substitue pas au RGAA ; il sert de garde-fou
pour les sujets de qualité périphériques (par exemple : titres de page explicites,
liens compréhensibles hors contexte, pas de piège au clavier).

### 1.3 Les 13 thématiques du RGAA

Le RGAA 4.1 organise ses 106 critères en 13 thématiques :

1. **Images** — alternatives textuelles, images décoratives ignorées, images légendées, CAPTCHA.
2. **Cadres** — titre de chaque cadre (iframe).
3. **Couleurs** — l'information ne doit pas être donnée uniquement par la couleur ; contrastes suffisants.
4. **Multimédia** — transcription, sous-titrage, audiodescription pour les médias temporels.
5. **Tableaux** — tableaux de données correctement structurés (en-têtes, titres, résumés).
6. **Liens** — liens explicites, intitulés compréhensibles.
7. **Scripts** — composants compatibles avec les technologies d'assistance, contrôlables au clavier.
8. **Éléments obligatoires** — doctype, langue, titre de page, métadonnées.
9. **Structuration de l'information** — titres hiérarchisés, listes, citations, regroupements.
10. **Présentation de l'information** — contenu restituable sans CSS, pas de balises de présentation, lisibilité.
11. **Formulaires** — étiquettes, regroupements, aide à la saisie, gestion des erreurs, contrôle de saisie.
12. **Navigation** — zones d'en-tête/navigation/pied, liens d'évitement, ordre de tabulation, plan du site.
13. **Consultation** — documents à télécharger accessibles, contenus en mouvement maîtrisables, pas d'ouverture de fenêtre sans avertissement.

---

## 2. Audit du prototype par thématique

Méthode : revue statique du code source du frontend (`frontend/src/`), complétée par un
calcul des ratios de contraste des couleurs principales du design system. Il ne s'agit
**pas** d'un audit RGAA complet (qui exige des tests avec technologies d'assistance sur
un échantillon de pages) mais d'une **auto-évaluation par inspection de code**, à faire
valider par un audit formel.

Statistique d'ensemble : **108 occurrences** d'attributs `aria-*` / `role` réparties sur
33 fichiers de `frontend/src/` (mesure par recherche textuelle).

### 2.1 Images

**Conforme (partiel).** L'interface ne contient **aucune balise `<img>`** : les visuels
sont des icônes SVG (lucide-react) et un canvas WebGL décoratif.

- Les icônes décoratives sont systématiquement masquées aux technologies d'assistance
  avec `aria-hidden="true"` : `frontend/src/components/ProtocolStack.tsx:72`,
  `frontend/src/components/MetricsBanner.tsx:14`, `frontend/src/components/Footer.tsx:34`,
  `frontend/src/components/TrustSection.tsx:75`, etc.
- Le canvas WebGL décoratif est explicitement ignoré :
  `frontend/src/components/WebGLSpiral.tsx:151` (`aria-hidden="true"`).
- Les icônes porteuses de sens sont accompagnées d'un libellé accessible sur le contrôle
  parent : `aria-label="Close"` (`frontend/src/components/app/WelcomeModal.tsx:52`),
  `aria-label={copied ? "Copied" : "Copy transaction hash"}`
  (`frontend/src/pages/app/IntentDetailPage.tsx:217`).

*Limite : pas de `<img>` donc le critère « alternative d'image » n'a pas de cas d'école
à tester ; si des images porteuses d'information sont ajoutées, le critère devra être
réévalué.*

### 2.2 Cadres

**Non applicable.** Aucun `<iframe>` dans le code source du frontend.

### 2.3 Couleurs

**Largement conforme, à valider exhaustivement.** Le design system est un thème sombre
dont les couleurs sont centralisées dans `frontend/src/styles/tokens.css`. Les ratios
de contraste ont été calculés (formule WCAG) à partir des valeurs hexadécimales du fichier :

| Paire texte / fond | Ratio | Seuil RGAA (AA, texte courant 4,5:1) |
|---|---|---|
| `#f4f4f5` (foreground) sur `#050505` (background) | 18,54:1 | Oui |
| `#a1a1aa` (muted-foreground) sur `#050505` | 7,95:1 | Oui |
| `#a1a1aa` sur `#0c0c0c` (card) | 7,63:1 | Oui |
| `#c8a46c` (accent) sur `#050505` | 8,72:1 | Oui |
| `#fb7185` (erreurs, `text-rose-400`) sur `#050505` | 7,57:1 | Oui |
| `#34d399` (succès, `text-emerald-400`) sur `#050505` | 10,60:1 | Oui |
| `#71717a` (gris secondaire) sur `#050505` | **4,22:1** | sous le seuil pour texte courant (OK pour grand texte, seuil 3:1) |

Mapping des tokens : `frontend/src/index.css:9-38` (variables `--color-*` du thème
Tailwind 4 pointant vers les primitives Otter).

L'information n'est pas véhiculée par la couleur seule : les erreurs de formulaire sont
des textes explicites en plus de la couleur rose (`role="alert"`, cf. §2.11), et les
états de copie affichent un texte (« Copied ») en plus du changement de couleur
(`frontend/src/pages/app/IntentDetailPage.tsx:225`).

*Limite : le calcul couvre les couleurs principales, pas chaque combinaison de classes
utilisée (opacités `bg-background/80`, superpositions de calques, états hover). Un audit
de contraste exhaustif reste à faire (voir §4).*

### 2.4 Multimédia

**Non applicable.** Aucun média temporel (`<video>`, `<audio>`) dans le frontend. Le seul
contenu animé est le canvas décoratif (masqué, cf. §2.1) et des animations CSS/Framer
Motion maîtrisées (cf. §2.13).

### 2.5 Tableaux

**Non applicable.** Aucun tableau de données (`<table>`) dans le frontend ; les listes
de données sont structurées avec des rôles de liste (cf. §2.9).

### 2.6 Liens

**Conforme.** Les liens et boutons ont des intitulés explicites :

- Les boutons iconiques sans texte visible portent un `aria-label` :
  `aria-label="Toggle menu"` (`frontend/src/components/Navigation.tsx:80`),
  `aria-label="View transaction on block explorer"`
  (`frontend/src/pages/app/IntentDetailPage.tsx:233`),
  `aria-label="Open navigation"` / `aria-label="Close navigation"`
  (`frontend/src/components/app/AppHeader.tsx:91`,
  `frontend/src/components/app/AppLayout.tsx:70`).
- Les liens sociaux du pied de page ont un libellé :
  `aria-label={social.label}` (`frontend/src/components/Footer.tsx:31`).
- L'état du lien courant est signalé : `aria-current={isCurrent ? "page" : undefined}`
  dans le fil d'Ariane (`frontend/src/components/app/AppHeader.tsx:108`) et
  `aria-pressed` pour les filtres (`frontend/src/pages/app/IntentsPage.tsx:86`).

### 2.7 Scripts

**Conforme (partiel).** Les composants interactifs reposent sur des éléments HTML
natifs focusables (`<button>`, `<input>`), ce qui garantit l'usage au clavier de base :

- Groupe de boutons radios custom implémenté avec des `<button type="button">` natifs
  + sémantique ARIA : `role="radiogroup"` / `role="radio"` / `aria-checked`
  (`frontend/src/pages/app/CreateDelegationPage.tsx:136-142`,
  `frontend/src/pages/app/CreateIntentPage.tsx:388-396`).
- Case à cocher custom : `role="checkbox"` + `aria-checked`
  (`frontend/src/components/ui/checkbox.tsx:17-18`).
- Fermeture des couches superposées au clavier via `Escape` : modale de bienvenue
  (`frontend/src/components/app/WelcomeModal.tsx:29-33`), tiroir de navigation mobile
  (`frontend/src/components/app/AppLayout.tsx:40-44`).
- États des disclosures : `aria-expanded` + `aria-controls` (FAQ,
  `frontend/src/components/Faq.tsx:79-80` ; détails de preuve,
  `frontend/src/pages/app/ProofsPage.tsx:86-87`).
- Menus : `aria-haspopup="menu"`, `role="menu"`, `role="menuitem"`
  (`frontend/src/components/app/AppHeader.tsx:146-191`).
- Primitives **Radix UI** (`@radix-ui/react-separator`, `@radix-ui/react-slot`,
  `frontend/package.json:16-17`), bibliothèque conçue pour l'accessibilité (gestion du
  focus, navigation clavier, attributs ARIA par construction).

*Limite honnête : l'usage de Radix est restreint à deux primitives de bas niveau ; les
dialogues, menus et tooltips sont des composants **maison**. Ils gèrent `Escape` et les
attributs ARIA, mais ne capturent pas le focus dans la modale (pas de focus trap ni de
retour du focus à la fermeture dans `WelcomeModal.tsx`), et le radiogroup ne gère pas la
navigation fléchée entre radios exigée par le pattern ARIA « radio group ».*

### 2.8 Éléments obligatoires

**Conforme (partiel).**

- Doctype et langue : `<!doctype html>` et `<html lang="en">`
  (`frontend/index.html:1-2`).
- Titre de page et description : `<title>Otter · Trustless DeFi intents</title>` et
  `<meta name="description">` (`frontend/index.html:10-14`).
- Attribut `dir` non requis (contenu LTR), encodage UTF-8 déclaré
  (`frontend/index.html:4`).

*Limite : le `<title>` est **unique et statique** pour toute l'application monopage —
aucune mise à jour de `document.title` à la navigation (aucune occurrence dans
`frontend/src/`). Non conforme au critère RGAA 8.5/8.6 (titre de page pertinent par
page). À corriger (voir §4).*

### 2.9 Structuration de l'information

**Conforme.**

- Hiérarchie de titres : `h1` unique par vue — page d'accueil
  (`frontend/src/components/HeroSection.tsx:39`), pages applicatives via le composant
  `PageHeader` (`frontend/src/components/app/PageHeader.tsx:20`) ; `h2`/`h3` pour les
  sous-sections (ex. modale : `frontend/src/components/app/WelcomeModal.tsx:58,78`).
- Listes sémantiques : `role="list"` / `role="listitem"` pour le stepper
  (`frontend/src/components/app/Stepper.tsx:13,20`) et la timeline
  (`frontend/src/components/app/KineticTimeline.tsx:34,41`).
- Regroupements de navigation nommés : `aria-label="App navigation"`
  (`frontend/src/components/app/AppSidebar.tsx:156`), `aria-label="Breadcrumb"`
  (`frontend/src/components/app/AppHeader.tsx:95`), section nommée
  `aria-label="Supported protocols and networks"`
  (`frontend/src/components/PoweredBy.tsx:18`).

### 2.10 Présentation de l'information

**Conforme (partiel).**

- Feuille de style externe unique, aucun style en ligne de présentation ni balise de
  présentation obsolète ; la couche CSS est centralisée (`frontend/src/index.css`).
- Contenu restitué dans un ordre logique : le DOM suit l'ordre visuel (en-tête,
  navigation, `<main>`, pied).
- Prise en compte de `prefers-reduced-motion` (cf. §2.13).

*Limite : la lisibilité à 200 % de zoom et en mode contraste élevé forcé
(`forced-colors`) n'a pas été testée.*

### 2.11 Formulaires

**Conforme (partiel) — c'est la thématique la mieux couverte.**

- **Étiquettes explicites** associées par `htmlFor`/`id` :
  `frontend/src/pages/app/CreateDelegationPage.tsx:173,264`,
  `frontend/src/pages/app/CreateIntentPage.tsx:295-332` ; étiquette visuellement
  masquée (`sr-only`) pour le champ e-mail (`frontend/src/components/Waitlist.tsx:102`).
- **Validation et erreurs accessibles** :
  - `aria-invalid` sur les champs en erreur
    (`frontend/src/pages/app/CreateDelegationPage.tsx:180,269`,
    `frontend/src/components/Waitlist.tsx:117`),
  - messages d'erreur annoncés via `role="alert"`
    (`frontend/src/pages/app/CreateDelegationPage.tsx:193,226,252,275,321`,
    `frontend/src/pages/app/CreateIntentPage.tsx:250,445,534`,
    `frontend/src/components/Waitlist.tsx:152`),
  - association champ ↔ message d'erreur par `aria-describedby`
    (`frontend/src/components/Waitlist.tsx:99,118`),
  - retour de soumission annoncé : `role="status"`
    (`frontend/src/pages/app/SettingsPage.tsx:96`), `aria-live="polite"`
    (`frontend/src/components/Waitlist.tsx:82`).
- Les erreurs sont textuelles (pas uniquement colorées), conformément au critère 3.1.

### 2.12 Navigation

**Conforme (partiel).**

- **Lien d'évitement** « Skip to main content » présent, visible au focus
  (`sr-only … focus:not-sr-only`), ciblant `<main id="main-content">`
  (`frontend/src/App.tsx:40-45,52`).
- Gestion du focus à la navigation côté client : le `<main>` reçoit le focus à chaque
  changement de route (`frontend/src/App.tsx:25-30`), avec `tabIndex={-1}` et
  `outline-none` (`frontend/src/App.tsx:52`).
- Zones repérables : `<main>`, `<nav>` étiquetées, fil d'Ariane avec
  `aria-current="page"` (`frontend/src/components/app/AppHeader.tsx:95-108`),
  indication d'étape courante `aria-current="step"`
  (`frontend/src/components/app/Stepper.tsx:23`).
- Indicateurs d'état non purement colorés : pastilles de statut masquées
  (`aria-hidden`) et accompagnées de texte
  (`frontend/src/components/app/AppSidebar.tsx:131,152`,
  `frontend/src/components/app/AppHeader.tsx:135`).

*Limites : le lien d'évitement n'existe que sur la page publique (`App.tsx`) ; le shell
applicatif `AppLayout.tsx` ne possède **pas** de skip link vers son propre `<main>`
(`frontend/src/components/app/AppLayout.tsx:82`). Pas de plan du site. L'ordre de
tabulation dans les couches superposées n'est pas verrouillé (cf. §2.7).*

### 2.13 Consultation

**Conforme (partiel).**

- **Contenus en mouvement maîtrisables** : respect global de
  `prefers-reduced-motion` —
  - désactivation CSS des animations longues
    (`frontend/src/index.css:149-153`),
  - configuration globale Framer Motion : `<MotionConfig reducedMotion="user">`
    (`frontend/src/main.tsx:72`),
  - le canvas WebGL n'est pas rendu si l'utilisateur préfère réduire les animations
    (`frontend/src/components/HeroSection.tsx:11-23`),
  - les compteurs animés sont court-circuités
    (`frontend/src/components/app/CountUp.tsx:23-24`).
- Pas d'ouverture de fenêtre intempestive ; les liens externes portent un libellé
  explicite (cf. §2.6).
- Chargement annoncé : `aria-busy="true"` + `aria-label="Loading page"` sur l'état de
  chargement des routes (`frontend/src/components/app/AppLayout.tsx:13`).

*Limite : l'animation marquee de la page publique (`.animate-marquee-linear`,
`frontend/src/index.css:107-119`) n'offre pas de contrôle pause/stop à l'utilisateur —
elle dure certes 90 s, mais le critère 13.8 exige une possibilité d'arrêt pour tout
mouvement déclenché automatiquement de plus de 5 s.*

---

## 3. Tableau de synthèse

| Thématique RGAA | Statut | Preuve principale | Action corrective |
|---|---|---|---|
| Images | Conforme | `aria-hidden` sur icônes/canvas (`WebGLSpiral.tsx:151`) | Réévaluer si ajout d'images informatives |
| Cadres | N/A | Aucun `<iframe>` | — |
| Couleurs | 🟡 Partiel | Ratios calculés : 7,5:1 à 18,5:1 sur les couleurs principales | Audit de contraste exhaustif (états hover, opacités) ; corriger `#71717a` (4,22:1 < 4,5:1) pour le texte courant |
| Multimédia | N/A | Aucun média temporel | — |
| Tableaux | N/A | Aucun `<table>` | — |
| Liens | Conforme | `aria-label` sur boutons iconiques, `aria-current` (`AppHeader.tsx:108`) | — |
| Scripts | 🟡 Partiel | Boutons natifs, `Escape`, ARIA complet sur radios/disclosures/menus | Focus trap + retour du focus dans les modales ; navigation fléchée dans les radiogroups |
| Éléments obligatoires | 🟡 Partiel | `lang="en"`, doctype, titre (`index.html:1-10`) | `document.title` dynamique par route |
| Structuration | Conforme | `h1` unique, `role="list"/"listitem"`, navs étiquetées | — |
| Présentation | 🟡 Partiel | CSS externe, ordre DOM logique | Test zoom 200 % et `forced-colors` |
| Formulaires | Conforme | `htmlFor`, `aria-invalid`, `role="alert"`, `aria-describedby` | Tests avec lecteur d'écran |
| Navigation | 🟡 Partiel | Skip link + focus management (`App.tsx:40-52`), `aria-current` | Skip link dans `AppLayout`, plan du site |
| Consultation | 🟡 Partiel | `prefers-reduced-motion` global (`main.tsx:72`, `index.css:149`) | Contrôle pause/stop pour le marquee |

---

## 4. Plan d'amélioration

Points **non couverts** ou partiellement couverts à date, par ordre de priorité :

1. **Titre de page dynamique** (critère RGAA 8.5) — ajouter un hook mettant à jour
   `document.title` à chaque route (le titre est aujourd'hui identique sur toute
   l'application).
2. **Gestion du focus dans les dialogues maison** (`WelcomeModal.tsx`, tiroir mobile
   d'`AppLayout.tsx`, menus d'`AppHeader.tsx`) — implémenter un focus trap, placer le
   focus à l'ouverture et le restituer à l'élément déclencheur à la fermeture. Le plus
   simple : remplacer ces composants par les primitives Radix UI (`Dialog`, `DropdownMenu`)
   qui fournissent ce comportement par construction, la dépendance étant déjà présente.
3. **Radiogroups custom** (`CreateDelegationPage.tsx:136`, `CreateIntentPage.tsx:388`) —
   ajouter la navigation au clavier par flèches et le roving tabindex conformément au
   pattern ARIA « radio group ».
4. **Skip link dans le shell applicatif** — le lien d'évitement n'existe que sur la page
   publique ; l'ajouter dans `AppLayout.tsx` vers le `<main>` interne.
5. **Audit de contraste exhaustif** — le calcul ne couvre que les couleurs principales
   en état de repos. Mesurer toutes les combinaisons réelles (états hover/focus/disabled,
   textes sur fonds translucides `bg-background/80`, badges de statut). Corriger notamment
   le gris `#71717a` (4,22:1) qui ne passe pas le seuil 4,5:1 pour le texte courant.
6. **Contrôle du marquee** (`index.css:107-119`) — ajouter un bouton pause/stop ou
   conditionner l'animation à `prefers-reduced-motion` comme les autres animations.
7. **Tests clavier systématiques** — parcours complet de chaque page à la seule
   tabulation, avec vérification de la visibilité du focus sur tous les composants
   (le style `outline-none` sur `<main>` est compensé par les `focus:ring-*` des
   composants, mais cela reste à vérifier page par page).
8. **Tests avec lecteurs d'écran** — aucun test réel NVDA/VoiceOver n'a été mené à ce
   stade ; la conformité ARIA n'a été vérifiée que par inspection statique du code.
9. **Tests zoom 200 % et mode contraste élevé** (`forced-colors: active`).
10. **Plan du site / page d'aide à la navigation** (critère RGAA 12.3).
11. **Déclaration d'accessibilité** — à rédiger après un audit RGAA formel (obligatoire
    pour les organismes assujettis ; non produite ici puisque le projet est un prototype).

---

*Document rédigé à partir d'une revue du code source au 20/07/2026. Toute évolution du
frontend doit déclencher une réévaluation des thématiques concernées.*
