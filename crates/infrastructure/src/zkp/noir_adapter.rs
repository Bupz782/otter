use crate::config::Config;
use domain::models::delegation::{
    DelegationProof, FieldBytes, PrivateDelegationInputs, ProposedDelegationIntent,
    PublicDelegationInputs, field_to_hex, serialize_public_inputs,
};
use domain::ports::zkp_port::{ZkpError, ZkpPort};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Cross-process lock that serializes `nargo execute` calls on a given circuit.
/// Noir locks the package manifest while generating witnesses; running multiple
/// proofs concurrently on the same package corrupts intermediate files. A
/// directory is used because directory creation is atomic on both Unix and
/// Windows, so this works across separate test binaries and API replicas that
/// share a circuit directory.
struct NargoLockGuard(PathBuf);

impl NargoLockGuard {
    fn acquire(circuit_dir: &Path) -> Result<Self, ZkpError> {
        let lock_dir = circuit_dir.join(".nargo_execute_lock");
        loop {
            match fs::create_dir(&lock_dir) {
                Ok(()) => return Ok(Self(lock_dir)),
                Err(_) => thread::sleep(Duration::from_millis(50)),
            }
        }
    }
}

impl Drop for NargoLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

/// Proof bytes and public inputs produced by the Barretenberg backend.
type BbProof = (Vec<u8>, Vec<u8>);

/// Adapter that drives the Noir `delegation_circuit` using `nargo execute` and,
/// when available, the Barretenberg `bb` backend.
///
/// # Workflow
/// 1. Serialize public + private inputs into a Noir `Prover.toml`.
/// 2. Run `nargo execute <witness> -p <prover>` to generate a witness.
///    This step enforces all circuit constraints and fails fast if the
///    delegation is invalid.
/// 3. If a `bb` binary is configured and works, run `bb prove` and capture
///    the proof bytes.
/// 4. Fall back to a witness-validated placeholder proof when `bb` is
///    unavailable or incompatible with the installed Noir version.
#[derive(Debug)]
pub struct NoirAdapter {
    /// Path to the Noir package directory containing `Nargo.toml`.
    circuit_dir: PathBuf,
    /// Name of the nargo binary.
    nargo_bin: String,
    /// Optional path to the `bb` binary.
    bb_bin: Option<String>,
    /// Package name as declared in `Nargo.toml`.
    package_name: String,
    /// Last witness generation duration in milliseconds.
    last_witness_ms: AtomicU64,
    /// Last bb prove duration in milliseconds.
    last_prove_ms: AtomicU64,
    /// Last verification duration in milliseconds.
    last_verify_ms: AtomicU64,
}

impl Clone for NoirAdapter {
    fn clone(&self) -> Self {
        Self {
            circuit_dir: self.circuit_dir.clone(),
            nargo_bin: self.nargo_bin.clone(),
            bb_bin: self.bb_bin.clone(),
            package_name: self.package_name.clone(),
            last_witness_ms: AtomicU64::new(self.last_witness_ms.load(Ordering::Relaxed)),
            last_prove_ms: AtomicU64::new(self.last_prove_ms.load(Ordering::Relaxed)),
            last_verify_ms: AtomicU64::new(self.last_verify_ms.load(Ordering::Relaxed)),
        }
    }
}

