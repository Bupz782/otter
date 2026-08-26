use domain::models::delegation::{field_from_u64, field_from_u128, field_to_hex};
use domain::ports::solvency_port::{
    PrivateSolvencyInputs, PublicSolvencyInputs, SolvencyError, SolvencyPort, SolvencyProof,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Number of leaves expected by the `delegation_solvency` circuit.
pub const SOLVENCY_LEAF_COUNT: usize = 16;

/// Proof bytes and public inputs produced by the Barretenberg backend.
type BbProof = (Vec<u8>, Vec<u8>);

/// Cross-process lock serializing `nargo execute` on the solvency circuit
/// directory (same mechanism as `NoirAdapter`).
struct NargoLockGuard(PathBuf);

impl NargoLockGuard {
    fn acquire(circuit_dir: &Path) -> Result<Self, SolvencyError> {
        let lock_dir = circuit_dir.join(".nargo_execute_lock");
        loop {
            match fs::create_dir(&lock_dir) {
                Ok(()) => return Ok(Self(lock_dir)),
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
    }
}

impl Drop for NargoLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

/// Adapter that drives the Noir `delegation_solvency` Merkle-sum
/// proof-of-solvency circuit using `nargo execute` and, when available, the
/// Barretenberg `bb` backend.
///
/// The workflow mirrors [`crate::zkp::NoirAdapter`]: serialize inputs into a
/// `Prover.toml`, run `nargo execute` (which enforces every circuit
/// constraint), then attempt a real UltraHonk proof with `bb`. When `bb` is
/// unavailable the adapter degrades to a witness-validated placeholder proof.
#[derive(Debug)]
pub struct SolvencyAdapter {
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
}

impl Clone for SolvencyAdapter {
    fn clone(&self) -> Self {
        Self {
            circuit_dir: self.circuit_dir.clone(),
            nargo_bin: self.nargo_bin.clone(),
            bb_bin: self.bb_bin.clone(),
            package_name: self.package_name.clone(),
            last_witness_ms: AtomicU64::new(self.last_witness_ms.load(Ordering::Relaxed)),
            last_prove_ms: AtomicU64::new(self.last_prove_ms.load(Ordering::Relaxed)),
        }
    }
}

impl SolvencyAdapter {
    /// Create an adapter from application configuration.
    ///
    /// Recognized environment variables:
    /// - `OTTER_SOLVENCY_CIRCUIT_DIR`: path to `delegation_solvency`
    ///   (default: `./delegation_solvency`)
    /// - `OTTER_NARGO_BIN`: nargo binary name/path (default: `nargo`)
    /// - `OTTER_BB_BIN`: bb binary name/path (optional)
    pub fn from_env() -> Self {
        let circuit_dir = std::env::var("OTTER_SOLVENCY_CIRCUIT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("delegation_solvency"));
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
            package_name: "delegation_solvency".to_string(),
            last_witness_ms: AtomicU64::new(0),
            last_prove_ms: AtomicU64::new(0),
        }
    }

    /// Return the last witness generation duration in milliseconds.
    pub fn last_witness_ms(&self) -> u64 {
        self.last_witness_ms.load(Ordering::Relaxed)
    }

    /// Return the last bb prove duration in milliseconds.
    pub fn last_prove_ms(&self) -> u64 {
        self.last_prove_ms.load(Ordering::Relaxed)
    }

    fn write_prover_toml(
        &self,
        prover_name: &str,
        public_inputs: &PublicSolvencyInputs,
        private_inputs: &PrivateSolvencyInputs,
    ) -> Result<PathBuf, SolvencyError> {
        if private_inputs.leaves.len() != SOLVENCY_LEAF_COUNT {
            return Err(SolvencyError::InvalidInput(format!(
                "expected {} leaves, got {}",
                SOLVENCY_LEAF_COUNT,
                private_inputs.leaves.len()
            )));
        }
        let path = self.circuit_dir.join(format!("{}.toml", prover_name));
        fs::write(&path, format_prover_toml(public_inputs, private_inputs))?;
        debug!(?path, "wrote solvency prover inputs");
        Ok(path)
    }

    fn run_nargo_execute(
        &self,
        witness_name: &str,
        prover_name: &str,
    ) -> Result<std::process::Output, SolvencyError> {
        let output = Command::new(&self.nargo_bin)
            .arg("execute")
            .arg(witness_name)
            .arg("--package")
            .arg(&self.package_name)
            .arg("-p")
            .arg(prover_name)
            .current_dir(&self.circuit_dir)
            .output()?;
        Ok(output)
    }

    fn try_bb_prove(
        &self,
        witness_name: &str,
        proof_dir_name: &str,
    ) -> Result<Option<BbProof>, SolvencyError> {
        let Some(bb_bin) = &self.bb_bin else {
            return Ok(None);
        };

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
            warn!(%stderr, "solvency bb prove failed; falling back to witness-only proof");
            return Ok(None);
        }

        let proof = fs::read(proof_dir_absolute.join("proof"))?;
        let public_inputs = fs::read(proof_dir_absolute.join("public_inputs"))?;
        Ok(Some((proof, public_inputs)))
    }

    fn ensure_vk(&self) -> Result<PathBuf, SolvencyError> {
        let bb_bin = self.bb_bin.as_ref().ok_or_else(|| {
            SolvencyError::BackendUnavailable("bb backend not configured".to_string())
        })?;
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
            return Err(SolvencyError::ProofGenerationFailed(format!(
                "bb write_vk failed: {}",
                stderr
            )));
        }

        Ok(vk_file)
    }
}

fn unique_id() -> String {
    use std::sync::atomic::AtomicU64;
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}{}", nanos, count)
}

