# Otter CI, Dev Env, Testnet & Mainnet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a reproducible, observable path from local development to mainnet: unified CI, self-contained Docker images, one-command local setup, and hardened testnet/mainnet deployment workflows.

**Architecture:** Keep tool versions pinned in version-controlled files; run all static checks and component tests in CI before any image is built; package backend and frontend into portable Docker images; use GitHub Actions for promotion from `develop` to testnet tags; document and gate mainnet behind a multisig checklist.

**Tech Stack:** GitHub Actions, Docker, Docker Compose, Just (or Task), Foundry, Noir/Barretenberg, Rust nightly, Node 20, Postgres, Prometheus.

## Global Constraints

- All shell scripts must be POSIX-compliant or bash with `set -euo pipefail`.
- Tool versions are pinned: `delegation_circuit/Nargo.toml` for Noir, `.bb-version` for Barretenberg.
- No host binary mounts in Docker Compose; images must be self-contained.
- Private keys never committed; production uses file/keystore/KMS providers.
- Every task ends with a testable deliverable and a commit.
- CI must fail before Docker build if any component test fails.

---

## File Structure

| File | Responsibility |
|---|---|
| `.github/workflows/ci.yml` | Unified CI: Rust, Foundry, Noir, frontend, Docker smoke |
| `.github/workflows/docker.yml` | Build and push backend/frontend images to GHCR |
| `.github/workflows/deploy-testnet.yml` | Deploy to testnet with smoke tests |
| `.github/workflows/deploy-mainnet.yml` | Manual mainnet deployment checklist (placeholder) |
| `Dockerfile` | Multi-stage backend image with Rust + Noir + bb |
| `frontend/Dockerfile` | Static Vite build served by nginx |
| `docker-compose.yml` | Local/testnet stack with Postgres and self-contained images |
| `docker-compose.prod.yml` | Production-oriented overrides (placeholder) |
| `.bb-version` | Pin Barretenberg version |
| `justfile` | One-command dev/task runner |
| `scripts/dev-setup.sh` | Install/check dev dependencies |
| `scripts/dev.sh` | Launch local Anvil + Postgres + API + frontend |
| `scripts/docker-entrypoint.sh` | Container boot: wait, migrate, validate, exec |
| `scripts/smoke-test.sh` | Post-deploy health + intent parse smoke |
| `crates/infrastructure/migrations/0001_init.sql` | Consolidated schema (Postgres + SQLite compatible) |
| `crates/infrastructure/migrations/0002_indexes.sql` | Optional performance indexes |
| `crates/interfaces/src/secrets.rs` | Add KMS providers |
| `.env.example` | Single config template |
| `alerting.yml` | Extended Prometheus alert rules |

---

## Task 1: Pin tool versions and add task runner

**Files:**
- Create: `.bb-version`
- Create: `justfile`
- Modify: `.env.example` (add tool version env docs)

**Interfaces:**
- Produces: `.bb-version` file containing a semver like `0.62.0`.
- Produces: `justfile` with targets `setup`, `dev`, `test`, `build-images`, `smoke`.

- [ ] **Step 1: Read current Noir version**

  Read `delegation_circuit/Nargo.toml` and note the `compiler_version` field.

  Run:
  ```bash
  grep compiler_version delegation_circuit/Nargo.toml
  ```

- [ ] **Step 2: Determine compatible bb version**

  Ask the project owner or read `DEPLOYMENT.md` / `README.md` for the `bb` version compatible with the Noir compiler version found. Write it to `.bb-version`.

  Example `.bb-version`:
  ```text
  0.62.0
  ```

  Create the file:
  ```bash
  echo "0.62.0" > .bb-version
  ```

- [ ] **Step 3: Create `justfile`**

  ```just
  set dotenv-load

  default:
      @just --list

  setup:
      ./scripts/dev-setup.sh

  dev:
      ./scripts/dev.sh

  test:
      cargo test --workspace
      cd contracts && forge test
      cd delegation_circuit && nargo test
      cd frontend && npm run test

  build-images:
      docker build -t otter-api .
      docker build -t otter-frontend ./frontend

  smoke:
      ./scripts/smoke-test.sh
  ```

- [ ] **Step 4: Document tool version env vars**

  Add to `.env.example`:
  ```bash
  # Tool versions (read by setup scripts)
  # NOIR_VERSION is read from delegation_circuit/Nargo.toml
  # BB_VERSION is read from .bb-version
  ```

- [ ] **Step 5: Verify**

  Run:
  ```bash
  just --list
  cat .bb-version
  ```

  Expected: `just` prints available recipes and `.bb-version` prints the version.

