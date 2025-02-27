use crate::idl::AuctionProgram;
use clap::{Parser, Subcommand};
use decimal::decimal_to_u64;
use idl::{BidArgs, InitAuctionArgs, InitHouseArgs};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    program_pack::Pack,
    pubkey::Pubkey,
    signer::{keypair::read_keypair_file, Signer},
};
use spl_token::state::Mint;
use std::path::PathBuf;

mod decimal;
mod idl;

#[derive(Parser)]
#[command(name = "auction-cli")]
#[command(about = " CLI for interacting with the Solana Aucton program")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Path to keypair file used for signing
    #[arg(short, long, value_name = "PATH")]
    keypair_path: Option<PathBuf>,

    /// Name to identify the auction house.
    #[arg(short, long, value_name = "NAME", default_value = "auction_house")]
    auction_house_name: String,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize the AuctionHouse.
    InitHouse {
        /// Fee in basis points taken for successful auctions.
        #[arg(value_name = "BASIS_POINTS")]
        fee: u16,
    },

    /// Initialize a new auction
    InitAuction {
        /// Mint of the token being listed for auction.
        /// #[clap(long, short)]
        listing_mint: Pubkey,
        /// Mint of the token used for bidding in auction.
        /// #[clap(long, short)]
        purchase_mint: Pubkey,

        /// The starting price.
        /// #[clap(long, short)]
        starting_price: String,
        /// The slot the auction will end on.
        /// #[clap(long, short)]
        end_slot: u64,
        /// The number of tokens to auction off.
        /// #[clap(long, short)]
        amount: String,

        /// The number of decimals to be used for the price.
        /// #[clap(long, short)]
        #[clap(long, short, default_value = "9")]
        decimals: u8,
    },

    /// bidder place bid
    Bid {
        /// Mint of the token being listed for auction.
        /// #[clap(long, short)]
        listing_mint: Pubkey,
        /// Mint of the token used for bidding in auction.
        /// #[clap(long, short)]
        purchase_mint: Pubkey,
        /// The seller in the auction.
        /// #[clap(long, short)]
        seller: Pubkey,
        /// bidder bid price, requring price higher than the current highest price
        /// #[clap(long, short)]
        price: String,
        /// The number of decimals to be used for the price.
        /// #[clap(long, short)]
        #[clap(long, short, default_value = "9")]
        decimals: u8,
    },
    /// bidder withdraw after falling out of highest price
    Withdraw {
        /// Mint of the token being listed for auction.
        // #[clap(long, short)]
        listing_mint: Pubkey,
        /// Mint of the token used for bidding in auction.
        purchase_mint: Pubkey,
        /// The seller in the auction.
        seller: Pubkey,
    },
    Finalize {
        /// auction house admin
        /// #[clap(long, short)]
        admin: Pubkey,
        /// Mint of the token being listed for auction.
        /// #[clap(long, short)]
        listing_mint: Pubkey,
        /// Mint of the token used for bidding in auction.
        /// #[clap(long, short)]
        purchase_mint: Pubkey,
        /// The seller in the auction.
        /// #[clap(long, short)]
        seller: Pubkey,
        /// The winner(bidder) in the auction.
        /// #[clap(long, short)]
        bidder: Pubkey,
    },
    Cancel {
        /// Mint of the token being listed for auction.
        /// #[clap(long, short)]
        listing_mint: Pubkey,
        /// Mint of the token used for bidding in auction.
        /// #[clap(long, short)]
        purchase_mint: Pubkey,
    },
}

