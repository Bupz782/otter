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

RUN cargo build --release -p interfaces --bin metis_api

# -----------------------------------------------------------------------------
# Stage 2: Noir tooling
# -----------------------------------------------------------------------------
FROM alpine:latest AS noir
ARG NOIR_VERSION
ENV SHELL=/bin/bash
RUN apk add --no-cache curl bash git \
    && curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash \
    && /root/.nargo/bin/noirup -v ${NOIR_VERSION} \
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
ENV OTTER_MIGRATIONS_DIR=/app/crates/infrastructure/migrations
ENV RUST_LOG=info

VOLUME ["/data"]
EXPOSE 3001

COPY --from=builder /app/target/release/metis_api /usr/local/bin/metis_api
COPY --from=builder /app/delegation_circuit /app/delegation_circuit
COPY --from=builder /app/crates/infrastructure/migrations /app/crates/infrastructure/migrations
COPY --from=noir /usr/local/bin/nargo /usr/local/bin/nargo
COPY --from=bb /usr/local/bin/bb /usr/local/bin/bb
COPY scripts/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ENTRYPOINT ["docker-entrypoint.sh"]
CMD ["metis_api"]