/// Serialize solvency public inputs to bytes: root (32) + total_deposits
/// (32-byte field) + timestamp (32-byte field) = 96 bytes. This matches the
/// on-chain ordering of the verifier's public-input array.
pub fn serialize_solvency_public_inputs(inputs: &PublicSolvencyInputs) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(96);
    bytes.extend_from_slice(&inputs.merkle_root);
    bytes.extend_from_slice(&field_from_u128(inputs.total_deposits));
    bytes.extend_from_slice(&field_from_u64(inputs.timestamp));
    bytes
}

fn format_byte_array(bytes: &[u8]) -> String {
    let items: Vec<String> = bytes.iter().map(|b| b.to_string()).collect();
    format!("[{}]", items.join(", "))
}

fn format_prover_toml(
    public_inputs: &PublicSolvencyInputs,
    private_inputs: &PrivateSolvencyInputs,
) -> String {
    let mut s = String::new();

    s.push_str(&format!(
        "merkle_root_pub = {}\n",
        format_byte_array(&public_inputs.merkle_root)
    ));
    s.push_str(&format!(
        "total_deposits = {}\n",
        field_to_hex(&field_from_u128(public_inputs.total_deposits))
    ));
    s.push_str(&format!(
        "timestamp = {}\n",
        field_to_hex(&field_from_u64(public_inputs.timestamp))
    ));

    for (i, leaf) in private_inputs.leaves.iter().enumerate() {
        s.push_str(&format!(
            "\n[[leaves]]\ncommitment_hash = {}\nbalance = \"0x{:032x}\"\n",
            format_byte_array(&leaf.commitment_hash),
            leaf.balance
        ));
        let _ = i;
    }

    s
}

