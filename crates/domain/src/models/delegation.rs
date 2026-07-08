use serde::Serialize;

/// A 32-byte big-endian representation of a Noir `Field` element.
pub type FieldBytes = [u8; 32];

/// Number of distinct intent types supported by a delegation.
pub const INTENT_TYPE_COUNT: usize = 10;

/// Number of protocols that can be whitelisted in a delegation.
pub const ALLOWED_PROTOCOL_COUNT: usize = 5;

/// A delegation message signed by the user. Limits what an agent can do on-chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DelegationMessage {
    /// secp256k1 public key X coordinate (32 bytes).
    pub pubkey_x: [u8; 32],
    /// secp256k1 public key Y coordinate (32 bytes).
    pub pubkey_y: [u8; 32],
    /// Bitfield of allowed intent types.
    pub allowed_intents: FieldBytes,
    /// Max amount allowed for each intent type.
    pub max_amounts: [FieldBytes; INTENT_TYPE_COUNT],
    /// Whitelisted protocol identifiers.
    pub allowed_protocols: [FieldBytes; ALLOWED_PROTOCOL_COUNT],
    /// Expiry timestamp (seconds since epoch).
    pub expiry: FieldBytes,
    /// Anti-replay nonce.
    pub nonce: FieldBytes,
    /// Allowed target contract (token address, or 0 for native ETH / unconstrained).
    pub target_contract: FieldBytes,
}

/// Intent proposed by the agent for execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedDelegationIntent {
    pub intent_type: FieldBytes,
    pub amount: FieldBytes,
    pub protocol: FieldBytes,
    pub target_contract: FieldBytes,
}

/// Public inputs to the delegation circuit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicDelegationInputs {
    pub delegation_hash: [u8; 32],
    pub proposed_intent: ProposedDelegationIntent,
    pub timestamp: FieldBytes,
    pub nonce: FieldBytes,
}

/// Private inputs to the delegation circuit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivateDelegationInputs {
    pub delegation: DelegationMessage,
    pub signature: [u8; 64],
}

/// A delegation proof produced by a ZKP backend.
///
/// The contents are opaque to the domain; verification is backend-specific.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationProof {
    pub proof: Vec<u8>,
    pub public_inputs: Vec<u8>,
}

/// Helper to convert a `u128` into a 32-byte big-endian Noir `Field`.
pub fn field_from_u128(value: u128) -> FieldBytes {
    let mut bytes = [0u8; 32];
    bytes[16..32].copy_from_slice(&value.to_be_bytes());
    bytes
}

/// Helper to convert a `u64` into a 32-byte big-endian Noir `Field`.
pub fn field_from_u64(value: u64) -> FieldBytes {
    let mut bytes = [0u8; 32];
    bytes[24..32].copy_from_slice(&value.to_be_bytes());
    bytes
}

/// Helper to convert a `u32` into a 32-byte big-endian Noir `Field`.
pub fn field_from_u32(value: u32) -> FieldBytes {
    let mut bytes = [0u8; 32];
    bytes[28..32].copy_from_slice(&value.to_be_bytes());
    bytes
}

/// Helper to convert a single byte into a 32-byte big-endian Noir `Field`.
pub fn field_from_u8(value: u8) -> FieldBytes {
    let mut bytes = [0u8; 32];
    bytes[31] = value;
    bytes
}

/// Serialize a `FieldBytes` as a quoted hexadecimal string for Noir TOML inputs.
///
/// Nargo requires large field elements to be wrapped in double quotes so that
/// they are parsed as arbitrary-precision Field values rather than native
/// integers.
pub fn field_to_hex(value: &FieldBytes) -> String {
    format!("\"0x{}\"", hex::encode(value))
}

/// Serialize the public inputs into the 38 × 32-byte layout expected by the
/// on-chain verifier and the Noir circuit:
///   - delegation_hash: 32 field elements, one per byte
///   - proposed_intent: intent_type, amount, protocol, target_contract
///   - timestamp
///   - nonce
pub fn serialize_public_inputs(public_inputs: &PublicDelegationInputs) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(38 * 32);

    for byte in &public_inputs.delegation_hash {
        bytes.extend_from_slice(&field_from_u8(*byte));
    }
    bytes.extend_from_slice(&public_inputs.proposed_intent.intent_type);
    bytes.extend_from_slice(&public_inputs.proposed_intent.amount);
    bytes.extend_from_slice(&public_inputs.proposed_intent.protocol);
    bytes.extend_from_slice(&public_inputs.proposed_intent.target_contract);
    bytes.extend_from_slice(&public_inputs.timestamp);
    bytes.extend_from_slice(&public_inputs.nonce);

    bytes
}

