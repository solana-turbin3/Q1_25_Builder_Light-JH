use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub points: u32,       // number of reward token
    pub amount_staked: u8, // nfts staked
    pub bump: u8,
}
