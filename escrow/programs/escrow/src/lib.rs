use anchor_lang::prelude::*;

pub mod instructions;
use instructions::*;

pub mod state;
pub use state::*;

declare_id!("EBTW9hWHnBqmy3guGE23fBR62CZYwEigTkCjLWmaT9kK");

#[program]
pub mod escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, receive: u64, deposit: u64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)?;
        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()?;
        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refunc_and_close_vault()?;
        Ok(())
    }
}
