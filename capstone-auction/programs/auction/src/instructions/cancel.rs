use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{close_account, CloseAccount},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::errors::AuctionError;
use crate::state::{Auction, AuctionHouse};

#[derive(Accounts)]
pub struct Cancel<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        // seeds = [b"house", auction_house.name.as_bytes()],
        // bump = auction_house.bump,
    )]
    pub auction_house: Account<'info, AuctionHouse>,
    // for the same house, seller can create auctions identified by different mints.
    // at the same time, seller can create auctions in different house when necessary.
    #[account(
        mut,
        close = seller,
        // seeds = [b"auction", auction_house.key().as_ref(), seller.key().as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref()],
        // bump = auction.bump,
    )]
    pub auction: Account<'info, Auction>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = seller,
    )]
    pub seller_mint_a_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = auction,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Cancel<'info> {
    pub fn cancel(&mut self) -> Result<()> {
        let current_slot = Clock::get()?.slot;

        require!(
            current_slot >= self.auction.end && self.auction.bidder.is_none(),
            AuctionError::NotEligibleToWithdraw
        );

        self.withdraw_and_close_vault()
    }

    fn withdraw_and_close_vault(&mut self) -> Result<()> {
        let seeds = &[
            b"auction",
            self.auction_house.to_account_info().key.as_ref(),
            self.seller.to_account_info().key.as_ref(),
            self.mint_a.to_account_info().key.as_ref(),
            self.mint_b.to_account_info().key.as_ref(),
            &[self.auction.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_program = self.token_program.to_account_info();
        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.seller_mint_a_ata.to_account_info(),
            authority: self.auction.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_accounts, signer_seeds);

        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        let accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.seller.to_account_info(),
            authority: self.auction.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        close_account(ctx)?;

        Ok(())
    }
}
