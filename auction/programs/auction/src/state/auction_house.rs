use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AuctionHouse {
    pub admin: Pubkey,
    pub fee: u16,
    pub bump: u8,
    #[max_len(32)]
    pub name: String,
}
