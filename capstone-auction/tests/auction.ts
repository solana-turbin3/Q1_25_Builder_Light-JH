import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Auction } from "../target/types/auction";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram, PublicKey, ComputeBudgetInstruction, ComputeBudgetProgram } from "@solana/web3.js";


import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddressSync,
    getMinimumBalanceForRentExemptMint,
    MINT_SIZE,
    createInitializeMint2Instruction,
    createAssociatedTokenAccountIdempotentInstruction,
    createMintToInstruction,
    getAccount


} from "@solana/spl-token";
import { assert, expect } from "chai";

describe("auction", () => {

    let provider = anchor.getProvider();
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Auction as Program<Auction>;

    const admin = Keypair.generate();
    const seller = Keypair.generate();
    const bidder = Keypair.generate();
    const mintA = Keypair.generate();
    const mintB = Keypair.generate();
    const tokenProgram = TOKEN_PROGRAM_ID;
    const sellerAtaA = getAssociatedTokenAddressSync(mintA.publicKey, seller.publicKey, true, tokenProgram);
    const bidderAtaA = getAssociatedTokenAddressSync(mintA.publicKey, bidder.publicKey, true, tokenProgram);
    const sellerAtaB = getAssociatedTokenAddressSync(mintB.publicKey, seller.publicKey, true, tokenProgram);
    const bidderAtaB = getAssociatedTokenAddressSync(mintB.publicKey, bidder.publicKey, true, tokenProgram);
    const houseAtaB = getAssociatedTokenAddressSync(mintB.publicKey, admin.publicKey, true, tokenProgram)
    const name = String("testAuction");
    const [auction_house] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("house"),
            Buffer.from("testAuction"),
        ],
        program.programId,
    );
    let end: anchor.BN;
    let auction: PublicKey;
    let vault: PublicKey;
    let bidderEscrow: PublicKey;
    let bidder2Escrow: PublicKey;
    let bidState: PublicKey;
    let bid2State: PublicKey;

    const starting_price = new anchor.BN(2000000);
    const amount = new anchor.BN(50);
    const bidPrice = new anchor.BN(3000000);
    const bidPrice2 = new anchor.BN(4000000);

    const bidder2 = Keypair.generate();
    const bidder2AtaB = getAssociatedTokenAddressSync(mintB.publicKey, bidder2.publicKey, true, tokenProgram);
    const bidder2AtaA = getAssociatedTokenAddressSync(mintA.publicKey, bidder2.publicKey, true, tokenProgram);


    before("airdrop", async () => {
        console.log("requesting airdrops");
        let lamports = await getMinimumBalanceForRentExemptMint(program.provider.connection);
        {
            let tx = new anchor.web3.Transaction();
            tx.instructions = [
                SystemProgram.transfer({
                    fromPubkey: provider.publicKey,
                    toPubkey: seller.publicKey,
                    lamports: 0.2 * LAMPORTS_PER_SOL,

                }),
                SystemProgram.transfer({
                    fromPubkey: provider.publicKey,
                    toPubkey: bidder.publicKey,
                    lamports: 0.2 * LAMPORTS_PER_SOL,

                }),
                SystemProgram.transfer({
                    fromPubkey: provider.publicKey,
                    toPubkey: bidder2.publicKey,
                    lamports: 0.2 * LAMPORTS_PER_SOL,

                }),
                SystemProgram.transfer({
                    fromPubkey: provider.publicKey,
                    toPubkey: admin.publicKey,
                    lamports: 0.2 * LAMPORTS_PER_SOL,

                }),
                SystemProgram.createAccount({
                    fromPubkey: provider.publicKey,
                    newAccountPubkey: mintA.publicKey,
                    lamports,
                    space: MINT_SIZE,
                    programId: tokenProgram,

                }),
                SystemProgram.createAccount({
                    fromPubkey: provider.publicKey,
                    newAccountPubkey: mintB.publicKey,
                    lamports,
                    space: MINT_SIZE,
                    programId: tokenProgram,

                })
            ];
            console.log("airdropping for: ", {
                seller: seller.publicKey.toString(),
                bidder: bidder.publicKey.toString(),
                bidder2: bidder2.publicKey.toString(),
                mintA: mintA.publicKey.toString(),
                mintB: mintB.publicKey.toString(),
            });
            await provider.sendAndConfirm(tx, [mintA, mintB]);
        }

        {
            let tx = new anchor.web3.Transaction();
            tx.instructions = [
                createInitializeMint2Instruction(mintA.publicKey, 6, seller.publicKey, null, tokenProgram),
                createAssociatedTokenAccountIdempotentInstruction(
                    provider.publicKey, sellerAtaA, seller.publicKey, mintA.publicKey, tokenProgram
                ),
                createMintToInstruction(mintA.publicKey, sellerAtaA, seller.publicKey, 1e9, undefined, tokenProgram)
            ];
            console.log("creating mintA, minting seller ATA: ", {
                seller: seller.publicKey.toString(),
                mintA: mintA.publicKey.toString(),
                sellerAtaA: sellerAtaA.toString(),
            });
            await provider.sendAndConfirm(tx, [seller]);
        }
        // bidderAtaB
        {
            let tx = new anchor.web3.Transaction();
            tx.instructions = [
                createInitializeMint2Instruction(mintB.publicKey, 6, bidder.publicKey, null, tokenProgram),
                createAssociatedTokenAccountIdempotentInstruction(
                    provider.publicKey, bidderAtaB, bidder.publicKey, mintB.publicKey, tokenProgram
                ),
                createMintToInstruction(mintB.publicKey, bidderAtaB, bidder.publicKey, 1e9, undefined, tokenProgram),
            ]
            console.log("creating mintB, minting bidder ATA: ", {
                bidder: bidder.publicKey.toString(),
                mintB: mintB.publicKey.toString(),
                bidderAtaB: bidderAtaB.toString()
            });
            await provider.sendAndConfirm(tx, [bidder]);

            const bidderAtaBAccount = await getAccount(provider.connection, bidderAtaB);
            assert.ok(bidderAtaBAccount.amount == BigInt(1e9));
        }
        // bidder2AtaB
        {
            let tx = new anchor.web3.Transaction();
            tx.instructions = [
                createAssociatedTokenAccountIdempotentInstruction(
                    provider.publicKey, bidder2AtaB, bidder2.publicKey, mintB.publicKey, tokenProgram
                ),
                createMintToInstruction(mintB.publicKey, bidder2AtaB, bidder.publicKey, 1e9, undefined, tokenProgram),
            ]
            console.log("minting bidder2 ATA: ", {
                bidder2: bidder2.publicKey.toString(),
                mintB: mintB.publicKey.toString(),
                bidder2AtaB: bidder2AtaB.toString()
            });
            await provider.sendAndConfirm(tx, [bidder]);

            const bidder2AtaBAccount = await getAccount(provider.connection, bidder2AtaB);
            assert.ok(bidder2AtaBAccount.amount == BigInt(1e9));
        }

        const connection = program.provider.connection;

        const mintAInfo = await connection.getAccountInfo(mintA.publicKey);
        console.log("MintA Account Info:", mintAInfo);

        const mintBInfo = await connection.getAccountInfo(mintB.publicKey);
        console.log("MintB Account Info:", mintAInfo);

        // Ensure the mints are initalized
        if (!mintAInfo || !mintBInfo) {
            throw new Error("Mint accounts are not initialized.")
        }

        const sellerAtaABalance = await connection.getTokenAccountBalance(sellerAtaA);
        console.log("Seller ATA A Balance:", sellerAtaABalance);

        // Ensure the correct amount of tokens was minted
        if (sellerAtaABalance.value.amount !== "1000000000") {
            throw new Error("Incorrect token balance in maker's ATA for mintA.");
        }

    })

    it("initialize house", async () => {
        console.log("initHouse");
        const accounts = {
            admin: admin.publicKey,
            auctionHouse: auction_house,
            systemProgram: SystemProgram.programId
        }

        const tx = await program.methods.initHouse(1, "testAuction")
            .accountsPartial({ ...accounts })
            .signers([admin])
            .rpc();

        console.log("Your transaction signature", tx);
    });

    it("initialize auction", async () => {
        end = new anchor.BN(await provider.connection.getSlot() + 10);

        const [auction_pk] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("auction"),
                auction_house.toBuffer(),
                seller.publicKey.toBuffer(),
                mintA.publicKey.toBuffer(),
                mintB.publicKey.toBuffer(),
            ],
            program.programId,
        );
        auction = auction_pk;
        vault = getAssociatedTokenAddressSync(mintA.publicKey, auction, true, tokenProgram);

        console.log("auction accounts for: ", {
            auction_pk: auction_pk.toString(),
            vault: vault.toString(),
        });
        const accounts = {
            seller: seller.publicKey,
            auctionHouse: auction_house,
            auction: auction,
            mintA: mintA.publicKey,
            mintB: mintB.publicKey,
            sellerAtaA: sellerAtaA,
            vault: vault,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        const tx = await program.methods.initAuction(starting_price, end, amount, 6)
            .accountsPartial({ ...accounts })
            .signers([seller])
            .rpc();
        console.log("Your transaction signature", tx);

        // Check auction state is set to expected values
        let auctionAccount = await program.account.auction.fetch(auction);
        console.log("auction acount: ", auctionAccount);
        assert.ok(auctionAccount.seller.equals(seller.publicKey));
        assert.ok(auctionAccount.mintA.equals(mintA.publicKey));
        assert.ok(auctionAccount.mintB.equals(mintB.publicKey));
        assert.ok(auctionAccount.bidder == null);
        assert.ok(auctionAccount.decimal == 6);
        assert.ok(auctionAccount.highestPrice.eq(starting_price.sub(new anchor.BN(1))));

        // Check the vault token account balance
        const vaultAccount = await getAccount(provider.connection, vault);
        assert.ok(new anchor.BN(vaultAccount.amount.toString()).eq(amount));
    })
    it("bid", async () => {
        const [bidState_pk] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("bid"),
                auction.toBuffer(),
                bidder.publicKey.toBuffer(),
            ],
            program.programId,
        );
        bidState = bidState_pk;
        bidderEscrow = getAssociatedTokenAddressSync(mintB.publicKey, bidState, true, tokenProgram);

        console.log("bid state and escrow accounts for: ", {
            bidState: bidState.toString(),
            bidderEscrow: bidderEscrow.toString(),
        })

        const accounts = {
            bidder: bidder.publicKey,
            mintA: mintA.publicKey,
            mintB: mintB.publicKey,
            auctionHouse: auction_house,
            auction: auction,
            bidderAtaB: bidderAtaB,
            bidderEscrow: bidderEscrow,
            bidState: bidState,
            vault: vault,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        const tx = await program.methods.bid(bidPrice)
            .accountsPartial({ ...accounts })
            .signers([bidder])
            .rpc();

        console.log("Your transaction signature", tx);

        // Check auction state is set to expected values
        let auctionAccount = await program.account.auction.fetch(auction);
        console.log("auction acount: ", auctionAccount);
        assert.ok(auctionAccount.seller.equals(seller.publicKey));
        assert.ok(auctionAccount.mintA.equals(mintA.publicKey));
        assert.ok(auctionAccount.mintB.equals(mintB.publicKey));
        assert.ok(auctionAccount.bidder.equals(bidder.publicKey));
        assert.ok(auctionAccount.decimal == 6);
        assert.ok(auctionAccount.highestPrice.eq(bidPrice));

        // Check the bid state is set to expected values
        const bidStateAccount = await program.account.bidState.fetch(bidState);
        console.log("bid state: {}", bidStateAccount);
        assert.ok(bidStateAccount.auction.equals(auction));
        assert.ok(bidStateAccount.bidder.equals(bidder.publicKey));

        // Check the bidder escrow token account
        const bidderEscrowAccount = await getAccount(provider.connection, bidderEscrow);
        console.log("bidder escrow: {}", bidderEscrowAccount);
        assert.ok(new anchor.BN(bidderEscrowAccount.amount.toString()).eq(bidPrice.mul(amount).div(new anchor.BN(1000000))));
    })
    it("bid2", async () => {
        const [bid2State_pk] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("bid"),
                auction.toBuffer(),
                bidder2.publicKey.toBuffer(),
            ],
            program.programId,
        );
        bid2State = bid2State_pk;
        bidder2Escrow = getAssociatedTokenAddressSync(mintB.publicKey, bid2State, true, tokenProgram);

        console.log("bid state and escrow accounts for: ", {
            bid2State: bid2State.toString(),
            bidder2Escrow: bidder2Escrow.toString(),
        })

        const accounts = {
            bidder: bidder2.publicKey,
            mintA: mintA.publicKey,
            mintB: mintB.publicKey,
            auctionHouse: auction_house,
            auction: auction,
            bidderAtaB: bidder2AtaB,
            bidderEscrow: bidder2Escrow,
            bidState: bid2State,
            vault: vault,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        const tx = await program.methods.bid(bidPrice2)
            .accountsPartial({ ...accounts })
            .signers([bidder2])
            .rpc();

        console.log("Your transaction signature", tx);

        // Check auction state is set to expected values
        let auctionAccount = await program.account.auction.fetch(auction);
        console.log("auction acount: ", auctionAccount);
        assert.ok(auctionAccount.seller.equals(seller.publicKey));
        assert.ok(auctionAccount.mintA.equals(mintA.publicKey));
        assert.ok(auctionAccount.mintB.equals(mintB.publicKey));
        assert.ok(auctionAccount.bidder.equals(bidder2.publicKey));
        assert.ok(auctionAccount.decimal == 6);
        assert.ok(auctionAccount.highestPrice.eq(bidPrice2));

        // Check the bid state is set to expected values
        const bid2StateAccount = await program.account.bidState.fetch(bid2State);
        console.log("bid2 state: {}", bid2StateAccount);
        assert.ok(bid2StateAccount.auction.equals(auction));
        assert.ok(bid2StateAccount.bidder.equals(bidder2.publicKey));

        // Check the bidder escrow token account
        const bidder2EscrowAccount = await getAccount(provider.connection, bidder2Escrow);
        console.log("bidder2 escrow: {}", bidder2EscrowAccount);
        assert.ok(new anchor.BN(bidder2EscrowAccount.amount.toString()).eq(bidPrice2.mul(amount).div(new anchor.BN(1000000))));
    })

    it("finalize", async () => {
        console.log("waiting for auction to end...");
        while (true) {
            const current_slot = new anchor.BN(await provider.connection.getSlot("processed"));
            if (current_slot.gt(end)) {
                break;
            }
        }
        console.log("auction over! finalizing...");
        const accounts = {
            payer: seller.publicKey,
            seller: seller.publicKey,
            bidder: bidder2.publicKey,
            admin: admin.publicKey,
            mintA: mintA.publicKey,
            mintB: mintB.publicKey,
            auctionHouse: auction_house,
            auction: auction,
            bidderAtaA: bidder2AtaA,
            sellerAtaB: sellerAtaB,
            sellerAtaA: sellerAtaA,
            houseMintB: houseAtaB,
            bidState: bid2State,
            bidderEscrow: bidder2Escrow,
            vault: vault,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
        };
        console.log(accounts);

        try {
            let tx = await program.methods.finalize()
                .accountsPartial({ ...accounts })
                .signers([seller])
                .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 })])
                .rpc();
            console.log("Your transaction signature", tx);
        } catch (err) {
            console.log(err);
            throw err;
        }
    })

    it("withdraw", async () => {
        const accounts = {
            bidder: bidder.publicKey,
            mintB: mintB.publicKey,
            auctionHouse: auction_house,
            auction: auction,
            bidderAtaB: bidderAtaB,
            bidderEscrow: bidderEscrow,
            bidState: bidState,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        const tx = await program.methods.withdraw()
            .accountsPartial({ ...accounts })
            .signers([bidder])
            .rpc();

        const bidderAtaBAccount = await getAccount(provider.connection, bidderAtaB);
        assert.ok(bidderAtaBAccount.amount === BigInt(1000000000));
        console.log("Your transaction signature", tx);
    })
});

