use anchor_lang::prelude::*;

use crate::errors::AuctionError;
use crate::state::AuctionHouse;

#[derive(Accounts)]
#[instruction(_fee: u16, name: String)]
pub struct InitHouse<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + AuctionHouse::INIT_SPACE,
        // convert the string slice (&str) into a byte array([&[u8]])
        // different accounts to be created under the same program but uniquelly identified by "name"
        seeds = [b"house", name.as_bytes()],
        bump,
    )]
    pub auction_house: Account<'info, AuctionHouse>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitHouse<'info> {
    pub fn init_house(&mut self, fee: u16, bumps: &InitHouseBumps, name: String) -> Result<()> {
        require!(
            !name.is_empty() && name.len() < 32,
            AuctionError::NameTooLong
        );

        self.auction_house.set_inner(AuctionHouse {
            admin: self.admin.key(),
            fee,
            bump: bumps.auction_house,
            name,
        });

        Ok(())
    }
}
