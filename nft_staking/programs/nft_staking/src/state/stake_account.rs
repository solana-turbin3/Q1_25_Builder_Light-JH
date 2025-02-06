use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub nft_mint: Pubkey, // nft_ mint, user can have different nfts for different stakeaccount
    pub staked_at: i64,   //time
    pub bump: u8,
}
