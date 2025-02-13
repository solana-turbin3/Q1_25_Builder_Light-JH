use anchor_lang::prelude::*;

#[account]
pub struct AuctionHouse {
    pub admin: Pubkey,
    pub fee: u16,
    pub bump: u8,
    pub name: String, // Set the limit to 32 bytes
}

impl Space for AuctionHouse {
    const INIT_SPACE: usize = 8 + 32 + 2 + 1 + 32;
}
