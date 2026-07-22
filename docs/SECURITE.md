# Mesures de sécurité — Projet Otter

Document de présentation des mesures de sécurité implémentées dans le dépôt
`otter` (projet **Otter** : automatisation DeFi trustless — vault on-chain,
délégations signées ECDSA, preuves ZK Noir vérifiées on-chain, backend
Rust/Axum hexagonal, frontend React).

La structure suit l'**OWASP Top 10 (édition 2021)**, de A01 à A10. Pour chaque
risque : une description courte, les mesures concrètes implémentées **dans ce
dépôt** avec les preuves (fichier:ligne), puis les points résiduels et axes
d'amélioration assumés.

> Toutes les références `fichier:ligne` ont été vérifiées dans le code source
> au moment de la rédaction.

---

## A01:2021 — Contrôles d'accès défaillants (Broken Access Control)

**Description du risque.** Des utilisateurs non autorisés accèdent à des
fonctions ou à des données qui ne leur sont pas destinées (élévation de
privilèges, accès à des ressources d'autrui, contournement de restrictions).

**Mesures implémentées.**

- *Côté API (backend Rust/Axum).* Les routes métier (`/api/v1/...`) sont
  regroupées dans un routeur « protégé » auquel est appliqué un middleware
  d'authentification systématique (`auth_middleware`) : toute requête sans
  en-tête `Authorization: Bearer <jwt>` valide reçoit un `401 Unauthorized`.
  — `crates/interfaces/src/bin/otter_api.rs:383-402` (routeur protégé),
  `crates/interfaces/src/bin/otter_api.rs:431-464` (middleware).
  Un test d'intégration vérifie le rejet `401` sans jeton et l'acceptation
  avec jeton valide — `crates/interfaces/src/bin/otter_api.rs:1954-1983`.
- *Côté smart contract.* Le contrat `DelegationVault` hérite d'`Ownable`
  (OpenZeppelin) et restreint la fonction sensible `setProtocolRouter`
  (whitelist des routeurs de protocoles) via le modificateur `onlyOwner`
  — `contracts/src/DelegationVault.sol:16` et
  `contracts/src/DelegationVault.sol:95-99`.
- *Séparation des privilèges par conception.* L'agent ne peut pas déplacer
  les fonds librement : toute exécution passe par `executeWithProof`, qui
  impose la vérification d'une preuve ZK **avant** tout mouvement de fonds,
  puis le respect des limites enregistrées on-chain (type d'intention
  autorisé, montant maximum, protocole whitelisté, expiration, nonce)
  — `contracts/src/DelegationVault.sol:176-210`.
- *Retraits limités au propriétaire des fonds.* Les retraits ne créditent que
  `msg.sender` après vérification de son propre solde
  — `contracts/src/DelegationVault.sol:152-171`.

**Points résiduels / axes d'amélioration.**

- L'authentification API est **désactivée par défaut** (`auth_enabled =
  false`) pour le développement — `crates/infrastructure/src/config/mod.rs:151-153`.
  Une mauvaise configuration de déploiement exposerait les routes protégées.
- Il n'y a pas de contrôle d'accès par rôle (RBAC) côté API : tout détenteur
  d'un JWT valide accède à l'ensemble des routes protégées ; les intentions ne
  sont pas filtrées par adresse propriétaire au niveau du routeur.
- Le propriétaire du contrat (clé `Ownable`) est un point de centralisation :
  pas de multisig ni de timelock dans cette version.

---

## A02:2021 — Défaillances cryptographiques (Cryptographic Failures)

**Description du risque.** Protection insuffisante des données sensibles en
transit ou au repos : algorithmes faibles, clés mal gérées, secrets exposés.

**Mesures implémentées.**

- *Pas de mots de passe.* L'authentification repose sur la possession d'une
  clé Ethereum : Sign-In with Ethereum (EIP-4361) — challenge signé par la
  clé privée de l'utilisateur, vérifié côté serveur
  — `crates/interfaces/src/auth.rs:106-155`. Aucun mot de passe n'est stocké.
- *Jetons de session signés.* JWT en **HS256** avec validation stricte de
  l'algorithme — `crates/interfaces/src/auth.rs:159-164` (validation) et
  `crates/interfaces/src/auth.rs:178-183` (émission). La durée de vie est
  configurable (`token_ttl_hours`, 24 h par défaut)
  — `crates/interfaces/src/auth.rs:48,172` et
  `crates/infrastructure/src/config/mod.rs:138-140,155-157`.
- *Challenges à usage unique et courte durée.* Nonce aléatoire de **16 octets**
  par challenge, expiration à **5 minutes**, comparaison du message signé au
  message émis, et suppression du challenge après usage (anti-rejeu)
  — `crates/interfaces/src/auth.rs:72-74` (nonce/expiration),
  `crates/interfaces/src/auth.rs:121-127` (contrôles),
  `crates/interfaces/src/auth.rs:147-151` (consommation).
- *Cryptographie on-chain.* Délégations signées **ECDSA secp256k1**, vérifiées
  dans le circuit ZK Noir — `delegation_circuit/src/main.nr:140-146` ; hash de
  délégation en **blake2s** — `delegation_circuit/src/main.nr:106`. La preuve
  UltraHonk est vérifiée on-chain par un contrat vérifieur dédié avant
  exécution — `contracts/src/DelegationVault.sol:180-182`.
- *Gestion des secrets.* Abstraction `SecretProvider` avec plusieurs
  stratégies : variable d'environnement (dev), fichier sur disque (mode
  `0600` recommandé et jamais commité), **HashiCorp Vault** (KV v2, feature
  cargo `vault`), **AWS KMS** (déchiffrement d'un blob, feature `aws-kms`),
  et **keystore Ethereum chiffré** (`eth-keystore`)
  — `crates/interfaces/src/secrets.rs:8-38` (trait + env/fichier),
  `crates/interfaces/src/secrets.rs:44-146` (Vault),
  `crates/interfaces/src/secrets.rs:148-243` (AWS KMS),
  `crates/interfaces/src/secrets.rs:342-359` (keystore).
  La précédence documentée est keystore > fichier > variable d'environnement,
  avec un avertissement explicite en logs quand la clé vient de l'environnement
  — `crates/interfaces/src/secrets.rs:272-287,372-374`.

**Points résiduels / axes d'amélioration.**

- **TLS non géré dans le code** : le serveur écoute en HTTP clair
  (`axum::serve` sur `0.0.0.0:<port>` — `crates/interfaces/src/bin/otter_api.rs:779-791`,
  aucune dépendance `rustls`). Le chiffrement en transit relève d'un reverse
  proxy externe. Une configuration TLS de référence est désormais versionnée
  (`deploy/Caddyfile` : terminaison TLS automatique via Caddy/Let's Encrypt,
  routage du frontend et de `/api/*`, headers de sécurité — cf.
  `DEPLOYMENT.md`, section « Terminaison TLS ») ; son déploiement effectif
  reste hors du périmètre du dépôt.
- Le secret JWT est **généré aléatoirement à chaque démarrage** s'il n'est pas
  configuré (acceptable en dev uniquement, signalé dans le code)
  — `crates/interfaces/src/auth.rs:52-61` et
  `crates/infrastructure/src/config/mod.rs:133-136`. En production il doit
  être fourni explicitement, sinon tout redémarrage invalide les sessions.
- HS256 (secret partagé) plutôt qu'un algorithme asymétrique (RS256/EdDSA) :
  acceptable pour un service unique, mais à revoir si plusieurs services
  doivent valider les jetons.

---

## A03:2021 — Injection

**Description du risque.** Données hostiles interprétées comme des commandes
ou requêtes (SQL, interpréteur de commandes, etc.).

**Mesures implémentées.**

- *Requêtes SQL paramétrées partout.* Les accès SQLite utilisent
  exclusivement des paramètres liés (`rusqlite::params!`, `?1, ?2, ...`)
  — par exemple `crates/infrastructure/src/storage/sqlite.rs:144-160` ; les
  accès PostgreSQL utilisent les requêtes `sqlx` avec `.bind(...)`. Une
  recherche sur `crates/infrastructure` ne trouve **aucune concaténation de
  SQL avec des entrées utilisateur**. Seule exception contrôlée : un
  `ALTER TABLE` formaté dans un helper de migration interne
  (`add_column_if_missing`, `crates/infrastructure/src/storage/sqlite.rs:113-133`),
  dont les identifiants table/colonne sont des constantes du code, jamais des
  données externes.
- *Désérialisation typée.* Toutes les entrées HTTP passent par des structures
  `serde` typées (`Json<ChallengeRequest>`, etc.), ce qui rejette les charges
  malformées avant tout traitement — `crates/interfaces/src/bin/otter_api.rs:510-536`.
- *Validation métier des entrées.* Longueur maximale du texte d'intention
  (`MAX_INTENT_TEXT_LEN = 2000`, rejet au-delà et rejet des textes vides)
  — `crates/interfaces/src/bin/otter_api.rs:1034-1047`. Validation des champs
  de délégation : cardinalités exactes (10 montants, 5 protocoles) et format
  hexadécimal de chaque champ (`validate_hex_field`)
  — `crates/interfaces/src/bin/otter_api.rs:1049-1079`.
- *Smart contract.* Pas d'`eval` ni d'appel dynamique : les transferts
  ERC-20 passent par `SafeERC20` d'OpenZeppelin
  — `contracts/src/DelegationVault.sol:5,17,145,168,249`, et les transferts
  ERC-20 d'exécution ne vont que vers des routeurs whitelistés par le
  propriétaire — `contracts/src/DelegationVault.sol:246-249`.

**Points résiduels / axes d'amélioration.**

- Le texte d'intention est transmis à un modèle de langage (LLM) : le risque
  d'« injection de prompt » n'est pas traité par des garde-fous dédiés dans le
  code ; la borne de 2000 caractères et les limites on-chain (montants,
  protocoles) constituent la ligne de défense finale.
- Les validations d'entrées restent minimales sur certains champs (formats
  d'adresses, etc.).

---

## A04:2021 — Conception non sécurisée (Insecure Design)

**Description du risque.** Faiblesses de conception (absence de modèle de
menace, limites métier non imposées) qu'aucun correctif de code ne peut
résorber.

**Mesures implémentées (conception).**

- *Architecture « trustless ».* Le principe directeur est que l'agent ne peut
  pas violer les limites fixées par l'utilisateur même s'il est compromis :
  les limites (types d'intentions, montants max par type, protocoles
  whitelistés, expiration, nonce) sont enregistrées **on-chain** et vérifiées
  **deux fois** — dans le circuit ZK (`delegation_circuit/src/main.nr:148-168`)
  et dans le contrat (`contracts/src/DelegationVault.sol:193-201`).
- *Anti-rejeu à deux niveaux.* Nonce de délégation vérifié dans le circuit
  (`delegation_circuit/src/main.nr:167-168`) puis marqué comme utilisé
  on-chain (`usedNonces`) — `contracts/src/DelegationVault.sol:197-198,203-204`.
- *Preuve obligatoire avant exécution.* Aucune exécution de fonds sans
  vérification cryptographique préalable : `verifier.verify(...)` sinon
  `revert InvalidProof()` — `contracts/src/DelegationVault.sol:179-182`.
- *Contrainte de cible.* La délégation peut épingler un `target_contract` :
  le circuit rejette toute intention visant un autre contrat
  — `delegation_circuit/src/main.nr:158-162`.
- *Drapeau d'exécution désactivé par défaut.* `OTTER_EXECUTION_ENABLED=false`
  dans la configuration d'exemple — `.env.example:40`.

**Points résiduels / axes d'amélioration.**

- Le modèle de menace n'est pas formalisé dans un document dédié (ce fichier
  en tient lieu partiellement).
- Le nonce de délégation étant unique par délégation, une délégation ne peut
  servir qu'une seule exécution ; toute réexécution exige une nouvelle
  délégation. C'est un choix de conception restrictif, à documenter comme tel.
- Pas de mécanisme de pause d'urgence (circuit breaker) sur le contrat dans
  cette version.

---

## A05:2021 — Mauvaise configuration de sécurité (Security Misconfiguration)

**Description du risque.** Configurations par défaut permissives, messages
d'erreur verbeux, fonctionnalités inutiles exposées.

**Mesures implémentées.**

- *CORS à liste blanche configurable.* `build_cors` accepte une liste
  d'origines explicites (séparées par des virgules) ; le joker `*` existe mais
  doit être choisi délibérément — `crates/interfaces/src/bin/otter_api.rs:411-429`.
  Tests : origine autorisée acceptée (`otter_api.rs:2017-2035`) et origine non
  configurée rejetée (`otter_api.rs:2037-2044` et suivantes).
- *Rate limiting par IP.* Middleware maison à fenêtre glissante de 60 s,
  seuil configurable (`rate_limit_per_minute`, défaut 100, 0 = désactivé),
  réponse `429 Too Many Requests` — `crates/interfaces/src/bin/otter_api.rs:476-508`
  et `crates/infrastructure/src/config/mod.rs:146-148,163-165`.
  Test dédié — `crates/interfaces/src/bin/otter_api.rs:1986-2014`.
- *Endpoints d'observabilité séparés.* Seuls `/health`, `/ready` et
  `/metrics` sont publics — `crates/interfaces/src/bin/otter_api.rs:380-381`.
- *Configuration TLS de référence versionnée.* `deploy/Caddyfile` fournit un
  reverse proxy Caddy avec terminaison TLS automatique (Let's Encrypt),
  routage du frontend et de `/api/*` vers l'API, et headers de sécurité de
  base (HSTS, `X-Content-Type-Options`, `X-Frame-Options`,
  `Referrer-Policy`). Les endpoints `/health`, `/ready` et `/metrics` ne sont
  pas exposés par ce proxy. Cf. `DEPLOYMENT.md`, section « Terminaison TLS ».
- *Configuration externalisée et documentée.* `.env.example` documente la
  précédence des sources de clés et interdit explicitement de committer une
  clé réelle — `.env.example:11-19`. Les secrets CI (clés SSH de déploiement,
  etc.) sont dans GitHub Secrets — `.github/workflows/deploy-testnet.yml:90-92`,
  `.github/workflows/deploy-mainnet.yml:38-40`.

**Points résiduels / axes d'amélioration.**

- **Défauts permissifs en dev** : `auth_enabled = false`
  (`crates/infrastructure/src/config/mod.rs:151-153`) et CORS `*` par défaut
  (`crates/infrastructure/src/config/mod.rs:159-161`). Ces défauts sont
  pensés pour le développement local ; un profil de production durci n'existe
  pas dans le dépôt.
- Le rate limiting est en mémoire et par instance : il ne protège pas derrière
  plusieurs réplicas ni contre une attaque distribuée (une IP = un compteur,
  un attaquant avec beaucoup d'IP le contourne).
- Les erreurs internes sont parfois renvoyées au client sous forme textuelle
  (`format!("invalid token: {}", err)` — `otter_api.rs:460`), ce qui peut
  divulguer des détails d'implémentation.

---

## A06:2021 — Composants vulnérables et obsolètes

**Description du risque.** Dépendances tierces comportant des vulnérabilités
connues ou non maintenues.

**Mesures implémentées.**

- *Bibliothèques reconnues pour le code critique.* Contrats basés sur
  **OpenZeppelin** (`Ownable`, `SafeERC20`, `IERC20`)
  — `contracts/src/DelegationVault.sol:5-7`. Authentification SIWE via la
  crate `siwe`, JWT via `jsonwebtoken`, clés via `eth-keystore`
  — `crates/interfaces/src/auth.rs:1-8`, `crates/interfaces/src/secrets.rs:344`.
- *Versions d'outillage épinglées.* Les versions de Noir et Barretenberg sont
  fixées dans `.noir-version` et `.bb-version` à la racine du dépôt, et
  référencées par `.env.example:29-31`.
- *Lint strict en CI.* `cargo clippy --workspace --all-targets -- -D warnings`
  bloque la CI sur tout avertissement — `.github/workflows/ci.yml:46-47`.
- *Verrouillage des dépendances Rust.* `Cargo.lock` est commité à la racine,
  garantissant des builds reproductibles.
- *Audit automatisé des dépendances Rust en CI.* Un job `security-audit`
  installe `cargo-audit` et exécute `cargo audit` (base RustSec) sur le
  workspace à chaque pipeline — `.github/workflows/ci.yml`, job
  `security-audit`. Le job est **bloquant** : toute nouvelle vulnérabilité
  fait échouer la CI. Seules des vulnérabilités sans correctif applicable
  sont ignorées via `--ignore`, chacune justifiée par un commentaire dans
  le workflow (voir points résiduels ci-dessous).
- *Mises à jour automatisées.* Dependabot ouvre chaque semaine des PR de
  mise à jour pour les écosystèmes cargo (racine), npm (`frontend/`),
  github-actions et docker — `.github/dependabot.yml`.

**Points résiduels / axes d'amélioration.**

- **Vulnérabilités connues : état au 2026-07-22.** L'audit du 2026-07-20
  (`cargo audit` sur `Cargo.lock`, 571 dépendances) signalait 6
  vulnérabilités. Traitement effectué le 2026-07-22 :
  - `sqlx` 0.7.4 (RUSTSEC-2024-0363) : **corrigé** par upgrade vers
    `sqlx` 0.8.6 (features inchangées) dans
    `crates/infrastructure/Cargo.toml`. `rusqlite` a dû être monté de
    0.30 à 0.32 pour lever le conflit de linkage `sqlite3` entre
    `libsqlite3-sys` 0.27 (rusqlite 0.30) et 0.30 (sqlx-sqlite 0.8).
    Aucun changement de code applicatif requis.
  - `rsa` 0.9.10 (RUSTSEC-2023-0071, attaque Marvin par canal auxiliaire
    temporel) : **sans correctif disponible**, ignorée dans le job CI.
    Exposition nulle : `rsa` n'est tirée que par `sqlx-mysql`, dépendance
    optionnelle de la facade `sqlx` dont la feature `mysql` n'est jamais
    activée ; `cargo tree -i rsa` sur les features par défaut retourne un
    graphe vide, le code `rsa` n'entre dans aucun binaire. Réévaluation :
    à chaque upgrade de sqlx ou si la feature `mysql` est un jour activée.
  - `rustls-webpki` 0.101.7 (RUSTSEC-2026-0098, RUSTSEC-2026-0099,
    RUSTSEC-2026-0104) : **ignorées** dans le job CI. La version vulnérable
    n'entre dans le graphe que via la feature optionnelle `aws-kms` de
    `interfaces` (`aws-smithy-http-client` 1.1.10 -> `rustls` 0.21),
    jamais activée dans les builds par défaut, la CI ou le Dockerfile ;
    le graphe par défaut utilise `rustls` 0.23 + `rustls-webpki`
    0.103.13 (corrigé). `aws-config` 1.8.14 et `aws-sdk-kms` 1.101.0 sont
    déjà aux dernières versions : pas de correctif applicable sans
    changement upstream du SDK AWS. Réévaluation : à chaque mise à jour
    Dependabot des SDK AWS.
  - `alloy-dyn-abi` 0.7.7 (RUSTSEC-2025-0073, DoS sur le hashing
    `TypedData` EIP-712, sévérité 7.5) : **ignorée** dans le job CI. Le
    correctif (>= 0.8.26) exige une migration majeure d'`alloy` 0.2 vers
    0.8, jugée trop risquée avant le rendu. Exposition nulle : aucun usage
    de `TypedData`/EIP-712 dans le code (grep sur `crates/`,
    `contracts/src`, `frontend/src`) ; l'authentification SIWE
    (`crates/interfaces/src/auth.rs`) repose sur EIP-191/personal_sign et
    n'appelle jamais le hashing `TypedData`. Réévaluation : lors de la
    migration `alloy` planifiée après le rendu.
  - Restent 5 avertissements non bloquants (crates non maintenues
    `derivative`, `paste`, `proc-macro-error` ; `lru` unsound ; `spin`
    yanké), remontés par le job sans faire échouer la CI.
- Pas de scan de vulnérabilités sur les dépendances Solidity (le vérifieur et
  OpenZeppelin sont intégrés via `contracts/lib/`) ni d'audit externe du
  contrat ou du circuit ZK dans cette version.

---

## A07:2021 — Défaillances d'identification et d'authentification

**Description du risque.** Authentification contournable : identifiants
faibles, sessions mal gérées, rejeu de jetons.

**Mesures implémentées.**

- *Authentification sans mot de passe (EIP-4361).* L'identité est prouvée par
  signature d'un message standardisé, ce qui élimine par construction les
  attaques par force brute sur des mots de passe, le credential stuffing et le
  stockage de hash — `crates/interfaces/src/auth.rs:69-103` (génération du
  challenge), `crates/interfaces/src/auth.rs:106-155` (vérification).
- *Anti-rejeu des challenges.* Nonce aléatoire 16 octets, expiration 5 min,
  message signé comparé à l'original, challenge détruit après usage
  — `crates/interfaces/src/auth.rs:72-74,121-127,147-151`.
- *Sessions courtes et signées.* JWT HS256 avec expiration (`exp`) et date
  d'émission (`iat`), TTL configurable — `crates/interfaces/src/auth.rs:170-184`.
- *Jeton transmis en en-tête, pas en cookie.* Le JWT est porté par
  `Authorization: Bearer`, ce qui le soustrait aux attaques CSRF classiques
  (voir A01/A05) — `crates/interfaces/src/bin/otter_api.rs:442-445`.
- *Tests unitaires.* Génération de challenge et cycle émission/validation de
  jeton — `crates/interfaces/src/auth.rs:194-215` ; test d'accès 401/200 en
  intégration — `crates/interfaces/src/bin/otter_api.rs:1954-1983`.

**Points résiduels / axes d'amélioration.**

- Le stockage des challenges est **en mémoire** (`HashMap` sous `Mutex` —
  `crates/interfaces/src/auth.rs:46-47`) : un redémarrage les invalide et un
  déploiement multi-instances casse le flux ; un stockage partagé (Redis)
  serait nécessaire en production.
- Pas de révocation de JWT avant expiration (pas de liste noire) : un jeton
  volé reste valide jusqu'à `exp`.
- Le défi SIWE utilise un domaine fixe `otter.local`
  (`crates/interfaces/src/auth.rs:79-85`) : à paramétrer par déploiement pour
  conserver la protection anti-phishing du standard.
- Le secret JWT aléatoire en dev (cf. A02) invalide toutes les sessions à
  chaque redémarrage.

---

## A08:2021 — Défaillances d'intégrité des logiciels et des données

**Description du risque.** Code ou données dont l'intégrité n'est pas vérifiée
: mises à jour non signées, CI/CD non maîtrisée, artefacts non vérifiables.

**Mesures implémentées.**

- *Intégrité cryptographique au cœur du produit.* L'exécution d'une intention
  exige une preuve ZK valide correspondant exactement aux entrées publiques
  (hash de délégation, montant, protocole, cible, timestamp, nonce) : toute
  altération des données invalide la preuve
  — `contracts/src/DelegationVault.sol:176-210` et
  `delegation_circuit/src/main.nr:127-168`.
- *Le hash de délégation lie les données privées aux entrées publiques.*
  Le circuit recalcule `blake2s(delegation)` et le compare au hash public
  — `delegation_circuit/src/main.nr:135-137` : impossible de substituer une
  délégation différente.
- *CI reproductible.* Dépendances Rust verrouillées par `Cargo.lock`,
  outillage ZK épinglé (`.noir-version`, `.bb-version`), clippy strict en CI
  — `.github/workflows/ci.yml:46-47`.
- *Secrets CI externalisés.* Les clés de déploiement ne figurent pas dans le
  code ni dans les workflows en clair (GitHub Secrets)
  — `.github/workflows/deploy-mainnet.yml:38-40`.

**Points résiduels / axes d'amélioration.**

- Pas de signature/vérification des artefacts de build (binaires, images
  Docker) ni de SBOM généré.
- Le workflow de déploiement mainnet (`.github/workflows/deploy-mainnet.yml:28`)
  se contente d'un rappel manuel (« verify testnet deployment, audit reports »)
  sans garde automatisée.
- Les sous-modules/dépendances Solidity (`contracts/lib/`) ne sont pas
  vérifiés par checksum en CI.

---

## A09:2021 — Défaillances de journalisation et de surveillance

**Description du risque.** Absence de logs et d'alertes empêchant la détection
et la réponse aux incidents.

**Mesures implémentées.**

- *Journalisation structurée.* Le backend utilise `tracing` dans tout le code
  (ex. `crates/interfaces/src/secrets.rs:66-77,93-95` ; avertissement explicite
  lors du chargement d'une clé privée depuis l'environnement —
  `crates/interfaces/src/secrets.rs:372-374`). Format JSON activable pour la
  production via `OTTER_LOG_FORMAT` — `.env.example:45-47`.
- *Endpoints de santé et de métriques.* `/health`, `/ready` et `/metrics`
  exposés par l'API — `crates/interfaces/src/bin/otter_api.rs:380-381` ; un
  fichier de configuration d'alerting (`alerting.yml`) est présent à la racine
  du dépôt.
- *Traçabilité on-chain.* Toutes les opérations sensibles du contrat émettent
  des événements (`Delegated`, `Deposited`, `Withdrawn`, `Executed`,
  `ProtocolRouterSet`) — `contracts/src/DelegationVault.sol:62-74`, permettant
  une surveillance et un audit a posteriori.
- *Journalisation des erreurs de persistance.* Les échecs de sauvegarde
  d'état sont tracés avec l'identifiant d'intention
  — `crates/interfaces/src/bin/otter_api.rs:1000-1002,1013-1017`.

**Points résiduels / axes d'amélioration.**

- Pas d'alerte spécifique sur les événements de sécurité (échecs
  d'authentification répétés, 429, `InvalidProof`) : ces événements sont
  journalisés au mieux au niveau debug/info, sans corrélation ni seuil
  d'alerte.
- Les échecs d'authentification renvoyés au client (`otter_api.rs:460`) ne
  sont pas journalisés côté serveur.
- Pas d'audit formel de ce qui est loggué : risque théorique de fuite de
  données sensibles dans les logs (aucun secret n'est loggué dans le code
  relu, mais ce n'est pas garanti par un mécanisme).

---

## A10:2021 — Falsification de requête côté serveur (SSRF)

**Description du risque.** Le serveur est amené à émettre des requêtes vers
des destinations contrôlées par un attaquant (réseau interne, métadonnées
cloud, etc.).

**Mesures implémentées.**

- *Destinations réseau configurées par l'opérateur, pas par l'utilisateur.*
  L'URL du nœud RPC Ethereum (`OTTER_RPC_URL`), l'adresse Vault, les chemins
  d'outillage et l'adresse du LLM proviennent de la configuration serveur
  — `.env.example:2,6-9,34-37`. Aucune route API n'accepte d'URL de
  destination fournie par le client.
- *Whitelist on-chain des routeurs.* Même côté contrat, les transferts ERC-20
  d'exécution ne partent que vers des adresses de routeurs enregistrées par le
  propriétaire — `contracts/src/DelegationVault.sol:95-99,246-249`, ce qui
  borne l'équivalent « sortant » des appels externes.
- *Validation des champs de délégation.* Les adresses et montants fournis à
  l'API sont validés en format (hex) et en cardinalité avant tout usage
  — `crates/interfaces/src/bin/otter_api.rs:1049-1079`.

**Points résiduels / axes d'amélioration.**

- Le `target_contract` d'une délégation est choisi par l'utilisateur (par
  conception, c'est sa délégation) ; la contrainte est vérifiée dans le
  circuit (`delegation_circuit/src/main.nr:158-162`) mais la politique de ce
  qui est une cible « légitime » reste hors du code.
- Le texte d'intention est interprété par un LLM qui peut déclencher des
  appels sortants (protocoles) : la borne on-chain (protocoles whitelistés,
  montants) limite l'impact, mais il n'y a pas de filtrage réseau dédié (ex.
  egress proxy) documenté.
- Le provider HashiCorp Vault interroge une URL configurable
  (`crates/interfaces/src/secrets.rs:44-58`) : comme toute configuration
  opérateur, elle doit être protégée contre une modification malveillante.

---

## Tableau de synthèse

| Risque OWASP 2021 | Mesure principale | Preuve (fichier) | Statut |
|---|---|---|---|
| A01 — Contrôles d'accès | Middleware JWT sur routes protégées ; `onlyOwner` ; preuve ZK obligatoire avant exécution | `crates/interfaces/src/bin/otter_api.rs:383-402,431-464` ; `contracts/src/DelegationVault.sol:95-99,176-210` | Implémenté (RBAC et multisig absents) |
| A02 — Crypto | SIWE + JWT HS256 ; ECDSA/blake2s/ZK on-chain ; `SecretProvider` (fichier 0600, Vault, KMS, keystore) ; config TLS de référence versionnée | `crates/interfaces/src/auth.rs` ; `delegation_circuit/src/main.nr:140-146` ; `crates/interfaces/src/secrets.rs` ; `deploy/Caddyfile` | Partiel : terminaison TLS externe (config de référence fournie), secret JWT aléatoire en dev |
| A03 — Injection | SQL paramétré (rusqlite/sqlx) ; serde typé ; `MAX_INTENT_TEXT_LEN = 2000` | `crates/infrastructure/src/storage/sqlite.rs:144-160` ; `crates/interfaces/src/bin/otter_api.rs:1034-1047` | Implémenté (prompt injection LLM non traitée) |
| A04 — Conception | Limites on-chain + double vérification (circuit + contrat) ; anti-rejeu nonce | `contracts/src/DelegationVault.sol:193-204` ; `delegation_circuit/src/main.nr:148-168` | Implémenté (pas de circuit breaker) |
| A05 — Configuration | CORS whitelist ; rate limiting/IP (défaut 100/min) ; secrets CI dans GitHub Secrets ; config TLS de référence avec headers de sécurité | `crates/interfaces/src/bin/otter_api.rs:411-429,476-508` ; `.env.example:11-19` ; `deploy/Caddyfile` | Partiel : défauts permissifs en dev (auth off, CORS `*`) |
| A06 — Composants | OpenZeppelin ; versions épinglées ; clippy `-D warnings` en CI ; job `cargo audit` bloquant en CI (ignores justifiés) ; Dependabot hebdomadaire | `contracts/src/DelegationVault.sol:5-7` ; `.github/workflows/ci.yml` (job `security-audit`) ; `.github/dependabot.yml` | Implémenté : job d'audit bloquant ; `sqlx` corrigé (0.8.6) ; 3 ignores justifiés (`rsa`, `alloy-dyn-abi`, `rustls-webpki` via `aws-kms`) avec exposition nulle |
| A07 — Authentification | EIP-4361 sans mot de passe ; challenges 16 octets/5 min à usage unique ; JWT court | `crates/interfaces/src/auth.rs:69-155,170-184` | Implémenté (challenges en mémoire, pas de révocation) |
| A08 — Intégrité | Preuve ZK liant toutes les entrées ; hash blake2s recalculé ; `Cargo.lock` commité | `delegation_circuit/src/main.nr:135-146` ; `contracts/src/DelegationVault.sol:179-182` | Partiel : artefacts non signés, pas de SBOM |
| A09 — Journalisation | `tracing` structuré ; `/health` `/ready` `/metrics` ; événements on-chain | `crates/interfaces/src/bin/otter_api.rs:380-381` ; `contracts/src/DelegationVault.sol:62-74` | Partiel : pas d'alerte sur les événements de sécurité |
| A10 — SSRF | Destinations réseau côté configuration uniquement ; routeurs whitelistés on-chain | `.env.example:2,34-37` ; `contracts/src/DelegationVault.sol:95-99,246-249` | Implémenté (pas de filtrage egress documenté) |

---

*Document rédigé pour le dossier de certification RNCP. Les références de
lignes correspondent à l'état du dépôt au moment de la rédaction ; elles
peuvent évoluer avec le code.*
