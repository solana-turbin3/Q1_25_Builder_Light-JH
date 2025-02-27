#!/bin/bash
solana-test-validator --reset

anchor build && anchor deploy

./setup.sh
solana-keygen pubkey auctioneer.json

cargo run -- init-house 100

Usage: cli init-auction <LISTING_MINT> <PURCHASE_MINT> <STARTING_PRICE> <END_SLOT> <AMOUNT>

cargo run -- -k ../demo/auctioneer.json init-auction GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35LGTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym 4.567 3000 100
cargo run -- -k ../demo/bidder1.json bid GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35LGTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym 5WoXGMRMRMV2qpyzoUy8rVQ7Ae91rUrxHCAEpjminD6n 4.6
cargo run -- -k ../demo/bidder2.json bid GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35LGTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym 5WoXGMRMRMV2qpyzoUy8rVQ7Ae91rUrxHCAEpjminD6n 4.65