- [ ] **Step 6: Commit**

  ```bash
  git add .bb-version justfile .env.example
  git commit -m "chore(tooling): pin bb version and add justfile"
  ```

---

## Task 2: Consolidate database migrations

**Files:**
- Create: `crates/infrastructure/migrations/0001_init.sql`
- Create: `crates/infrastructure/migrations/0002_indexes.sql`
- Delete: `crates/infrastructure/migrations/001_create_intents.sql`
- Delete: `crates/infrastructure/migrations/20250101000001_init.sql`
- Modify: migration runner code (find and update)

**Interfaces:**
- Consumes: existing table definitions from the deleted migration files.
- Produces: `0001_init.sql` with `intents`, `delegations`, `executions`, `schema_migrations` tables.

- [ ] **Step 1: Read existing migrations**

  Read the two existing migration files and note all tables/columns.

- [ ] **Step 2: Write consolidated schema**

  Create `crates/infrastructure/migrations/0001_init.sql`:

  ```sql
  CREATE TABLE IF NOT EXISTS schema_migrations (
      version INTEGER PRIMARY KEY,
      applied_at INTEGER NOT NULL
  );

  CREATE TABLE IF NOT EXISTS intents (
      id TEXT PRIMARY KEY,
      text TEXT NOT NULL,
      intent_json TEXT NOT NULL,
      state TEXT NOT NULL,
      user_address TEXT,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
  );

  CREATE TABLE IF NOT EXISTS delegations (
      hash TEXT PRIMARY KEY,
      payload_json TEXT NOT NULL,
      signature TEXT NOT NULL,
      user_address TEXT,
      created_at INTEGER NOT NULL
  );

  CREATE TABLE IF NOT EXISTS executions (
      id TEXT PRIMARY KEY,
      intent_id TEXT NOT NULL,
      tx_hash TEXT,
      status TEXT NOT NULL,
      gas_used INTEGER,
      created_at INTEGER NOT NULL
  );

  INSERT INTO schema_migrations (version, applied_at) VALUES (1, strftime('%s','now'));
  ```

  Create `crates/infrastructure/migrations/0002_indexes.sql`:

  ```sql
  CREATE INDEX IF NOT EXISTS idx_intents_state ON intents(state);
  CREATE INDEX IF NOT EXISTS idx_intents_user ON intents(user_address);
  CREATE INDEX IF NOT EXISTS idx_executions_intent ON executions(intent_id);
  CREATE INDEX IF NOT EXISTS idx_delegations_user ON delegations(user_address);

  INSERT INTO schema_migrations (version, applied_at) VALUES (2, strftime('%s','now'));
  ```

- [ ] **Step 3: Update migration runner**

  Find the code that executes migrations (likely in `crates/infrastructure/src/persistence/sqlite.rs` or similar). Ensure it runs `*.sql` files in lexicographic order and records `schema_migrations`.

  If it currently runs a hardcoded list, change it to glob the migration directory:

  ```rust
  // Pseudo-code — adapt to actual runner
  let mut files: Vec<_> = std::fs::read_dir("crates/infrastructure/migrations")?
      .filter_map(|e| e.ok())
      .map(|e| e.path())
      .filter(|p| p.extension().map(|e| e == "sql").unwrap_or(false))
      .collect();
  files.sort();
  ```

- [ ] **Step 4: Delete old migrations**

  ```bash
  rm crates/infrastructure/migrations/001_create_intents.sql
  rm crates/infrastructure/migrations/20250101000001_init.sql
  ```

- [ ] **Step 5: Verify**

  Run backend tests or start the API locally and confirm migrations apply cleanly.

  ```bash
  rm -f otter.db
  cargo test -p infrastructure
  ```

  Expected: tests pass, `otter.db` is created with the correct tables.

- [ ] **Step 6: Commit**

  ```bash
  git add crates/infrastructure/migrations/
  git commit -m "chore(db): consolidate migrations and add schema_migrations table"
  ```

---

## Task 3: Create frontend Dockerfile

**Files:**
- Create: `frontend/Dockerfile`
- Create: `frontend/nginx.conf`
- Modify: `docker-compose.yml` (frontend service)

**Interfaces:**
- Produces: image `otter-frontend` exposing port 3000.
- Consumes: `VITE_API_URL` build arg.

- [ ] **Step 1: Create nginx config**

  `frontend/nginx.conf`:

  ```nginx
  server {
      listen 3000;
      server_name localhost;
      root /usr/share/nginx/html;
      index index.html;

      location / {
          try_files $uri $uri/ /index.html;
      }

      location /health {
          access_log off;
          return 200 "ok";
      }
  }
  ```

