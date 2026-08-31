# Bridge cross-chain EVM — V1 design

## Goal
Move ERC-20 value from a source EVM chain to a destination EVM chain while keeping
the bridge trust-minimized enough for a first production iteration.

## V1 scope (implemented)

```
User ──lock──▶ OtterBridge (source) ──event Lock────┐
                                                   │ trusted relayer
                                                   ▼
                                          OtterBridge (destination)
                                                   │
                                                   └──mint──▶ BridgeToken
```

- `OtterBridge.sol` deployed on both chains.
- Source side locks an underlying ERC-20 and emits `Lock(user, amount, destinationChainId, bridgeId, nonce)`.
- Destination side exposes `mint(user, amount, bridgeId)` callable only by the bridge owner.
- `BridgeToken.sol` is the wrapped ERC-20 minted 1:1 against locked underlying.
- Replay protection via `mapping(bytes32 => bool) minted`.

### Trust assumptions
V1 is **owner-gated**: the owner of the destination bridge is the relayer. It
can mint unbacked tokens if compromised. This is documented and acceptable only
for a controlled launch.

## V2 direction
- Replace the owner-gated `mint` with a messaging/ZK proof that the source
  `Lock` event was emitted.
- Options:
  1. **LayerZero / Axelar / CCIP** generic messaging with delivered payload.
  2. **ZK light client** proving a source-chain block/header containing the Lock
     event, verified on the destination chain.
  3. **Native mint/burn** across canonical token bridges for chains where they
     exist.

## Files
- `contracts/src/OtterBridge.sol`
- `contracts/src/BridgeToken.sol`
- `contracts/test/OtterBridge.t.sol`

## Operational notes
- The locking account must `approve` the bridge for the ERC-20 amount first;
  `lock` reverts with `ERC20InsufficientAllowance` otherwise.
- E2E verified against anvil on 2026-08-31 via the API
  (`POST /bridge/lock` → `GET /bridge/transfers` (pending) →
  `POST /bridge/mint` → status `minted`), wrapped balance checked on-chain.
