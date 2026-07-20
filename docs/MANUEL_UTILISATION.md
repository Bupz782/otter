# Manuel d'utilisation — Otter

> Application d'automatisation DeFi : vous décrivez une stratégie en langage
> courant, vous signez une délégation limitée, et un agent l'exécute
> automatiquement avec une preuve à divulgation nulle (ZKP).

Ce manuel s'adresse à un utilisateur non-développeur. Il explique comment
installer, lancer et utiliser l'application pas à pas.

---

## 1. Présentation et concepts clés

### 1.1 À quoi sert Otter ?

Otter automatise vos opérations de finance décentralisée (DeFi). Au lieu de
surveiller vous-même les marchés et de signer chaque transaction, vous :

1. décrivez ce que vous voulez en langage courant (par exemple
   *« prête 1000 USDC sur Aave si le rendement dépasse 3 % »*) ;
2. fixez des limites signées avec votre wallet (montants, protocoles, durée) ;
3. laissez un agent surveiller la condition et exécuter l'action pour vous.

Chaque exécution est accompagnée d'une preuve cryptographique qui garantit que
l'agent est resté dans les limites que vous avez signées. L'agent ne peut pas
tricher : la preuve serait refusée par le contrat.

### 1.2 Les cinq concepts à connaître

**L'intention (intent).** C'est la règle que vous écrivez en langage courant :
une action (prêter, échanger, retirer, réclamer), un montant, un actif, un
protocole, et éventuellement une condition (rendement, prix). Exemple :
*« Lend 1000 USDC on Aave if yield > 3% »*.

**Le plan d'exécution (plan).** Une fois votre phrase comprise, Otter la
traduit en une suite d'étapes techniques : vérifier la condition sur la
blockchain, préparer la transaction, prouver, exécuter. Vous validez ce plan
avant de lancer quoi que ce soit.

**La délégation.** C'est l'autorisation que vous signez avec votre wallet. Elle
définit des limites strictes : montant maximum par type d'action, protocoles
autorisés (Aave, Compound, Uniswap), date d'expiration. L'agent ne peut agir
qu'à l'intérieur de ces limites. Vos clés restent dans votre wallet : Otter ne
détient jamais vos fonds ni vos clés privées.

**L'exécution.** Quand la condition de votre intention est remplie (par exemple
le rendement Aave dépasse 3 %), l'agent exécute l'action sur la blockchain. Il
peut aussi capturer une partie du « MEV » (valeur extraite des transactions) et
vous la reverser sous forme de rabais d'exécution.

**La preuve à divulgation nulle (preuve ZK).** Avant chaque exécution, l'agent
génère une preuve mathématique (circuit Noir + Barretenberg) attestant que
l'action respecte votre délégation — sans révéler d'information inutile. Le
contrat `DelegationVault` vérifie cette preuve sur la blockchain avant de
laisser passer la transaction. La preuve reste vérifiable par n'importe qui,
pour toujours.

### 1.3 Le cycle de vie d'une intention

```
Décrire → Vérifier le plan → Choisir une délégation → Confirmer
        → Surveillance (Monitoring) → Condition remplie (Condition Met)
        → Génération de la preuve (Proving) → Transaction envoyée (Submitted)
        → Confirmée (Confirmed)   ou   Échouée (Failed)
```

Vous pouvez annuler une intention tant qu'elle n'est pas terminée : elle passe
alors au statut « Revoked ».

---

## 2. Prérequis

| Élément | Détail |
|---|---|
| **Navigateur** | Un navigateur récent (Chrome, Firefox, Brave…). |
| **Wallet Ethereum** | Une extension wallet compatible, par exemple MetaMask. La connexion se fait via RainbowKit, qui accepte la plupart des wallets (WalletConnect inclus). |
| **Réseau** | L'application est configurée pour le réseau de test **Sepolia** uniquement. Votre wallet doit être connecté à Sepolia. |
| **Fonds de test** | Des ETH Sepolia (testnet) pour payer les frais de transaction. Ils s'obtiennent gratuitement sur un « faucet » Sepolia. Ces fonds n'ont aucune valeur réelle. |
| **Docker** (installation locale) | Docker Engine + Docker Compose v2, pour lancer l'application complète en local. |