impl NoirAdapter {
    /// Create an adapter from application configuration.
    ///
    /// Recognized environment variables:
    /// - `OTTER_CIRCUIT_DIR`: path to `delegation_circuit` (default: `./delegation_circuit`)
    /// - `OTTER_NARGO_BIN`: nargo binary name/path (default: `nargo`)
    /// - `OTTER_BB_BIN`: bb binary name/path (optional)
    pub fn from_config(_config: &Config) -> Self {
        let circuit_dir = std::env::var("OTTER_CIRCUIT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("delegation_circuit"));
        let nargo_bin = std::env::var("OTTER_NARGO_BIN").unwrap_or_else(|_| "nargo".to_string());
        let bb_bin = std::env::var("OTTER_BB_BIN").ok();

        Self::new(circuit_dir, nargo_bin, bb_bin)
    }

    /// Create an adapter with explicit paths.
    pub fn new(
        circuit_dir: impl Into<PathBuf>,
        nargo_bin: impl Into<String>,
        bb_bin: Option<impl Into<String>>,
    ) -> Self {
        Self {
            circuit_dir: circuit_dir.into(),
            nargo_bin: nargo_bin.into(),
            bb_bin: bb_bin.map(Into::into),
            package_name: "delegation_circuit".to_string(),
            last_witness_ms: AtomicU64::new(0),
            last_prove_ms: AtomicU64::new(0),
            last_verify_ms: AtomicU64::new(0),
        }
    }

    fn write_prover_toml(
        &self,
        prover_name: &str,
        public_inputs: &PublicDelegationInputs,
        private_inputs: &PrivateDelegationInputs,
    ) -> Result<PathBuf, ZkpError> {
        let path = self.circuit_dir.join(format!("{}.toml", prover_name));
        let content = format_prover_toml(public_inputs, private_inputs);
        fs::write(&path, content)?;
        debug!(?path, "wrote Noir prover inputs");
        Ok(path)
    }

    fn run_nargo_execute(
        &self,
        witness_name: &str,
        prover_name: &str,
    ) -> Result<std::process::Output, ZkpError> {
        let output = Command::new(&self.nargo_bin)
            .arg("execute")
            .arg(witness_name)
            .arg("--package")
            .arg(&self.package_name)
            .arg("-p")
            .arg(prover_name)
            .current_dir(&self.circuit_dir)
            .output()?;

        debug!(
            status = ?output.status,
            stdout = %String::from_utf8_lossy(&output.stdout),
            stderr = %String::from_utf8_lossy(&output.stderr),
            "nargo execute finished"
        );
        Ok(output)
    }

    fn try_bb_prove(
        &self,
        witness_name: &str,
        proof_dir_name: &str,
    ) -> Result<Option<BbProof>, ZkpError> {
        let Some(bb_bin) = &self.bb_bin else {
            return Ok(None);
        };

        // All commands run with `current_dir` set to `self.circuit_dir`, so use
        // relative paths that resolve correctly regardless of whether the user
        // passed an absolute or relative circuit directory.
        let circuit_path = PathBuf::from(format!("target/{}.json", self.package_name));
        let witness_path = PathBuf::from(format!("target/{}.gz", witness_name));
        let proof_dir_absolute = self.circuit_dir.join(proof_dir_name);

        fs::create_dir_all(&proof_dir_absolute)?;

        let output = Command::new(bb_bin)
            .arg("prove")
            .arg("--scheme")
            .arg("ultra_honk")
            .arg("-t")
            .arg("evm-no-zk")
            .arg("--write_vk")
            .arg("-b")
            .arg(&circuit_path)
            .arg("-w")
            .arg(&witness_path)
            .arg("-o")
            .arg(proof_dir_name)
            .current_dir(&self.circuit_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            eprintln!("[NoirAdapter] bb prove stdout: {}", stdout);
            eprintln!("[NoirAdapter] bb prove stderr: {}", stderr);
            warn!(%stderr, "bb prove failed; falling back to witness-only proof");
            return Ok(None);
        }

        let proof_file = proof_dir_absolute.join("proof");
        let public_inputs_file = proof_dir_absolute.join("public_inputs");
        let proof = fs::read(&proof_file).map_err(|e| {
            warn!(%e, "bb prove succeeded but proof file could not be read");
            e
        })?;
        let public_inputs = fs::read(&public_inputs_file).map_err(|e| {
            warn!(%e, "bb prove succeeded but public inputs file could not be read");
            e
        })?;

        Ok(Some((proof, public_inputs)))
    }

    fn ensure_vk(&self) -> Result<PathBuf, ZkpError> {
        let bb_bin = self
            .bb_bin
            .as_ref()
            .ok_or_else(|| ZkpError::BackendUnavailable("bb backend not configured".to_string()))?;
        let vk_dir = self
            .circuit_dir
            .join(format!("target/{}_evm_no_zk_vk", self.package_name));
        let vk_file = vk_dir.join("vk");
        if vk_file.exists() {
            return Ok(vk_file);
        }

        fs::create_dir_all(&vk_dir)?;
        let circuit_path = PathBuf::from(format!("target/{}.json", self.package_name));
        let output = Command::new(bb_bin)
            .arg("write_vk")
            .arg("--scheme")
            .arg("ultra_honk")
            .arg("-t")
            .arg("evm-no-zk")
            .arg("-b")
            .arg(&circuit_path)
            .arg("-o")
            .arg(&vk_dir)
            .current_dir(&self.circuit_dir)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ZkpError::ProofGenerationFailed(format!(
                "bb write_vk failed: {}",
                stderr
            )));
        }

        Ok(vk_file)
    }
}

