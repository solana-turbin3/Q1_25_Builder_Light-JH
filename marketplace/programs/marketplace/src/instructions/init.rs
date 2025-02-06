use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::state::Marketplace;
use crate::MarketplaceError;

#[derive(Accounts)]
#[instruction(name: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Marketplace::INIT_SPACE,
        // convert the string slice (&str) into a byte array([&[u8]])
        // different accounts to be created under the same program but uniquelly identified by "name"
        seeds = [b"marketplace", name.as_str().as_bytes()],
        bump,
    )]
    pub marketplace: Account<'info, Marketplace>,
    #[account(
        // each unique marketplace account will result in a different PDA for treasury
        seeds = [b"treasury", marketplace.key().as_ref()],
        bump,
    )]
    pub treasury: SystemAccount<'info>, // hold sol for fee
    #[account(
        init,
        payer = admin,
        seeds = [b"rewards", marketplace.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = marketplace,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>, // reward_mint for rewarding participations
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Initialize<'info> {
    pub fn init(&mut self, fee: u16, bumps: &InitializeBumps, name: String) -> Result<()> {
        require!(
            name.len() > 0 && name.len() < 4 + 33,
            MarketplaceError::NameTooLong
        );

        self.marketplace.set_inner(Marketplace {
            admin: self.admin.key(),
            fee,
            bump: bumps.marketplace,
            treasury_bump: bumps.treasury,
            rewards_bump: bumps.rewards_mint,
            name,
        });

        Ok(())
    }
}
