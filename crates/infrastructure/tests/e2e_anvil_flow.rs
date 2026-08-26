use blake2::{Blake2s256, Digest};
use domain::models::delegation::{
    DelegationMessage, PrivateDelegationInputs, ProposedDelegationIntent, PublicDelegationInputs,
    field_from_u32, field_from_u64, field_from_u128, hash_delegation, serialize_delegation,
};
use domain::models::transaction::Transaction;
use domain::ports::evm_port::EvmPort;
use domain::ports::zkp_port::ZkpPort;
use infrastructure::blockchain::AlloyEvmAdapter;
use infrastructure::zkp::NoirAdapter;
use k256::ecdsa::{SigningKey, signature::DigestSigner};
use rand::rngs::OsRng;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Fresh timestamp in seconds: the vault rejects proofs older than
/// MAX_PROOF_AGE (staleness bound on the delegation).
fn fresh_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

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
fn deposit_delegate_and_execute_with_real_proof_on_anvil() {
    let rpc_url = match std::env::var("OTTER_TEST_RPC_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping e2e_anvil_flow: OTTER_TEST_RPC_URL not set");
            return;
        }
    };
    let vault_address = match std::env::var("OTTER_TEST_VAULT_ADDRESS") {
        Ok(addr) => addr,
        Err(_) => {
            eprintln!("Skipping e2e_anvil_flow: OTTER_TEST_VAULT_ADDRESS not set");
            return;
        }
    };
    let private_key = match std::env::var("OTTER_TEST_PRIVATE_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Skipping e2e_anvil_flow: OTTER_TEST_PRIVATE_KEY not set");
            return;
        }
    };

    let bb_bin = std::env::var("BB_BIN")
        .ok()
        .or_else(|| {
            let home = std::env::var("HOME").ok()?;
            Some(format!("{}/.bb/bb", home))
        })
        .filter(|p| std::path::Path::new(p).exists());

    if bb_bin.is_none() {
        eprintln!("Skipping e2e_anvil_flow: BB_BIN not found");
        return;
    }

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
        timestamp: field_from_u64(fresh_timestamp()),
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
    let zkp = NoirAdapter::new(circuit_dir, "nargo", bb_bin.as_deref());
    let proof = zkp
        .prove_delegation(&public_inputs, &private_inputs)
        .expect("proof generation should succeed");
    assert!(
        !proof.proof.is_empty(),
        "bb should produce real proof bytes"
    );

    let evm = AlloyEvmAdapter::new(rpc_url.clone(), &private_key, &vault_address)
        .expect("evm adapter should build");

    // Register the delegation on-chain (same key acts as owner/agent for this test).
    evm.ensure_delegated(&private_inputs.delegation)
        .expect("delegation should be registered");

    // Whitelist a router for protocol 1: the vault transfers funds there on
    // execution (both ETH and ERC-20 paths).
    evm.set_protocol_router(1, "0x2222222222222222222222222222222222222222")
        .expect("protocol router should be registered");

    // Deposit native ETH into the vault.
    let deposit = Transaction::new(&vault_address, 100_000_000_000_000_000u128, 100_000);
    let deposit_tx = evm
        .send_transaction(&deposit)
        .expect("deposit transaction should be submitted");
    let deposit_receipt = evm
        .get_transaction_receipt(&deposit_tx)
        .expect("deposit receipt fetch should not fail")
        .expect("deposit should be mined");
    assert!(deposit_receipt.status, "deposit transaction should succeed");

    // Execute the intent via the vault using the ZK proof.
    let execute_tx = evm
        .execute_with_proof(&proof, &public_inputs)
        .expect("executeWithProof should be submitted");
    let execute_receipt = evm
        .get_transaction_receipt(&execute_tx)
        .expect("execute receipt fetch should not fail")
        .expect("execute should be mined");
    assert!(
        execute_receipt.status,
        "executeWithProof transaction should succeed"
    );
}
