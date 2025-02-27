#!/bin/bash

cargo run -- init-house 100
cargo run -- -k ../demo/auctioneer.json init-auction GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35LGTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym 4.567 700 100
cargo run -- -k ../demo/bidder1.json bid GRXHSrCmGPAsEDTFtrKH78xjHznWaFXBzdvdV35LGTTZ 7GhJv6M85G59zHSvZBjZtaVEWExC5JLeFQASC2NNUgym 5WoXGMRMRMV2qpyzoUy8rVQ7Ae91rUrxHCAEpjminD6n 4.0