impl ZkpPort for NoirAdapter {
    fn prove_delegation(
        &self,
        public_inputs: &PublicDelegationInputs,
        private_inputs: &PrivateDelegationInputs,
    ) -> Result<DelegationProof, ZkpError> {
        let prover_name = format!("otter_prover_{}", unique_id());
        let witness_name = format!("otter_witness_{}", unique_id());
        let proof_dir_name = format!("otter_proof_{}", unique_id());
        // bb writes into a directory relative to `circuit_dir` (its cwd), but we
        // read back using the Rust process cwd, so keep an absolute path.
        let proof_path = self.circuit_dir.join(&proof_dir_name);

        let _prover_path = self.write_prover_toml(&prover_name, public_inputs, private_inputs)?;

        let nargo_start = Instant::now();
        let nargo_output = {
            let _guard = NargoLockGuard::acquire(&self.circuit_dir)?;
            self.run_nargo_execute(&witness_name, &prover_name)?
        };
        let nargo_elapsed = nargo_start.elapsed();
        if !nargo_output.status.success() {
            let stderr = String::from_utf8_lossy(&nargo_output.stderr);
            return Err(ZkpError::WitnessGenerationFailed(stderr.to_string()));
        }

        let witness_ms = nargo_elapsed.as_millis() as u64;
        self.last_witness_ms.store(witness_ms, Ordering::Relaxed);
        info!(
            elapsed_ms = witness_ms,
            "Noir witness generation succeeded; attempting bb prove"
        );

        let bb_start = Instant::now();
        let (proof, public_inputs_bytes) = match self.try_bb_prove(&witness_name, &proof_dir_name) {
            Ok(Some((proof, bb_public_inputs))) => (proof, bb_public_inputs),
            Ok(None) => {
                eprintln!("[NoirAdapter] bb prove unavailable; falling back to witness-only proof");
                let fallback = serialize_public_inputs(public_inputs);
                (Vec::new(), fallback)
            }
            Err(err) => {
                eprintln!(
                    "[NoirAdapter] bb prove failed: {}; falling back to witness-only proof",
                    err
                );
                let fallback = serialize_public_inputs(public_inputs);
                (Vec::new(), fallback)
            }
        };
        let prove_ms = bb_start.elapsed().as_millis() as u64;
        self.last_prove_ms.store(prove_ms, Ordering::Relaxed);
        info!(
            elapsed_ms = prove_ms,
            has_proof = !proof.is_empty(),
            "bb prove step finished"
        );

        // Best-effort cleanup of temporary files.
        let proof_dir_absolute = self.circuit_dir.join(&proof_path);
        let _ = fs::remove_file(self.circuit_dir.join(format!("{}.toml", prover_name)));
        let _ = fs::remove_file(self.circuit_dir.join(format!("target/{}.gz", witness_name)));
        let _ = fs::remove_file(proof_dir_absolute.join("proof"));
        let _ = fs::remove_file(proof_dir_absolute.join("public_inputs"));
        let _ = fs::remove_file(proof_dir_absolute.join("vk"));
        let _ = fs::remove_dir(&proof_dir_absolute);

        Ok(DelegationProof {
            proof,
            public_inputs: public_inputs_bytes,
        })
    }

    fn verify_delegation(
        &self,
        proof: &DelegationProof,
        public_inputs: &PublicDelegationInputs,
    ) -> Result<bool, ZkpError> {
        let expected_public_inputs = serialize_public_inputs(public_inputs);
        if proof.public_inputs != expected_public_inputs {
            return Ok(false);
        }

        if proof.proof.is_empty() {
            warn!("NoirAdapter::verify_delegation: no proof bytes available");
            return Err(ZkpError::BackendUnavailable(
                "bb backend required for verification".to_string(),
            ));
        }

        let bb_bin = match &self.bb_bin {
            Some(bin) => bin,
            None => {
                warn!("NoirAdapter::verify_delegation: bb backend not configured");
                return Err(ZkpError::BackendUnavailable(
                    "bb backend required for verification".to_string(),
                ));
            }
        };

        let vk_path = self.ensure_vk()?;
        let id = unique_id();
        let proof_path = self.circuit_dir.join(format!("otter_verify_proof_{}", id));
        let public_inputs_path = self
            .circuit_dir
            .join(format!("otter_verify_public_inputs_{}", id));

        fs::write(&proof_path, &proof.proof)?;
        fs::write(&public_inputs_path, &proof.public_inputs)?;

        let verify_start = Instant::now();
        let output = Command::new(bb_bin)
            .arg("verify")
            .arg("--scheme")
            .arg("ultra_honk")
            .arg("-t")
            .arg("evm-no-zk")
            .arg("-p")
            .arg(&proof_path)
            .arg("-i")
            .arg(&public_inputs_path)
            .arg("-k")
            .arg(&vk_path)
            .current_dir(&self.circuit_dir)
            .output()?;

        let _ = fs::remove_file(&proof_path);
        let _ = fs::remove_file(&public_inputs_path);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!(%stderr, "bb verify returned false");
            return Ok(false);
        }

        let verify_ms = verify_start.elapsed().as_millis() as u64;
        self.last_verify_ms.store(verify_ms, Ordering::Relaxed);
        info!(elapsed_ms = verify_ms, "Noir proof verification succeeded");
        Ok(true)
    }
}