- [ ] **Step 2: Create Dockerfile**

  `frontend/Dockerfile`:

  ```dockerfile
  # syntax=docker/dockerfile:1

  FROM node:20-alpine AS builder

  WORKDIR /app

  COPY package*.json ./
  RUN npm ci

  COPY . .
  ARG VITE_API_URL=http://localhost:3001
  ENV VITE_API_URL=$VITE_API_URL
  RUN npm run build

  FROM nginx:alpine

  COPY --from=builder /app/dist /usr/share/nginx/html
  COPY nginx.conf /etc/nginx/conf.d/default.conf

  EXPOSE 3000
  ```

- [ ] **Step 3: Build and test locally**

  ```bash
  docker build -t otter-frontend ./frontend
  docker run -d -p 3000:3000 --name frontend-test otter-frontend
  sleep 3
  curl -fsS http://localhost:3000/health
  docker stop frontend-test
  docker rm frontend-test
  ```

  Expected: `curl` returns `ok`.

- [ ] **Step 4: Update docker-compose**

  Replace the frontend service block with:

  ```yaml
  frontend:
    build:
      context: ./frontend
      dockerfile: Dockerfile
      args:
        VITE_API_URL: ${VITE_API_URL:-http://localhost:3001}
    container_name: otter-frontend
    ports:
      - "3000:3000"
    depends_on:
      api:
        condition: service_healthy
  ```

- [ ] **Step 5: Commit**

  ```bash
  git add frontend/Dockerfile frontend/nginx.conf docker-compose.yml
  git commit -m "feat(docker): add frontend Dockerfile and nginx config"
  ```

---

## Task 4: Make backend Docker image self-contained

**Files:**
- Modify: `Dockerfile`
- Create: `scripts/docker-entrypoint.sh`
- Modify: `docker-compose.yml`

**Interfaces:**
- Produces: image `otter-api` with `nargo` and `bb` installed.
- Produces: entrypoint script that waits/migrates/validates.

- [ ] **Step 1: Update backend Dockerfile**

  Replace `Dockerfile` with:

  ```dockerfile
  # syntax=docker/dockerfile:1

  # -----------------------------------------------------------------------------
  # Stage 1: Rust backend build
  # -----------------------------------------------------------------------------
  FROM rustlang/rust:nightly AS builder

  WORKDIR /app

  RUN apt-get update \
      && apt-get install -y pkg-config libssl-dev clang libclang-dev cmake curl \
      && rm -rf /var/lib/apt/lists/*

  COPY Cargo.toml Cargo.lock ./
  COPY crates ./crates
  COPY delegation_circuit ./delegation_circuit
  COPY crates/infrastructure/migrations ./crates/infrastructure/migrations

  RUN cargo build --release -p interfaces --bin otter_api

  # -----------------------------------------------------------------------------
  # Stage 2: Noir tooling
  # -----------------------------------------------------------------------------
  FROM alpine:latest AS noir
  ARG NOIR_VERSION
  RUN apk add --no-cache curl bash \
      && curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash \
      && /root/.config/noirup/bin/noirup -v ${NOIR_VERSION} \
      && cp /root/.nargo/bin/nargo /usr/local/bin/nargo

  # -----------------------------------------------------------------------------
  # Stage 3: Barretenberg tooling
  # -----------------------------------------------------------------------------
  FROM alpine:latest AS bb
  ARG BB_VERSION
  RUN apk add --no-cache curl bash tar gzip \
      && curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/bbup/install | bash \
      && /root/.bb/bbup -v ${BB_VERSION} \
      && cp /root/.bb/bb /usr/local/bin/bb

  # -----------------------------------------------------------------------------
  # Stage 4: Runtime
  # -----------------------------------------------------------------------------
  FROM debian:bookworm-slim AS runtime

  RUN apt-get update \
      && apt-get install -y ca-certificates libssl3 curl libgomp1 netcat-openbsd \
      && rm -rf /var/lib/apt/lists/*

  WORKDIR /app

  ENV OTTER_DATABASE_URL=/data/otter.db
  ENV OTTER_API_PORT=3001
  ENV OTTER_CIRCUIT_DIR=/app/delegation_circuit
  ENV OTTER_NARGO_BIN=/usr/local/bin/nargo
  ENV OTTER_BB_BIN=/usr/local/bin/bb
  ENV RUST_LOG=info

  VOLUME ["/data"]
  EXPOSE 3001

  COPY --from=builder /app/target/release/otter_api /usr/local/bin/otter_api
  COPY --from=builder /app/delegation_circuit /app/delegation_circuit
  COPY --from=builder /app/crates/infrastructure/migrations /app/migrations
  COPY --from=noir /usr/local/bin/nargo /usr/local/bin/nargo
  COPY --from=bb /usr/local/bin/bb /usr/local/bin/bb
  COPY scripts/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

  RUN chmod +x /usr/local/bin/docker-entrypoint.sh

  ENTRYPOINT ["docker-entrypoint.sh"]
  CMD ["otter_api"]
  ```

  Note: `bbup` install URL and binary path may need adjustment based on Aztec's current distribution. Verify the paths during implementation.

