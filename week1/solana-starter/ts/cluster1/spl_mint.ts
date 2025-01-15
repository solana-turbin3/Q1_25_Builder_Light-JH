import { Keypair, PublicKey, Connection, Commitment } from "@solana/web3.js";
import { ASSOCIATED_TOKEN_PROGRAM_ID, createAssociatedTokenAccount, getOrCreateAssociatedTokenAccount, mintTo, mintToChecked, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import wallet from "../../../../devnet.json"

// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

//Create a Solana devnet connection
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

const token_decimals = 1_000_000n;

// Mint address
const mint = new PublicKey("7rpjuQQRGYeC4SDuDTcMMTwd7r7jQuUpdPNUnVeAbLkp");

(async () => {
    try {
        // Create an ATA

        const ata = await createAssociatedTokenAccount(connection, keypair, mint, keypair.publicKey, {},
            TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID);
        console.log(`Your ata is: ${ata.toBase58()}`);

        // Mint to ATA
        //4an1meRt2LtBCALeBxbuUQRRr6zap4A4bZoJKSctgAeT
        const mintTx = await mintToChecked(connection, keypair, mint, ata, keypair.publicKey, 120000000000, 2);
        console.log(`Your mint txid: ${mintTx}`);
    } catch (error) {
        console.log(`Oops, something went wrong: ${error}`)
    }
})()
