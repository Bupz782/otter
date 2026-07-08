use domain::models::delegation::{
    DelegationMessage, DelegationProof, FieldBytes, PrivateDelegationInputs,
    ProposedDelegationIntent, PublicDelegationInputs, field_from_u32, field_from_u64,
    field_from_u128,
};
use domain::models::intent::{Asset, DexType, Intent, LendingType, ValidationError};
use domain::ports::evm_port::{EvmError, EvmPort};
use domain::ports::intent_parser_port::{IntentParserError, IntentParserPort};
use domain::ports::price_oracle_port::{OracleError, PriceOraclePort};
use domain::ports::zkp_port::{ZkpError, ZkpPort};

use super::evaluate_condition::{EvaluateConditionUseCase, EvaluationError};

/// Execute a natural-language intent end-to-end:
/// parse → evaluate condition → validate → prove delegation → submit on-chain.
pub struct ExecuteIntentUseCase<P, O, Z, E> {
    parser: P,
    oracle: O,
    zkp: Z,
    evm: E,
    chain_id: u64,
}

#[derive(Debug)]
pub enum ExecutionError {
    ParsingFailed(String),
    ConditionNotMet,
    InvalidIntent(String),
    UnsupportedIntentType(String),
    OracleUnavailable(String),
    ProofFailed(String),
    SubmissionFailed(String),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::ParsingFailed(msg) => write!(f, "parsing failed: {}", msg),
            ExecutionError::ConditionNotMet => write!(f, "condition not met"),
            ExecutionError::InvalidIntent(msg) => write!(f, "invalid intent: {}", msg),
            ExecutionError::UnsupportedIntentType(msg) => write!(f, "unsupported intent: {}", msg),
            ExecutionError::OracleUnavailable(msg) => write!(f, "oracle unavailable: {}", msg),
            ExecutionError::ProofFailed(msg) => write!(f, "proof failed: {}", msg),
            ExecutionError::SubmissionFailed(msg) => write!(f, "submission failed: {}", msg),
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<IntentParserError> for ExecutionError {
    fn from(err: IntentParserError) -> Self {
        ExecutionError::ParsingFailed(err.to_string())
    }
}

impl From<EvaluationError> for ExecutionError {
    fn from(err: EvaluationError) -> Self {
        match err {
            EvaluationError::OracleUnavailable(msg) => ExecutionError::OracleUnavailable(msg),
            EvaluationError::InvalidMetric(msg) => ExecutionError::OracleUnavailable(msg),
            EvaluationError::EvaluationFailed(msg) => ExecutionError::OracleUnavailable(msg),
        }
    }
}

impl From<OracleError> for ExecutionError {
    fn from(err: OracleError) -> Self {
        ExecutionError::OracleUnavailable(err.to_string())
    }
}

impl From<ValidationError> for ExecutionError {
    fn from(err: ValidationError) -> Self {
        ExecutionError::InvalidIntent(format!("{:?}", err))
    }
}

impl From<ZkpError> for ExecutionError {
    fn from(err: ZkpError) -> Self {
        ExecutionError::ProofFailed(err.to_string())
    }
}

impl From<EvmError> for ExecutionError {
    fn from(err: EvmError) -> Self {
        ExecutionError::SubmissionFailed(err.to_string())
    }
}

impl<P, O, Z, E> ExecuteIntentUseCase<P, O, Z, E>
where
    P: IntentParserPort,
    O: PriceOraclePort,
    Z: ZkpPort,
    E: EvmPort,
{
    pub fn new(parser: P, oracle: O, zkp: Z, evm: E, chain_id: u64) -> Self {
        Self {
            parser,
            oracle,
            zkp,
            evm,
            chain_id,
        }
    }

    /// Execute an intent described in natural language.
    ///
    /// # Arguments
    /// - `input` — natural language intent, e.g. "swap 1 ETH for USDC on Uniswap if ETH > 2000".
    /// - `delegation` — user's delegation limits.
    /// - `signature` — secp256k1 signature over the delegation hash.
    /// - `timestamp` — current timestamp in seconds.
    ///
    /// Returns the on-chain transaction identifier when the full pipeline succeeds.
    pub fn execute(
        &self,
        input: &str,
        delegation: &DelegationMessage,
        signature: &[u8; 64],
        timestamp: u64,
    ) -> Result<String, ExecutionError> {
        // 1. Parse the natural-language intent.
        let conditional_intent = self.parser.parse(input)?;
        let intent = conditional_intent.intent;

        // 2. Evaluate the condition if one is present.
        if let Some(condition) = &conditional_intent.condition {
            let evaluator = EvaluateConditionUseCase::new(&self.oracle);
            let asset = primary_asset(&intent);
            if !evaluator.execute(condition, Some(&asset))? {
                return Err(ExecutionError::ConditionNotMet);
            }
        }

        // 3. Validate the intent structure.
        intent.validate()?;

        // 4. Map the domain intent to a circuit-friendly proposed intent.
        let proposed_intent = intent_to_proposed(&intent, self.chain_id)?;

        // 5. Build public and private inputs for the ZKP.
        let public_inputs = PublicDelegationInputs {
            delegation_hash: domain::models::delegation::hash_delegation(delegation),
            proposed_intent,
            timestamp: field_from_u64(timestamp),
            nonce: delegation.nonce,
        };
        let private_inputs = PrivateDelegationInputs {
            delegation: delegation.clone(),
            signature: *signature,
        };

        // 6. Generate the delegation proof.
        let proof = self.zkp.prove_delegation(&public_inputs, &private_inputs)?;

        // 7. Submit the proof on-chain.
        let tx_hash = self.evm.execute_with_proof(&proof, &public_inputs)?;
        Ok(tx_hash)
    }

    /// Generate a proof without submitting it. Useful for testing or off-chain
    /// verification.
    pub fn prove(
        &self,
        input: &str,
        delegation: &DelegationMessage,
        signature: &[u8; 64],
        timestamp: u64,
    ) -> Result<(DelegationProof, PublicDelegationInputs), ExecutionError> {
        let conditional_intent = self.parser.parse(input)?;
        conditional_intent.intent.validate()?;
        let proposed_intent = intent_to_proposed(&conditional_intent.intent, self.chain_id)?;

        let public_inputs = PublicDelegationInputs {
            delegation_hash: domain::models::delegation::hash_delegation(delegation),
            proposed_intent,
            timestamp: field_from_u64(timestamp),
            nonce: delegation.nonce,
        };
        let private_inputs = PrivateDelegationInputs {
            delegation: delegation.clone(),
            signature: *signature,
        };

        let proof = self.zkp.prove_delegation(&public_inputs, &private_inputs)?;
        Ok((proof, public_inputs))
    }
}

fn primary_asset(intent: &Intent) -> Asset {
    match intent {
        Intent::Lend { asset, .. } | Intent::Stake { asset, .. } | Intent::Borrow { asset, .. } => {
            asset.clone()
        }
        Intent::Swap { from_asset, .. } => from_asset.clone(),
        Intent::Composite { intents } => intents.first().map(primary_asset).unwrap_or(Asset::Eth),
    }
}

fn intent_to_proposed(
    intent: &Intent,
    chain_id: u64,
) -> Result<ProposedDelegationIntent, ExecutionError> {
    let asset = primary_asset(intent);
    let target_contract = asset_to_target_contract(&asset, chain_id);

    let (intent_type, amount, protocol) = match intent {
        Intent::Swap {
            amount, protocol, ..
        } => (0u32, *amount, dex_protocol_id(protocol)),
        Intent::Stake {
            amount, protocol, ..
        } => (1u32, *amount, lending_protocol_id(protocol)),
        Intent::Borrow {
            amount, protocol, ..
        } => (2u32, *amount, lending_protocol_id(protocol)),
        Intent::Lend {
            amount, protocol, ..
        } => (3u32, *amount, lending_protocol_id(protocol)),
        Intent::Composite { .. } => {
            return Err(ExecutionError::UnsupportedIntentType(
                "composite intents are not supported in the ZKP flow".to_string(),
            ));
        }
    };

    Ok(ProposedDelegationIntent {
        intent_type: field_from_u32(intent_type),
        amount: field_from_u128(amount),
        protocol,
        target_contract,
    })
}

fn asset_to_target_contract(asset: &Asset, chain_id: u64) -> FieldBytes {
    match asset.token_address(chain_id) {
        Some(addr) => {
            let mut bytes = [0u8; 32];
            bytes[12..32].copy_from_slice(&addr);
            bytes
        }
        None => field_from_u32(0),
    }
}

fn dex_protocol_id(protocol: &DexType) -> domain::models::delegation::FieldBytes {
    match protocol {
        DexType::Uniswap => field_from_u32(1),
        DexType::Sushiswap => field_from_u32(2),
        DexType::Balancer => field_from_u32(3),
    }
}

fn lending_protocol_id(protocol: &LendingType) -> domain::models::delegation::FieldBytes {
    match protocol {
        LendingType::Aave => field_from_u32(4),
        LendingType::Compound => field_from_u32(5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::delegation::{DelegationMessage, field_from_u32};
    use infrastructure::blockchain::{MockEvmAdapter, MockOracleAdapter};
    use infrastructure::parsers::RegexParser;
    use infrastructure::zkp::MockZkpAdapter;

    fn sample_delegation() -> DelegationMessage {
        DelegationMessage {
            pubkey_x: [0u8; 32],
            pubkey_y: [0u8; 32],
            allowed_intents: field_from_u32(0x0f), // allow types 0-3
            max_amounts: [field_from_u128(10_000); 10],
            allowed_protocols: [
                field_from_u32(1),
                field_from_u32(2),
                field_from_u32(4),
                field_from_u32(0),
                field_from_u32(0),
            ],
            expiry: field_from_u64(4_000_000_000),
            nonce: field_from_u64(42),
            target_contract: field_from_u32(0),
        }
    }

    #[test]
    fn execute_parses_validates_proves_and_submits() {
        let parser = RegexParser::new();
        let oracle =
            MockOracleAdapter::with_default(domain::models::condition::Metric::Price, 3000_000000);
        let zkp = MockZkpAdapter::new();
        let evm = MockEvmAdapter::new();

        let use_case = ExecuteIntentUseCase::new(parser, oracle, zkp, evm, 11155111);
        let tx = use_case
            .execute(
                "swap 1000 USDC for ETH on Uniswap",
                &sample_delegation(),
                &[0u8; 64],
                1_000_000,
            )
            .unwrap();

        assert!(!tx.is_empty());
    }

    #[test]
    fn execute_rejects_unparseable_input() {
        let parser = RegexParser::new();
        let oracle = MockOracleAdapter::new();
        let zkp = MockZkpAdapter::new();
        let evm = MockEvmAdapter::new();

        let use_case = ExecuteIntentUseCase::new(parser, oracle, zkp, evm, 11155111);
        let result = use_case.execute(
            "do something weird with all my money",
            &sample_delegation(),
            &[0u8; 64],
            1_000_000,
        );

        assert!(matches!(result, Err(ExecutionError::ParsingFailed(_))));
    }

    #[test]
    fn intent_to_proposed_rejects_composite() {
        let composite = Intent::Composite {
            intents: vec![Intent::Swap {
                from_asset: domain::models::intent::Asset::Usdc,
                to_asset: domain::models::intent::Asset::Eth,
                amount: 1000,
                protocol: DexType::Uniswap,
            }],
        };
        assert!(matches!(
            intent_to_proposed(&composite, 11155111),
            Err(ExecutionError::UnsupportedIntentType(_))
        ));
    }
}
