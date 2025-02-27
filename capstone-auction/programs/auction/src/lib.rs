use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Slot;
mod errors;
mod instructions;
mod state;

// use errors::*;
use instructions::*;

declare_id!("7VNBDULA3eH3ctDqx5ckpfZA1Xe2AkjUnGjuXe7de6bf");

#[program]
pub mod auction {

    use super::*;

    pub fn init_house(ctx: Context<InitHouse>, fee: u16, name: String) -> Result<()> {
        ctx.accounts.init_house(fee, &ctx.bumps, name)?;
        Ok(())
    }

    pub fn init_auction(
        ctx: Context<InitAuction>,
        starting_price: u64,
        end: Slot,
        amount: u64,
        decimal: u8,
    ) -> Result<()> {
        ctx.accounts
            .init_auction(starting_price, end, amount, decimal, &ctx.bumps)?;
        Ok(())
    }

    pub fn bid(ctx: Context<Bid>, price: u64) -> Result<()> {
        ctx.accounts.bid(price, &ctx.bumps)?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        ctx.accounts.withdraw()?;
        Ok(())
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        ctx.accounts.finalize()?;
        Ok(())
    }

    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        ctx.accounts.cancel()?;
        Ok(())
    }
}
