use helius::types::*;
use helius::Helius;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{
    instruction::Instruction, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_instruction::transfer,
};
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let api_key: &str = "YOUR_API_KEY";
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Replace with your actual keypair
    let from_keypair: Keypair = Keypair::new();
    let from_pubkey: Pubkey = from_keypair.pubkey();

    // Replace with the recipient's public key
    let to_pubkey: Pubkey = Pubkey::from_str("RecipientPublicKeyHere").unwrap();

    // Create a simple instruction (transfer 0.01 SOL from from_pubkey to to_pubkey)
    let transfer_amount: u64 = 100_000; // 0.01 SOL in lamports
    let instructions: Vec<Instruction> = vec![transfer(&from_pubkey, &to_pubkey, transfer_amount)];

    let create_config: CreateSmartTransactionConfig = CreateSmartTransactionConfig {
        instructions,
        signers: vec![&from_keypair],
        lookup_tables: None,
        fee_payer: None,
        priority_fee: None,
    };

    let config: SmartTransactionConfig = SmartTransactionConfig {
        create_config,
        send_options: RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: None,
            encoding: None,
            max_retries: None,
            min_context_slot: None,
        },
        timeout: Timeout::default(),
    };

    // Send the optimized transaction with a 10k lamport tip using the New York region's API URL
    match helius
        .send_smart_transaction_with_tip(config, Some(10000), Some("NY"))
        .await
    {
        Ok(bundle_id) => {
            println!("Transaction sent successfully: {}", bundle_id);
            sleep(Duration::from_secs(5)).await;

            // Get final balances
            let balance_from = helius.connection().get_balance(&from_pubkey).unwrap_or(0);
            println!(
                "From Wallet Balance: {} SOL",
                balance_from as f64 / LAMPORTS_PER_SOL as f64
            );

            let balance_to = helius.connection().get_balance(&to_pubkey).unwrap_or(0);
            println!("To Wallet Balance: {} SOL", balance_to as f64 / LAMPORTS_PER_SOL as f64);
        }
        Err(e) => {
            eprintln!("Failed to send transaction: {:?}", e);
        }
    }
}
