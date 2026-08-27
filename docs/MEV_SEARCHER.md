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
