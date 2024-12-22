use helius::types::*;
use helius::Helius;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::{bs58, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, system_instruction};
use std::{str::FromStr, time::Duration};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        let api_key: &str = "your_api_key";
        let cluster: Cluster = Cluster::MainnetBeta;
        let helius: Helius = Helius::new(api_key, cluster).unwrap();

        // Convert your base58 private key to a seed
        let keypair_base58 = "your_keypair_as_base58";
        let keypair_bytes = bs58::decode(keypair_base58).into_vec().unwrap();

        // Create the recipient address
        let to_pubkey: Pubkey = Pubkey::from_str("recipient_address").unwrap();

        // Get the sender's public key for balance checking
        let from_pubkey: Pubkey = Keypair::from_bytes(&keypair_bytes).unwrap().pubkey();

        println!("From wallet address: {}", from_pubkey);
        println!("To wallet address: {}", to_pubkey);

        // Get initial balances
        let balance_from: u64 = helius.connection().get_balance(&from_pubkey).unwrap_or(0);
        let balance_to: u64 = helius.connection().get_balance(&to_pubkey).unwrap_or(0);

        println!(
            "From wallet balance: {} SOL",
            balance_from as f64 / LAMPORTS_PER_SOL as f64
        );
        println!("To wallet balance: {} SOL", balance_to as f64 / LAMPORTS_PER_SOL as f64);

        // Create the transfer instruction
        let transfer_amount: u64 = (0.01 * LAMPORTS_PER_SOL as f64) as u64;
        let instruction: solana_sdk::instruction::Instruction =
            system_instruction::transfer(&from_pubkey, &to_pubkey, transfer_amount);

        // Convert keypair bytes to a 32-byte seed array
        let mut seed: [u8; 32] = [0u8; 32];
        seed.copy_from_slice(&keypair_bytes[..32]);

        // For testing purposes. In a production setting, you'd actually create or pass in an existing ATL
        let address_lut: Vec<solana_sdk::address_lookup_table::AddressLookupTableAccount> = vec![];

        // Configure the smart transaction
        let config: CreateSmartTransactionSeedConfig = CreateSmartTransactionSeedConfig {
            instructions: vec![instruction],
            signer_seeds: vec![seed],
            fee_payer_seed: None,
            lookup_tables: Some(address_lut),
            priority_fee_cap: Some(100000),
        };

        // Configure send options (optional)
        let send_options: Option<RpcSendTransactionConfig> = Some(RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: None,
            encoding: None,
            max_retries: None,
            min_context_slot: None,
        });

        // Set a timeout (optional)
        let timeout: Option<Timeout> = Some(Timeout {
            duration: Duration::from_secs(60),
        });

        // Send the transaction
        match helius
            .send_smart_transaction_with_seeds(config, send_options, timeout)
            .await
        {
            Ok(signature) => {
                println!("Transaction sent successfully: {}", signature);
                sleep(Duration::from_secs(5)).await;

                // Get final balances
                let balance_from: u64 = helius.connection().get_balance(&from_pubkey).unwrap_or(0);
                println!(
                    "Final From Wallet Balance: {} SOL",
                    balance_from as f64 / LAMPORTS_PER_SOL as f64
                );
                let balance_to: u64 = helius.connection().get_balance(&to_pubkey).unwrap_or(0);
                println!(
                    "Final To Wallet Balance: {} SOL",
                    balance_to as f64 / LAMPORTS_PER_SOL as f64
                );
            }
            Err(e) => {
                eprintln!("Failed to send transaction: {:?}", e);
            }
        }
    })
    .await
    .unwrap();
}
