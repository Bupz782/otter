use blake2::{Blake2s256, Digest};
use domain::models::delegation::{
    DelegationMessage, PrivateDelegationInputs, ProposedDelegationIntent, PublicDelegationInputs,
    field_from_u32, field_from_u64, field_from_u128, hash_delegation, serialize_delegation,
};
use domain::ports::zkp_port::ZkpPort;
use infrastructure::zkp::NoirAdapter;
use k256::ecdsa::{SigningKey, signature::DigestSigner};
use std::path::PathBuf;

fn sign_delegation(delegation: &DelegationMessage, signing_key: &SigningKey) -> [u8; 64] {
    let serialized = serialize_delegation(delegation);
    let signature: k256::ecdsa::Signature =
        signing_key.sign_digest(Blake2s256::new().chain_update(serialized));
    signature.to_bytes().into()
}

fn main() {
    let home = std::env::var("HOME").expect("HOME not set");
    let bb_bin = format!("{}/.bb/bb", home);
    if !std::path::Path::new(&bb_bin).exists() {
        eprintln!(
            "bb binary not found at {}. Install it with bbup first.",
            bb_bin
        );
        std::process::exit(1);
    }

    let circuit_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("delegation_circuit");

    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("contracts")
        .join("test")
        .join("fixtures");
    std::fs::create_dir_all(&fixture_dir).expect("create fixture dir");

    // Deterministic signing key: a fixed 32-byte scalar.
    let mut key_bytes = [0u8; 32];
    key_bytes[31] = 0x01;
    let signing_key = SigningKey::from_bytes(&key_bytes.into()).expect("valid signing key");
    let verifying_key = signing_key.verifying_key();
    let encoded = verifying_key.to_encoded_point(false);
    let pubkey_bytes = encoded.as_bytes();
    assert_eq!(pubkey_bytes.len(), 65);
    assert_eq!(pubkey_bytes[0], 0x04);

    let mut pubkey_x = [0u8; 32];
    let mut pubkey_y = [0u8; 32];
    pubkey_x.copy_from_slice(&pubkey_bytes[1..33]);
    pubkey_y.copy_from_slice(&pubkey_bytes[33..65]);

    let delegation = DelegationMessage {
        pubkey_x,
        pubkey_y,
        allowed_intents: field_from_u32(0x05),
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
    };

    let delegation_hash = hash_delegation(&delegation);
    let signature = sign_delegation(&delegation, &signing_key);

    let proposed_intent = ProposedDelegationIntent {
        intent_type: field_from_u32(2),
        amount: field_from_u128(2_000_000),
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

    let adapter = NoirAdapter::new(circuit_dir, "nargo", Some(bb_bin));
    let proof = adapter
        .prove_delegation(&public_inputs, &private_inputs)
        .expect("proof generation failed");

    assert!(!proof.proof.is_empty(), "proof bytes are empty");

    std::fs::write(fixture_dir.join("proof.bin"), &proof.proof).expect("write proof");
    std::fs::write(fixture_dir.join("public_inputs.bin"), &proof.public_inputs)
        .expect("write public inputs");

    let delegation_json = serde_json::json!({
        "delegation_hash": format!("0x{}", hex::encode(delegation_hash)),
        "allowed_intents": "0x05",
        "max_amounts": [
            "1000000", "2000000", "3000000", "0", "0", "0", "0", "0", "0", "0"
        ],
        "allowed_protocols": ["1", "2", "0", "0", "0"],
        "expiry": "4000000000",
        "nonce": "42",
        "intent_type": "2",
        "amount": "2000000",
        "protocol": "1",
        "target_contract": "0",
        "timestamp": "1000000",
    });
    std::fs::write(
        fixture_dir.join("delegation.json"),
        serde_json::to_string_pretty(&delegation_json).unwrap(),
    )
    .expect("write delegation json");

    println!("Wrote fixtures:");
    println!("  proof bytes: {}", proof.proof.len());
    println!("  public inputs bytes: {}", proof.public_inputs.len());
    println!("  fixture dir: {}", fixture_dir.display());
    println!(
        "  delegation_hash: {}",
        delegation_json["delegation_hash"].as_str().unwrap()
    );
}
