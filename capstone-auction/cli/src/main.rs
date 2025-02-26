use crate::idl::AuctionProgram;
use clap::{Parser, Subcommand};
use idl::InitHouseArgs;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::read_keypair_file, Signer},
};
use std::path::PathBuf;

mod idl;

#[derive(Parser)]
#[command(name = "auction-cli")]
#[command(about = " CLI for interacting with the Solana Aucton program")]
struct Cli {
    /// Path to keypair file used for signing
    #[arg(short, long, value_name = "PATH")]
    keypair_path: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize the AuctionHouse.
    InitHouse {
        /// Fee in basis points taken for successful auctions.
        fee: u16,
        /// Name to identify the auction house.
        name: String,
    },

    InitAuction {
        starting_price: u64,
        end_slot: u64,
        amount: u64,
        decimal: u8,
    },

    Bid {
        price: u64,
        decimal: u8,
    },

    Withdraw,
    Finalize,
}

fn main() {
    // let cli = Cli::parse();
    let Cli {
        keypair_path,
        command,
    } = Cli::parse();
    let rpc_url = "http://127.0.0.1:8899";
    let client = RpcClient::new(rpc_url);

    let keypair_path = keypair_path.unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Failed to get home directory")
            .join(".config/solana/id.json")
    });
    let keypair = read_keypair_file(&keypair_path).expect("Couldn't find wallet file");

    match command {
        Command::InitHouse { fee, name } => {
            println!("Initializing house with fee: {} and name: {}", fee, name);

            let recent_blockhash = client
                .get_latest_blockhash()
                .expect("recent blockhash exists");
            let (auction_house, _auction_house_bump) =
                Pubkey::find_program_address(&[b"house", name.as_bytes()], &AuctionProgram::id());
            let transaction = AuctionProgram::init_house(
                &[
                    &keypair.pubkey(),
                    &auction_house,
                    &solana_sdk::system_program::id(),
                ],
                &InitHouseArgs { fee, name },
                Some(&keypair.pubkey()),
                &[&keypair],
                recent_blockhash,
            );
            let signature = client
                .send_and_confirm_transaction(&transaction)
                .expect("confirmed transaction");
            println!(
                "Initialized auction house account: {} at {}",
                signature, auction_house
            )
        }

        Command::InitAuction {
            starting_price,
            end_slot,
            amount,
            decimal,
        } => {
            println!(
                "Initializing auction with starting price: {}, end slot: {}, amount: {}, decimal: {}",
                starting_price, end_slot, amount, decimal);
        }

        Command::Bid { price, decimal } => {
            println!("Placing bid with price: {} (decimal={})", price, decimal)
        }

        Command::Withdraw => {
            println!("Withdraw funds")
        }

        Command::Finalize => {
            println!("Finalizing Auction")
        }
    }
}
