use domain::models::delegation::{
    DelegationProof, PrivateDelegationInputs, PublicDelegationInputs, serialize_public_inputs,
};
use domain::ports::zkp_port::{ZkpError, ZkpPort};

/// In-memory ZKP adapter for development and testing.
///
/// `prove_delegation` returns a deterministic mock proof. It does **not**
/// perform any cryptographic work, so it should never be used in production.
#[derive(Clone)]
pub struct MockZkpAdapter {
    verify_result: bool,
}

impl MockZkpAdapter {
    pub fn new() -> Self {
        Self {
            verify_result: true,
        }
    }

    pub fn failing() -> Self {
        Self {
            verify_result: false,
        }
    }

    pub fn with_verify_result(mut self, result: bool) -> Self {
        self.verify_result = result;
        self
    }
}

impl Default for MockZkpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ZkpPort for MockZkpAdapter {
    fn prove_delegation(
        &self,
        public_inputs: &PublicDelegationInputs,
        _private_inputs: &PrivateDelegationInputs,
    ) -> Result<DelegationProof, ZkpError> {
        Ok(DelegationProof {
            proof: b"mock-proof".to_vec(),
            public_inputs: serialize_public_inputs(public_inputs),
        })
    }

    fn verify_delegation(
        &self,
        _proof: &DelegationProof,
        _public_inputs: &PublicDelegationInputs,
    ) -> Result<bool, ZkpError> {
        Ok(self.verify_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::delegation::{
        DelegationMessage, ProposedDelegationIntent, field_from_u32, field_from_u64,
        field_from_u128,
    };

    fn sample_public_inputs() -> PublicDelegationInputs {
        PublicDelegationInputs {
            delegation_hash: [0u8; 32],
            proposed_intent: ProposedDelegationIntent {
                intent_type: field_from_u32(1),
                amount: field_from_u128(1000),
                protocol: field_from_u32(1),
                target_contract: field_from_u32(0),
            },
            timestamp: field_from_u64(1234567890),
            nonce: field_from_u64(42),
        }
    }

    fn sample_private_inputs() -> PrivateDelegationInputs {
        PrivateDelegationInputs {
            delegation: DelegationMessage {
                pubkey_x: [0u8; 32],
                pubkey_y: [0u8; 32],
                allowed_intents: field_from_u32(0x02),
                max_amounts: [field_from_u128(1000); 10],
                allowed_protocols: [field_from_u32(1); 5],
                expiry: field_from_u64(9999999999),
                nonce: field_from_u64(42),
                target_contract: field_from_u32(0),
            },
            signature: [0u8; 64],
        }
    }

    #[test]
    fn prove_returns_mock_proof() {
        let adapter = MockZkpAdapter::new();
        let proof = adapter
            .prove_delegation(&sample_public_inputs(), &sample_private_inputs())
            .unwrap();
        assert_eq!(proof.proof, b"mock-proof");
    }

    #[test]
    fn verify_returns_configured_result() {
        let adapter = MockZkpAdapter::new().with_verify_result(false);
        let proof = adapter
            .prove_delegation(&sample_public_inputs(), &sample_private_inputs())
            .unwrap();
        assert!(
            !adapter
                .verify_delegation(&proof, &sample_public_inputs())
                .unwrap()
        );
    }
}
