import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Auction } from "../target/types/auction";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram, PublicKey } from "@solana/web3.js";
import { randomBytes } from 'crypto';

import {
    TOKEN_PROGRAM_ID,
    getAssociatedTokenAddressSync,
    getMinimumBalanceForRentExemptMint,
    MINT_SIZE,
    createInitializeMint2Instruction,
    createAssociatedTokenAccountIdempotentInstruction,
    createMintToInstruction


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
    const name = String("testAuction");
    const [auction_house] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("house"),
            Buffer.from("testAuction"),
        ],
        program.programId,
    );
    let end: anchor.BN;
    // let end = new anchor.BN(await provider.connection.getSlot() + 20);
    // const [auction] = PublicKey.findProgramAddressSync(
    //     [
    //         Buffer.from("auction"),
    //         auction_house.toBuffer(),
    //         seller.publicKey.toBuffer(),
    //         mintA.publicKey.toBuffer(),
    //         mintB.publicKey.toBuffer(),
    //         end.toArrayLike(Buffer, 'le', 8)
    //     ],
    //     program.programId,
    // );

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

    })
    it("follow up", async () => {
        console.log("end still = ", end);
    })
});
