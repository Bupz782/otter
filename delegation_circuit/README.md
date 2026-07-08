# Delegation Circuit

Noir circuit that proves an agent's proposed on-chain intent is authorized by a
user-signed delegation message.

## Circuit Behavior

`main` enforces the following constraints:

1. The public `delegation_hash` equals `hash_delegation(delegation)`.
2. The delegation hash is signed by the secp256k1 public key in the delegation.
3. The proposed intent type is allowed by the delegation bitfield.
4. The proposed amount does not exceed the per-intent maximum.
5. The proposed protocol is in the delegation whitelist.
6. If `delegation.target_contract` is non-zero, the proposed intent must target
   the same contract (used to constrain ERC-20 tokens).
7. The delegation has not expired (`timestamp < expiry`).
8. The public nonce matches the delegation nonce (anti-replay).

## Structs

- `DelegationMessage` — signed limits (pubkey, allowed intents, max amounts,
  allowed protocols, expiry, nonce, target contract).
- `ProposedIntent` — intent the agent wants to execute.

## Hashing

`hash_delegation` serializes the delegation into 640 bytes and applies
`blake2s`:

```text
pubkey_x (32) || pubkey_y (32) || allowed_intents (32) ||
max_amounts[0..10] (10 * 32) || allowed_protocols[0..5] (5 * 32) ||
expiry (32) || nonce (32) || target_contract (32)
```

## Signature Scheme

The circuit uses Noir's built-in `std::ecdsa_secp256k1::verify_signature`.
Users can reuse existing EVM wallets; the agent never sees the private key.

## Commands

```bash
# Run circuit unit tests
nargo test

# Compile the circuit
nargo compile

# Generate a witness from Prover.toml
nargo execute
```

## Toolchain

- Noir `1.0.0-beta.22`
- Barretenberg `bb` `5.0.0-nightly.20260522`

Install both via `noirup` / `bbup`:

```bash
noirup -v 1.0.0-beta.22
bbup --version 5.0.0-nightly.20260522
```

## Solidity Verifier Generation

The Solidity verifier is generated with Barretenberg (`bb`). We use the
`evm-no-zk` target so that the verifier expects exactly the 38 user public
inputs declared in the circuit (the 8 pairing-point limbs stay inside the
proof bytes). This keeps the `DelegationVault` public-input layout simple.

From this directory:

```bash
# Compile the circuit
nargo compile

# Generate the verification key for the non-ZK EVM target
bb write_vk --scheme ultra_honk -t evm-no-zk \
  -b target/delegation_circuit.json -o /tmp/delegation_vk

# Generate the verifier contract
bb write_solidity_verifier --scheme ultra_honk -t evm-no-zk \
  -k /tmp/delegation_vk/vk -o /tmp/Verifier.sol

# The generated contract is named `HonkVerifier`; rename it to `DelegationVerifier`
# so the rest of the repo keeps a stable import.
sed 's/contract HonkVerifier/contract DelegationVerifier/' /tmp/Verifier.sol \
  > ../contracts/src/DelegationVerifier.sol
```

## End-to-End Smoke Test

```bash
# Rust side: generate real proof fixtures for the Foundry tests
cargo run -p infrastructure --bin generate-fixture

# Solidity side: verify the generated proof on a local EVM
cd ../contracts && forge test
```

## CLI Prove / Verify-Onchain

You can generate a proof for any intent from the command line and then verify
it on-chain against the vault's verifier contract (view call, no state change).

```bash
# Generate proof.bin + public_inputs.bin for an intent
cargo run -p interfaces --bin metis_cli -- prove \
  "lend 1000 USDC on Aave" \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --output-dir ./tmp

# Verify the proof on-chain (vault address is used to discover the verifier)
cargo run -p interfaces --bin metis_cli -- verify-onchain \
  --proof ./tmp/proof.bin \
  --public-inputs ./tmp/public_inputs.bin \
  --rpc-url http://localhost:8545 \
  --vault 0x... \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

## See Also

- `crates/domain/src/models/delegation.rs` — domain types.
- `crates/domain/src/ports/zkp_port.rs` — `ZkpPort` trait.
- `crates/infrastructure/src/zkp/noir_adapter.rs` — `NoirAdapter`.
