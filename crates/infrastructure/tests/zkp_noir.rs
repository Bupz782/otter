use blake2::{Blake2s256, Digest};
use domain::models::delegation::{
    DelegationMessage, PrivateDelegationInputs, ProposedDelegationIntent, PublicDelegationInputs,
    field_from_u32, field_from_u64, field_from_u128, hash_delegation, serialize_delegation,
};
use domain::ports::zkp_port::ZkpPort;
use infrastructure::zkp::NoirAdapter;
use k256::ecdsa::{SigningKey, signature::DigestSigner};
use rand::rngs::OsRng;
use serial_test::serial;
use std::path::PathBuf;

fn sign_delegation(delegation: &DelegationMessage, signing_key: &SigningKey) -> [u8; 64] {
    let serialized = serialize_delegation(delegation);
    let signature: k256::ecdsa::Signature =
        signing_key.sign_digest(Blake2s256::new().chain_update(serialized));
    signature.to_bytes().into()
}

fn generate_test_keypair() -> (SigningKey, [u8; 32], [u8; 32]) {
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    let encoded = verifying_key.to_encoded_point(false);
    let pubkey_bytes = encoded.as_bytes();
    assert_eq!(pubkey_bytes.len(), 65);
    assert_eq!(pubkey_bytes[0], 0x04);

    let mut pubkey_x = [0u8; 32];
    let mut pubkey_y = [0u8; 32];
    pubkey_x.copy_from_slice(&pubkey_bytes[1..33]);
    pubkey_y.copy_from_slice(&pubkey_bytes[33..65]);
    (signing_key, pubkey_x, pubkey_y)
}

fn make_delegation(pubkey_x: [u8; 32], pubkey_y: [u8; 32]) -> DelegationMessage {
    DelegationMessage {
        pubkey_x,
        pubkey_y,
        allowed_intents: field_from_u32(0x05), // intent types 0 and 2 allowed
        max_amounts: [
            field_from_u128(1_000_000),
            field_from_u128(2_000_000),
            field_from_u128(3_000_000),
            field_from_u128(0),
            field_from_u128(0),
            field_from_u128(0),
            field_from_u128(0),
            field_from_u128(0),
            field_from_u128(0),
            field_from_u128(0),
        ],
        allowed_protocols: [
            field_from_u32(1),
            field_from_u32(2),
            field_from_u32(0),
            field_from_u32(0),
            field_from_u32(0),
        ],
        expiry: field_from_u64(4_000_000_000),
        nonce: field_from_u64(42),
        target_contract: field_from_u32(0),
    }
}

#[test]
#[serial]
fn noir_adapter_generates_witness_for_valid_delegation() {
    let (signing_key, pubkey_x, pubkey_y) = generate_test_keypair();
    let delegation = make_delegation(pubkey_x, pubkey_y);
    let delegation_hash = hash_delegation(&delegation);
    let signature = sign_delegation(&delegation, &signing_key);

    let proposed_intent = ProposedDelegationIntent {
        intent_type: field_from_u32(2),     // allowed (bit 2 set)
        amount: field_from_u128(2_000_000), // within max for type 2
        protocol: field_from_u32(1),        // whitelisted
        target_contract: field_from_u32(0),
    };

    let public_inputs = PublicDelegationInputs {
        delegation_hash,
        proposed_intent,
        timestamp: field_from_u64(1_000_000),
        nonce: delegation.nonce,
    };

    let private_inputs = PrivateDelegationInputs {
        delegation,
        signature,
    };

    let circuit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("delegation_circuit");
    let bb_bin = std::env::var("BB_BIN")
        .ok()
        .or_else(|| {
            let home = std::env::var("HOME").ok()?;
            Some(format!("{}/.bb/bb", home))
        })
        .filter(|p| std::path::Path::new(p).exists());
    let adapter = NoirAdapter::new(circuit_dir, "nargo", bb_bin.as_deref());
    let result = adapter.prove_delegation(&public_inputs, &private_inputs);
    let proof = result.unwrap_or_else(|e| {
        panic!("witness generation should succeed for valid delegation: {e:?}")
    });

    if bb_bin.is_some() {
        assert!(
            !proof.proof.is_empty(),
            "bb was available but proof bytes are empty"
        );
    }
    assert_eq!(
        proof.public_inputs.len(),
        38 * 32,
        "public inputs must contain 38 field elements"
    );

    // The first 32 fields hold the hash bytes in their low byte.
    for (i, byte) in public_inputs.delegation_hash.iter().enumerate() {
        assert_eq!(proof.public_inputs[i * 32 + 31], *byte);
    }
}

#[test]
#[serial]
fn noir_adapter_rejects_invalid_delegation() {
    let (signing_key, pubkey_x, pubkey_y) = generate_test_keypair();
    let delegation = make_delegation(pubkey_x, pubkey_y);
    let delegation_hash = hash_delegation(&delegation);
    let signature = sign_delegation(&delegation, &signing_key);

    // Intent type 5 is not allowed (allowed_intents = 0x05 allows bits 0 and 2).
    let proposed_intent = ProposedDelegationIntent {
        intent_type: field_from_u32(5),
        amount: field_from_u128(100),
        protocol: field_from_u32(1),
        target_contract: field_from_u32(0),
    };

    let public_inputs = PublicDelegationInputs {
        delegation_hash,
        proposed_intent,
        timestamp: field_from_u64(1_000_000),
        nonce: delegation.nonce,
    };

    let private_inputs = PrivateDelegationInputs {
        delegation,
        signature,
    };

    let circuit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("delegation_circuit");
    let adapter = NoirAdapter::new(circuit_dir, "nargo", None::<String>);
    let result = adapter.prove_delegation(&public_inputs, &private_inputs);
    assert!(
        result.is_err(),
        "expected constraint failure for disallowed intent type"
    );
}