impl SolvencyPort for SolvencyAdapter {
    fn prove_solvency(
        &self,
        public_inputs: &PublicSolvencyInputs,
        private_inputs: &PrivateSolvencyInputs,
    ) -> Result<SolvencyProof, SolvencyError> {
        let prover_name = format!("solvency_prover_{}", unique_id());
        let witness_name = format!("solvency_witness_{}", unique_id());
        let proof_dir_name = format!("solvency_proof_{}", unique_id());

        self.write_prover_toml(&prover_name, public_inputs, private_inputs)?;

        let nargo_start = Instant::now();
        let nargo_output = {
            let _guard = NargoLockGuard::acquire(&self.circuit_dir)?;
            self.run_nargo_execute(&witness_name, &prover_name)?
        };
        if !nargo_output.status.success() {
            let stderr = String::from_utf8_lossy(&nargo_output.stderr);
            return Err(SolvencyError::WitnessGenerationFailed(stderr.to_string()));
        }
        self.last_witness_ms
            .store(nargo_start.elapsed().as_millis() as u64, Ordering::Relaxed);
        info!("solvency witness generation succeeded; attempting bb prove");

        let prove_start = Instant::now();
        let (proof, public_inputs_bytes) = match self.try_bb_prove(&witness_name, &proof_dir_name) {
            Ok(Some((proof, pi))) => (proof, pi),
            Ok(None) => {
                eprintln!("[SolvencyAdapter] bb unavailable; falling back to witness-only proof");
                (Vec::new(), serialize_solvency_public_inputs(public_inputs))
            }
            Err(err) => {
                eprintln!(
                    "[SolvencyAdapter] bb prove failed: {}; falling back to witness-only proof",
                    err
                );
                (Vec::new(), serialize_solvency_public_inputs(public_inputs))
            }
        };
        self.last_prove_ms
            .store(prove_start.elapsed().as_millis() as u64, Ordering::Relaxed);

        // Best-effort cleanup.
        let proof_dir_absolute = self.circuit_dir.join(&proof_dir_name);
        let _ = fs::remove_file(self.circuit_dir.join(format!("{}.toml", prover_name)));
        let _ = fs::remove_file(self.circuit_dir.join(format!("target/{}.gz", witness_name)));
        let _ = fs::remove_dir_all(&proof_dir_absolute);

        Ok(SolvencyProof {
            proof,
            public_inputs: public_inputs_bytes,
        })
    }

    fn verify_solvency(
        &self,
        proof: &SolvencyProof,
        public_inputs: &PublicSolvencyInputs,
    ) -> Result<bool, SolvencyError> {
        let expected = serialize_solvency_public_inputs(public_inputs);
        if proof.public_inputs.len() >= expected.len()
            && proof.public_inputs[..expected.len()] != expected[..]
        {
            return Ok(false);
        }
        if proof.public_inputs.is_empty() {
            return Ok(false);
        }

        if proof.proof.is_empty() {
            warn!("SolvencyAdapter::verify_solvency: no proof bytes available");
            return Err(SolvencyError::BackendUnavailable(
                "bb backend required for verification".to_string(),
            ));
        }

        let bb_bin = match &self.bb_bin {
            Some(bin) => bin,
            None => {
                return Err(SolvencyError::BackendUnavailable(
                    "bb backend required for verification".to_string(),
                ));
            }
        };

        let vk_path = self.ensure_vk()?;
        let id = unique_id();
        let proof_path = self
            .circuit_dir
            .join(format!("solvency_verify_proof_{}", id));
        let public_inputs_path = self
            .circuit_dir
            .join(format!("solvency_verify_public_inputs_{}", id));

        fs::write(&proof_path, &proof.proof)?;
        fs::write(&public_inputs_path, &proof.public_inputs)?;

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

        Ok(output.status.success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::ports::solvency_port::SolvencyLeaf;

    fn sample_leaves() -> [SolvencyLeaf; SOLVENCY_LEAF_COUNT] {
        let mut leaves = [SolvencyLeaf {
            commitment_hash: [0u8; 32],
            balance: 0,
        }; SOLVENCY_LEAF_COUNT];
        for (i, leaf) in leaves.iter_mut().enumerate() {
            leaf.commitment_hash = [(i as u8) + 1; 32];
            leaf.balance = 100 * (i as u128 + 1);
        }
        leaves
    }

    #[test]
    fn serialize_solvency_public_inputs_is_96_bytes_and_ordered() {
        let inputs = PublicSolvencyInputs {
            merkle_root: [7u8; 32],
            total_deposits: 1600,
            timestamp: 1_000_000,
        };
        let bytes = serialize_solvency_public_inputs(&inputs);
        assert_eq!(bytes.len(), 96);
        assert_eq!(&bytes[0..32], &[7u8; 32]);
    }

    #[test]
    fn format_prover_toml_contains_all_sections() {
        let public = PublicSolvencyInputs {
            merkle_root: [1u8; 32],
            total_deposits: 1600,
            timestamp: 42,
        };
        let private = PrivateSolvencyInputs {
            leaves: sample_leaves(),
        };
        let toml = format_prover_toml(&public, &private);
        assert!(toml.contains("merkle_root_pub"));
        assert!(toml.contains("total_deposits"));
        assert!(toml.contains("timestamp"));
        assert_eq!(toml.matches("[[leaves]]").count(), SOLVENCY_LEAF_COUNT);
    }
}
