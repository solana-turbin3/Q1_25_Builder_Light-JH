use anchor_lang::error_code;

#[error_code]
pub enum AuctionError {
    #[msg("The given name is too long")]
    NameTooLong,
    #[msg("ArithematicOverflow")]
    ArithematicOverflow,
    #[msg("PriceTooLow")]
    PriceTooLow,
    #[msg("NotEligibleToWithdraw")]
    NotEligibleToWithdraw,
}
