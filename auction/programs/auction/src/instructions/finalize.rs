use crate::errors::AuctionError;
use crate::state::{Auction, AuctionHouse, BidState};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{close_account, CloseAccount},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub seller: SystemAccount<'info>,
    #[account(mut)]
    pub bidder: SystemAccount<'info>,
    pub admin: SystemAccount<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    pub auction_house: Account<'info, AuctionHouse>,
    #[account(
        mut,
        close = seller,
        has_one = mint_a,
        has_one = mint_b,
        // seeds = [b"auction", auction_house.key().as_ref(), seller.key().as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref(), auction.end.to_le_bytes().as_ref()],
        // bump = auction.bump,
        constraint = auction.bidder == Some(bidder.key()),
    )]
    pub auction: Account<'info, Auction>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_a,
        associated_token::authority = bidder,
    )]
    pub bidder_mint_a_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_b,
        associated_token::authority = seller,
    )]
    pub seller_mint_b_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_a,
        associated_token::authority = seller,
    )]
    pub seller_mint_a_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_b,
        associated_token::authority = admin,
    )]
    pub house_mint_b_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = bidder,
        seeds = [b"bid", auction.key().as_ref(), bidder.key().as_ref()],
        bump = bid_state.bump,
        constraint = bid_state.bidder == bidder.key(),
    )]
    pub bid_state: Account<'info, BidState>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = bid_state,
    )]
    pub bidder_escrow: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = auction.mint_a,
        associated_token::authority = auction,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

// there is a winner
impl<'info> Finalize<'info> {
    pub fn finalize(&mut self) -> Result<()> {
        msg!("bidder.key(): {:?}", self.bidder.key());
        msg!("bid_state.bidder: {:?}", self.bid_state.bidder);
        msg!("bid_state = {:?}", self.bid_state);
        self.winner_withdraw_and_close_vault()?;
        self.seller_withdraw_and_close_escrow()?;
        Ok(())
    }

    fn winner_withdraw_and_close_vault(&mut self) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        require!(
            current_slot >= self.auction.end,
            AuctionError::NotEligibleToWithdraw
        );
        require!(
            self.bid_state.bidder == self.bidder.key(),
            AuctionError::NotEligibleToWithdraw
        );
        require!(
            self.auction.bidder == Some(self.bidder.key()),
            AuctionError::NotEligibleToWithdraw
        );
        require!(
            Some(self.bid_state.bidder) == self.auction.bidder,
            AuctionError::NotEligibleToWithdraw
        );

        let seeds = &[
            b"auction",
            self.auction_house.to_account_info().key.as_ref(),
            self.seller.to_account_info().key.as_ref(),
            self.mint_a.to_account_info().key.as_ref(),
            self.mint_b.to_account_info().key.as_ref(),
            &self.auction.end.to_le_bytes()[..],
            &[self.auction.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.bidder_mint_a_ata.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.auction.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        // msg!("transfering to bidder ata a.");
        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        // close vault to refund rent exemption
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

    fn seller_withdraw_and_close_escrow(&mut self) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        require!(
            (current_slot >= self.auction.end),
            AuctionError::NotEligibleToWithdraw
        );
        let seeds = &[
            b"bid",
            self.auction.to_account_info().key.as_ref(),
            self.bidder.to_account_info().key.as_ref(),
            &[self.bid_state.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // transfer mintB from bidder_escrow to seller

        let transfer_accounts = TransferChecked {
            from: self.bidder_escrow.to_account_info(),
            to: self.seller_mint_b_ata.to_account_info(),
            mint: self.mint_b.to_account_info(),
            authority: self.bid_state.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        let house_fee = self
            .bidder_escrow
            .amount
            .checked_mul(u64::from(self.auction_house.fee))
            .unwrap()
            .checked_div(10_000)
            .unwrap();

        // msg!(&format!(
        //     "bidder escrow: {}. house_fee={}. vault={}.",
        //     self.bidder_escrow.amount, house_fee, self.vault.amount,
        // ));
        let amount = self.bidder_escrow.amount - house_fee;

        // msg!("transfering to bidder");
        transfer_checked(cpi_ctx, amount, self.mint_b.decimals)?;

        // transfer mintB from bidder_escrow to auction house

        let transfer_accounts = TransferChecked {
            from: self.bidder_escrow.to_account_info(),
            to: self.house_mint_b_ata.to_account_info(),
            mint: self.mint_b.to_account_info(),
            authority: self.bid_state.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        // msg!("transfering to house");
        transfer_checked(cpi_ctx, house_fee, self.mint_b.decimals)?;

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

    //Unsuccessful auction
    pub fn cancel(&mut self) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        require!(
            (self.auction.bidder.is_none() && current_slot >= self.auction.end),
            AuctionError::NotEligibleToWithdraw
        );

        // seller get back mintA and close auction
        let seeds = &[
            b"auction",
            self.auction_house.to_account_info().key.as_ref(),
            self.seller.to_account_info().key.as_ref(),
            self.mint_a.to_account_info().key.as_ref(),
            self.mint_b.to_account_info().key.as_ref(),
            &self.auction.end.to_le_bytes()[..],
            &[self.auction.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.seller_mint_a_ata.to_account_info(),
            mint: self.mint_a.to_account_info(),
            authority: self.auction.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        // msg!("transfering to seller ata a.");
        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        // close vault to refund rent exemption
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