#[test]
#[serial]
fn noir_adapter_enforces_matching_target_contract() {
    let (signing_key, pubkey_x, pubkey_y) = generate_test_keypair();
    let mut delegation = make_delegation(pubkey_x, pubkey_y);
    delegation.target_contract = field_from_u32(0x12345678);
    let delegation_hash = hash_delegation(&delegation);
    let signature = sign_delegation(&delegation, &signing_key);

    let proposed_intent = ProposedDelegationIntent {
        intent_type: field_from_u32(2),
        amount: field_from_u128(2_000_000),
        protocol: field_from_u32(1),
        target_contract: field_from_u32(0x12345678),
    };

    let public_inputs = PublicDelegationInputs {
        delegation_hash,
        proposed_intent,
        timestamp: field_from_u64(1_000_000),
        nonce: delegation.nonce,
    };

    let private_inputs = PrivateDelegationInputs {
        delegation,
        signature,
    };

    let circuit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("delegation_circuit");
    let adapter = NoirAdapter::new(circuit_dir, "nargo", None::<String>);
    let result = adapter.prove_delegation(&public_inputs, &private_inputs);
    assert!(
        result.is_ok(),
        "expected witness generation to succeed when target_contract matches: {:?}",
        result.err()
    );
}

#[test]
#[serial]
fn noir_adapter_rejects_mismatched_target_contract() {
    let (signing_key, pubkey_x, pubkey_y) = generate_test_keypair();
    let mut delegation = make_delegation(pubkey_x, pubkey_y);
    delegation.target_contract = field_from_u32(0x12345678);
    let delegation_hash = hash_delegation(&delegation);
    let signature = sign_delegation(&delegation, &signing_key);

    let proposed_intent = ProposedDelegationIntent {
        intent_type: field_from_u32(2),
        amount: field_from_u128(2_000_000),
        protocol: field_from_u32(1),
        target_contract: field_from_u32(0xdeadbeef),
    };

    let public_inputs = PublicDelegationInputs {
        delegation_hash,
        proposed_intent,
        timestamp: field_from_u64(1_000_000),
        nonce: delegation.nonce,
    };

    let private_inputs = PrivateDelegationInputs {
        delegation,
        signature,
    };

    let circuit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("delegation_circuit");
    let adapter = NoirAdapter::new(circuit_dir, "nargo", None::<String>);
    let result = adapter.prove_delegation(&public_inputs, &private_inputs);
    assert!(
        result.is_err(),
        "expected constraint failure when target_contract does not match"
    );
}

type ConstraintMutation = Box<dyn Fn(&mut PublicDelegationInputs, &mut PrivateDelegationInputs)>;

#[test]
#[serial]
fn noir_adapter_rejects_invalid_constraints() {
    let (signing_key, pubkey_x, pubkey_y) = generate_test_keypair();
    let delegation = make_delegation(pubkey_x, pubkey_y);
    let delegation_hash = hash_delegation(&delegation);
    let signature = sign_delegation(&delegation, &signing_key);
    let base_public_inputs = PublicDelegationInputs {
        delegation_hash,
        proposed_intent: ProposedDelegationIntent {
            intent_type: field_from_u32(2),
            amount: field_from_u128(2_000_000),
            protocol: field_from_u32(1),
            target_contract: field_from_u32(0),
        },
        timestamp: field_from_u64(1_000_000),
        nonce: delegation.nonce,
    };
    let base_private_inputs = PrivateDelegationInputs {
        delegation: delegation.clone(),
        signature,
    };

    let circuit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("delegation_circuit");
    let adapter = NoirAdapter::new(circuit_dir, "nargo", None::<String>);

    let mut cases: Vec<(ConstraintMutation, &str)> = Vec::new();

    cases.push((
        Box::new(|pi, _| {
            pi.proposed_intent.intent_type = field_from_u32(5);
        }),
        "disallowed intent type",
    ));

    cases.push((
        Box::new(|pi, _| {
            pi.proposed_intent.amount = field_from_u128(5_000_000);
        }),
        "amount exceeds max",
    ));

    cases.push((
        Box::new(|pi, _| {
            pi.proposed_intent.protocol = field_from_u32(5);
        }),
        "protocol not whitelisted",
    ));

    cases.push((
        Box::new(|pi, _| {
            pi.timestamp = field_from_u64(4_000_000_001);
        }),
        "delegation expired",
    ));

    cases.push((
        Box::new(|pi, _| {
            pi.nonce = field_from_u64(99);
        }),
        "nonce mismatch",
    ));

    cases.push((
        Box::new(|pi, _| {
            pi.delegation_hash[0] ^= 0xff;
        }),
        "delegation hash mismatch",
    ));

    cases.push((
        Box::new(|_, priv_in| {
            let other_key = SigningKey::random(&mut OsRng);
            priv_in.signature = sign_delegation(&priv_in.delegation, &other_key);
        }),
        "invalid signature",
    ));

    for (mutator, description) in cases {
        let mut pi = base_public_inputs.clone();
        let mut priv_in = base_private_inputs.clone();
        mutator(&mut pi, &mut priv_in);
        let result = adapter.prove_delegation(&pi, &priv_in);
        assert!(
            result.is_err(),
            "expected constraint failure for {}",
            description
        );
    }
}
