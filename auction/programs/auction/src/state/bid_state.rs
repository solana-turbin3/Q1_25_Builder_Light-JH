use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct BidState {
    pub bidder: Pubkey,
    pub bump: u8,
}
