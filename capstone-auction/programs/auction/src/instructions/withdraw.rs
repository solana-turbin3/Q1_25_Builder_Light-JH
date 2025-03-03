use crate::errors::AuctionError;
use crate::state::{Auction, AuctionHouse, BidState};

use anchor_lang::prelude::*;
use anchor_spl::token::{close_account, CloseAccount};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub bidder: Signer<'info>,
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        seeds = [b"house", auction_house.name.as_bytes()],
        bump = auction_house.bump,
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,
    /// CHECK: This is unchecked, because the account may or may not exist at this point.
    pub auction: AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = bidder,
        associated_token::mint = mint_b,
        associated_token::authority = bidder,
    )]
    pub bidder_mint_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = bid_state,
    )]
    pub bidder_escrow: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        close = bidder,
        seeds = [b"bid", auction.key().as_ref(), bidder.key().as_ref()],
        bump = bid_state.bump,
        constraint = bid_state.auction == auction.key()
    )]
    pub bid_state: Box<Account<'info, BidState>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
// bidder withdraw for falling out the highest price
impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self) -> Result<()> {
        msg!("withdraw");
        // If the `auction` account does not exist (no data)
        // then no additional verification is necessary.
        // If the `auction` account does exist, it must be Auction,
        // and the withdrawing bidder must not be the highest bidder.
        if !self.auction.data_is_empty() {
            msg!("auction still exists...");
            let auction = Auction::try_deserialize(&mut self.auction.data.borrow().as_ref())?;
            require!(
                auction.bidder != Some(self.bid_state.bidder),
                AuctionError::NotEligibleToWithdraw
            );
            msg!("and i'm not highest bidder");
        }

        let seeds = &[
            b"bid",
            self.auction.to_account_info().key.as_ref(),
            self.bidder.to_account_info().key.as_ref(),
            &[self.bid_state.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_program = self.token_program.to_account_info();

        let transfer_accounts = TransferChecked {
            from: self.bidder_escrow.to_account_info(),
            to: self.bidder_mint_b_ata.to_account_info(),
            mint: self.mint_b.to_account_info(),
            authority: self.bid_state.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, transfer_accounts, signer_seeds);

        msg!("transfering back to bidder");
        transfer_checked(cpi_ctx, self.bidder_escrow.amount, self.mint_b.decimals)?;

        // close escrow
        let seeds = &[
            b"bid",
            self.auction.to_account_info().key.as_ref(),
            self.bidder.to_account_info().key.as_ref(),
            &[self.bid_state.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let accounts = CloseAccount {
            account: self.bidder_escrow.to_account_info(),
            destination: self.bidder.to_account_info(),
            authority: self.bid_state.to_account_info(),
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
