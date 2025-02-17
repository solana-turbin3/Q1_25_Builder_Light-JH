import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Auction } from "../target/types/auction";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram, PublicKey } from "@solana/web3.js";
import { randomBytes } from 'crypto';

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
import { assert } from "chai";

describe("auction", () => {

    // Configure the client to use the local cluster.
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
    let bidState: PublicKey;


    // const [bid_state] = PublicKey.findProgramAddressSync(
    //     [
    //         Buffer.from("bid"),
    //         auction.toBuffer(),
    //         bidder.publicKey.toBuffer(),
    //     ],
    //     program.programId,
    // );

    // const vault = getAssociatedTokenAddressSync(mintA.publicKey, auction, true, tokenProgram);
    // const bidder_escrow = getAssociatedTokenAddressSync(mintB.publicKey, bid_state, true, tokenProgram);
    // const seed = new anchor.BN(randomBytes(8));
    const starting_price = new anchor.BN(2000000);
    const amount = new anchor.BN(50);
    const bidPrice = new anchor.BN(3000000);

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
        const connection = program.provider.connection;

        const mintAInfo = await connection.getAccountInfo(mintA.publicKey);
        console.log("MintA Account Info:", mintAInfo);

        const mintBInfo = await connection.getAccountInfo(mintA.publicKey);
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
        end = new anchor.BN(await provider.connection.getSlot() + 20);

        const [auction_pk] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("auction"),
                auction_house.toBuffer(),
                seller.publicKey.toBuffer(),
                mintA.publicKey.toBuffer(),
                mintB.publicKey.toBuffer(),
                end.toArrayLike(Buffer, 'le', 8)
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
        const bidState = bidState_pk;
        const bidderEscrow = getAssociatedTokenAddressSync(mintB.publicKey, bidState, true, tokenProgram);

        console.log("bid state and escrow accounts for: ", {
            bidState: bidState.toString(),
            bidderEscrow: bidderEscrow.toString(),
        })

        const accounts = {
            bidder: bidder.publicKey,
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
        const tx = await program.methods.bid(bidPrice, 6)
            .accountsPartial({ ...accounts })
            .signers([bidder])
            .rpc();

        console.log("Your transaction signature", tx);

        // const bidderEscrowAccount = await getAccount(provider.connection, bidderEscrow);
        // assert.ok(new anchor.BN(bidderEscrowAccount.amount.toString()).eq(new anchor.BN(bidPrice * amount / 1000000)));
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
            bidder: bidder.publicKey,
            admin: admin.publicKey,
            mintA: mintA.publicKey,
            mintB: mintB.publicKey,
            auctionHouse: auction_house,
            auction: auction,
            bidderAtaA: bidderAtaA,
            sellerAtaB: sellerAtaB,
            houseMintB: houseAtaB,
            bidderEscrow: bidderEscrow,
            bidState: bidState,
            vault: vault,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        }
        const tx = await program.methods.finalize()
            .accountsPartial({ ...accounts })
            .signers([seller])
            .rpc();

        console.log("Your transaction signature", tx);
    })

    // it("withdraw", async () => {
    //     end = new anchor.BN(await provider.connection.getSlot() + 20);
    //     const accounts = {
    //         bidder: bidder.publicKey,
    //         mintB: mintB.publicKey,
    //         auctionHouse: auction_house,
    //         auction: auction,
    //         bidderAtaB: bidderAtaB,
    //         bidderEscrow: bidder_escrow,
    //         bidState: bid_state,
    //         systemProgram: SystemProgram.programId,
    //         tokenProgram: TOKEN_PROGRAM_ID,
    //         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    //     }
    //     const tx = await program.methods.withdraw()
    //         .accountsPartial({ ...accounts })
    //         .signers([bidder])
    //         .rpc();

    //     console.log("Your transaction signature", tx);
    // })
});