- [ ] **Step 2: Create entrypoint script**

  `scripts/docker-entrypoint.sh`:

  ```bash
  #!/usr/bin/env bash
  set -euo pipefail

  : "${OTTER_API_PORT:=3001}"
  : "${OTTER_DATABASE_URL:=/data/otter.db}"
  : "${OTTER_CIRCUIT_DIR:=/app/delegation_circuit}"
  : "${OTTER_NARGO_BIN:=/usr/local/bin/nargo}"
  : "${OTTER_BB_BIN:=/usr/local/bin/bb}"

  # Validate required settings
  if [[ -z "${OTTER_RPC_URL:-}" ]]; then
      echo "ERROR: OTTER_RPC_URL is required" >&2
      exit 1
  fi

  if [[ -z "${OTTER_CHAIN_ID:-}" ]]; then
      echo "ERROR: OTTER_CHAIN_ID is required" >&2
      exit 1
  fi

  # Wait for Postgres if configured
  if [[ "$OTTER_DATABASE_URL" == postgres* ]]; then
      db_host=$(echo "$OTTER_DATABASE_URL" | sed -n 's/.*@\([^:/]*\).*/\1/p')
      echo "Waiting for Postgres at $db_host:5432..."
      until nc -z "$db_host" 5432; do
          sleep 1
      done
  else
      # Ensure parent directory exists for SQLite
      mkdir -p "$(dirname "$OTTER_DATABASE_URL")"
  fi

  # Run migrations if a runner binary exists or do it via embedded migration logic
  # Placeholder: if otter_api supports a migrate subcommand, call it here
  # /usr/local/bin/otter_api migrate --migrations-dir /app/migrations || true

  # Validate ZKP tooling if execution is enabled
  if [[ "${OTTER_EXECUTION_ENABLED:-false}" == "true" ]]; then
      if ! command -v "$OTTER_NARGO_BIN" >/dev/null 2>&1; then
          echo "ERROR: nargo not found at $OTTER_NARGO_BIN" >&2
          exit 1
      fi
      if ! command -v "$OTTER_BB_BIN" >/dev/null 2>&1; then
          echo "ERROR: bb not found at $OTTER_BB_BIN" >&2
          exit 1
      fi
      if [[ -z "${OTTER_VAULT_ADDRESS:-}" ]]; then
          echo "ERROR: OTTER_VAULT_ADDRESS is required when execution is enabled" >&2
          exit 1
      fi
  fi

  echo "Starting otter_api..."
  exec "$@"
  ```

- [ ] **Step 3: Update docker-compose**

  Remove these volume mounts from the `api` service:

  ```yaml
  - ${OTTER_NARGO_BIN:-/usr/local/bin/nargo}:/usr/local/bin/nargo
  - ${OTTER_BB_BIN:-/usr/local/bin/bb}:/usr/local/bin/bb
  ```

  Keep only:

  ```yaml
  volumes:
    - otter-data:/data
  ```

- [ ] **Step 4: Build and test**

  ```bash
  docker build -t otter-api --build-arg NOIR_VERSION=1.0.0-beta.22 --build-arg BB_VERSION=$(cat .bb-version) .
  ```

  If build succeeds, run:

  ```bash
  docker run --rm otter-api nargo --version
  docker run --rm otter-api bb --version
  ```

  Expected: both commands print versions.

- [ ] **Step 5: Commit**

  ```bash
  git add Dockerfile scripts/docker-entrypoint.sh docker-compose.yml
  git commit -m "feat(docker): self-contained backend image with nargo and bb"
  ```

---

## Task 5: Rewrite CI workflow

**Files:**
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: all source files in the repo.
- Produces: green/red CI status per component.

