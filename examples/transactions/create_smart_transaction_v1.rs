use helius::types::*;
use helius::Helius;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::{native_token::LAMPORTS_PER_SOL, pubkey::Pubkey};
use solana_system_interface::instruction as system_instruction;
use std::sync::Arc;

/// Builds and sends a **Transaction v1** (SIMD-0385) smart transaction.
///
/// v1 unlocks larger transactions (up to 4,096 bytes, SIMD-0296). Compared to legacy/v0 it:
/// - carries the compute-unit limit and **total** priority fee (in lamports) in the message
///   header config instead of `ComputeBudget` instructions, and
/// - does **not** support address lookup tables.
///
/// Opt in with `version: TransactionVersion::V1`; everything else about smart transactions
/// (priority-fee estimation, compute-unit optimization, retries) works the same.
#[tokio::main]
async fn main() {
    let api_key: &str = "your_api_key";
    let helius: Helius = Helius::new(api_key, Cluster::MainnetBeta).unwrap();

    // Replace with your funded keypair.
    let payer: Keypair = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());
    let recipient: Pubkey = Pubkey::new_unique();

    let create_config: CreateSmartTransactionConfig = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &payer.pubkey(),
            &recipient,
            LAMPORTS_PER_SOL / 100,
        )],
        signers: vec![payer_signer],
        // Opt into Transaction v1.
        version: TransactionVersion::V1,
        // Optional: cap the absolute priority fee. v1 pays a flat lamport amount rather than a
        // per-compute-unit rate, so this caps total spend.
        priority_fee_lamports_cap: Some(1_000_000),
        ..Default::default()
    };

    let config: SmartTransactionConfig = SmartTransactionConfig {
        create_config,
        send_options: RpcSendTransactionConfig::default(),
        timeout: Timeout::default(),
    };

    match helius.send_smart_transaction(config).await {
        Ok(signature) => println!("Sent Transaction v1: {signature}"),
        Err(error) => eprintln!("Failed to send Transaction v1: {error}"),
    }
}