---

## 3. Installation et accès

### 3.1 Démarrage local avec Docker Compose (recommandé)

Le fichier `docker-compose.yml` lance trois services : une base PostgreSQL,
l'API Rust et le frontend web.

```bash
# 1. Préparer la configuration
cp .env.example .env
#    (éditez .env si besoin ; les valeurs par défaut pointent vers Sepolia)

# 2. Construire et démarrer tout
docker compose up --build

# Variante en arrière-plan :
docker compose up -d --build

# 3. Suivre les logs
docker compose logs -f
```

Une fois démarré :

- **Interface web** : http://localhost:3000
- **API** : http://localhost:3001
- **Métriques** : http://localhost:3001/metrics

Pour vérifier que l'API est prête :

```bash
curl -fsS http://localhost:3001/ready
```

Pour arrêter :

```bash
docker compose down
```

### 3.2 Accès au testnet Sepolia

L'application est pensée pour Sepolia (chain ID `11155111`). La configuration
par défaut de `.env.example` utilise le RPC public
`https://ethereum-sepolia-rpc.publicnode.com`.

> ⚠️ Les adresses de contrats de la démo V1 documentées dans `DEPLOYMENT.md`
> sont **obsolètes** : le circuit Noir a changé (ajout de `target_contract`),
> ce qui a modifié la clé de vérification. Un nouveau déploiement des contrats
> est nécessaire et l'adresse du vault doit être renseignée dans
> `OTTER_VAULT_ADDRESS`. En attendant, l'interface web reste utilisable en
> mode démonstration (voir §4.2).

Le déploiement complet des contrats sur Sepolia (script Foundry, variables
d'environnement, clé de l'agent) est une opération technique décrite dans
`DEPLOYMENT.md` ; elle n'est pas nécessaire pour découvrir l'interface.

---

## 4. Guide pas à pas des écrans

L'interface se compose d'une **page d'accueil** (`/`) et de l'**application**
proprement dite, accessible sous `/app`. L'application comporte une barre
latérale avec quatre groupes : *Manage* (Dashboard, Intents, Delegations),
*Discover* (Otter Agents), *Verify* (Proofs), *System* (Settings).

### 4.1 Page d'accueil (`/`)

C'est la vitrine du produit : présentation du fonctionnement (schéma du flux),
aperçu de démonstration, cas d'usage, intentions en direct, section de
confiance, FAQ et liste d'attente. Le menu **Help** de l'application permet d'y
revenir à tout moment (*Back to landing*).

### 4.2 Connexion du wallet et mode démo

En haut à droite de l'en-tête se trouve le bouton de connexion (RainbowKit).

1. Cliquez sur le bouton **Connect** et choisissez votre wallet (MetaMask,
   WalletConnect…).
2. Vérifiez que le wallet est sur le réseau **Sepolia** (icône de chaîne dans
   le bouton).
3. Une fois le wallet connecté, un bouton **Sign In** apparaît. Cliquez-le :
   votre wallet vous demande de signer un message « Sign in to Otter agent ».
   Cette signature ne coûte rien et ne déplace aucun fonds : elle prouve que
   vous êtes bien le propriétaire de l'adresse et ouvre votre session.

Le bas de la barre latérale indique l'état de la connexion :

- point gris *Not connected* : aucun wallet connecté ;
- point orange + adresse : wallet connecté mais session non signée ;
- point vert + adresse : connecté et authentifié.

**Mode démo.** Tant que vous n'êtes pas authentifié, un badge orange
*« Demo data »* s'affiche dans l'en-tête et les pages montrent des données de
démonstration. Vous pouvez parcourir tout le parcours (création d'intention,
délégation) mais les boutons finaux sont remplacés par un encadré orange
*« Connect wallet to set it live. »* / *« Connect wallet to sign. »*.

**Erreurs possibles :** si la signature échoue (refus dans le wallet, perte
réseau), un message d'erreur rouge s'affiche à côté du bouton Sign In. Si vous
déconnectez le wallet, la session est fermée automatiquement.

