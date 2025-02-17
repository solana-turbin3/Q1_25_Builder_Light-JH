import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import { LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getMinimumBalanceForRentExemptMint,
} from "@solana/spl-token";
import { randomBytes } from "crypto";

describe("escrow", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const connection = provider.connection;

  const program = anchor.workspace.Escrow as Program<Escrow>;

  const tokenProgram = TOKEN_2022_PROGRAM_ID;

  const confirm = async (signature: string): Promise<string> => {
    const block = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
      signature,
      ...block,
    });
    return signature;
  };

  const log = async (signature: string): Promise<string> => {
    console.log(
      `Your transaction signature: https://explorer.solana.com/transaction/${signature}?cluster=custom&customUrl=${connection.rpcEndpoint}`
    );
    return signature;
  };


  const seed = new anchor.BN(randomBytes(8));
  const [maker, taker, mintA, mintB] = Array.from({ length: 4 }, () => Keypair.generate())
  const [makerAtaA, makerAtaB, takerAtaA, takerAtaB] = [maker, taker]
    .map((a) => [mintA, mintB].map((m) =>
      getAssociatedTokenAddressSync(m.publicKey, a.publicKey, false, tokenProgram))).flat(1);

  const escrow = PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), maker.publicKey.toBuffer(), seed.toArrayLike(Buffer, "le", 8)],
    program.programId
  )[0];


  const vault = getAssociatedTokenAddressSync(mintA.publicKey, escrow, true, tokenProgram);

  const accounts = {
    maker: maker.publicKey,
    taker: taker.publicKey,
    mintA: mintA.publicKey,
    mintB: mintB.publicKey,
    takerAtaA,
    takerAtaB,
    makerAtaA,
    makerAtaB,
    escrow,
    vault,
    tokenProgram,
  }

  it("Airdrop and create mints", async () => {
    await provider.connection.requestAirdrop(maker.publicKey, 10 * LAMPORTS_PER_SOL);
    let lamports = await getMinimumBalanceForRentExemptMint(connection);
    const tx = await program.methods.make({ seed: 1, receive: 2, deposit: 3 }).accountsPartial(accounts).signers([maker]).rpc();
    console.log("Your transaction signature", tx);
  });
});