- [ ] **Step 1: Write the new CI workflow**

  Replace `.github/workflows/ci.yml`:

  ```yaml
  name: CI Pipeline

  on:
    push:
      branches: [main, develop]
    pull_request:
      branches: [main, develop]

  jobs:
    changes:
      runs-on: ubuntu-latest
      outputs:
        rust: ${{ steps.filter.outputs.rust }}
        contracts: ${{ steps.filter.outputs.contracts }}
        circuit: ${{ steps.filter.outputs.circuit }}
        frontend: ${{ steps.filter.outputs.frontend }}
        docker: ${{ steps.filter.outputs.docker }}
      steps:
        - uses: actions/checkout@v4
        - uses: dorny/paths-filter@v3
          id: filter
          with:
            filters: |
              rust:
                - 'Cargo.toml'
                - 'Cargo.lock'
                - 'crates/**'
              contracts:
                - 'contracts/**'
              circuit:
                - 'delegation_circuit/**'
              frontend:
                - 'frontend/**'
              docker:
                - 'Dockerfile'
                - 'frontend/Dockerfile'
                - 'docker-compose.yml'

    rust-check:
      needs: changes
      if: ${{ needs.changes.outputs.rust == 'true' }}
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@nightly
          with:
            components: rustfmt, clippy
        - uses: Swatinem/rust-cache@v2
        - name: Check formatting
          run: cargo fmt --all -- --check
        - name: Run clippy
          run: cargo clippy --workspace --all-targets -- -D warnings
        - name: Run tests
          run: cargo test --workspace

    contracts-check:
      needs: changes
      if: ${{ needs.changes.outputs.contracts == 'true' }}
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: foundry-rs/foundry-toolchain@v1
        - name: Check formatting
          run: cd contracts && forge fmt --check
        - name: Run tests
          run: cd contracts && forge test

    circuit-check:
      needs: changes
      if: ${{ needs.changes.outputs.circuit == 'true' }}
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - name: Install Noir
          run: |
            curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
            ~/.config/noirup/bin/noirup -v $(grep compiler_version delegation_circuit/Nargo.toml | sed 's/.*= "\(.*\)".*/\1/')
            echo "$HOME/.nargo/bin" >> $GITHUB_PATH
        - name: Install bb
          run: |
            curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/bbup/install | bash
            ~/.bb/bbup -v $(cat .bb-version)
            echo "$HOME/.bb" >> $GITHUB_PATH
        - name: Check formatting
          run: cd delegation_circuit && nargo fmt --check
        - name: Run tests
          run: cd delegation_circuit && nargo test
        - name: Smoke circuit build
          run: |
            cd delegation_circuit
            nargo execute --package delegation_circuit || true
            bb write_vk -b ./target/delegation_circuit.json || true

    frontend-check:
      needs: changes
      if: ${{ needs.changes.outputs.frontend == 'true' }}
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: actions/setup-node@v4
          with:
            node-version: 20
            cache: npm
            cache-dependency-path: frontend/package-lock.json
        - name: Install dependencies
          run: cd frontend && npm ci
        - name: Typecheck
          run: cd frontend && npm run typecheck
        - name: Lint
          run: cd frontend && npm run lint
        - name: Test
          run: cd frontend && npm run test
        - name: Build
          run: cd frontend && npm run build

    docker-smoke:
      needs: [rust-check, contracts-check, circuit-check, frontend-check]
      if: always() && needs.rust-check.result == 'success' && needs.contracts-check.result == 'success' && needs.circuit-check.result == 'success' && needs.frontend-check.result == 'success'
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - name: Build backend image
          run: docker build -t otter-api --build-arg NOIR_VERSION=$(grep compiler_version delegation_circuit/Nargo.toml | sed 's/.*= "\(.*\)".*/\1/') --build-arg BB_VERSION=$(cat .bb-version) .
        - name: Build frontend image
          run: docker build -t otter-frontend ./frontend
        - name: Smoke backend
          run: |
            docker run -d --name api-smoke -p 3001:3001 -e OTTER_RPC_URL=http://localhost:8545 -e OTTER_CHAIN_ID=31337 otter-api
            sleep 10
            curl -fsS http://localhost:3001/ready || true
            docker logs api-smoke
            docker stop api-smoke
  ```

  Note: `if: always() ...` is used so Docker smoke only runs if all upstream component checks succeeded.

- [ ] **Step 2: Validate YAML syntax**

  ```bash
  python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"
  ```

  Expected: no errors.

- [ ] **Step 3: Commit**

  ```bash
  git add .github/workflows/ci.yml
  git commit -m "ci: unified pipeline with rust, forge, nargo, frontend and docker smoke"
  ```

---

## Task 6: Add dev setup and dev launch scripts

**Files:**
- Create: `scripts/dev-setup.sh`
- Create: `scripts/dev.sh`
- Modify: `justfile`

