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
```

The adapter is built only when all three variables are present. The API
routes live under `/api/v1/solana/*` and require authentication.

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

To build the Anchor program you need the Solana/Anchor toolchain installed:

```bash
cd solana/attestation_registry
anchor build
```
