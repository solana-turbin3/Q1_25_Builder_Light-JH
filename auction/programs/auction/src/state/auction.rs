use anchor_lang::{prelude::*, solana_program::clock::Slot};

#[account]
pub struct Auction {
    pub seller: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub bump: u8,
    pub end: Slot,
    pub highest_price: u64,
    pub decimal: u8,
    pub bidder: Pubkey,
}

impl Space for Auction {
    const INIT_SPACE: usize = 8 + 32 + 32 + 32 + 1 + 8 + 8 + 1 + 32;
}
