use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct BidState {
    pub bidder: Pubkey,
    pub auction: Pubkey, // make sure the bidder withdraw from the correct auction
    pub bump: u8,
}
