use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Auction {
    pub seller: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub bump: u8,
    pub end: u64,
    pub highest_price: u64,
    pub decimal: u8,
    pub bidder: Option<Pubkey>,
}
