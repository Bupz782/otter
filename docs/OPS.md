# Operations

## Local Docker Compose

A full local stack (anvil + backend + frontend) is available via Docker Compose.

```bash
docker compose up --build
```

Services:
- `anvil` on http://localhost:8545
- `api` on http://localhost:3001
- `frontend` on http://localhost:3000

### Deploying contracts in the compose stack

The backend needs the vault address in `OTTER_NETWORKS`. Deploy it with:

```bash
cd contracts
forge script script/DeployDelegationVault.s.sol \
  --rpc-url http://localhost:8545 --broadcast \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
# Copy the deployed vault address.
```

Then restart the API with the correct `OTTER_NETWORKS`:

```bash
OTTER_NETWORKS="default=http://anvil:8545|<vault_address>|31337" \
docker compose up -d api
```

## Environment variables

See `.env.example` for the full list. Key variables:

- `OTTER_NETWORKS` — multi-network EVM config.
- `OTTER_PRIVATE_KEY` — agent signing key.
- `OTTER_SOLVENCY_REGISTRY` — on-chain solvency registry.
- `OTTER_SOLANA_ENABLED` / `OTTER_SOLANA_RPC_URL` / `OTTER_SOLANA_PROGRAM_ID` /
  `OTTER_SOLANA_AUTHORITY_KEYPAIR` — Solana adapter.
- `OTTER_MEV_SEARCHER_ENABLED` / `OTTER_MEV_SEARCHER_RPC_URL` — private-tx MEV.

## Deployment checklist

1. Build backend image: `docker build -t otter-api .`
2. Build frontend image: `cd frontend && docker build -t otter-frontend .`
3. Deploy contracts to the target chain (see `contracts/script/`).
4. Configure env vars for the target RPC, vault and registry addresses.
5. Run migrations and start the API.
