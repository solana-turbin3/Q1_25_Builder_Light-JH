use crate::errors::AuctionError;
use crate::state::{Auction, AuctionHouse, BidState};

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct Bid<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [b"house", auction_house.name.as_str().as_bytes()],
        bump = auction_house.bump,
    )]
    pub auction_house: Account<'info, AuctionHouse>,
    #[account(
        mut,
        seeds = [b"auction", auction_house.key().as_ref(), auction.seller.key().as_ref(), auction.mint_a.key().as_ref(), mint_b.key().as_ref(), auction.end.to_le_bytes().as_ref()],
        bump = auction.bump,
    )]
    pub auction: Account<'info, Auction>,
    #[account(
        associated_token::mint = mint_b,
        associated_token::authority = bidder,
    )]
    pub bidder_mint_b_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = bidder,
        associated_token::mint = mint_b,
        associated_token::authority = bid_state,
    )]
    pub bidder_escrow: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = bidder,
        space = 8 + BidState::INIT_SPACE,
        seeds = [b"bid", auction.key().as_ref(), bidder.key().as_ref()],
        bump,
    )]
    pub bid_state: Account<'info, BidState>,
    #[account(
        associated_token::mint = auction.mint_a,
        associated_token::authority = auction,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Bid<'info> {
    pub fn place_and_update_bid(&mut self, price: Decimal, bump: &BidBumps) -> Result<()> {
        require!(
            price > self.auction.highest_price,
            AuctionError::PriceTooLow
        );

        self.auction.highest_price = price;
        self.auction.bidder = self.bidder.key();

        // create bid account to store the bid account
        self.bid_state.set_inner(BidState {
            bidder: self.bidder.key(),
            bump: bump.bid_state,
        });
        Ok(())
    }
    pub fn deposit(&mut self) -> Result<()> {
        let amount = self
            .vault
            .amount
            .checked_mul(self.auction.highest_price)
            .ok_or(AuctionError::ArithematicOverflow)?
            .checked_div(10u64.pow(u32::from(self.auction.decimal)))
            .ok_or(AuctionError::ArithematicOverflow)?;

        let cpi_program = self.token_program.to_account_info();

        let transfer_accounts = TransferChecked {
            from: self.bidder_mint_b_ata.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.bidder_escrow.to_account_info(),
            authority: self.bid_state.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, transfer_accounts);

        transfer_checked(cpi_ctx, amount, self.mint_b.decimals)?;

        Ok(())
    }
}