fn main() {
    // let cli = Cli::parse();
    let Cli {
        keypair_path,
        command,
        auction_house_name,
    } = Cli::parse();

    let (auction_house, _auction_house_bump) = Pubkey::find_program_address(
        &[b"house", auction_house_name.as_bytes()],
        &AuctionProgram::id(),
    );

    let rpc_url = "http://127.0.0.1:8899";
    let client = RpcClient::new(rpc_url);

    let keypair_path = keypair_path.unwrap_or_else(|| {
        dirs::home_dir()
            .expect("Failed to get home directory")
            .join(".config/solana/id.json")
    });
    let keypair = read_keypair_file(&keypair_path).expect("Couldn't find wallet file");

    match command {
        Command::InitHouse { fee } => {
            println!(
                "Initializing house with fee: {} and name: {}",
                fee, auction_house_name
            );

            let recent_blockhash = client
                .get_latest_blockhash()
                .expect("recent blockhash exists");
            let transaction = AuctionProgram::init_house(
                &[
                    &keypair.pubkey(),
                    &auction_house,
                    &solana_sdk::system_program::id(),
                ],
                &InitHouseArgs {
                    fee,
                    name: auction_house_name,
                },
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
            listing_mint,
            purchase_mint,
            starting_price,
            end_slot,
            amount,
            decimals,
        } => {
            let seller = keypair.pubkey();
            let AuctionSellerKeys {
                auction,
                vault,
                seller_listing_mint_ata,
                ..
            } = derive_auction_keys(&auction_house, &listing_mint, &purchase_mint, &seller);

            let starting_price = decimal_to_u64(&starting_price, decimals).expect("invalid price");

            let listing_mint_account = client
                .get_account(&listing_mint)
                .expect("could not get listing mint account");
            let listing_mint_account =
                Mint::unpack(&listing_mint_account.data).expect("invalid mint account data");
            let listing_mint_decimals = listing_mint_account.decimals;
            let amount = decimal_to_u64(&amount, listing_mint_decimals).expect("invalid amount");

            let recent_blockhash = client
                .get_latest_blockhash()
                .expect("recent blockhash exists");
            let transaction = AuctionProgram::init_auction(
                &[
                    &seller,
                    &auction_house,
                    &auction,
                    &listing_mint,
                    &purchase_mint,
                    &seller_listing_mint_ata,
                    &vault,
                    &spl_associated_token_account::ID,
                    &solana_sdk::system_program::ID,
                    &spl_token::ID,
                ],
                &InitAuctionArgs {
                    starting_price,
                    end: end_slot,
                    amount,
                    decimal: decimals,
                },
                Some(&seller),
                &[&keypair],
                recent_blockhash,
            );
            let signature = client
                .send_and_confirm_transaction(&transaction)
                .expect("confirmed transaction");
            println!("Initialized auction account: {} at {}", signature, auction)
        }

        Command::Bid {
            listing_mint,
            purchase_mint,
            seller,
            price,
            decimals,
        } => {
            println!("Placing bid with price: {} (decimal={})", price, decimals);
            let AuctionSellerKeys { auction, vault, .. } =
                derive_auction_keys(&auction_house, &listing_mint, &purchase_mint, &seller);

            let bidder = keypair.pubkey();
            let BidderKeys {
                bidder_purchase_mint_ata,
                bid_state,
                bid_escrow,
                ..
            } = derive_bidder_keys(&bidder, &purchase_mint, &listing_mint, &auction);

            let recent_blockhash = client
                .get_latest_blockhash()
                .expect("recent blockhash exists");
            let transaction = AuctionProgram::bid(
                &[
                    &bidder,
                    &listing_mint,
                    &purchase_mint,
                    &auction_house,
                    &auction,
                    &bidder_purchase_mint_ata,
                    &bid_state,
                    &bid_escrow,
                    &vault,
                    &spl_associated_token_account::ID,
                    &spl_token::ID,
                    &solana_sdk::system_program::ID,
                ],
                &BidArgs {
                    price: decimal_to_u64(&price, decimals).expect("invalid price"), // TODO: make sure decimals matches auction decimals.
                },
                Some(&bidder),
                &[&keypair],
                recent_blockhash,
            );
            let signature = client
                .send_and_confirm_transaction(&transaction)
                .expect("confirmed transaction");
            println!("Placed bid and bid state: {} at {}", signature, bid_state)
        }

        Command::Withdraw {
            purchase_mint,
            listing_mint,
            seller,
        } => {
            let AuctionSellerKeys { auction, .. } =
                derive_auction_keys(&auction_house, &listing_mint, &purchase_mint, &seller);

            let bidder = keypair.pubkey();
            let BidderKeys {
                bidder_purchase_mint_ata,
                bid_state,
                bid_escrow,
                ..
            } = derive_bidder_keys(&bidder, &purchase_mint, &listing_mint, &auction);
            println!("bid_purchase_mint_ata={bidder_purchase_mint_ata}");
            println!("bid_state={bid_state}");
            println!("bid_escrow={bid_escrow}");

            let recent_blockhash = client
                .get_latest_blockhash()
                .expect("recent blockhash exists");

            let transaction = AuctionProgram::withdraw(
                &[
                    &bidder,
                    &purchase_mint,
                    &auction_house,
                    &auction,
                    &bidder_purchase_mint_ata,
                    &bid_escrow,
                    &bid_state,
                    &spl_associated_token_account::ID,
                    &spl_token::ID,
                    &solana_sdk::system_program::ID,
                ],
                Some(&bidder),
                &[&keypair],
                recent_blockhash,
            );
            let signature = client
                .send_and_confirm_transaction(&transaction)
                .expect("confirmed transaction");
            println!("Withdrawed bid:  {} at {}", signature, bid_escrow);
        }

        Command::Finalize {
            admin,
            listing_mint,
            purchase_mint,
            seller,
            bidder,
        } => {
            println!("Finalizing and close auction");
            let AuctionSellerKeys {
                auction,
                vault,
                seller_listing_mint_ata,
                seller_purchase_mint_ata,
            } = derive_auction_keys(&auction_house, &listing_mint, &purchase_mint, &seller);

            let BidderKeys {
                bidder_listing_mint_ata,
                bid_state,
                bid_escrow,
                ..
            } = derive_bidder_keys(&bidder, &purchase_mint, &listing_mint, &auction);

            let house_purchase_mint_ata =
                spl_associated_token_account::get_associated_token_address(&admin, &purchase_mint);

            let recent_blockhash = client
                .get_latest_blockhash()
                .expect("recent blockhash exists");

            let transaction = AuctionProgram::finalize(
                &[
                    &keypair.pubkey(),
                    &seller,
                    &bidder,
                    &admin,
                    &listing_mint,
                    &purchase_mint,
                    &auction_house,
                    &auction,
                    &bidder_listing_mint_ata,
                    &seller_purchase_mint_ata,
                    &seller_listing_mint_ata,
                    &house_purchase_mint_ata,
                    &bid_state,
                    &bid_escrow,
                    &vault,
                    &spl_associated_token_account::ID,
                    &spl_token::ID,
                    &solana_sdk::system_program::ID,
                ],
                Some(&keypair.pubkey()),
                &[&keypair],
                recent_blockhash,
            );

            let signature = client
                .send_and_confirm_transaction(&transaction)
                .expect("confirmed transaction");
            println!("Withdrawed bid: {}", signature);
        }

        Command::Cancel {
            listing_mint,
            purchase_mint,
        } => {
            println!("Auctioneer withdraw and cancel auction due to unsuccessful auction");
            let seller = keypair.pubkey();
            let AuctionSellerKeys {
                auction,
                vault,
                seller_listing_mint_ata,
                ..
            } = derive_auction_keys(&auction_house, &listing_mint, &purchase_mint, &seller);

            let recent_blockhash = client
                .get_latest_blockhash()
                .expect("recent blockhash exists");

            let transaction = AuctionProgram::cancel(
                &[
                    &seller,
                    &auction_house,
                    &auction,
                    &listing_mint,
                    &purchase_mint,
                    &seller_listing_mint_ata,
                    &vault,
                    &spl_associated_token_account::ID,
                    &solana_sdk::system_program::ID,
                    &spl_token::ID,
                ],
                Some(&seller),
                &[&keypair],
                recent_blockhash,
            );

            let signature = client
                .send_and_confirm_transaction(&transaction)
                .expect("confirmed transaction");

            println!("Canceled Auction {} at {}", auction, signature);
        }
    }
}

struct AuctionSellerKeys {
    auction: Pubkey,
    vault: Pubkey,
    seller_listing_mint_ata: Pubkey,
    seller_purchase_mint_ata: Pubkey,
}

fn derive_auction_keys(
    auction_house: &Pubkey,
    listing_mint: &Pubkey,
    purchase_mint: &Pubkey,
    seller: &Pubkey,
) -> AuctionSellerKeys {
    let (auction, _auction_bump) = Pubkey::find_program_address(
        &[
            b"auction",
            auction_house.as_ref(),
            seller.as_ref(),
            listing_mint.as_ref(),
            purchase_mint.as_ref(),
        ],
        &AuctionProgram::id(),
    );
    let vault = spl_associated_token_account::get_associated_token_address(&auction, listing_mint);
    let seller_listing_mint_ata =
        spl_associated_token_account::get_associated_token_address(seller, listing_mint);
    let seller_purchase_mint_ata =
        spl_associated_token_account::get_associated_token_address(seller, purchase_mint);

    AuctionSellerKeys {
        auction,
        vault,
        seller_listing_mint_ata,
        seller_purchase_mint_ata,
    }
}

struct BidderKeys {
    bidder_purchase_mint_ata: Pubkey,
    bidder_listing_mint_ata: Pubkey,
    bid_state: Pubkey,
    bid_escrow: Pubkey,
}

fn derive_bidder_keys(
    bidder: &Pubkey,
    purchase_mint: &Pubkey,
    listing_mint: &Pubkey,
    auction: &Pubkey,
) -> BidderKeys {
    let bidder_purchase_mint_ata =
        spl_associated_token_account::get_associated_token_address(bidder, purchase_mint);
    let bidder_listing_mint_ata =
        spl_associated_token_account::get_associated_token_address(bidder, listing_mint);
    let (bid_state, _bid_state_bump) = Pubkey::find_program_address(
        &[b"bid", auction.as_ref(), bidder.as_ref()],
        &AuctionProgram::id(),
    );
    let bid_escrow =
        spl_associated_token_account::get_associated_token_address(&bid_state, purchase_mint);

    BidderKeys {
        bidder_purchase_mint_ata,
        bidder_listing_mint_ata,
        bid_state,
        bid_escrow,
    }
}
