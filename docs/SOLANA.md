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
OTTER_SOLANA_PROGRAM_ID=<deployed-program-id>
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
# 3. Build the program
cd solana/attestation_registry
anchor build
```

The compiled program lands at
`solana/attestation_registry/target/deploy/attestation_registry.so`. The
workspace `.gitignore` excludes every `target/` directory, so committing the
artifact requires an explicit force-add:

```bash
git add -f solana/attestation_registry/target/deploy/attestation_registry.so
```

Note: this build was not run on the development machine used for V1 (no
Solana toolchain installed); the documented procedure above is the reference
for reproducing the artifact before a devnet deployment.
