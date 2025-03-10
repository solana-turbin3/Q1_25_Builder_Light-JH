import { Keypair, Connection, Commitment } from "@solana/web3.js";
import { createMint, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import wallet from "../../../../devnet.json"

// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

//Create a Solana devnet connection
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

(async () => {
    try {
        const mint = await createMint(connection, keypair, keypair.publicKey, keypair.publicKey,
            2, undefined, {}, TOKEN_PROGRAM_ID);
        console.log(`mint account: ${mint}`)
    } catch (error) {
        console.log(`Oops, something went wrong: ${error}`)
    }
})()