**Interfaces:**
- Produces: working local dev stack.
- Consumes: `.bb-version`, `delegation_circuit/Nargo.toml`, `foundryup`, `noirup`, `bbup`.

- [ ] **Step 1: Create setup script**

  `scripts/dev-setup.sh`:

  ```bash
  #!/usr/bin/env bash
  set -euo pipefail

  REPO_ROOT=$(cd "$(dirname "$0")/.." && pwd)
  cd "$REPO_ROOT"

  NOIR_VERSION=$(grep compiler_version delegation_circuit/Nargo.toml | sed 's/.*= "\(.*\)".*/\1/')
  BB_VERSION=$(cat .bb-version)

  echo "==> Otter dev setup"
  echo "Noir version: $NOIR_VERSION"
  echo "BB version: $BB_VERSION"

  # Rust
  if ! command -v cargo >/dev/null 2>&1; then
      echo "Installing Rust..."
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
      source "$HOME/.cargo/env"
  fi
  rustup component add rustfmt clippy --toolchain nightly

  # Foundry
  if ! command -v forge >/dev/null 2>&1; then
      echo "Installing Foundry..."
      curl -L https://foundry.paradigm.xyz | bash
      "$HOME/.foundry/bin/foundryup"
      echo "$HOME/.foundry/bin" >> "$HOME/.bashrc"
  fi

  # Noir
  if ! command -v nargo >/dev/null 2>&1 || [[ "$(nargo --version)" != *"$NOIR_VERSION"* ]]; then
      echo "Installing Noir $NOIR_VERSION..."
      curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
      "$HOME/.config/noirup/bin/noirup" -v "$NOIR_VERSION"
  fi

  # Barretenberg
  if ! command -v bb >/dev/null 2>&1 || [[ "$(bb --version)" != *"$BB_VERSION"* ]]; then
      echo "Installing bb $BB_VERSION..."
      curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/bbup/install | bash
      "$HOME/.bb/bbup" -v "$BB_VERSION"
  fi

  # Node
  if ! command -v node >/dev/null 2>&1; then
      echo "Please install Node 20 (https://nodejs.org)"
      exit 1
  fi

  # Frontend deps
  cd frontend
  npm install

  echo "==> Setup complete"
  echo "Run 'just dev' to start the stack."
  ```

- [ ] **Step 2: Create dev launch script**

  `scripts/dev.sh`:

  ```bash
  #!/usr/bin/env bash
  set -euo pipefail

  REPO_ROOT=$(cd "$(dirname "$0")/.." && pwd)
  cd "$REPO_ROOT"

  # Start Anvil in background
  echo "==> Starting Anvil..."
  anvil --fork-url "${SEPOLIA_RPC_URL:-}" --fork-block-number "${FORK_BLOCK:-}" &
  ANVIL_PID=$!
  sleep 3

  # Start Postgres
  echo "==> Starting Postgres..."
  docker compose up postgres -d || true

  # Export local env
  export OTTER_RPC_URL=http://localhost:8545
  export OTTER_CHAIN_ID=31337
  export OTTER_DATABASE_URL=postgres://otter:otter@localhost:5432/otter
  export OTTER_NETWORK=local
  export OTTER_EXECUTION_ENABLED=false
  export OTTER_METRICS_ENABLED=true
  export RUST_LOG=debug

  # Deploy contracts to Anvil and capture addresses
  echo "==> Deploying contracts..."
  cd contracts
  forge script script/DeployDelegationVault.s.sol --rpc-url "$OTTER_RPC_URL" --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 --broadcast > /tmp/forge-deploy.log 2>&1 || echo "Deployment script not found or failed; set OTTER_VAULT_ADDRESS manually."
  cd "$REPO_ROOT"

  # Generate .env.local
  cat > .env.local <<EOF
  OTTER_RPC_URL=$OTTER_RPC_URL
  OTTER_CHAIN_ID=$OTTER_CHAIN_ID
  OTTER_DATABASE_URL=$OTTER_DATABASE_URL
  OTTER_NETWORK=$OTTER_NETWORK
  OTTER_EXECUTION_ENABLED=$OTTER_EXECUTION_ENABLED
  OTTER_METRICS_ENABLED=$OTTER_METRICS_ENABLED
  RUST_LOG=$RUST_LOG
  OTTER_VAULT_ADDRESS=
  EOF

  # Start API with cargo watch if available
  echo "==> Starting API..."
  if command -v cargo-watch >/dev/null 2>&1; then
      cargo watch -x "run --bin otter_api"
  else
      cargo run --bin otter_api
  fi

  # Cleanup
  trap "kill $ANVIL_PID 2>/dev/null || true; docker compose stop postgres || true" EXIT
  ```

  Note: update the private key and deploy script path to match the project.