### 4.3 Dashboard (`/app/dashboard`)

C'est l'écran d'accueil de l'application : « What your capital is doing right
now. »

Vous y voyez :

- le **solde total du vault**, avec la répartition *Allocated* (fonds engagés)
  / *Available* (fonds disponibles), le rendement gagné (*Yield earned*) et les
  rabais d'exécution (*Execution rebates*) ;
- l'**activité récente** (dépôts, exécutions, rabais MEV…) ;
- quatre cartes de synthèse : Allocated, Yield earned, Execution rebates,
  Active intents ;
- la liste des **intentions actives** (avec leur statut coloré) ;
- vos **positions** ouvertes (actif, protocole, chaîne, montant, APY).

Le bouton **Create intent** en haut à droite mène à la création d'intention.
Un bouton **Take the tour** (menu Help ou page Settings) relance la visite
guidée en cinq étapes du tableau de bord.

**Erreurs possibles :** si l'API est injoignable, chaque bloc affiche un état
d'erreur avec un bouton *Retry*.

### 4.4 Créer une intention (`/app/intents/new`)

Un assistant en **quatre étapes** (indiquées par un stepper en haut de page) :
**Describe → Review → Delegate → Confirm**.

**Étape 1 — Describe (« What do you want to do? »).** Écrivez votre intention
en langage courant dans la zone de texte, ou cliquez sur un des exemples
proposés :

- *Lend 1000 USDC on Aave if yield > 3%*
- *Swap 500 USDC to ETH on Uniswap when gas < 20 gwei*
- *Claim Aave rewards every Monday*
- *Withdraw 2000 USDC from Compound if utilization > 85%*

Cliquez ensuite sur **Parse intent**. Le bouton est grisé tant que le champ
est vide. Pendant l'analyse, il affiche *Parsing…*.

**Erreur possible :** *« Couldn't parse that intent. Try rephrasing. »* — la
phrase n'a pas été comprise. Reformulez en gardant la structure
action + montant + actif + protocole (+ condition).

**Étape 2 — Review (« How Otter read it »).** Otter affiche sa lecture de
votre phrase : Action, Montant, Actif, Protocole, Chaîne, Condition. **Tous
les champs sauf l'action sont modifiables** : corrigez ici toute erreur de
compréhension avant de continuer. Cliquez sur **Choose delegation** (ou
**Back** pour réécrire la phrase).

**Étape 3 — Delegate (« Choose a delegation »).** Choisissez la délégation
signée sous laquelle l'intention s'exécutera. Chaque carte rappelle l'agent,
ses limites (montant max, protocoles) et sa date d'expiration.

- Si vous n'avez **aucune délégation**, un message *« No active delegations »*
  propose le bouton **Create delegation** (voir §4.6).
- Si votre intention dépasse les limites de la délégation choisie, un encadré
  orange *« Heads up: this intent may not run under the selected delegation. »*
  liste les incompatibilités (protocole non autorisé, montant supérieur à la
  limite). Vous pouvez continuer, mais l'exécution échouera : préférez corriger
  l'intention ou choisir une autre délégation.

Cliquez sur **Confirm delegation**.

**Étape 4 — Confirm (« One last look before it goes live »).** Récapitulatif
complet : phrase d'origine, action, montant, protocole, chaîne, condition,
délégation. Une note rappelle qu'Otter prouvera le respect de vos limites avant
toute exécution. Cliquez sur **Set intent**. Le bouton affiche *Creating…*
puis *Created*, et vous êtes redirigé vers la liste des intentions.

**Erreur possible :** *« Couldn't create the intent. Try again. »*

### 4.5 Suivre ses intentions (`/app/intents` et `/app/intents/:id`)

La page **Intents** liste toutes vos règles. Des filtres en haut permettent
de n'afficher qu'un statut : All, Monitoring, Condition met, Proving,
Confirmed, Failed.

Signification des statuts :

