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
    /// CHECK: Just need pubkey...
    #[account(mut)]
    pub seller: AccountInfo<'info>,
    /// CHECK: Just need pubkey...
    #[account(mut)]
    pub bidder: AccountInfo<'info>,
    /// CHECK: Just need pubkey...
    pub admin: AccountInfo<'info>,
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        seeds = [b"house", auction_house.name.as_bytes()],
        bump = auction_house.bump,
    )]
    pub auction_house: Box<Account<'info, AuctionHouse>>,
    #[account(
        mut,
        close = seller,
        seeds = [b"auction", auction_house.key().as_ref(), seller.key().as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump = auction.bump,
        constraint = auction.bidder == Some(bidder.key()),
    )]
    pub auction: Box<Account<'info, Auction>>,
    #[account(
        mut,
        close = bidder,
        seeds = [b"bid", auction.key().as_ref(), bidder.key().as_ref()],
        bump = bid_state.bump,
        constraint = bid_state.bidder == bidder.key(),
    )]
    pub bid_state: Box<Account<'info, BidState>>,
    #[account(
        mut,
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

    /// CHECK: Must be token account (mint_a, bidder).
    #[account(
        mut,
        seeds = [
            bidder.key().as_ref(),
            token_program.key().as_ref(),
            mint_a.key().as_ref(),
        ],
        seeds::program = associated_token_program.key(),
        bump,
    )]
    pub bidder_mint_a_ata: AccountInfo<'info>,
    /// CHECK: Must be token account (mint_b, seller).
    #[account(
        mut,
        seeds = [
            seller.key().as_ref(),
            token_program.key().as_ref(),
            mint_b.key().as_ref(),
        ],
        seeds::program = associated_token_program.key(),
        bump,
    )]
    pub seller_mint_b_ata: AccountInfo<'info>,
    /// CHECK: Must be token account (mint_b, admin).
    #[account(
        mut,
        seeds = [
            admin.key().as_ref(),
            token_program.key().as_ref(),
            mint_b.key().as_ref(),
        ],
        seeds::program = associated_token_program.key(),
        bump,
    )]
    pub admin_mint_b_ata: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

// there is a winner
impl<'info> Finalize<'info> {
    pub fn finalize(&mut self) -> Result<()> {
        // msg!("bidder.key(): {:?}", self.bidder.key());
        // msg!("bid_state.bidder: {:?}", self.bid_state.bidder);
        // msg!("bid_state = {:?}", self.bid_state);
        self.winner_withdraw_and_close_vault()?;
        self.seller_withdraw_and_close_escrow()?; // house fee transfered covered here as well
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

        self.initialize_token_account_if_needed(
            self.mint_a.to_account_info(),
            self.bidder.to_account_info(),
            self.bidder_mint_a_ata.to_account_info(),
        )?;

        let seeds = &[
            b"auction",
            self.auction_house.to_account_info().key.as_ref(),
            self.seller.to_account_info().key.as_ref(),
            self.mint_a.to_account_info().key.as_ref(),
            self.mint_b.to_account_info().key.as_ref(),
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

        self.initialize_token_account_if_needed(
            self.mint_b.to_account_info(),
            self.seller.to_account_info(),
            self.seller_mint_b_ata.to_account_info(),
        )?;

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

        self.initialize_token_account_if_needed(
            self.mint_b.to_account_info(),
            self.admin.to_account_info(),
            self.admin_mint_b_ata.to_account_info(),
        )?;

        // transfer mintB from bidder_escrow to auction house

        let transfer_accounts = TransferChecked {
            from: self.bidder_escrow.to_account_info(),
            to: self.admin_mint_b_ata.to_account_info(),
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

    fn initialize_token_account_if_needed(
        &mut self,
        mint: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        associated_token: AccountInfo<'info>,
    ) -> Result<()> {
        if associated_token.data_is_empty() {
            anchor_spl::associated_token::create(CpiContext::new(
                self.associated_token_program.to_account_info(),
                anchor_spl::associated_token::Create {
                    payer: self.payer.to_account_info(),
                    associated_token,
                    authority,
                    mint,
                    system_program: self.system_program.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                },
            ))
        } else {
            Ok(())
        }
    }
}
