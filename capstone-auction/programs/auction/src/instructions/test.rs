use crate::{
    errors::AuctionError,
    state::{Auction, AuctionHouse, BidState},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[derive(Accounts)]
pub struct Test<'info> {
    mint_a: InterfaceAccount<'info, Mint>,
    mint_b: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    user: Signer<'info>,

    admin: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_b,
        associated_token::authority = admin,
    )]
    admin_b: InterfaceAccount<'info, TokenAccount>,
    #[account(
        constraint = &auction_house.admin == admin.key
    )]
    auction_house: Account<'info, AuctionHouse>,

    seller: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_a,
        associated_token::authority = seller,
    )]
    seller_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_b,
        associated_token::authority = seller,
    )]
    seller_b: InterfaceAccount<'info, TokenAccount>,

    auction: Account<'info, Auction>,
    #[account(
        mut,
        associated_token::mint = auction.mint_a,
        associated_token::authority = auction,
    )]
    vault: InterfaceAccount<'info, TokenAccount>,

    bidder: SystemAccount<'info>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_a,
        associated_token::authority = bidder,
    )]
    bidder_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        constraint = bid_state.auction == auction.key(),
        constraint = &bid_state.bidder == bidder.key,
    )]
    bid_state: Account<'info, BidState>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = bid_state,
    )]
    bidder_escrow: InterfaceAccount<'info, TokenAccount>,

    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}

impl<'info> Test<'info> {
    pub fn test(&mut self) -> Result<()> {
        msg!("mints = {} / {}", self.mint_a.key(), self.mint_b.key());
        msg!("user = {}", self.user.key());
        msg!("admin = {}", self.admin.key());
        msg!(
            "admin_b = {} w/ {}",
            self.admin_b.key(),
            self.admin_b.amount
        );
        msg!("auction house = {}", self.auction_house.key());
        msg!("seller = {}", self.seller.key());
        msg!(
            "seller_a = {} w/ {}",
            self.seller_a.key(),
            self.seller_a.amount
        );
        msg!(
            "seller_b = {} w/ {}",
            self.seller_b.key(),
            self.seller_b.amount
        );
        msg!("auction = {}", self.auction.key());
        msg!("vault = {} w/ {}", self.vault.key(), self.vault.amount);
        msg!("bidder = {}", self.bidder.key());
        msg!(
            "bidder_a = {} w/ {}",
            self.bidder_a.key(),
            self.bidder_a.amount
        );
        msg!("bid_state = {}", self.bid_state.key());
        msg!("bid_state.bidder = {}", self.bid_state.bidder);
        msg!(
            "bidder escrow = {} w/ {}",
            self.bidder_escrow.key(),
            self.bidder_escrow.amount
        );
        require!(false, AuctionError::TestError);
        Ok(())
    }
}