| Statut | Signification |
|---|---|
| **Monitoring** (orange) | Otter surveille la condition. |
| **Condition Met** | La condition vient d'être remplie. |
| **Proving** | La preuve ZK est en cours de génération. |
| **Submitted** | La transaction est envoyée, en attente de confirmation. |
| **Confirmed** (vert) | Exécution confirmée sur la blockchain. |
| **Failed** (rouge) | L'exécution a échoué. |
| **Revoked** (grisé) | Vous avez annulé l'intention. |

Cliquez sur une intention pour ouvrir sa **page de détail** :

- **Parameters** : ce qu'Otter a compris de votre phrase.
- **Transaction** (si exécutée) : le hash de la transaction, un bouton pour le
  copier, un lien vers l'explorateur de blocs (Etherscan ou Arbiscan selon la
  chaîne), et le rabais d'exécution éventuel (*+X USDC execution rebate*).
- **Execution** : une frise chronologique animée du cycle de vie. Si aucune
  exécution n'est enregistrée, un message l'indique clairement (*« Otter
  checks the condition and executes when it hits. »*).

**Annuler une intention.** Tant que le statut n'est pas terminal (Confirmed,
Failed, Revoked), le bouton **Cancel intent** est disponible en bas du panneau
Execution. Une confirmation vous est demandée (*« Cancel this intent? Otter
stops monitoring it. »* → **Confirm cancel** ou **Keep it**).
**Erreur possible :** *« Couldn't cancel the intent. Try again. »*

**Erreurs possibles :** si l'identifiant n'existe pas, la page affiche
*« Intent not found »* avec un lien de retour. Si le serveur ne répond pas,
un état d'erreur avec *Retry* s'affiche.

### 4.6 Créer une délégation (`/app/delegations/new`)

Formulaire en cinq sections :

1. **Agent** : choisissez l'agent Otter qui exécutera pour vous. Chaque carte
   indique sa réputation (étoiles) et son nombre de preuves soumises.
2. **Limits** : montant maximum (en USDC) que l'agent peut mouvementer par
   type d'action — *lend*, *swap*, *withdraw*, *claim*.
3. **Protocols & chains** : cochez les protocoles autorisés (Aave, Compound,
   Uniswap) et les chaînes (Ethereum, Arbitrum). Tout ce qui n'est pas coché
   reste interdit. Note : la version actuelle du circuit ne fait respecter que
   les montants et les protocoles ; les chaînes ne font pas encore partie du
   message signé.
4. **Expiry** : durée de validité de la délégation, en jours (minimum 1).
5. **Review & sign** : récapitulatif de ce que votre signature engage. Cliquez
   sur **Sign delegation** ; le bouton affiche *Signing…* puis *Delegated* et
   vous êtes redirigé vers la liste.

**Messages d'erreur de validation :**

- *« Limits must be numbers above 0. »* — une limite est vide ou négative ;
- *« Pick at least one protocol. »* ;
- *« Pick at least one chain. »* ;
- *« Expiry must be at least 1 day. »* ;
- *« Couldn't sign the delegation. Try again. »* — échec à l'envoi.

Le bouton **Sign delegation** reste grisé tant que le formulaire est incomplet.

### 4.7 Liste des délégations (`/app/delegations`)

Liste vos délégations signées : identifiant (hash tronqué), date de signature,
agent concerné, protocoles et chaînes autorisés (badges), et statut
(*Active*, *Expired* ou *Revoked*). Le bouton **New delegation** ouvre le
formulaire de création.

> Note : il n'existe pas de bouton de révocation dans l'interface actuelle —
> l'API ne propose pas de point de terminaison de suppression de délégation.
> La protection passe par la date d'expiration choisie à la création.

Si la liste est vide : *« No delegations yet »* avec un bouton **Create your
first delegation**.

### 4.8 Agents et stratégies (`/app/agents` et `/app/agents/:id`)

