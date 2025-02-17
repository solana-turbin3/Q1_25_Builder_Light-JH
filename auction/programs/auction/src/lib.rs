use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Slot;
mod errors;
mod instructions;
mod state;

// use errors::*;
use instructions::*;

declare_id!("83hbyGzsQ2Ekje4oD1g87dS9KmRVxbK4bJq2DHZCHFZR");

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
            .init_auction(starting_price, end, decimal, &ctx.bumps)?;
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    pub fn bid(ctx: Context<Bid>, price: u64, decimal: u8) -> Result<()> {
        ctx.accounts
            .place_and_update_bid(price, decimal, &ctx.bumps)?;
        ctx.accounts.deposit()?;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        ctx.accounts.withdraw()?;
        Ok(())
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
        ctx.accounts.winner_withdraw_and_close_vault()?;
        ctx.accounts.seller_withdraw_and_close_escrow()?;
        Ok(())
    }
}
