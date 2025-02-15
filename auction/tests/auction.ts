import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Auction } from "../target/types/auction";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";
import { Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import { randomBytes } from 'crypto';

import {
    TOKEN_PROGRAM_ID,
    createMint,
    createAccount,
    mintTo,
    getAccount,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getMint
} from "@solana/spl-token";
import { assert } from "chai";

describe("auction", () => {
    // Configure the client to use the local cluster.
    let provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.Auction as Program<Auction>;

    let mintA: anchor.web3.PublicKey;
    let mintB: anchor.web3.PublicKey;
    let sellerAtaA: anchor.web3.PublicKey;
    let bidderAtaA: anchor.web3.PublicKey;
    let sellerAtaB: anchor.web3.PublicKey;
    let bidderAtaB: anchor.web3.PublicKey;
    let auction: anchor.web3.PublicKey;
    let vault: anchor.web3.PublicKey;
    let bidder_escrow: anchor.web3.PublicKey;
    let bid_state: anchor.web3.PublicKey;
    let auction_house: anchor.web3.PublicKey;


    const seller = Keypair.generate();
    const bidder = Keypair.generate();


    const name = String("testAuction");

    // anchor.BN() to represent numbers safely and precisely when interacting with solana programs
    //Solana and many other blockchains use 64-bit or even 128-bit integers, which can exceed JavaScript's safe range.
    const seed = new anchor.BN(randomBytes(8));
    const depositAmount = new anchor.BN(50);

    before(async () => {
        const sellerAirdrop = await provider.connection.requestAirdrop(seller.publicKey, 10 * LAMPORTS_PER_SOL);
        const bidderAirdrop = await provider.connection.requestAirdrop(bidder.publicKey, 10 * LAMPORTS_PER_SOL);

        const lastestBlockhash = await provider.connection.getLatestBlockhash();
        await provider.connection.confirmTransaction({
            signature: sellerAirdrop,
            blockhash: lastestBlockhash.blockhash,
            lastValidBlockHeight: lastestBlockhash.lastValidBlockHeight,
        });


        await provider.connection.confirmTransaction({
            signature: bidderAirdrop,
            blockhash: lastestBlockhash.blockhash,
            lastValidBlockHeight: lastestBlockhash.lastValidBlockHeight,
        });

        mintA = await createMint(provider.connection, seller, seller.publicKey, null, 6);
        mintB = await createMint(provider.connection, bidder, bidder.publicKey, null, 6);

        sellerAtaA = await createAccount(provider.connection, seller, mintA, seller.publicKey);
        bidderAtaA = await createAccount(provider.connection, bidder, mintA, bidder.publicKey);
        sellerAtaB = await createAccount(provider.connection, seller, mintB, seller.publicKey);
        bidderAtaB = await createAccount(provider.connection, bidder, mintB, bidder.publicKey);

        await mintTo(provider.connection, seller, mintA, sellerAtaA, seller, 100000000);
        await mintTo(provider.connection, bidder, mintB, bidderAtaB, bidder, 100000000);

        [auction] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("auction"), auction_house.toBuffer(), seller.publicKey.toBuffer(), mintA.toBuffer(), mintB.toBuffer(), end.toBuffer('le', 8)],
            program.programId,
        );

        [auction_house] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("house"), Buffer.from(name)],
            program.programId,
        );

        [bid_state] = anchor.web3.PublicKey.findProgramAddressSync(
            [Buffer.from("auction"), auction.toBuffer(), bidder.publicKey.toBuffer()],
            program.programId,
        );


        vault = await anchor.utils.token.associatedAddress({
            mint: mintA,
            owner: auction
        });
        bidder_escrow = await anchor.utils.token.associatedAddress({
            mint: mintB,
            owner: bid_state,
        })

        let end = await provider.connection.getSlot() + 20;

    })

    it("initialize house", async () => {
        // Add your test here.
        const tx = await program.methods.initHouse(2000, name).rpc();
        console.log("Your transaction signature", tx);
    });
});