La page **Otter Agents** présente les agents vérifiés du protocole (chacun est
opéré par le protocole, doté d'une caution et audité). Pour chaque agent :
profil de risque (*Conservative*, *Balanced*, *Advanced*), rendement routé,
nombre de preuves, disponibilité (*Uptime*) et rabais capturés.

La page contient aussi :

- **Official strategies** : stratégies pré-écrites et auditées. Le bouton
  **Use strategy** pré-remplit le formulaire de création d'intention avec le
  texte de la stratégie.
- **Agent Leaderboard** : les cinq premiers agents classés par nombre de
  preuves.

La **page de détail d'un agent** affiche ses statistiques (rendement généré,
preuves soumises, rabais capturés, uptime) et sa fiche (*Operated by*, profil
de risque, caution, réputation, nombre de délégataires, stratégies). Le bouton
**Create delegation** ouvre le formulaire de délégation avec cet agent
présélectionné.

### 4.9 Preuves (`/app/proofs`)

Le registre de toutes les preuves : délégation, exécution et solvabilité du
vault. En haut, un bandeau **Vault solvency** indique le résultat de la
dernière preuve de solvabilité (le vault prouve que ses actifs couvrent les
dépôts, sans révéler les soldes individuels).

Chaque ligne affiche le type de preuve, le circuit vérificateur, le nombre de
contraintes, le temps de génération et un badge **Verified** (vert) ou
**Invalid** (rouge). Cliquez sur la flèche à droite d'une ligne pour déplier
le détail : type, circuit, contraintes, temps de génération, horodatage, lien
vers l'intention concernée et hash de transaction.

### 4.10 Paramètres (`/app/settings`)

Trois sections :

- **Wallet** : adresse connectée et réseau. Rappel : les signatures se font
  dans votre wallet ; Otter ne détient jamais vos clés.
- **Tour** : bouton **Take the tour** pour rejouer la visite guidée en cinq
  étapes.
- **Session** : bouton **Sign out** pour fermer la session. La fermeture lie
  la déconnexion de vos intentions et délégations à ce navigateur ; vos données
  on-chain ne sont pas affectées.

---

## 5. API REST (utilisateurs avancés)

L'API Rust (`metis_api`) écoute par défaut sur le port **3001**. Les routes
métier sont préfixées par `/api/v1`.

### 5.1 Authentification

Quand l'authentification est activée, les routes métier exigent un jeton
Bearer. On l'obtient en deux appels :

```bash
# 1. Demander un challenge de signature pour votre adresse
curl -X POST http://localhost:3001/api/v1/auth/challenge \
  -H 'Content-Type: application/json' \
  -d '{"address":"0xVotreAdresse"}'
# → {"message":"Sign in to Otter agent ... Nonce: ..."}

# 2. Signer ce message avec votre wallet, puis l'envoyer
curl -X POST http://localhost:3001/api/v1/auth/verify \
  -H 'Content-Type: application/json' \
  -d '{"message":"<message signé>","signature":"0x..."}'
# → {"token":"..."}

# Utilisation : -H "Authorization: Bearer <token>"
```

### 5.2 Routes principales

**Santé et supervision (publiques) :**

```bash
curl http://localhost:3001/health       # état simple
curl http://localhost:3001/ready        # disponibilité (healthcheck Docker)
curl http://localhost:3001/metrics      # métriques Prometheus
```

**Intentions :**

```bash
# Analyser une phrase sans rien créer
curl -X POST http://localhost:3001/api/v1/intents/parse \
  -H 'Content-Type: application/json' \
  -d '{"text":"lend 1000 USDC on Aave"}'

# Construire le plan d'exécution
curl -X POST http://localhost:3001/api/v1/intents/plan \
  -H 'Content-Type: application/json' \
  -d '{"text":"lend 1000 USDC on Aave if yield > 3"}'

# Créer une intention
curl -X POST http://localhost:3001/api/v1/intents \
  -H 'Content-Type: application/json' \
  -d '{"text":"swap 1 ETH for USDC on Uniswap"}'

# Lister les intentions actives
curl http://localhost:3001/api/v1/intents

# Consulter / supprimer une intention
curl http://localhost:3001/api/v1/intents/<id>
curl -X DELETE http://localhost:3001/api/v1/intents/<id>
```

**Délégations :**

```bash
curl http://localhost:3001/api/v1/delegation          # lister
curl -X POST http://localhost:3001/api/v1/delegation  # enregistrer une délégation signée
curl -X POST http://localhost:3001/api/v1/delegation/hash  # calculer le hash d'une délégation
```

