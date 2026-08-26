#![allow(deprecated)]

use async_trait::async_trait;
use borsh::{BorshDeserialize, BorshSerialize};
use domain::ports::solana_port::{AttestationRecord, SolanaError, SolanaPort};
use solana_client::rpc_client::RpcClient;
use solana_program::system_program;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};

const ATTEST_DISCRIMINATOR: [u8; 8] = [0x53, 0x94, 0x78, 0x77, 0x90, 0x8b, 0x75, 0xa0];
const ATTESTATION_SEED: &[u8] = b"attestation";

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct AttestationAccount {
    authority: [u8; 32],
    payload_hash: [u8; 32],
    timestamp: i64,
    bump: u8,
}

/// Solana attestation adapter that talks to an Anchor `attestation_registry`
/// program directly over JSON-RPC.
pub struct SolanaAttestationAdapter {
    rpc: RpcClient,
    program_id: Pubkey,
    authority: Keypair,
}

impl SolanaAttestationAdapter {
    pub fn new(
        rpc_url: &str,
        program_id: &str,
        authority_keypair: &str,
    ) -> Result<Self, SolanaError> {
        let rpc = RpcClient::new(rpc_url.to_string());
        let program_id: Pubkey = program_id
            .parse()
            .map_err(|e| SolanaError::InvalidInput(format!("invalid program id: {}", e)))?;
        let bytes = bs58::decode(authority_keypair)
            .into_vec()
            .map_err(|e| SolanaError::InvalidInput(format!("invalid authority keypair: {}", e)))?;
        let authority = Keypair::try_from(&bytes[..])
            .map_err(|e| SolanaError::InvalidInput(format!("invalid keypair bytes: {}", e)))?;
        Ok(Self {
            rpc,
            program_id,
            authority,
        })
    }

    fn attestation_pda(&self) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[ATTESTATION_SEED, self.authority.pubkey().as_ref()],
            &self.program_id,
        )
    }
}

#[async_trait]
impl SolanaPort for SolanaAttestationAdapter {
    async fn attest(&self, payload_hash: [u8; 32]) -> Result<String, SolanaError> {
        let (pda, _bump) = self.attestation_pda();

        let mut data = ATTEST_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&payload_hash);

        let instruction = Instruction::new_with_bytes(
            self.program_id,
            &data,
            vec![
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(self.authority.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let blockhash = self
            .rpc
            .get_latest_blockhash()
            .map_err(|e| SolanaError::SubmissionFailed(format!("blockhash failed: {}", e)))?;
        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.authority.pubkey()),
            &[&self.authority],
            blockhash,
        );

        let signature = self
            .rpc
            .send_and_confirm_transaction(&tx)
            .map_err(|e| SolanaError::SubmissionFailed(format!("attest failed: {}", e)))?;
        Ok(signature.to_string())
    }

    async fn get_attestation(&self, authority: &str) -> Result<AttestationRecord, SolanaError> {
        let authority: Pubkey = authority
            .parse()
            .map_err(|e| SolanaError::InvalidInput(format!("invalid authority: {}", e)))?;
        let (pda, _bump) =
            Pubkey::find_program_address(&[ATTESTATION_SEED, authority.as_ref()], &self.program_id);
        let account = self
            .rpc
            .get_account(&pda)
            .map_err(|e| SolanaError::SubmissionFailed(format!("fetch account failed: {}", e)))?;
        if account.data.len() < 8 {
            return Err(SolanaError::NotFound);
        }
        let record = AttestationAccount::try_from_slice(&account.data[8..])
            .map_err(|e| SolanaError::SubmissionFailed(format!("deserialize failed: {}", e)))?;
        Ok(AttestationRecord {
            authority: authority.to_string(),
            payload_hash: record.payload_hash,
            timestamp: record.timestamp,
        })
    }

    async fn verify(&self, authority: &str, payload_hash: [u8; 32]) -> Result<bool, SolanaError> {
        let record = self.get_attestation(authority).await?;
        Ok(record.payload_hash == payload_hash)
    }
}
