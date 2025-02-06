use anchor_lang::prelude::*;

declare_id!("FUFLpiJu29U6534qeu8X6aA4RBK47fdoZePow9EHEpHY");

mod errors;
mod instructions;
mod state;

use errors::*;
use instructions::*;

#[program]
pub mod marketplace {
    use super::*;
    // bumps get from ctx.bumps
    pub fn initialize(ctx: Context<Initialize>, fee: u16, name: String) -> Result<()> {
        ctx.accounts.init(fee, &ctx.bumps, name)?;
        Ok(())
    }

    pub fn list(ctx: Context<List>, price: u64) -> Result<()> {
        ctx.accounts.create_listing(price, &ctx.bumps)?;
        ctx.accounts.deposit()?;
        Ok(())
    }

    pub fn purchase(ctx: Context<Purchase>) -> Result<()> {
        ctx.accounts.send_sol()?;
        ctx.accounts.send_nft()?;
        ctx.accounts.close_mint_vault()?;
        Ok(())
    }

    pub fn delist(ctx: Context<Delist>) -> Result<()> {
        ctx.accounts.withdraw_nft()?;
        ctx.accounts.close_vault()?;
        Ok(())
    }
}