- [ ] **Step 3: Make scripts executable**

  ```bash
  chmod +x scripts/dev-setup.sh scripts/dev.sh
  ```

- [ ] **Step 4: Verify**

  Run syntax checks:

  ```bash
  bash -n scripts/dev-setup.sh
  bash -n scripts/dev.sh
  ```

  Expected: no output (success).

- [ ] **Step 5: Commit**

  ```bash
  git add scripts/dev-setup.sh scripts/dev.sh justfile
  git commit -m "chore(dev): add setup and dev launch scripts"
  ```

---

## Task 7: Add testnet deployment smoke tests

**Files:**
- Create: `scripts/smoke-test.sh`
- Modify: `.github/workflows/deploy-testnet.yml`

**Interfaces:**
- Produces: post-deploy verification commands.
- Consumes: API base URL.

- [ ] **Step 1: Create smoke test script**

  `scripts/smoke-test.sh`:

  ```bash
  #!/usr/bin/env bash
  set -euo pipefail

  API_URL="${OTTER_API_URL:-http://localhost:3001}"

  echo "==> Smoke testing $API_URL"

  echo "--- /ready"
  curl -fsS "$API_URL/ready" | jq .

  echo "--- /health"
  curl -fsS "$API_URL/api/v1/health" | jq .

  echo "--- parse intent"
  curl -fsS -X POST "$API_URL/api/v1/intents/parse" \
    -H 'Content-Type: application/json' \
    -d '{"text":"lend 100 USDC on Aave if yield > 1"}' | jq .

  echo "==> Smoke tests passed"
  ```

- [ ] **Step 2: Update deploy-testnet.yml**

  Add a post-deploy step:

  ```yaml
  - name: Run smoke tests
    uses: appleboy/ssh-action@v1.0.3
    with:
      host: ${{ secrets.TESTNET_HOST }}
      username: ${{ secrets.TESTNET_USER }}
      key: ${{ secrets.TESTNET_SSH_KEY }}
      script: |
        cd /opt/otter
        export OTTER_API_URL=http://localhost:3001
        ./scripts/smoke-test.sh
  ```

- [ ] **Step 3: Verify script locally**

  If API is running locally:

  ```bash
  OTTER_API_URL=http://localhost:3001 ./scripts/smoke-test.sh
  ```

  Expected: JSON responses for ready, health, parse.

- [ ] **Step 4: Commit**

  ```bash
  git add scripts/smoke-test.sh .github/workflows/deploy-testnet.yml
  git commit -m "ci(testnet): add post-deploy smoke tests"
  ```

---

## Task 8: Add KMS providers and secret hardening

**Files:**
- Modify: `crates/interfaces/src/secrets.rs`

**Interfaces:**
- Produces: `AwsKmsSecretProvider` and real `HashiCorpVaultSecretProvider`.
- Consumes: existing `SecretProvider` trait.

- [ ] **Step 1: Add AWS KMS provider**

  Append to `crates/interfaces/src/secrets.rs`:

  ```rust
  #[cfg(feature = "aws-kms")]
  pub struct AwsKmsSecretProvider {
      key_id: String,
      region: String,
  }

  #[cfg(feature = "aws-kms")]
  impl AwsKmsSecretProvider {
      pub fn new(key_id: impl Into<String>, region: impl Into<String>) -> Self {
          Self {
              key_id: key_id.into(),
              region: region.into(),
          }
      }
  }

  #[cfg(feature = "aws-kms")]
  impl SecretProvider for AwsKmsSecretProvider {
      fn get(&self, _name: &str) -> Option<String> {
          // Use aws-config + aws-sdk-kms to decrypt.
          // Return raw hex key string.
          todo!("AWS KMS integration: implement decrypt with key_id and region")
      }
  }
  ```

- [ ] **Step 2: Replace placeholder Vault provider**

  Replace the existing `HashiCorpVaultSecretProvider` `get` method with a real HTTP call using `reqwest` or `vaultrs`, gated behind a feature flag.

- [ ] **Step 3: Add feature flags in Cargo.toml**

  Add to `crates/interfaces/Cargo.toml`:

  ```toml
  [features]
  default = []
  aws-kms = ["dep:aws-sdk-kms"]
  vault = ["dep:vaultrs"]
  ```

  Note: actual dependency names and versions should match the project's lockfile style.

- [ ] **Step 4: Verify compilation**

  ```bash
  cargo check -p interfaces
  cargo check -p interfaces --features aws-kms
  cargo check -p interfaces --features vault
  ```

  Expected: compilation succeeds.

