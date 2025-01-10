mod programs;

#[cfg(test)]
mod tests {
    use crate::programs::turbin3_prereq::{CompleteArgs, TurbinePrereqProgram};
    use solana_sdk::signature::{Keypair, Signer};
    #[test]
    fn keygen() {
        let kp = Keypair::new();
        println!(
            "You've generated a new Solana wallet: {}",
            kp.pubkey().to_string()
        );
        println!("");
        println!("To save your wallet, copy and paste the folling into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }

    #[test]
    fn airdop() {
        use solana_client::rpc_client::RpcClient;
        use solana_sdk::signature::{read_keypair_file, Signer};

        const PRC_URL: &str = "https://api.devnet.solana.com/";
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");

        let client = RpcClient::new(PRC_URL);

        match client.request_airdrop(&keypair.pubkey(), 2000000000) {
            Ok(s) => {
                println!("Success! Check out your TX here:");
                println!(
                    "https://explorer.solana.com/tx/{}?cluster=devenet",
                    s.to_string()
                );
            }
            Err(e) => println!("Oops, something went wrong: {}", e.to_string()),
        }
    }

    #[test]
    fn transfer_sol() {
        use solana_client::rpc_client::RpcClient;
        use solana_program::system_instruction::transfer;
        use solana_sdk::{
            message::Message,
            signature::{read_keypair_file, Signer},
            transaction::Transaction,
        };

        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        let turbin_key = read_keypair_file("../../devnet.json").unwrap();
        let to_pubkey = turbin_key.pubkey();

        const PRC_URL: &str = "https://api.devnet.solana.com/";
        let rpc_client = RpcClient::new(PRC_URL);

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");

        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );

        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }
    #[test]
    fn enroll() {
        use solana_client::rpc_client::RpcClient;
        use solana_program::system_program;
        use solana_sdk::signature::{read_keypair_file, Signer};

        const PRC_URL: &str = "https://api.devnet.solana.com/";
        let rpc_client = RpcClient::new(PRC_URL);

        let signer = read_keypair_file("../../devnet.json").expect("Couldn't find wallet file");

        let prereq = TurbinePrereqProgram::derive_program_address(&[
            b"prereq",
            signer.pubkey().to_bytes().as_ref(),
        ]);
        let args = CompleteArgs {
            github: b"Light-JH".to_vec(),
        };
        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let transaction = TurbinePrereqProgram::complete(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],
            blockhash,
        );
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here:https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }
}