//test cancel seperately because no success bid

describe("auction cancel", () => {
    let provider = anchor.getProvider();
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Auction as Program<Auction>;

    const admin = Keypair.generate();
    const seller = Keypair.generate();
    const mintA = Keypair.generate();
    const mintB = Keypair.generate();
    const tokenProgram = TOKEN_PROGRAM_ID;
    const sellerAtaA = getAssociatedTokenAddressSync(mintA.publicKey, seller.publicKey, true, tokenProgram);
    const sellerAtaB = getAssociatedTokenAddressSync(mintB.publicKey, seller.publicKey, true, tokenProgram);
    const name = String("testAuctionCancel");
    const [auction_house] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("house"),
            Buffer.from(name),
        ],
        program.programId,
    );
    let end: anchor.BN;
    let auction: PublicKey;
    let vault: PublicKey;

    const starting_price = new anchor.BN(2000000);
    const amount = new anchor.BN(50);

    before("airdrop", async () => {
        console.log("requesting airdrops");
        let lamports = await getMinimumBalanceForRentExemptMint(program.provider.connection);
        {
            let tx = new anchor.web3.Transaction();
            tx.instructions = [
                SystemProgram.transfer({
                    fromPubkey: provider.publicKey,
                    toPubkey: seller.publicKey,
                    lamports: 0.2 * LAMPORTS_PER_SOL,

                }),
                SystemProgram.transfer({
                    fromPubkey: provider.publicKey,
                    toPubkey: admin.publicKey,
                    lamports: 0.2 * LAMPORTS_PER_SOL,

                }),
                SystemProgram.createAccount({
                    fromPubkey: provider.publicKey,
                    newAccountPubkey: mintA.publicKey,
                    lamports,
                    space: MINT_SIZE,
                    programId: tokenProgram,

                }),
                SystemProgram.createAccount({
                    fromPubkey: provider.publicKey,
                    newAccountPubkey: mintB.publicKey,
                    lamports,
                    space: MINT_SIZE,
                    programId: tokenProgram,

                })
            ];
            console.log("airdropping for: ", {
                seller: seller.publicKey.toString(),
                mintA: mintA.publicKey.toString(),
                mintB: mintB.publicKey.toString(),
            });
            await provider.sendAndConfirm(tx, [mintA, mintB]);
        }

        {
            let tx = new anchor.web3.Transaction();
            tx.instructions = [
                createInitializeMint2Instruction(mintA.publicKey, 6, seller.publicKey, null, tokenProgram),
                createAssociatedTokenAccountIdempotentInstruction(
                    provider.publicKey, sellerAtaA, seller.publicKey, mintA.publicKey, tokenProgram
                ),
                createMintToInstruction(mintA.publicKey, sellerAtaA, seller.publicKey, 1e9, undefined, tokenProgram)
            ];
            console.log("creating mintA, minting seller ATA: ", {
                seller: seller.publicKey.toString(),
                mintA: mintA.publicKey.toString(),
                sellerAtaA: sellerAtaA.toString(),
            });
            await provider.sendAndConfirm(tx, [seller]);
        }

        {
            let tx = new anchor.web3.Transaction();
            tx.instructions = [
                createInitializeMint2Instruction(mintB.publicKey, 6, seller.publicKey, null, tokenProgram),
                createAssociatedTokenAccountIdempotentInstruction(
                    provider.publicKey, sellerAtaB, seller.publicKey, mintB.publicKey, tokenProgram
                ),
                createMintToInstruction(mintB.publicKey, sellerAtaB, seller.publicKey, 1e9, undefined, tokenProgram)
            ];
            console.log("creating mintB, minting seller ATA: ", {
                seller: seller.publicKey.toString(),
                mintB: mintB.publicKey.toString(),
                sellerAtaB: sellerAtaB.toString(),
            });
            await provider.sendAndConfirm(tx, [seller]);
        }

        const connection = program.provider.connection;

        const mintAInfo = await connection.getAccountInfo(mintA.publicKey);
        console.log("MintA Account Info:", mintAInfo);

        const mintBInfo = await connection.getAccountInfo(mintB.publicKey);
        console.log("MintB Account Info:", mintAInfo);

        // Ensure the mints are initalized
        if (!mintAInfo || !mintBInfo) {
            throw new Error("Mint accounts are not initialized.")
        }

        const sellerAtaABalance = await connection.getTokenAccountBalance(sellerAtaA);
        console.log("Seller ATA A Balance:", sellerAtaABalance);

        // Ensure the correct amount of tokens was minted
        if (sellerAtaABalance.value.amount !== "1000000000") {
            throw new Error("Incorrect token balance in maker's ATA for mintA.");
        }

    })

    it("initialize house", async () => {
        console.log("initHouse");
        const accounts = {
            admin: admin.publicKey,
            auctionHouse: auction_house,
            systemProgram: SystemProgram.programId
        }

        const tx = await program.methods.initHouse(1, name)
            .accountsPartial({ ...accounts })
            .signers([admin])
            .rpc();

        console.log("Your transaction signature", tx);
    });

    it("initialize auction", async () => {
        end = new anchor.BN(await provider.connection.getSlot() + 10);

        const [auction_pk] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("auction"),
                auction_house.toBuffer(),
                seller.publicKey.toBuffer(),
                mintA.publicKey.toBuffer(),
                mintB.publicKey.toBuffer(),
            ],
            program.programId,
        );
        auction = auction_pk;
        vault = getAssociatedTokenAddressSync(mintA.publicKey, auction, true, tokenProgram);

        console.log("auction accounts for: ", {
            auction_pk: auction_pk.toString(),
            vault: vault.toString(),
        });
        const accounts = {
            seller: seller.publicKey,
            auctionHouse: auction_house,
            auction: auction,
            mintA: mintA.publicKey,
            mintB: mintB.publicKey,
            sellerAtaA: sellerAtaA,
            vault: vault,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        const tx = await program.methods.initAuction(starting_price, end, amount, 6)
            .accountsPartial({ ...accounts })
            .signers([seller])
            .rpc();
        console.log("Your transaction signature", tx);

        // Check auction state is set to expected values
        let auctionAccount = await program.account.auction.fetch(auction);
        console.log("auction acount: ", auctionAccount);
        assert.ok(auctionAccount.seller.equals(seller.publicKey));
        assert.ok(auctionAccount.mintA.equals(mintA.publicKey));
        assert.ok(auctionAccount.mintB.equals(mintB.publicKey));
        assert.ok(auctionAccount.bidder == null);
        assert.ok(auctionAccount.decimal == 6);
        assert.ok(auctionAccount.highestPrice.eq(starting_price.sub(new anchor.BN(1))));

        // Check the vault token account balance
        const vaultAccount = await getAccount(provider.connection, vault);
        assert.ok(new anchor.BN(vaultAccount.amount.toString()).eq(amount));
    })
    it("cancel", async () => {
        console.log("waiting for auction to end...");
        while (true) {
            const current_slot = new anchor.BN(await provider.connection.getSlot("processed"));
            if (current_slot.gt(end)) {
                break;
            }
        }
        console.log("auction over! canceling...");
        const accounts = {
            payer: seller.publicKey,
            seller: seller.publicKey,
            admin: admin.publicKey,
            mintA: mintA.publicKey,
            mintB: mintB.publicKey,
            auctionHouse: auction_house,
            auction: auction,
            sellerAtaA: sellerAtaA,
            vault: vault,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
        };
        console.log(accounts);

        try {
            let tx = await program.methods.cancel()
                .accountsPartial({ ...accounts })
                .signers([seller])
                .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 })])
                .rpc();
            console.log("Your transaction signature", tx);
        } catch (err) {
            console.log(err);
            throw err;
        }

        const sellerAtaABalance = await program.provider.connection.getTokenAccountBalance(sellerAtaA);
        console.log("Seller ATA A Balance:", sellerAtaABalance);

        try {
            await program.account.auction.fetch(auction)
            throw new Error("Auction account was expected to be closed, but it still exists.");
        } catch (error) {
            expect(error).to.exist;
        };
        try {
            await getAccount(provider.connection, vault);
            throw new Error("Vault account was expected to be closed, but it still exists.");
        } catch (error) {
            expect(error).to.exist;
        };
    })
});
