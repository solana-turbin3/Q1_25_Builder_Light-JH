# Capstone project - On-chain Auction
## Overview
A blockchain-based auction that leverages smart contracts for price discovery and efficient resource allocation.
## Use case
+ Dutch Auctions for Fair and Transparent Token Launches
+ Optimized English Auctions for Rare Digital Collectibles
## Program Architecture
![architecture diagram](images/architecture-diagram.png)
## Accounts setup before running the program
Run below command in the cli folder to create necessary accounts for the program
```
./setup.sh
```
## Command line usage
After account
Use the following help command to know the auction program cli usage:
```
cargo run -- --help
```
Example to create an auction house with default house name and required input of house fee
```
cargo run -- init-house 100
```
Example to create an auction
```
cargo run -- -k ../demo/auctioneer.json init-auction GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35L
GTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym 4.567 5000 100
```