impl NoirAdapter {
    /// Return the last witness generation duration in milliseconds.
    pub fn last_witness_ms(&self) -> u64 {
        self.last_witness_ms.load(Ordering::Relaxed)
    }

    /// Return the last bb prove duration in milliseconds.
    pub fn last_prove_ms(&self) -> u64 {
        self.last_prove_ms.load(Ordering::Relaxed)
    }

    /// Return the last verification duration in milliseconds.
    pub fn last_verify_ms(&self) -> u64 {
        self.last_verify_ms.load(Ordering::Relaxed)
    }
}

fn unique_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", now)
}

fn format_prover_toml(
    public_inputs: &PublicDelegationInputs,
    private_inputs: &PrivateDelegationInputs,
) -> String {
    let mut s = String::new();

    s.push_str(&format!(
        "delegation_hash = {}\n",
        format_byte_array(&public_inputs.delegation_hash)
    ));
    s.push_str(&format!(
        "proposed_intent = {}\n",
        format_proposed_intent(&public_inputs.proposed_intent)
    ));
    s.push_str(&format!(
        "timestamp = {}\n",
        field_to_hex(&public_inputs.timestamp)
    ));
    s.push_str(&format!("nonce = {}\n", field_to_hex(&public_inputs.nonce)));
    s.push_str(&format!(
        "signature = {}\n",
        format_byte_array(&private_inputs.signature)
    ));
    s.push('\n');
    s.push_str("[delegation]\n");
    s.push_str(&format!(
        "pubkey_x = {}\n",
        format_byte_array(&private_inputs.delegation.pubkey_x)
    ));
    s.push_str(&format!(
        "pubkey_y = {}\n",
        format_byte_array(&private_inputs.delegation.pubkey_y)
    ));
    s.push_str(&format!(
        "allowed_intents = {}\n",
        field_to_hex(&private_inputs.delegation.allowed_intents)
    ));
    s.push_str(&format!(
        "max_amounts = {}\n",
        format_field_array(&private_inputs.delegation.max_amounts)
    ));
    s.push_str(&format!(
        "allowed_protocols = {}\n",
        format_field_array(&private_inputs.delegation.allowed_protocols)
    ));
    s.push_str(&format!(
        "expiry = {}\n",
        field_to_hex(&private_inputs.delegation.expiry)
    ));
    s.push_str(&format!(
        "nonce = {}\n",
        field_to_hex(&private_inputs.delegation.nonce)
    ));
    s.push_str(&format!(
        "target_contract = {}\n",
        field_to_hex(&private_inputs.delegation.target_contract)
    ));

    s
}

fn format_byte_array(bytes: &[u8]) -> String {
    let items: Vec<String> = bytes.iter().map(|b| format!("\"0x{:02x}\"", b)).collect();
    format!("[{}]", items.join(", "))
}

fn format_field_array<const N: usize>(fields: &[FieldBytes; N]) -> String {
    let items: Vec<String> = fields.iter().map(field_to_hex).collect();
    format!("[{}]", items.join(", "))
}

fn format_proposed_intent(intent: &ProposedDelegationIntent) -> String {
    format!(
        "{{ intent_type = {}, amount = {}, protocol = {}, target_contract = {} }}",
        field_to_hex(&intent.intent_type),
        field_to_hex(&intent.amount),
        field_to_hex(&intent.protocol),
        field_to_hex(&intent.target_contract)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::models::delegation::{
        DelegationMessage, field_from_u32, field_from_u64, field_from_u128,
    };

    #[test]
    fn prover_toml_format_is_valid() {
        let public_inputs = PublicDelegationInputs {
            delegation_hash: [0u8; 32],
            proposed_intent: ProposedDelegationIntent {
                intent_type: field_from_u32(1),
                amount: field_from_u128(1000),
                protocol: field_from_u32(1),
                target_contract: field_from_u32(0),
            },
            timestamp: field_from_u64(1234567890),
            nonce: field_from_u64(42),
        };
        let private_inputs = PrivateDelegationInputs {
            delegation: DelegationMessage {
                pubkey_x: [0u8; 32],
                pubkey_y: [1u8; 32],
                allowed_intents: field_from_u32(0x05),
                max_amounts: [field_from_u128(1000); 10],
                allowed_protocols: [field_from_u32(1); 5],
                expiry: field_from_u64(9999999999),
                nonce: field_from_u64(42),
                target_contract: field_from_u32(0),
            },
            signature: [0u8; 64],
        };

        let toml = format_prover_toml(&public_inputs, &private_inputs);
        assert!(toml.contains("delegation_hash"));
        assert!(toml.contains("[delegation]"));
        assert!(toml.contains("signature"));
    }
}
