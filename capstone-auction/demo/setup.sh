#!/bin/bash

# Exit on errors
# set -e
# solana-keygen new --no-bip39-passphrase -s -o xxx.json

echo "Airdrop to default id..."
solana airdrop 1

echo "Transferring funds for fees..."
solana transfer --allow-unfunded-recipient admin.json 0.2
solana transfer --allow-unfunded-recipient auctioneer.json 0.2
solana transfer --allow-unfunded-recipient bidder1.json 0.2
solana transfer --allow-unfunded-recipient bidder2.json 0.2

echo "Creating mints..."
spl-token create-token mintA.json
spl-token create-token mintB.json

echo "Creating ATAs..."
spl-token create-account mintA.json
spl-token create-account mintB.json

echo "Minting tokens..."
spl-token mint mintA.json 1000
spl-token mint mintB.json 1000

echo "Transfer tokens..."
spl-token transfer mintA.json 1000 auctioneer.json --fund-recipient
spl-token transfer mintB.json 500 bidder1.json --fund-recipient
spl-token transfer mintB.json 500 bidder2.json --fund-recipient

# Display balances (optional)
# spl-token balance --owner $(solana address)

# echo "Both mint accounts have been created successfully!"
