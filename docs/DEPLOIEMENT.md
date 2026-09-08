# Déploiement Otter — guide et estimation de coûts

> État au 08/09/2026. Tous les chiffres de gas sont **mesurés** (broadcast réel
> sur anvil + exécution confirmée on-chain), pas estimés. Conversion
> ETH = 2 100 € (début sept. 2026) — la formule est donnée pour recalculer.

## 1. Ce qu'il y a à déployer

| Composant | Type | Où |
|---|---|---|
| Frontend (Vite/React) | Statique (CDN) | Vercel / Cloudflare Pages / S3 |
| API `otter_api` (Rust + bb + nargo) | Container | VPS / Kubernetes / Fly.io |
| Base de données | SQLite (volume) ou Postgres | VPS / managé |
| Contrats : DelegationVerifier + DelegationVault | EVM | L2 (Base, Arbitrum…) — voir §5 |
| Contrats : SolvencyVerifier + SolvencyRegistry | EVM | idem |
| Contrats : OtterBridge + BridgeToken | EVM | idem, par chaîne bridgée |
| Circuit Noir + fixtures bb | Embarqué dans l'image API | — |

L'agent (clé signante) vit côté API : c'est lui qui paie le gas des
exécutions. Le user ne signe que des délégations (gratuit, pas de tx).

## 2. Local (déjà opérationnel)

```
docker compose up --build -d          # anvil + api + frontend
# après un reset anvil UNIQUEMENT :
docker compose stop api
docker compose restart anvil
cd contracts && forge script script/DeployLocalStack.s.sol \
  --rpc-url http://localhost:8545 --private-key <clé-anvil-0> --broadcast
docker compose up -d api
```

`DeployLocalStack.s.sol` déploie les 7 contrats en un seul broadcast avec des
adresses déterministes (documentées dans son header et épinglées dans
`docker-compose.yml`). L'API doit être stoppée pendant le déploiement :
l'orchestrateur consomme des nonces avec la même clé, ce qui décalerait
toutes les adresses.

## 3. Testnet publique (étape suivante recommandée)

Cible conseillée : **Base Sepolia** (L2, faucet gratuit, explorer, mêmes
coûts réels que la mainnet L2 à un facteur près).

1. Secrets : clé dédiée testnet dans KMS ou `.env` hors git ; jamais la clé
   anvil. RPC payant ou free tier (Alchemy/QuickNode Base Sepolia).
2. Contrats : rejouer `DeployLocalStack.s.sol` avec
   `--rpc-url $BASE_SEPOLIA_RPC --broadcast --verify --etherscan-api-key …`
   (sans TestToken si on veut un ERC-20 réel : passer une adresse existante
   en `UNDERLYING_ADDRESS` à `DeployOtterBridge.s.sol`).
3. API : `OTTER_PROFILE=prod` (validate() refuse alors auth off, CORS `*` et
   clé hex en clair en ligne),
   `OTTER_NETWORKS=default=<rpc>|<vault>|84532|<bridge>`,
   `OTTER_AUTH_ENABLED=true`, CORS restreint au domaine du frontend.
4. Frontend : build statique avec `VITE_API_URL` pointant l'API, déployé sur
   Vercel/Cloudflare.
5. Vérifications : run de `scripts/demo.sh` adapté testnet + un intent
   « gas < N » créé via l'UI exécuté de bout en bout.

## 4. Mainnet — gates obligatoires (pas avant)

- Audit externe contrats + circuit (voir §5) et correctifs mergés.
- `OTTER_PROFILE=prod` validé (auth, CORS, KMS obligatoires).
- Owner des contrats = multisig (Safe), pas une EOA ; pausable si ajouté.
- 2 semaines de testnet sans incident + monitoring/alerting en place.
- Bridge : owner-gated actuel inacceptable en mainnet → multisig relayers
  (V1.5) minimum, voir `docs/BRIDGE.md` §V2.

## 5. Coûts

### 5.1 Déploiement des contrats (one-time, gas mesuré)

| Contrat | Gas mesuré |
|---|---:|
| RelationsLib ×2 (libs Honk) | 5 254 956 |
| ZKTranscriptLib | 2 386 004 |
| DelegationVerifier | 5 072 801 |
| DelegationVault | 2 567 968 |
| SolvencyVerifier | 5 189 337 |
| SolvencyRegistry | 529 026 |
| OtterBridge + BridgeToken + wiring | 2 353 251 |
| **Total prod** | **≈ 23 353 000** |

Formule : coût(€) = gas × gwei × 1e-9 × prix_ETH.

| Réseau | Hypothèse gas price | Coût déploiement |
|---|---|---:|
| Ethereum L1 | 10 gwei | ≈ 490 € |
| Ethereum L1 | 30 gwei | ≈ 1 470 € |
| Base / Arbitrum | 0,05 gwei | ≈ 2,50 € |
| Base / Arbitrum | 1 gwei (pic) | ≈ 49 € |

### 5.2 Par exécution d'intent (le chiffre qui décide de l'architecture)

`executeWithProof` (vérification ZK incluse) : **2 114 141 gas mesuré**.

| Réseau | Hypothèse | Coût / exécution |
|---|---|---:|
| Ethereum L1 | 10 gwei | ≈ 44 € |
| Ethereum L1 | 30 gwei | ≈ 133 € |
| Base / Arbitrum | 0,05 gwei | ≈ 0,22 € |
| Base / Arbitrum | 1 gwei (pic) | ≈ 4,40 € |

**Conclusion : L1 exclu pour l'exécution** — le produit doit vivre sur L2.
La génération de preuve bb (~0,5 s CPU mesuré) se fait côté serveur, pas de
GPU nécessaire. Optimisation future documentée : vérification via
`ecrecover`-style precompile-friendly proof system ou batching d'intents.

### 5.3 Infrastructure mensuelle

| Poste | Beta testnet | Prod L2 |
|---|---:|---:|
| API (2 vCPU/4 Go, bb inclus) | 8–20 € | 80–160 € (×2 + LB) |
| Postgres managé | 0 (SQLite volume) | 40–80 € (HA) |
| RPC (Alchemy/QuickNode) | 0 (free tier) | 45–90 € |
| Frontend (Vercel/Cloudflare) | 0 | 0–20 € |
| KMS (AWS) | ~1 € | ~5 € |
| Monitoring (Grafana/Sentry) | 0 (free) | 25–50 € |
| **Total** | **≈ 10–25 €/mois** | **≈ 200–400 €/mois** |

### 5.4 One-time hors gas

| Poste | Fourchette |
|---|---:|
| Audit externe (3 contrats + verifier + circuit Noir) | 30–150 k€ |
| Bug bounty (pool de lancement) | 10–50 k$ |
| Nom de domaine, divers | < 100 €/an |

## 6. Runbook opérationnel

- **Reset anvil** : procédure §2 (api stoppée → anvil restart → script → api).
- **Clé compromise** : rotation KMS, `revoke()` sur les délégations actives
  via le nouveau endpoint, redéploiement vault si la clé était owner.
- **Upgrade contrats** : non-upgradeables par design ; nouveau déploiement +
  ré-enregistrement des délégations (les users re-signent, expiry courte par
  défaut le permet sans drame).
- **Monitoring** : `/metrics` Prometheus exposé par l'API ; alertes minimales
  = exécutions en échec, solde de la clé agent (gas), healthcheck `/health`.
