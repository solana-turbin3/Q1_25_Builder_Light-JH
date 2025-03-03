#!/bin/bash
solana-test-validator --reset

anchor build && anchor deploy

./setup.sh
solana-keygen pubkey auctioneer.json

export HOUSE="auction_house2"
auction-cli -a $HOUSE init-house 100
auction-cli -a $HOUSE -k auctioneer.json init-auction GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35LGTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym 4.567 $(( $(solana -ud slot) + 300 )) 100
auction-cli -a $HOUSE -k bidder1.json bid GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35LGTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym 5WoXGMRMRMV2qpyzoUy8rVQ7Ae91rUrxHCAEpjminD6n 4.6
auction-cli -a $HOUSE -k bidder2.json bid GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35LGTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym 5WoXGMRMRMV2qpyzoUy8rVQ7Ae91rUrxHCAEpjminD6n 4.65
auction-cli -a $HOUSE -k auctioneer.json finalize DbosQhf29wdLJP7iCp2r2AHjdVow781nPqjhwqu4t41i GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35LGTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym 5WoXGMRMRMV2qpyzoUy8rVQ7Ae91rUrxHCAEpjminD6n 8DPrt4VU4qmc9jT925zWSmSE7gqRmqSGftfLQvAE7R2a
auction-cli -a $HOUSE -k auctioneer.json cancel GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35LGTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym