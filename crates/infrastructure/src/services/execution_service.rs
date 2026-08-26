use application::ports::{ExecutionPort, ExecutionResult};
use application::use_cases::execute_intent::{ExecuteIntentUseCase, ExecutionError};
use domain::models::delegation::{
    DelegationMessage, FieldBytes, field_from_u32, field_from_u64, field_from_u128, hash_delegation,
};
use domain::ports::evm_port::EvmPort;
use domain::ports::intent_parser_port::IntentParserPort;
use domain::ports::mev_port::MevPort;
use domain::ports::wallet_port::WalletPort;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;

/// A user-signed delegation together with its secp256k1 signature.
type UserDelegation = Option<(DelegationMessage, [u8; 64])>;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::blockchain::AlloyEvmAdapter;
use crate::blockchain::LocalWalletAdapter;
use crate::blockchain::composite_oracle::CompositeOracle;
use crate::parsers::RegexParser;
use crate::zkp::NoirAdapter;

/// End-to-end execution service that re-creates the on-chain delegation with a
/// fresh nonce for every intent.
///
/// This guarantees that multiple intents can be executed against the same
/// `DelegationVault` without replaying a consumed delegation. The service also
/// re-registers the delegation via `AlloyEvmAdapter::ensure_delegated` before
/// submitting the proof.
#[derive(Clone)]
pub struct OnChainExecutionService {
    parser: RegexParser,
    oracle: CompositeOracle,
    zkp: NoirAdapter,
    evm: AlloyEvmAdapter,
    wallet: LocalWalletAdapter,
    nonce: FileNonceStore,
    template: DelegationMessage,
    chain_id: u64,
    /// Delegation hashes already registered on-chain. The mutex also serializes
    /// delegate transactions so concurrent executions do not submit with the
    /// same EOA nonce.
    delegated_hashes: Arc<Mutex<HashSet<[u8; 32]>>>,
    /// Optional user-signed delegation. When present, executions use this
    /// delegation and signature instead of generating a fresh agent-signed one.
    user_delegation: Arc<Mutex<UserDelegation>>,
    /// Optional simulated MEV capture port, invoked after each successful
    /// execution so captured profit can be rebated to the vault owner.
    mev: Option<Arc<dyn MevPort>>,
}

impl OnChainExecutionService {
    /// Build a service from concrete adapters and a 32-byte agent private key.
    ///
    /// `starting_nonce` is used as the first delegation nonce; pass the signer's
    /// current on-chain transaction count + 1 to avoid reusing a consumed nonce.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parser: RegexParser,
        oracle: CompositeOracle,
        zkp: NoirAdapter,
        evm: AlloyEvmAdapter,
        private_key: &[u8; 32],
        starting_nonce: u64,
        nonce_store_path: &str,
        chain_id: u64,
    ) -> Result<Self, ExecutionError> {
        let wallet = LocalWalletAdapter::from_bytes(private_key)
            .map_err(|e| ExecutionError::InvalidIntent(format!("wallet: {e}")))?;
        let (pubkey_x, pubkey_y) = wallet
            .pubkey()
            .map_err(|e| ExecutionError::InvalidIntent(format!("failed to derive pubkey: {e}")))?;

        let template = DelegationMessage {
            pubkey_x,
            pubkey_y,
            allowed_intents: field_from_u32(0x0f), // allow all four intent types
            max_amounts: [field_from_u128(u128::MAX); 10],
            allowed_protocols: [
                field_from_u32(1), // Uniswap
                field_from_u32(2), // Sushiswap
                field_from_u32(4), // Aave
                field_from_u32(5), // Compound
                field_from_u32(0),
            ],
            expiry: field_from_u64(u64::MAX),
            nonce: field_from_u64(0),
            target_contract: field_from_u32(0),
        };

        Ok(Self {
            parser,
            oracle,
            zkp,
            evm,
            wallet,
            nonce: FileNonceStore::new(nonce_store_path, starting_nonce),
            template,
            chain_id,
            delegated_hashes: Arc::new(Mutex::new(HashSet::new())),
            user_delegation: Arc::new(Mutex::new(None)),
            mev: None,
        })
    }

    /// Attach a simulated MEV capture port (see `infrastructure::mev`).
    pub fn with_mev(mut self, mev: Arc<dyn MevPort>) -> Self {
        self.mev = Some(mev);
        self
    }
}

impl OnChainExecutionService {
    /// Store a user-signed delegation for subsequent executions.
    pub fn set_user_delegation(&self, delegation: DelegationMessage, signature: [u8; 64]) {
        let mut guard = self
            .user_delegation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *guard = Some((delegation, signature));
    }

    fn prepare_delegation(
        &self,
        target_contract: FieldBytes,
    ) -> Result<(DelegationMessage, [u8; 64]), ExecutionError> {
        let guard = self
            .user_delegation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some((delegation, signature)) = guard.as_ref() {
            return Ok((delegation.clone(), *signature));
        }
        drop(guard);

        let nonce = self.nonce.next();
        let mut delegation = self.template.clone();
        delegation.nonce = field_from_u64(nonce);
        delegation.target_contract = target_contract;

        let hash = hash_delegation(&delegation);
        let signature = self
            .wallet
            .sign_hash(&hash)
            .map_err(|e| ExecutionError::ProofFailed(format!("delegation sign failed: {e}")))?;
        Ok((delegation, signature))
    }

    fn target_contract_for_input(&self, input: &str) -> Result<FieldBytes, ExecutionError> {
        use domain::models::intent::{Asset, Intent};

        let conditional = self
            .parser
            .parse(input)
            .map_err(|e| ExecutionError::InvalidIntent(format!("parse failed: {e}")))?;
        let asset = match &conditional.intent {
            Intent::Lend { asset, .. }
            | Intent::Stake { asset, .. }
            | Intent::Borrow { asset, .. } => asset.clone(),
            Intent::Swap { from_asset, .. } => from_asset.clone(),
            Intent::Composite { intents } => intents
                .first()
                .map(|i| match i {
                    Intent::Lend { asset, .. }
                    | Intent::Stake { asset, .. }
                    | Intent::Borrow { asset, .. } => asset.clone(),
                    Intent::Swap { from_asset, .. } => from_asset.clone(),
                    Intent::Composite { .. } => Asset::Eth,
                })
                .unwrap_or(Asset::Eth),
        };

        match asset.token_address(self.chain_id) {
            Some(addr) => {
                let mut bytes = [0u8; 32];
                bytes[12..32].copy_from_slice(&addr);
                Ok(bytes)
            }
            None => Ok(field_from_u32(0)),
        }
    }
}

impl ExecutionPort for OnChainExecutionService {
    fn execute(&self, input: &str) -> Result<String, ExecutionError> {
        let target_contract = self.target_contract_for_input(input)?;
        let (delegation, signature) = self.prepare_delegation(target_contract)?;
        let hash = hash_delegation(&delegation);

        {
            let mut delegated = self.delegated_hashes.lock().map_err(|e| {
                ExecutionError::SubmissionFailed(format!("delegated_hashes lock poisoned: {e}"))
            })?;
            if !delegated.contains(&hash) {
                self.evm.ensure_delegated(&delegation).map_err(|e| {
                    ExecutionError::SubmissionFailed(format!("ensure_delegated: {e}"))
                })?;
                delegated.insert(hash);
            }
        }

        let mut use_case = ExecuteIntentUseCase::new(
            self.parser.clone(),
            self.oracle.clone(),
            self.zkp.clone(),
            self.evm.clone(),
            self.chain_id,
        );
        if let Some(mev) = &self.mev {
            use_case = use_case.with_mev(Arc::clone(mev));
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        use_case.execute(input, &delegation, &signature, timestamp)
    }

    fn set_delegation(&self, delegation: DelegationMessage, signature: [u8; 64]) {
        self.set_user_delegation(delegation, signature);
    }

    fn confirm(&self, tx_hash: &str) -> Result<ExecutionResult, ExecutionError> {
        self.evm
            .confirm_transaction(tx_hash)
            .map_err(|e| ExecutionError::SubmissionFailed(format!("confirmation failed: {e}")))?;

        let receipt = self
            .evm
            .get_transaction_receipt(tx_hash)
            .map_err(|e| ExecutionError::SubmissionFailed(format!("receipt fetch failed: {e}")))?;

        Ok(match receipt {
            Some(r) => ExecutionResult {
                success: r.status,
                gas_used: r.gas_used,
            },
            None => ExecutionResult {
                success: false,
                gas_used: 0,
            },
        })
    }
}

/// File-backed nonce store that persists the next usable nonce across
/// process restarts. On creation it picks the maximum between the on-chain
/// starting nonce and the value stored on disk, so restarts never reuse a
/// consumed delegation nonce.
#[derive(Clone)]
struct FileNonceStore {
    inner: Arc<Mutex<u64>>,
    path: Arc<PathBuf>,
}

impl FileNonceStore {
    fn new(path: impl AsRef<Path>, starting_nonce: u64) -> Self {
        let path = path.as_ref().to_path_buf();
        let file_nonce = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let initial = file_nonce.max(starting_nonce);
        Self {
            inner: Arc::new(Mutex::new(initial)),
            path: Arc::new(path),
        }
    }

    fn next(&self) -> u64 {
        let mut guard = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let value = *guard;
        *guard = value + 1;

        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let tmp = self.path.with_extension("tmp");
        let _ = std::fs::write(&tmp, guard.to_string());
        let _ = std::fs::rename(&tmp, &*self.path);
        value
    }
}
