use std::ecdsa_secp256k1;
use std::hash::blake2s;

/// Number of distinct intent types supported by a delegation.
global INTENT_TYPE_COUNT: u32 = 10;

/// Number of protocols that can be whitelisted in a delegation.
global ALLOWED_PROTOCOL_COUNT: u32 = 5;

/// Total serialized size of a DelegationMessage in bytes:
/// pubkey_x (32) + pubkey_y (32) + allowed_intents (32) +
/// max_amounts (10 * 32) + allowed_protocols (5 * 32) +
/// expiry (32) + nonce (32) + target_contract (32) = 640 bytes.
global DELEGATION_SERIALIZED_SIZE: u32 = 32 + 32 + 32 + (10 * 32) + (5 * 32) + 32 + 32 + 32;

/// A delegation message signed by the user. Limits what an agent can do.
struct DelegationMessage {
    /// secp256k1 public key X coordinate (32 bytes).
    pubkey_x: [u8; 32],
    /// secp256k1 public key Y coordinate (32 bytes).
    pubkey_y: [u8; 32],
    /// Bitfield of allowed intent types.
    allowed_intents: Field,
    /// Max amount allowed for each intent type.
    max_amounts: [Field; INTENT_TYPE_COUNT],
    /// Whitelisted protocol identifiers.
    allowed_protocols: [Field; ALLOWED_PROTOCOL_COUNT],
    /// Expiry timestamp (seconds since epoch).
    expiry: Field,
    /// Anti-replay nonce.
    nonce: Field,
    /// Allowed target contract (token address, or 0 for native ETH / unconstrained).
    target_contract: Field,
}

/// Intent proposed by the agent for execution.
struct ProposedIntent {
    intent_type: Field,
    amount: Field,
    protocol: Field,
    target_contract: Field,
}

/// Serialize a DelegationMessage into a fixed-size byte array and hash it.
fn hash_delegation(delegation: DelegationMessage) -> [u8; 32] {
    let mut bytes: [u8; DELEGATION_SERIALIZED_SIZE] = [0; DELEGATION_SERIALIZED_SIZE];
    let mut offset: u32 = 0;

    // pubkey_x
    for i in 0..32 {
        bytes[offset + i] = delegation.pubkey_x[i];
    }
    offset += 32;

    // pubkey_y
    for i in 0..32 {
        bytes[offset + i] = delegation.pubkey_y[i];
    }
    offset += 32;

    // allowed_intents
    let allowed_intents_bytes = delegation.allowed_intents.to_be_bytes::<32>();
    for i in 0..32 {
        bytes[offset + i] = allowed_intents_bytes[i];
    }
    offset += 32;

    // max_amounts
    for i in 0..INTENT_TYPE_COUNT {
        let amount_bytes = delegation.max_amounts[i].to_be_bytes::<32>();
        for j in 0..32 {
            bytes[offset + j] = amount_bytes[j];
        }
        offset += 32;
    }

    // allowed_protocols
    for i in 0..ALLOWED_PROTOCOL_COUNT {
        let protocol_bytes = delegation.allowed_protocols[i].to_be_bytes::<32>();
        for j in 0..32 {
            bytes[offset + j] = protocol_bytes[j];
        }
        offset += 32;
    }

    // expiry
    let expiry_bytes = delegation.expiry.to_be_bytes::<32>();
    for i in 0..32 {
        bytes[offset + i] = expiry_bytes[i];
    }
    offset += 32;

    // nonce
    let nonce_bytes = delegation.nonce.to_be_bytes::<32>();
    for i in 0..32 {
        bytes[offset + i] = nonce_bytes[i];
    }
    offset += 32;

    // target_contract
    let target_contract_bytes = delegation.target_contract.to_be_bytes::<32>();
    for i in 0..32 {
        bytes[offset + i] = target_contract_bytes[i];
    }

    blake2s(bytes)
}

/// Check if the bit at position `bit` is set in `bitfield`.
fn is_bit_set(bitfield: Field, bit: Field) -> bool {
    let bit_as_u32: u32 = bit as u32;
    let bits: [bool; 32] = bitfield.to_le_bits::<32>();
    bits[bit_as_u32]
}

/// Check if `value` is present in `array`.
fn contains(array: [Field; ALLOWED_PROTOCOL_COUNT], value: Field) -> bool {
    let mut found = false;
    for i in 0..ALLOWED_PROTOCOL_COUNT {
        if array[i] == value {
            found = true;
        }
    }
    found
}

fn main(
    delegation_hash: pub [u8; 32],
    proposed_intent: pub ProposedIntent,
    timestamp: pub Field,
    nonce: pub Field,
    delegation: DelegationMessage,
    signature: [u8; 64],
) {
    // 1. The public delegation_hash must match the hash of the private delegation.
    let computed_hash = hash_delegation(delegation);
    assert(computed_hash == delegation_hash);

    // 2. The user must have signed the delegation_hash.
    let signature_valid = ecdsa_secp256k1::verify_signature(
        delegation.pubkey_x,
        delegation.pubkey_y,
        signature,
        delegation_hash,
    );
    assert(signature_valid as bool);

    // 3. The proposed intent type must be allowed.
    assert(is_bit_set(delegation.allowed_intents, proposed_intent.intent_type));

    // 4. The proposed amount must not exceed the max for this intent type.
    let intent_index: u32 = proposed_intent.intent_type as u32;
    assert((proposed_intent.amount as u128) <= (delegation.max_amounts[intent_index] as u128));

    // 5. The proposed protocol must be whitelisted.
    assert(contains(delegation.allowed_protocols, proposed_intent.protocol));

    // 5b. If a target contract is specified, the proposed intent must target it.
    // A value of 0 means "any target" (typically native ETH / unconstrained).
    if delegation.target_contract != 0 {
        assert(proposed_intent.target_contract == delegation.target_contract);
    }

    // 6. The delegation must not have expired.
    assert((timestamp as u64) < (delegation.expiry as u64));

    // 7. The nonce must match the delegation nonce.
    assert(nonce == delegation.nonce);
}

#[test]
fn test_hash_delegation_is_deterministic() {
    let delegation = DelegationMessage {
        pubkey_x: [0; 32],
        pubkey_y: [1; 32],
        allowed_intents: 0x05, // bits 0 and 2 set
        max_amounts: [1000, 2000, 3000, 0, 0, 0, 0, 0, 0, 0],
        allowed_protocols: [1, 2, 0, 0, 0],
        expiry: 1_000_000,
        nonce: 42,
        target_contract: 0,
    };

    let hash1 = hash_delegation(delegation);
    let hash2 = hash_delegation(delegation);
    assert(hash1 == hash2);
}

#[test]
fn test_is_bit_set() {
    assert(is_bit_set(0x05, 0)); // bit 0 set
    assert(is_bit_set(0x05, 2)); // bit 2 set
    assert(!is_bit_set(0x05, 1)); // bit 1 not set
}

#[test]
fn test_contains() {
    let array = [1, 2, 3, 4, 5];
    assert(contains(array, 3));
    assert(!contains(array, 99));
}
