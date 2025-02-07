use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;
use instructions::*;

declare_id!("9dHN5L6sSZwQgnGojDAGhtGTxKGsqsn2PQUUfdErvQrb");

#[program]
pub mod amm {
    use super::*;

    pub fn init(ctx: Context<Init>) -> Result<()> {
        Ok(())
    }
}
