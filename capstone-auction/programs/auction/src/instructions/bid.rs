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
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        seeds = [b"house", auction_house.name.as_bytes()],
        bump = auction_house.bump,
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,
    #[account(
        mut,
        seeds = [b"auction", auction_house.key().as_ref(), auction.seller.key().as_ref(),mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump = auction.bump,
    )]
    pub auction: Box<Account<'info, Auction>>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = bidder,
    )]
    pub bidder_mint_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init,
        payer = bidder,
        space = 8 + BidState::INIT_SPACE,
        seeds = [b"bid", auction.key().as_ref(), bidder.key().as_ref()],
        bump,
    )]
    pub bid_state: Box<Account<'info, BidState>>,
    #[account(
        init,
        payer = bidder,
        associated_token::mint = mint_b,
        associated_token::authority = bid_state,
    )]
    pub bidder_escrow: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = auction.mint_a,
        associated_token::authority = auction,
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Bid<'info> {
    pub fn bid(&mut self, price: u64, bump: &BidBumps) -> Result<()> {
        require!(
            price > self.auction.highest_price,
            AuctionError::PriceTooLow
        );
        self.update_auction(price);
        self.create_bid_state(bump);
        self.deposit_bid()?;
        Ok(())
    }

    fn update_auction(&mut self, price: u64) {
        self.auction.bidder = Some(self.bidder.key());
        self.auction.highest_price = price;
    }

    fn create_bid_state(&mut self, bump: &BidBumps) {
        self.bid_state.set_inner(BidState {
            bidder: self.bidder.key(),
            bump: bump.bid_state,
            auction: self.auction.key(),
        });
    }

    fn deposit_bid(&mut self) -> Result<()> {
        let amount = calculate_amount_b(
            self.vault.amount,
            self.auction.highest_price,
            self.mint_a.decimals,
            self.mint_b.decimals,
            self.auction.decimal,
        )
        .ok_or(AuctionError::ArithematicOverflow)?;

        let cpi_program = self.token_program.to_account_info();
        //example bid_price = 2B/A, amount = 50, 2*50 = 100B
        let transfer_accounts = TransferChecked {
            from: self.bidder_mint_b_ata.to_account_info(),
            to: self.bidder_escrow.to_account_info(),
            mint: self.mint_b.to_account_info(),
            authority: self.bidder.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, transfer_accounts);

        transfer_checked(cpi_ctx, amount, self.mint_b.decimals)?;

        Ok(())
    }
}

fn calculate_amount_b(
    amount_a: u64,
    price: u64,
    decimals_a: u8,
    decimals_b: u8,
    price_decimals: u8,
) -> Option<u64> {
    // Scale factors
    let scale_a = 10u128.pow(decimals_a as u32);
    let scale_b = 10u128.pow(decimals_b as u32);
    let scale_price = 10u128.pow(price_decimals as u32);

    // Convert to u128 to prevent overflow
    let amount_a_128 = amount_a as u128;
    let price_128 = price as u128;

    // Compute: (amount_a * price * 10^decimals_b) / (10^(decimals_a + price_decimals))
    let numerator = amount_a_128.checked_mul(price_128)?.checked_mul(scale_b)?;

    let denominator = scale_a.checked_mul(scale_price)?;

    let amount_b = numerator.checked_div(denominator)?;

    // Convert back to u64 if safe
    amount_b.try_into().ok()
}