**Suivi et référentiels :**

```bash
curl http://localhost:3001/api/v1/orchestrator/state  # état de l'orchestrateur
curl http://localhost:3001/api/v1/executions          # historique des exécutions
curl http://localhost:3001/api/v1/proofs              # preuves enregistrées
curl http://localhost:3001/api/v1/portfolio           # portefeuille / vault
curl http://localhost:3001/api/v1/agents              # agents
curl http://localhost:3001/api/v1/agents/<id>         # détail d'un agent
curl http://localhost:3001/api/v1/strategies          # stratégies publiées
curl http://localhost:3001/api/v1/leaderboard         # classement des agents
```

Un flux WebSocket d'événements temps réel est disponible sur `/api/v1/ws`.

---

## 6. Ligne de commande `metis_cli`

Le binaire CLI permet de piloter le moteur sans l'interface web. Commandes
réelles (`metis_cli --help` les liste) :

```bash
# Analyser une intention et afficher le résultat structuré
cargo run -p interfaces --bin metis_cli -- parse "lend 1000 USDC on Aave"

# Construire le plan d'exécution
cargo run -p interfaces --bin metis_cli -- plan "swap 1 ETH for USDC on Uniswap"

# Lancer le démon de surveillance d'une condition (réseau Sepolia par défaut)
cargo run -p interfaces --bin metis_cli -- start \
  "lend 1000 USDC on Aave if yield > 3" \
  --network sepolia --interval 60

# Afficher l'état du démon et les intentions actives
cargo run -p interfaces --bin metis_cli -- status
```

**Exécution complète.** La commande `execute` enchaîne analyse → condition →
preuve → soumission on-chain. Sans `--vault`, elle tourne en mode simulé
(mock) ; avec `--vault`, `--private-key` et `--rpc-url`, elle génère une vraie
preuve Noir/Barretenberg et appelle `executeWithProof` sur le vault :

```bash
# Mode simulé (aucune blockchain requise)
cargo run -p interfaces --bin metis_cli -- execute "swap 1000 USDC for ETH on Uniswap"

# Mode on-chain (nœud local Anvil, vault déployé)
cargo run -p interfaces --bin metis_cli -- execute \
  "swap 1000 USDC for ETH on Uniswap" \
  --rpc-url http://localhost:8545 \
  --private-key 0x... \
  --vault $VAULT \
  --delegate
```

L'option `--delegate` enregistre la délégation on-chain avant l'exécution.

**Preuves manuelles :**

```bash
# Générer une preuve (écrit proof.bin et public_inputs.bin)
cargo run -p interfaces --bin metis_cli -- prove \
  "lend 1000 USDC on Aave" \
  --private-key 0x... \
  --output-dir ./tmp

# Vérifier une preuve on-chain (appel en lecture contre le vérificateur du vault)
cargo run -p interfaces --bin metis_cli -- verify-onchain \
  --proof ./tmp/proof.bin \
  --public-inputs ./tmp/public_inputs.bin \
  --rpc-url http://localhost:8545 \
  --vault $VAULT \
  --private-key 0x...
```

La clé privée peut aussi être fournie par la variable d'environnement
`OTTER_PRIVATE_KEY`. Une démo de bout en bout automatisée (nœud Anvil, déploiement,
dépôt, preuve réelle) est fournie par le script `./lab/zkp_e2e.sh`.

---

## 7. FAQ et dépannage

### Questions fréquentes

**Mon wallet est connecté mais l'application affiche « Demo data ».**
Vous n'êtes pas authentifié. Cliquez sur **Sign In** dans l'en-tête et signez
le message dans votre wallet. Le badge disparaît une fois la session ouverte.

