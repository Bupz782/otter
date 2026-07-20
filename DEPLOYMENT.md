# Otter Testnet Deployment Guide

This document explains how to deploy the Otter V1 testnet stack on **Sepolia**
using Docker Compose and the GitHub Actions workflow.

---

## What is deployed

- `otter-api` — Rust Axum daemon that parses intents, monitors on-chain
  conditions, generates ZK proofs and submits transactions to the vault.
- `otter-frontend` — React 18 + Vite UI for creating intents and watching
  execution status.
- Prometheus `/metrics` endpoint and alerting rules for production observability.

---

## Prerequisites

1. A machine with Docker Engine + Docker Compose v2.
2. `nargo` (Noir) and `bb` (Barretenberg) installed on the host and available
   in `PATH`.
   - https://noir-lang.org/docs/getting_started/installation
   - https://github.com/AztecProtocol/aztec-packages/tree/master/barretenberg
3. A funded Sepolia wallet for the agent.
4. A Sepolia RPC endpoint (public node or Alchemy/Infura).
5. The `DelegationVault` contract deployed on Sepolia.

---

## V1 testnet proof

Otter V1 was validated end-to-end on Sepolia. The contracts are deployed and
an intent created via the API was executed automatically by the agent:

- **DelegationVerifier**: `0x4Bd78ec7DA6d2789Bbe01a79ea6eAaBC0271A58c`
- **DelegationVault**: `0x2e3a565bb92bC46150259F2559320ce79EC751F2`
- **Execution tx**: `0xe3cd33b2c6697e4ad16416cf0d3c84cde6a92bcd606b0d8ffd83da33d69d54a5`

> **Stale after circuit change.** Adding `target_contract` to the Noir
circuit changed the verification key. The addresses above correspond to the
previous verifier and must be redeployed. Follow the steps below, then update
`OTTER_VAULT_ADDRESS` with the newly deployed vault address.

The agent monitored the price condition, generated a Noir/Barretenberg proof and
called `DelegationVault.executeWithProof` without manual intervention.

---

## 1. Deploy the contracts

```bash
cd contracts
forge script script/DeployDelegationVault.s.sol \
  --rpc-url $SEPOLIA_RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --verify \
  --etherscan-api-key $ETHERSCAN_API_KEY
```

Export the printed vault address:

```bash
export VAULT_ADDRESS=0x...
```

---

## 2. Configure secrets and environment

```bash
cp .env.example .env
```

Edit `.env`:

```bash
OTTER_RPC_URL=https://ethereum-sepolia-rpc.publicnode.com
OTTER_CHAIN_ID=11155111
OTTER_NETWORK=sepolia
OTTER_VAULT_ADDRESS=0x...

# Recommended for production: read the key from a file or secret manager.
# OTTER_PRIVATE_KEY_FILE=/run/secrets/otter_private_key
# OTTER_PRIVATE_KEY_SOURCE=vault-file

# Acceptable for testnet only — the agent will log a warning on startup.
OTTER_PRIVATE_KEY=0x...

# Nonce persistence. The file is updated after each on-chain delegation and is
# read on startup so restarts never reuse a consumed nonce.
OTTER_NONCE_STORE_PATH=/data/otter-nonce.txt

OTTER_CIRCUIT_DIR=./delegation_circuit
OTTER_NARGO_BIN=/usr/local/bin/nargo
OTTER_BB_BIN=/usr/local/bin/bb

OTTER_EXECUTION_ENABLED=true
OTTER_DELEGATE_ON_CREATE=false
OTTER_METRICS_ENABLED=true
OTTER_MONITORING_INTERVAL_SECS=60

RUST_LOG=info
```

### Secret management strategy

The API loads the agent private key in this order:

1. `OTTER_PRIVATE_KEY_FILE` — read the hex key from a file. Use this in
   production and restrict the file to the service user (`chmod 0600`).
2. `OTTER_PRIVATE_KEY` — read the key from an environment variable or the
   config file. The API logs a warning on startup because the value is visible
   in the process environment.

For a production hardening path beyond V1, implement a `SecretProvider` trait
(see `crates/interfaces/src/secrets.rs`) backed by HashiCorp Vault, AWS KMS,
Azure Key Vault or Kubernetes secrets. Rotate the key regularly and update
`OTTER_PRIVATE_KEY_SOURCE` so the audit log reflects the source.

### Agent wallet funding

The agent private key must hold ETH to pay for:

- `DelegationVault.delegate()` calls (one per executed intent)
- `DelegationVault.executeWithProof()` calls

---

## 3. Start the stack

```bash
docker compose up -d --build
```

Services:

- API: http://localhost:3001
- Frontend: http://localhost:3000
- Prometheus-style metrics: http://localhost:3001/metrics (when
  `OTTER_METRICS_ENABLED=true`)

Health checks run automatically via the `/ready` endpoint.

---

## 4. Verify it works

Create an intent:

