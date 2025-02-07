use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, TokenAccount},
};

use crate::state::*;
#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Init<'info> {
    #[account(mut)]
    pub init_user: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    #[account(
        init,
        payer = init_user,
        seeds = [b"lp", config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub mint_lp: Account<'info, Mint>,
    #[account(
        init,
        payer = init_user,
        associated_token::mint = mint_x,
        associated_token::authority = config,
    )]
    pub mint_x_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = init_user,
        associated_token::mint = mint_y,
        associated_token::authority = config,
    )]
    pub mint_y_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = init_user,
        seeds = [b"config", seed.to_le_bytes().as_ref()],
        bump,
        space = Config::INIT_SPACE,
    )]
    pub config: Account<'info, Config>,
    pub token_program: Program<'info, TokenProgram>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Init<'info> {
    pub fn init(&mut self, seed: u64, authority: Option<Pubkey>, bumps: &InitBumps) -> Result<()> {
        self.config.set_inner(Config {
            authority,
            seed,
            fee,
            mint_x: self.mint_x.key(),
            mint_y: self.mint_x.key(),
            locked: false,
            config_bump: bumps.config,
            lp_bump: bumps.mint_lp,
        });
    }
}
