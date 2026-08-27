# MEV searcher pipeline

## V1 (implemented): private transaction submission

When `OTTER_MEV_SEARCHER_ENABLED=true` and `OTTER_MEV_SEARCHER_RPC_URL` is set,
the execution layer routes on-chain intent transactions through a private mempool
endpoint such as Flashbots Protect or MEV-Blocker. This avoids public mempool
frontrunning but does not actively capture MEV.

The private RPC is used **only for transaction submission**. Gas estimation,
balance checks, receipt polling and other reads still go through the normal
`OTTER_RPC_URL` / `OTTER_NETWORKS` RPC.

## V2 (implemented): bundle-based searcher

`crates/infrastructure/src/mev/bundle_searcher.rs` contains a
Flashbots-compatible bundle client. It signs `eth_sendBundle` requests with a
secp256k1 private key and submits raw transaction bundles to any relay supporting
the Flashbots RPC format (Flashbots, bloXroute, etc.).

### Configuration

```bash
OTTER_MEV_BUNDLE_ENABLED=true
OTTER_MEV_BUNDLE_RELAY_URL=https://relay.flashbots.net
OTTER_MEV_BUNDLE_PRIVATE_KEY=0x...
OTTER_MEV_BUNDLE_BENEFICIARY=0x...
# Backrun monitor (optional): watch a target address and auto-submit a rebate
# bundle for every pending transaction hitting it.
OTTER_MEV_BUNDLE_TARGET_ADDRESS=0x...
OTTER_MEV_BUNDLE_POLL_INTERVAL_MS=1000
```

### API

`POST /api/v1/mev/bundle` accepts:

```json
{
  "txs": ["0x...", "0x..."],
  "block_number": 12345678
}
```

and returns:

```json
{ "bundle_hash": "0x..." }
```

`GET /api/v1/mev/bundles` lists recorded submissions (hash, triggering target
transaction when automated, status, timestamp), most recent first.

`GET /api/v1/mev/config` / `POST /api/v1/mev/config` (`{ "rebate_bps": 5000 }`,
writer role) read and override the rebate share at runtime. The override is
in-memory only: `OTTER_MEV_REBATE_BPS` remains the boot value.

### Backrun handler

When `OTTER_MEV_BUNDLE_TARGET_ADDRESS` is set, the API spawns the mempool
monitor at startup. `crates/infrastructure/src/mev/backrun.rs` implements the
handler: for each detected target transaction it signs a zero-value rebate
transfer (searcher key → beneficiary, falling back to the searcher address
itself) and submits it as a single-transaction bundle for the next block. Every
attempt — submitted or failed — is recorded in the `mev_bundles` table.
Including the target transaction itself in the bundle requires its raw signed
bytes, which the monitor does not reconstruct; that ordering guarantee is a
strategy-level follow-up.

### Mempool monitor

`crates/infrastructure/src/mev/mempool_monitor.rs` polls
`watch_pending_transactions` and emits every pending transaction whose `to`
address matches the configured target. A handler can then build a backrun bundle
and submit it through the bundle client.

### Split rebates

Rebate accounting still flows through the existing `mev_captures` SQLite table
and the `OTTER_MEV_REBATE_BPS` share. In a production deployment the searcher
would update the table with the actual coinbase transfer or balance change from
the winning bundle; the current implementation provides the submission
infrastructure and leaves profit extraction to the backrun strategy.
