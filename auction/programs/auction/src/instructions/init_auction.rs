use anchor_lang::{prelude::*, solana_program::clock::Slot};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::state::{Auction, AuctionHouse};

#[derive(Accounts)]
#[instruction(starting_price: u64, end: Slot, amount: u64, decimal: u8)]
pub struct InitAuction<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        seeds = [b"house", auction_house.name.as_str().as_bytes()],
        bump = auction_house.bump,
    )]
    pub auction_house: Account<'info, AuctionHouse>,
    // for the same house, seller can create auctions identified by different mints.
    // at the same time, seller can create auctions in different house when necessary.
    #[account(
        init,
        payer = seller,
        space = 8 + Auction::INIT_SPACE,
        seeds = [b"auction", auction_house.key().as_ref(), seller.key().as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref(), end.to_le_bytes().as_ref()],
        bump,
    )]
    pub auction: Account<'info, Auction>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        associated_token::mint = mint_a,
        associated_token::authority = seller,
    )]
    pub seller_mint_a_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = seller,
        associated_token::mint = mint_a,
        associated_token::authority = auction,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> InitAuction<'info> {
    pub fn init_auction(
        &mut self,
        starting_price: u64,
        end: Slot,
        decimal: u8,
        bumps: &InitAuctionBumps,
    ) -> Result<()> {
        self.auction.set_inner(Auction {
            seller: self.seller.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            bump: bumps.auction,
            end,
            highest_price: starting_price.saturating_sub(1),
            decimal,
            bidder: Pubkey::default(),
        });

        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let transfer_accounts = TransferChecked {
            from: self.seller_mint_a_ata.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.seller.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, transfer_accounts);

        transfer_checked(cpi_ctx, amount, self.mint_a.decimals)?;

        Ok(())
    }
}
