use crate::{
    errors::MarketplaceError,
    state::{Listing, Marketplace},
};
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

#[derive(Accounts)]
pub struct Purchase<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub seller: SystemAccount<'info>,
    pub seller_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = seller_mint,
        associated_token::authority = buyer,
    )]
    pub buyer_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,
    #[account(
        mut,
        // close = seller, ???
        associated_token::mint = seller_mint,
        associated_token::authority = listing,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"listing",marketplace.key().as_ref(), seller_mint.key().as_ref()],
        bump = listing.bump,
        close = seller,
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        // mut,
        seeds = [b"treasury", marketplace.key().as_ref()],
        // bump = maketplace.treasury_bump,???
        bump,
    )]
    pub treasury: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"rewards", marketplace.key().as_ref()],
        bump = marketplace.rewards_bump,
        mint::decimals = 6,
        mint::authority = marketplace,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Purchase<'info> {
    pub fn send_sol(&mut self) -> Result<()> {
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.seller.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        let marketplace_fee = self
            .listing
            .price
            .checked_mul(self.marketplace.fee as u64)
            .ok_or(MarketplaceError::ArithematicOverflow)?
            .checked_div(10000_u64)
            .ok_or(MarketplaceError::ArithematicOverflow)?;

        let amount = self.listing.price - marketplace_fee;

        // let fee = (self.marketplace.fee as u64).checked_mul(self.listing.price).unwrap().checked_div(10000_u64).unwrap();

        // let amount = self.listing.price.checked_sub(fee).unwrap();

        transfer(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn send_nft(&mut self) -> Result<()> {
        let seeds = &[
            b"listing",
            &self.marketplace.key().to_bytes()[..],
            &self.seller_mint.key().to_bytes()[..],
            &[self.listing.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let transfer_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.buyer_ata.to_account_info(),
            mint: self.seller_mint.to_account_info(),
            authority: self.listing.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        transfer_checked(cpi_ctx, 1, self.seller_mint.decimals)?;
        Ok(())
    }

    pub fn close_mint_vault(&mut self) -> Result<()> {
        let seeds = &[
            &self.marketplace.key().to_bytes()[..],
            &self.seller_mint.key().to_bytes()[..],
            &[self.listing.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let accounts = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.seller.to_account_info(),
            authority: self.listing.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        close_account(cpi_ctx);
        Ok(())
    }
}