```bash
curl -X POST http://localhost:3001/api/v1/intents \
  -H 'Content-Type: application/json' \
  -d '{"text":"lend 1000 USDC on Aave if yield > 3"}'
```

Watch the orchestrator state:

```bash
curl http://localhost:3001/api/v1/orchestrator/state | jq
```

Watch logs:

```bash
docker compose logs -f api
```

When the Aave supply APY on Sepolia satisfies the condition, the agent will:

1. Build a fresh delegation with a unique nonce.
2. Sign it locally.
3. Register it on-chain via `ensure_delegated()`.
4. Generate a Noir proof.
5. Call `executeWithProof()` on the vault.

---

## 5. Observability

### Prometheus scraping

Add the API to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'otter-api'
    static_configs:
      - targets: ['localhost:3001']
    metrics_path: /metrics
```

The endpoint exposes:

- `otter_price_updates_total`
- `otter_conditions_met_total`
- `otter_executions_total`
- `otter_errors_total`
- `otter_active_intents`
- `otter_execution_enabled`

### Alerting rules

Load `alerting.yml` into Prometheus:

```yaml
rule_files:
  - alerting.yml
```

Key alerts:

- `OtterAgentDown` — API unreachable.
- `OtterHighErrorRate` — error rate > 0.1/sec for 2 minutes.
- `OtterExecutionStalled` — conditions met but no executions confirmed.
- `OtterNoPriceUpdates` — oracle/monitoring stalled.

### Runbook snippets

```bash
# Check readiness
curl -fsS http://localhost:3001/ready | jq

# Inspect metrics
curl -fsS http://localhost:3001/metrics

# Check current nonce persisted on disk
cat /data/otter-nonce.txt
```

---

## 6. Load test

A load-test script is provided in `scripts/load_test.py`. Run it against a
running API:

```bash
# Start the API locally (execution disabled for a safe parse-only test)
OTTER_API_PORT=3002 OTTER_EXECUTION_ENABLED=false OTTER_METRICS_ENABLED=true \
  ./target/debug/otter_api

# In another terminal
export OTTER_API_URL=http://localhost:3002
python3 scripts/load_test.py --intents 100 --concurrency 5
```

Example output on a local machine:

```text
Parse latency
  samples: 30
  success: 23/30 (76.7%)
  p50:     0.006s
  p95:     0.028s

Create-intent latency
  samples: 30
  success: 23/30 (76.7%)
  p50:     0.002s
  p95:     0.010s
```

> Note: Docker Compose runs the API against PostgreSQL, which removes SQLite
> concurrency limits. For local dev without Docker you can still use SQLite by
> setting `OTTER_DATABASE_URL=./otter.db`.

---

## 7. CI/CD deployment

The repository includes two workflows:

- `.github/workflows/docker.yml` — builds and pushes images to GHCR on every
  push to `main`/`develop` and on version tags.
- `.github/workflows/deploy-testnet.yml` — triggered on `v*` tags or manually,
  pushes images and deploys them to the configured testnet host.

Configure these repository secrets for the testnet workflow:

| Secret | Description |
|--------|-------------|
| `TESTNET_HOST` | IP or hostname of the deployment server |
| `TESTNET_USER` | SSH user |
| `TESTNET_SSH_KEY` | Private key for SSH access |

On the testnet host, place a `docker-compose.yml` and `.env` in `/opt/otter`.
The workflow runs:

```bash
cd /opt/otter
docker compose pull
docker compose up -d --remove-orphans
```

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `ready` healthcheck fails | Postgres unreachable or migrations failed | Check that the `postgres` service is healthy and the API can reach it on port 5432. |
| `bb prove failed` | `bb` binary missing or incompatible | Check `OTTER_BB_BIN` points to a working `bb` built for the same Noir version as the circuit. |
| `delegate()` reverts | Wrong vault address or nonce reuse | Verify `OTTER_VAULT_ADDRESS` and that the agent key has ETH. Check `/data/otter-nonce.txt`. |
| No intents execute | `OTTER_EXECUTION_ENABLED=false` or condition never met | Enable execution and check `/metrics` / logs. |
| `nargo execute` not found | Host `nargo` not mounted correctly | Ensure the binary path is valid and executable. |
| High error rate alert fires | RPC throttling | Add retries or use a dedicated RPC endpoint. |

---

## Production hardening (beyond V1)

- Do not store the agent private key in plain `.env`; use `OTTER_PRIVATE_KEY_FILE`,
  a secret manager, or a KMS/HSM via the `SecretProvider` trait.
- Run `bb` / `nargo` in a side-car container instead of mounting host binaries.
- The Docker Compose stack already uses Postgres; for other deployments ensure
  `OTTER_DATABASE_URL` points to a managed PostgreSQL instance with backups.
- Add TLS termination and authentication on `/metrics` and the API.
- Keep the `DelegationVault` contract upgrade path documented and test it on
  a fork before mainnet.
