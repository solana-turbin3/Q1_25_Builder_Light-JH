use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{MasterEditionAccount, Metadata, MetadataAccount},
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::state::{Listing, Marketplace};

#[derive(Accounts)]
pub struct List<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    #[account(
        seeds = [b"marketplace", marketplace.name.as_str().as_bytes()],
        bump = marketplace.bump,
    )]
    pub marketplace: Account<'info, Marketplace>,
    pub seller_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = seller,
        associated_token::mint = seller_mint,
        associated_token::authority = seller,
    )]
    pub seller_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = seller,
        associated_token::mint = seller_mint,
        associated_token::authority = listing,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = seller,
        space = Listing::INIT_SPACE,
        seeds = [b"listing", marketplace.key().as_ref(), seller_mint.key().as_ref()],
        bump,
    )]
    pub listing: Account<'info, Listing>, //hold listing info include price
    //verify that seller_mint (the NFT) is part of the collection identified by collection_mint.
    pub collection_mint: InterfaceAccount<'info, Mint>,
    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            seller_mint.key().as_ref(),
        ], //Metaplex uses a deterministic Program Derived Address (PDA) to store metadata:
        //metadata_pda = [ "metadata", metadata_program_id, mint ]
        // the seeds confirm that this metadata account belongs to the seller_mint NFT
        seeds::program = metadata_program.key(),
        bump,
        // ensure the NFT belongs to this collection
        constraint =
        metadata.collection.as_ref().unwrap().key.as_ref() ==
        collection_mint.key().as_ref(),
        // the NFT has been offically verified as part of the collection
        constraint = metadata.collection.as_ref().unwrap().verified == true,

    )]
    pub metadata: Account<'info, MetadataAccount>, // store the metadata for a specific NFT(seller_mint)
    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            seller_mint.key().as_ref(),
            b"edition"
        ],
        seeds::program = metadata_program.key(),
        bump
    )] // additional check that seller_mint is a valid NFT with a master edition
    pub master_edition: Account<'info, MasterEditionAccount>,

    pub metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    // interface allows the program to interact with different versions of the SPL TokenProgram
    //spl-token & spl-token-2022
    //Anchor resolves the correct program at runtime
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> List<'info> {
    pub fn create_listing(&mut self, price: u64, bumps: &ListBumps) -> Result<()> {
        self.listing.set_inner(Listing {
            seller: self.seller.key(),
            seller_mint: self.seller_mint.key(),
            price,
            bump: bumps.listing,
        });

        Ok(())
    }

    pub fn deposit(&mut self) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();
        let transfer_accounts = TransferChecked {
            from: self.seller_ata.to_account_info(),
            mint: self.seller_mint.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.seller.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, transfer_accounts);

        transfer_checked(cpi_ctx, 1, self.seller_mint.decimals)?;

        Ok(())
    }
}
