import {
  Connection,
  Keypair,
  SystemProgram,
  PublicKey,
  Commitment,
} from "@solana/web3.js";
import { Program, Wallet, AnchorProvider, Address } from "@coral-xyz/anchor";
import { WbaVault, IDL } from "./programs/wba_vault";
import wallet from "../../../../devnet.json";


// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

// Commitment
const commitment: Commitment = "confirmed";

// Create a devnet connection
const connection = new Connection("https://api.devnet.solana.com");

// Create our anchor provider
const provider = new AnchorProvider(connection, new Wallet(keypair), {
  commitment,
});

// Create our program
const program = new Program<WbaVault>(
  IDL,
  "D51uEDHLbWAxNfodfQDv7qkp8WZtxrhi3uganGbNos7o" as Address,
  provider,
);

// Create a random keypair
//G3qUbdGqv3AGzjhHfwKRWUJ2CdQPn5GWsvVS8VU8XLN
const vaultState = Keypair.generate();
console.log(`Vault public key: ${vaultState.publicKey.toBase58()}`);

// Create the PDA for our enrollment account
// Seeds are "auth", vaultState
// const enrollment_seeds = [Buffer.from("prereq"), keypair.publicKey.toBuffer()];
// const [enrollment_key, _bump] = PublicKey.findProgramAddressSync(enrollment_seeds, program.programId);

const auth_seeds = [Buffer.from("auth"), vaultState.publicKey.toBuffer()];
const [vaultAuth, vaultAuthBump] = PublicKey.findProgramAddressSync(auth_seeds, program.programId);

// Create the vault key
// Seeds are "vault", vaultAuth
const vault_seeds = [Buffer.from("vault"), vaultAuth.toBuffer()];
const [vault, vaultBump] = PublicKey.findProgramAddressSync(vault_seeds, program.programId);

// Execute our enrollment transaction
(async () => {
  try {
    const signature = await program.methods.initialize()
      .accounts({

        owner: keypair.publicKey,
        vaultState: vaultState.publicKey,
        vaultAuth: vaultAuth,
        vault: vault,
        systemProgram: SystemProgram.programId,

      }).signers([keypair, vaultState]).rpc();
    console.log(`Init success! Check out your TX here:\n\nhttps://explorer.solana.com/tx/${signature}?cluster=devnet`);
  } catch (e) {
    console.error(`Oops, something went wrong: ${e}`);
  }
})();
