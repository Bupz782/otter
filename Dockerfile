# syntax=docker/dockerfile:1

# ------------------------------------------------------------------------------
# Stage 1: Build the Rust backend
# ------------------------------------------------------------------------------
FROM rustlang/rust:nightly AS builder

WORKDIR /app

# Install system dependencies required by native crates (openssl, etc.)
RUN apt-get update \
    && apt-get install -y pkg-config libssl-dev clang libclang-dev cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests and source
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY delegation_circuit ./delegation_circuit
COPY crates/infrastructure/migrations ./crates/infrastructure/migrations

# Build the API binary in release mode
RUN cargo build --release -p interfaces --bin metis_api

# ------------------------------------------------------------------------------
# TODO(frontend-tooling-ci): Add a stage that installs pinned Noir / bb binaries.
# Pinned versions used locally:
#   - nargo: 1.0.0-beta.22
#   - bb (barretenberg): 5.0.0-nightly.20260522
# These are currently supplied via host mounts in docker-compose.yml
# (OTTER_NARGO_BIN / OTTER_BB_BIN). To make the image self-contained, add a
# stage here that installs nargo (e.g. via noirup) and bb (e.g. from the Aztec
# releases), then copy the binaries into the runtime stage below.
# ------------------------------------------------------------------------------

# ------------------------------------------------------------------------------
# Stage 2: Runtime image
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y ca-certificates libssl3 curl libgomp1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Persisted SQLite database lives in /data so it can be mounted as a volume
ENV OTTER_DATABASE_URL=/data/otter.db
ENV OTTER_API_PORT=3001
ENV OTTER_CIRCUIT_DIR=/app/delegation_circuit
ENV RUST_LOG=info

VOLUME ["/data"]
EXPOSE 3001

COPY --from=builder /app/target/release/metis_api /usr/local/bin/metis_api
COPY --from=builder /app/delegation_circuit /app/delegation_circuit

CMD ["metis_api"]