**L'application ne comprend pas mon intention
(« Couldn't parse that intent. Try rephrasing. »).**
Reformulez avec la structure action + montant + actif + protocole, en anglais
si besoin (les exemples fournis sont en anglais) : *« Lend 1000 USDC on Aave if
yield > 3% »*. À l'étape Review, corrigez à la main les champs mal compris.

**Mon intention reste en « Monitoring » et ne s'exécute jamais.**
La condition n'est tout simplement pas remplie (par exemple le rendement Aave
est sous votre seuil de 3 %). Vérifiez aussi côté serveur que l'exécution est
activée : si `OTTER_EXECUTION_ENABLED=false`, l'agent surveille mais n'exécute
rien (c'est le réglage par défaut de `.env.example`).

**L'avertissement « this intent may not run under the selected delegation »
s'affiche.**
Votre intention dépasse les limites signées : protocole non autorisé ou montant
supérieur à la limite du type d'action. Réduisez le montant, changez de
protocole, ou créez une délégation plus large.

**La génération de la preuve prend plus de 30 secondes.**
C'est un comportement connu sur les machines lentes : la supervision déclenche
l'alerte `OtterProofPipelineSlow` au-delà de 30 s. La preuve finit en général
par aboutir ; si elle échoue, l'intention passe en *Failed* et vous pouvez la
recréer.

**Puis-je révoquer une délégation ?**
Pas depuis l'interface actuelle (l'API n'a pas de route de suppression de
délégation). Choisissez une date d'expiration courte à la création ; passée
cette date, la délégation n'est plus valable.

**Otter peut-il déplacer mes fonds sans mon accord ?**
Non. L'agent ne peut agir qu'à l'intérieur des limites de votre délégation
signée, et chaque exécution exige une preuve ZK vérifiée par le contrat
`DelegationVault`. Vos clés privées restent dans votre wallet.

### Problèmes techniques (côté serveur)

Ces symptômes concernent la personne qui héberge la stack Docker.

| Symptôme | Cause probable | Solution |
|---|---|---|
| Le healthcheck `/ready` échoue | PostgreSQL injoignable ou migrations en échec | Vérifier que le service `postgres` est healthy et joignable par l'API sur le port 5432 (`docker compose ps`, `docker compose logs postgres`). |
| `bb prove failed` dans les logs | Binaire `bb` absent ou incompatible | Vérifier que `OTTER_BB_BIN` pointe vers un `bb` compilé pour la même version de Noir que le circuit. |
| `delegate()` reverts | Mauvaise adresse de vault ou nonce réutilisé | Vérifier `OTTER_VAULT_ADDRESS` et que la clé de l'agent a des ETH. Contrôler le fichier de nonce (`/data/otter-nonce.txt`). |
| Aucune intention ne s'exécute | `OTTER_EXECUTION_ENABLED=false` ou condition jamais remplie | Activer l'exécution et surveiller `/metrics` et les logs (`docker compose logs -f api`). |
| `nargo execute` introuvable | Binaire hôte mal monté | Vérifier le chemin `OTTER_NARGO_BIN` (binaire présent et exécutable). |
| Alerte `OtterHighErrorRate` | RPC qui limite le débit (throttling) | Utiliser un endpoint RPC dédié (Alchemy/Infura) plutôt qu'un nœud public. |

### Alertes de supervision (Prometheus)

Le fichier `alerting.yml` définit les alertes suivantes, utiles pour
comprendre un comportement anormal :

- `OtterAgentDown` — l'API ne répond plus depuis plus d'une minute ;
- `OtterNoPriceUpdates` — plus de mise à jour de prix depuis 10 minutes
  (vérifier la connectivité RPC et l'oracle) ;
- `OtterExecutionStalled` — des conditions sont remplies mais aucune exécution
  n'aboutit (vérifier le solde ETH de l'agent et l'adresse du vault) ;
- `OtterProofPipelineSlow` — la dernière génération de preuve a dépassé 30 s ;
- `OtterAgentLowBalance` — le solde ETH de l'agent passe sous 0,01 ETH :
  recharger le wallet de l'agent ;
- `OtterProofVerificationFailing` — la vérification on-chain des preuves
  échoue (circuit et vérificateur probablement désynchronisés) ;
- `OtterRpcUnhealthy` — trop d'erreurs RPC : changer d'endpoint.

Commandes de diagnostic rapide :

```bash
curl -fsS http://localhost:3001/ready      # l'API est-elle prête ?
curl -fsS http://localhost:3001/metrics    # compteurs internes
docker compose logs -f api                 # logs applicatifs
```