/// Total serialized size of a DelegationMessage in bytes, matching the Noir
/// circuit layout:
/// pubkey_x (32) + pubkey_y (32) + allowed_intents (32) +
/// max_amounts (10 * 32) + allowed_protocols (5 * 32) +
/// expiry (32) + nonce (32) + target_contract (32).
pub const DELEGATION_SERIALIZED_SIZE: usize =
    32 + 32 + 32 + (INTENT_TYPE_COUNT * 32) + (ALLOWED_PROTOCOL_COUNT * 32) + 32 + 32 + 32;

/// Serialize a delegation message into the exact byte layout expected by the
/// Noir `hash_delegation` function.
pub fn serialize_delegation(delegation: &DelegationMessage) -> [u8; DELEGATION_SERIALIZED_SIZE] {
    let mut bytes = [0u8; DELEGATION_SERIALIZED_SIZE];
    let mut offset = 0usize;

    bytes[offset..offset + 32].copy_from_slice(&delegation.pubkey_x);
    offset += 32;
    bytes[offset..offset + 32].copy_from_slice(&delegation.pubkey_y);
    offset += 32;
    bytes[offset..offset + 32].copy_from_slice(&delegation.allowed_intents);
    offset += 32;

    for i in 0..INTENT_TYPE_COUNT {
        bytes[offset..offset + 32].copy_from_slice(&delegation.max_amounts[i]);
        offset += 32;
    }

    for i in 0..ALLOWED_PROTOCOL_COUNT {
        bytes[offset..offset + 32].copy_from_slice(&delegation.allowed_protocols[i]);
        offset += 32;
    }

    bytes[offset..offset + 32].copy_from_slice(&delegation.expiry);
    offset += 32;
    bytes[offset..offset + 32].copy_from_slice(&delegation.nonce);
    offset += 32;
    bytes[offset..offset + 32].copy_from_slice(&delegation.target_contract);

    bytes
}

/// Compute the blake2s hash of a serialized delegation message.
///
/// This must match the `hash_delegation` function in the Noir circuit.
pub fn hash_delegation(delegation: &DelegationMessage) -> [u8; 32] {
    use blake2::{Blake2s256, Digest};
    let serialized = serialize_delegation(delegation);
    let mut hasher = Blake2s256::new();
    hasher.update(serialized);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_from_u128_is_big_endian_padded() {
        let field = field_from_u128(0x0102_0304_0506_0708_090a_0b0c_0d0e_0f10);
        assert_eq!(
            field[16..],
            [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
                0x0f, 0x10
            ]
        );
        assert!(field[..16].iter().all(|&b| b == 0));
    }

    #[test]
    fn field_from_u64_is_big_endian_padded() {
        let field = field_from_u64(0x0102_0304_0506_0708);
        assert_eq!(
            field[24..],
            [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]
        );
        assert!(field[..24].iter().all(|&b| b == 0));
    }

    #[test]
    fn serialize_public_inputs_produces_38_fields() {
        let public_inputs = PublicDelegationInputs {
            delegation_hash: [0xab; 32],
            proposed_intent: ProposedDelegationIntent {
                intent_type: field_from_u32(1),
                amount: field_from_u128(1000),
                protocol: field_from_u32(4),
                target_contract: field_from_u32(0),
            },
            timestamp: field_from_u64(1_000_000),
            nonce: field_from_u64(42),
        };
        let bytes = serialize_public_inputs(&public_inputs);
        assert_eq!(bytes.len(), 38 * 32);

        // First 32 fields should each hold one byte of the hash in their low byte.
        for (i, byte) in public_inputs.delegation_hash.iter().enumerate() {
            let field_start = i * 32;
            let field = &bytes[field_start..field_start + 32];
            assert_eq!(field[31], *byte, "hash byte {} mismatch", i);
            assert!(field[..31].iter().all(|&b| b == 0));
        }
    }
}
