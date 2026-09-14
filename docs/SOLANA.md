# Solana attestation registry

## Goal
Provide a lightweight Solana-side attestation layer so Otter can anchor
arbitrary 32-byte hashes on-chain. V1 stores one attestation per authority;
later versions can add revocation, expiration, and multi-sig notaries.

## Files

```
solana/attestation_registry/
├── Anchor.toml
├── Cargo.toml
└── programs/attestation_registry/
    ├── Cargo.toml
    ├── Xargo.toml
    └── src/lib.rs
```

- `Attestation` account: `authority: Pubkey`, `payload_hash: [u8; 32]`,
  `timestamp: i64`, `bump: u8`.
- Instructions: `attest(payload_hash)`, `revoke`.
- PDA seeds: `["attestation", authority.as_ref()]`.

## Rust adapter

Feature-gated behind the `solana` Cargo feature in `crates/infrastructure`.

- `crates/infrastructure/src/solana/adapter.rs` — real implementation using
  `solana-client` + raw Anchor instruction data.
- `crates/infrastructure/src/solana/disabled.rs` — stub returned when the
  feature is off.

## Configuration

```bash
OTTER_SOLANA_ENABLED=true
OTTER_SOLANA_RPC_URL=https://api.devnet.solana.com
OTTER_SOLANA_PROGRAM_ID=2gdcNaGNfaqMJEzNRXa21pCzxQeDNS4Hg3b4KH8tiCXb
OTTER_SOLANA_AUTHORITY_KEYPAIR=<base58-encoded-keypair>
# Optional: scheduler attestation interval in seconds (default 3600, min 60).
OTTER_SOLANA_ATTEST_INTERVAL_SECS=3600
```

The adapter is built only when all three variables are present. The API
routes live under `/api/v1/solana/*` and require authentication.

## Attestation scheduler

When both the adapter and an on-chain SolvencyRegistry
(`OTTER_SOLVENCY_REGISTRY`) are configured, the API spawns a background task
at boot (`infrastructure::solana::scheduler::spawn_attestation_scheduler`,
same idiom as the MEV backrun monitor). Every
`OTTER_SOLANA_ATTEST_INTERVAL_SECS` it reads the proven solvency Merkle root
from the EVM registry and anchors it on Solana via `attest`. Ticks are
skipped while the registry has no proven root; transient failures are logged
and retried on the next tick.

## API endpoints

- `POST /api/v1/solana/attest` — store a payload hash, returns tx signature.
- `GET /api/v1/solana/attestations/:authority` — read the on-chain record.
- `POST /api/v1/solana/verify` — check whether an authority attested to a hash.

## Build

Default build excludes Solana:

```bash
cargo build --workspace
```

To compile the Solana adapter:

```bash
cargo build -p infrastructure --features solana
```

To build the Anchor program you need the Solana/Anchor toolchain installed.
The program pins `anchor-lang = "0.30.1"`, so use Anchor CLI 0.30.1:

```bash
# 1. Solana (Agave) toolchain — provides cargo-build-sbf
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
# 2. Anchor version manager + CLI 0.30.1
cargo install --git https://github.com/coral-xyz/anchor avm --force
avm install 0.30.1 && avm use 0.30.1
# 3. Build the program (see toolchain notes below)
cd solana/attestation_registry
anchor build
```

### Toolchain pinning (required, 2026)

The Anchor workspace carries its own `rust-toolchain.toml` (stable **1.79.0**)
— do NOT build it with the repo-root nightly:

- the SBF platform-tools cargo is rust 1.75: it cannot read v4 lockfiles
  (repo nightly writes them) nor parse edition-2024 crates;
- the lock must therefore stay at `version = 3` with transitive deps pinned to
  mid-2024 versions (blake3 1.6.1, proc-macro2 **1.0.94 exactly**, indexmap
  2.2.6, toml_edit 0.21.1, jobserver 0.32, borsh 1.5.1, …);
- anchor-syn 0.30.1's IDL generation calls `Span::source_file()`, removed in
  proc-macro2 1.0.95, and proc-macro2 ≤ 1.0.94 only compiles on nightlies
  older than 2025-04-16 → run the build as:

```bash
RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor build
```

`scripts/fix-anchor-lock-and-build.sh` (in this directory) automates the
lock repair: it builds, identifies the failing crate and pins the curated
compatible version, looping until the build succeeds.

The compiled program lands at
`solana/attestation_registry/target/deploy/attestation_registry.so`
(**~216 KB** — if it is ~800 bytes you built a no-entrypoint stub: the
program's `default` feature must be empty, never `["cpi"]`).

## Local E2E (no faucet needed)

```bash
# 1. Local validator (fresh ledger outside the repo avoids macOS AppleDouble
#    pollution of the genesis archive)
solana-test-validator --reset --quiet --ledger /tmp/otter-sol-ledger
# 2. Deploy
solana airdrop 10 <authority> --url http://localhost:8899
solana program deploy target/deploy/attestation_registry.so \
  --url http://localhost:8899 --keypair ~/.config/solana/id.json
# 3. Run the API with the adapter (binary built with
#    --features infrastructure/solana)
OTTER_SOLANA_ENABLED=true OTTER_SOLANA_RPC_URL=http://localhost:8899 \
OTTER_SOLANA_PROGRAM_ID=2gdcNaGNfaqMJEzNRXa21pCzxQeDNS4Hg3b4KH8tiCXb \
OTTER_SOLANA_AUTHORITY_KEYPAIR=<base58-of-id.json> ./target/debug/otter_api
# 4. Round-trip (verified 2026-09-14: attest -> signature, GET -> record,
#    verify -> {"valid": true})
curl -X POST localhost:3001/api/v1/solana/attest \
  -H 'Content-Type: application/json' -d '{"payload_hash":"abab...ab"}'
```

Devnet deployment is the same procedure with `--url https://api.devnet.solana.com`
once the authority holds devnet SOL (public faucet rate-limits CLI airdrops —
use https://faucet.solana.com).
