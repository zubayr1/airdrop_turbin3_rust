use solana_sdk::{
    bs58,
    message::Message,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_instruction, system_program,
    transaction::Transaction,
};

use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
// use solana_program::{pubkey::Pubkey, system_instruction::transfer};

use std::{
    fs,
    io::{self, BufRead},
    str::FromStr,
};

mod programs;

use crate::programs::Turbin3_prereq::{CompleteArgs, UpdateArgs, Turbin3PrereqProgram};

#[derive(Deserialize)]
struct DevWallet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pubkey: Option<String>,
    private_key: Vec<u8>,
}

const RPC_URL: &str = "https://api.devnet.solana.com";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as base58:");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your wallet file is:");
        let wallet = bs58::decode(base58).into_vec().unwrap();
        println!("{:?}", wallet);
    }

    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a wallet file byte array:");
        let stdin = io::stdin();
        let wallet = stdin
            .lock()
            .lines()
            .next()
            .unwrap()
            .unwrap()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();

        println!("Your private key is:");
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }

    #[test]
    fn keygen() {
        // Create a new keypair
        let kp = Keypair::new();

        println!(
            "You've generated a new Solana wallet: {}",
            kp.pubkey().to_string()
        );
        println!();

        // instructions to save the keypair
        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }

    #[test]
    fn airdrop() {
        let json_data = fs::read_to_string("dev-wallet.json").expect("Couldn't find wallet file");
        let wallet: DevWallet =
            serde_json::from_str(&json_data).expect("Couldn't parse wallet file");

        let keypair = Keypair::from_bytes(&wallet.private_key).expect("Invalid private key");

        println!("Airdropping to {}", keypair.pubkey().to_string());

        // Connected to Solana Devnet RPC Client
        let client = RpcClient::new(RPC_URL);

        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(s) => {
                println!("Success! Check out your TX here:");

                println!(
                    "https://explorer.solana.com/tx/{}?cluster=devnet",
                    s.to_string()
                );
            }

            Err(e) => println!("Oops, something went wrong: {}", e.to_string()),
        };
    }

    #[test]
    fn get_devnet_balance() {
        let client = RpcClient::new(RPC_URL.to_string());

        let json_data = fs::read_to_string("dev-wallet.json").expect("Couldn't find wallet file");
        let wallet: DevWallet =
            serde_json::from_str(&json_data).expect("Couldn't parse wallet file");

        let keypair = Keypair::from_bytes(&wallet.private_key).expect("Invalid private key");

        let input_pubkey = keypair.pubkey().to_string();
        let pubkey = Pubkey::from_str(&input_pubkey).expect("Invalid public key format");

        // Get the balance
        match client.get_balance(&pubkey) {
            Ok(balance) => {
                println!("The balance for {} is: {} lamports", pubkey, balance);
            }
            Err(err) => {
                println!("Failed to fetch balance: {}", err);
            }
        }
    }

    #[test]
    fn transfer() {
        let json_data = fs::read_to_string("dev-wallet.json").expect("Couldn't find wallet file");
        let wallet: DevWallet =
            serde_json::from_str(&json_data).expect("Couldn't parse wallet file");

        let keypair = Keypair::from_bytes(&wallet.private_key).expect("Invalid private key");

        // Define our Turbin3 public key
        let to_pubkey = Pubkey::from_str("DYD14Q9hked8pWxFHPLpJdTcXpSX4S4in9FYd82nnfFE").unwrap();

        let rpc_client = RpcClient::new(RPC_URL);

        // Get recent blockhash
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        // Create a transfer instruction
        let transfer_instruction =
            system_instruction::transfer(&keypair.pubkey(), &to_pubkey, 1_000_000);

        // Create a transaction
        let transaction = Transaction::new_signed_with_payer(
            &[transfer_instruction],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        // Send the transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn rest_amount() {
        let json_data = fs::read_to_string("dev-wallet.json").expect("Couldn't find wallet file");
        let wallet: DevWallet =
            serde_json::from_str(&json_data).expect("Couldn't parse wallet file");

        let keypair = Keypair::from_bytes(&wallet.private_key).expect("Invalid private key");

        // Define our Turbin3 public key
        let to_pubkey = Pubkey::from_str("DYD14Q9hked8pWxFHPLpJdTcXpSX4S4in9FYd82nnfFE").unwrap();

        let rpc_client = RpcClient::new(RPC_URL);

        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");

        println!("Current devnet balance: {}", balance);

        // Get recent blockhash
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        // Create a transfer instruction
        let mut transfer_instruction =
            system_instruction::transfer(&keypair.pubkey(), &to_pubkey, balance);

        // Create a test transaction to calculate fees
        let message = Message::new_with_blockhash(
            &[transfer_instruction],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );

        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");

        println!("Fee: {}", fee);

        transfer_instruction =
            system_instruction::transfer(&keypair.pubkey(), &to_pubkey, balance - fee);

        // Create a transaction
        let transaction = Transaction::new_signed_with_payer(
            &[transfer_instruction],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        // Send the transaction
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
        let rpc_client = RpcClient::new(RPC_URL);

        // Add better error handling for the wallet file
        let json_data = fs::read_to_string("turbin3-wallet.json")
            .expect("Error reading turbin3-wallet.json - make sure the file exists");

        println!("Wallet file contents: {}", json_data); // Debug print

        let wallet: DevWallet = serde_json::from_str(&json_data)
            .expect("Error parsing wallet JSON - check the file format");

        let signer =
            Keypair::from_bytes(&wallet.private_key).expect("Invalid private key in wallet file");

        // creating PDA
        let prereq = Turbin3PrereqProgram::derive_program_address(&[
            b"prereq",
            signer.pubkey().to_bytes().as_ref(),
        ]);

        println!("Prereq PDA: {}", prereq);

        // Define our instruction data
        let args = CompleteArgs {
            github: b"zubayr1".to_vec(),
        };

        // Get recent blockhash
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        // Create a transaction
        let transaction = Turbin3PrereqProgram::complete(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],
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
    fn update() {
        let rpc_client = RpcClient::new(RPC_URL);

        // Add better error handling for the wallet file
        let json_data = fs::read_to_string("turbin3-wallet.json")
            .expect("Error reading turbin3-wallet.json - make sure the file exists");

        println!("Wallet file contents: {}", json_data); // Debug print

        let wallet: DevWallet = serde_json::from_str(&json_data)
            .expect("Error parsing wallet JSON - check the file format");

        let signer =
            Keypair::from_bytes(&wallet.private_key).expect("Invalid private key in wallet file");

        // creating PDA
        let prereq = Turbin3PrereqProgram::derive_program_address(&[
            b"prereq",
            signer.pubkey().to_bytes().as_ref(),
        ]);

        println!("Prereq PDA: {}", prereq);

        // Define our instruction data
        let args = UpdateArgs {
            github: b"zubayr1".to_vec(),
        };

        // Get recent blockhash
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        // Create a transaction
        let transaction = Turbin3PrereqProgram::update(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],
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
}