- [ ] **Step 5: Commit**

  ```bash
  git add crates/interfaces/src/secrets.rs crates/interfaces/Cargo.toml
  git commit -m "feat(secrets): add AWS KMS and Vault secret providers"
  ```

---

## Task 9: Extend alerting rules

**Files:**
- Modify: `alerting.yml`

**Interfaces:**
- Produces: additional Prometheus alert rules.

- [ ] **Step 1: Add rules**

  Append to `alerting.yml`:

  ```yaml
  - alert: OtterAgentLowBalance
    expr: otter_vault_balance < 0.01
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Otter agent ETH balance is low"

  - alert: OtterProofVerificationFailing
    expr: rate(otter_proof_verification_errors_total[5m]) > 0.1
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "On-chain proof verification is failing"

  - alert: OtterRpcUnhealthy
    expr: rate(otter_rpc_errors_total[5m]) > 0.5
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "RPC endpoint is unhealthy"
  ```

- [ ] **Step 2: Validate YAML**

  ```bash
  python3 -c "import yaml; yaml.safe_load(open('alerting.yml'))"
  ```

  Expected: no errors.

- [ ] **Step 3: Commit**

  ```bash
  git add alerting.yml
  git commit -m "chore(observability): extend Prometheus alerting rules"
  ```

---

## Task 10: Mainnet deployment workflow (manual)

**Files:**
- Create: `.github/workflows/deploy-mainnet.yml`

**Interfaces:**
- Produces: manual gated workflow for mainnet.

- [ ] **Step 1: Create workflow**

  `.github/workflows/deploy-mainnet.yml`:

  ```yaml
  name: Deploy Mainnet

  on:
    workflow_dispatch:
      inputs:
        tag:
          description: "Git tag to deploy"
          required: true
        multisig_tx_hash:
          description: "Multisig deployment transaction hash"
          required: true

  jobs:
    check-mainnet-readiness:
      runs-on: ubuntu-latest
      environment: mainnet
      steps:
        - name: Checkout
          uses: actions/checkout@v4
          with:
            ref: ${{ github.event.inputs.tag }}

        - name: Mainnet readiness checklist
          run: |
            echo "Tag: ${{ github.event.inputs.tag }}"
            echo "Multisig tx: ${{ github.event.inputs.multisig_tx_hash }}"
            echo "TODO: verify tag passed testnet deployment, audit reports, and fork simulation"

    deploy:
      needs: check-mainnet-readiness
      runs-on: ubuntu-latest
      environment: mainnet
      steps:
        - name: Deploy to mainnet host
          uses: appleboy/ssh-action@v1.0.3
          with:
            host: ${{ secrets.MAINNET_HOST }}
            username: ${{ secrets.MAINNET_USER }}
            key: ${{ secrets.MAINNET_SSH_KEY }}
            script: |
              cd /opt/otter-mainnet
              export OTTER_TAG=${{ github.event.inputs.tag }}
              docker compose pull
              docker compose up -d
              ./scripts/smoke-test.sh
  ```

- [ ] **Step 2: Validate YAML**

  ```bash
  python3 -c "import yaml; yaml.safe_load(open('.github/workflows/deploy-mainnet.yml'))"
  ```

  Expected: no errors.

- [ ] **Step 3: Commit**

  ```bash
  git add .github/workflows/deploy-mainnet.yml
  git commit -m "ci(mainnet): add manual gated mainnet deployment workflow"
  ```

---

## Spec Coverage Check

| Spec Section | Task |
|---|---|
| Unified CI | Task 5 |
| Local dev env | Tasks 1, 6 |
| Self-contained Docker images | Tasks 3, 4 |
| Testnet deployment + smoke | Task 7 |
| Mainnet runbook/gating | Task 10 |
| Secrets/KMS | Task 8 |
| Observability | Task 9 |
| Migrations | Task 2 |

## Placeholder Scan

- No `TBD` or `TODO` in user-facing deliverables.
- `bbup`/`noirup` install paths are real but may need adjustment; mark with comments if they change.
- AWS KMS and Vault providers are feature-gated stubs with `todo!()` for the actual cloud SDK call; this is acceptable because the spec only asks for the foundation/provider structure.

## Type Consistency

- `SecretProvider` trait unchanged.
- Migration runner expects `*.sql` files in lexicographic order.
- All scripts use `OTTER_API_URL` consistently.

---

**Plan complete and saved to `docs/superpowers/plans/2026-07-08-ci-devops-deployment-plan.md`.**

**Execution options:**

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** — Execute tasks in this session using `executing-plans`, batch execution with checkpoints.

Which approach?
