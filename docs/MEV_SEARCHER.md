# MEV searcher pipeline

## V1 (implemented): private transaction submission

When `OTTER_MEV_SEARCHER_ENABLED=true` and `OTTER_MEV_SEARCHER_RPC_URL` is set,
the execution layer routes on-chain intent transactions through that RPC
instead of the public mempool. This gives users pre-confirmation privacy and
protection from public frontrunning, using relays such as:

- Flashbots Protect (`https://rpc.flashbots.net`)
- MEV-Blocker (`https://rpc.mevblocker.io`)
- Any `eth_sendRawTransaction` endpoint that keeps transactions private until
  inclusion.

The private RPC is used **only for transaction submission**. Gas estimation,
balance checks, receipt polling and other reads still go through the normal
`OTTER_RPC_URL` / `OTTER_NETWORKS` RPC.

## V2 direction: bundle backrun detection

A full searcher pipeline will:

1. Monitor the public mempool for target DEX swap transactions that create
   price dislocation.
2. Compose a bundle containing:
   - the user's intent transaction,
   - a backrun arbitrage transaction that captures the dislocation,
   - optionally a payment to the builder/validator.
3. Submit the bundle to a relay (Flashbots, bloXroute, Eden) with
   `eth_sendBundle`.
4. Split captured value between rebates to the user and protocol treasury.

This requires:
- a mempool streaming adapter,
- DEX pricing / simulation,
- bundle health scoring and re-submission logic,
- a searcher signer and relay permissions.
