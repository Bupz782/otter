use anchor_lang::prelude::*;

/// The Otter Solana attestation registry. V1 stores a single attestation per
/// authority: a 32-byte payload hash plus a UNIX timestamp. It is intentionally
/// small so it can be deployed on devnet/mainnet with minimal rent.
#[program]
pub mod attestation_registry {
    use super::*;

    /// Store (or overwrite) the caller's attestation with a new payload hash.
    pub fn attest(ctx: Context<Attest>, payload_hash: [u8; 32]) -> Result<()> {
        let attestation = &mut ctx.accounts.attestation;
        attestation.authority = ctx.accounts.authority.key();
        attestation.payload_hash = payload_hash;
        attestation.timestamp = Clock::get()?.unix_timestamp;
        attestation.bump = ctx.bumps.attestation;
        Ok(())
    }

    /// Remove the caller's attestation. Mainly useful for testing / revocation.
    pub fn revoke(ctx: Context<Revoke>) -> Result<()> {
        ctx.accounts.attestation.payload_hash = [0u8; 32];
        ctx.accounts.attestation.timestamp = 0;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(payload_hash: [u8; 32])]
pub struct Attest<'info> {
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + Attestation::SIZE,
        seeds = [b"attestation", authority.key().as_ref()],
        bump
    )]
    pub attestation: Account<'info, Attestation>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Revoke<'info> {
    #[account(
        mut,
        seeds = [b"attestation", authority.key().as_ref()],
        bump = attestation.bump,
        has_one = authority
    )]
    pub attestation: Account<'info, Attestation>,
    pub authority: Signer<'info>,
}

#[account]
pub struct Attestation {
    pub authority: Pubkey,
    pub payload_hash: [u8; 32],
    pub timestamp: i64,
    pub bump: u8,
}

impl Attestation {
    /// Size excluding the 8-byte Anchor account discriminator.
    pub const SIZE: usize = 32 + 32 + 8 + 1;
}
