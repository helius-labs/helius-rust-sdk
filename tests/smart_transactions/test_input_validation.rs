use helius::error::HeliusError;
use helius::types::{CreateSmartTransactionConfig, CreateSmartTransactionSeedConfig};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use solana_system_interface::instruction as system_instruction;
use std::sync::Arc;

use super::helpers::setup_mock;

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_rejects_empty_signers() {
    let (_server, helius) = setup_mock().await;

    // No RPC mocks needed — should fail before any network call
    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &Pubkey::new_unique(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![],
        lookup_tables: None,
        fee_payer: None,
        priority_fee_cap: None,
        cu_buffer_multiplier: None,
    };

    let result = helius.create_smart_transaction(&config).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        HeliusError::InvalidInput(msg) => {
            assert!(
                msg.contains("fee payer must sign"),
                "Expected fee payer error, got: {}",
                msg
            );
        }
        other => panic!("Expected InvalidInput, got: {:?}", other),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_rejects_compute_budget_instructions() {
    let (mut server, helius) = setup_mock().await;

    // Mock getLatestBlockhash since the check happens after fetching it
    super::helpers::mock_latest_blockhash(&mut server);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig {
        instructions: vec![
            ComputeBudgetInstruction::set_compute_unit_price(1000),
            system_instruction::transfer(&payer.pubkey(), &Pubkey::new_unique(), 1000),
        ],
        signers: vec![payer_signer],
        lookup_tables: None,
        fee_payer: None,
        priority_fee_cap: None,
        cu_buffer_multiplier: None,
    };

    let result = helius.create_smart_transaction(&config).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        HeliusError::InvalidInput(msg) => {
            assert!(
                msg.contains("compute unit price"),
                "Expected compute budget error, got: {}",
                msg
            );
        }
        other => panic!("Expected InvalidInput, got: {:?}", other),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_rejects_compute_unit_limit_instruction() {
    let (mut server, helius) = setup_mock().await;
    super::helpers::mock_latest_blockhash(&mut server);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig {
        instructions: vec![
            ComputeBudgetInstruction::set_compute_unit_limit(200_000),
            system_instruction::transfer(&payer.pubkey(), &Pubkey::new_unique(), 1000),
        ],
        signers: vec![payer_signer],
        lookup_tables: None,
        fee_payer: None,
        priority_fee_cap: None,
        cu_buffer_multiplier: None,
    };

    let result = helius.create_smart_transaction(&config).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        HeliusError::InvalidInput(msg) => {
            assert!(
                msg.contains("compute unit price") || msg.contains("compute unit"),
                "Expected compute budget error, got: {}",
                msg
            );
        }
        other => panic!("Expected InvalidInput, got: {:?}", other),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_with_seeds_rejects_empty_seeds() {
    let (_server, helius) = setup_mock().await;

    let config = CreateSmartTransactionSeedConfig {
        instructions: vec![system_instruction::transfer(
            &Pubkey::new_unique(),
            &Pubkey::new_unique(),
            1000,
        )],
        signer_seeds: vec![],
        fee_payer_seed: None,
        lookup_tables: None,
        priority_fee_cap: None,
        cu_buffer_multiplier: None,
    };

    let result = helius.create_smart_transaction_with_seeds(&config).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        HeliusError::InvalidInput(msg) => {
            assert!(
                msg.contains("signer seed"),
                "Expected signer seed error, got: {}",
                msg
            );
        }
        other => panic!("Expected InvalidInput, got: {:?}", other),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_without_signers_rejects_missing_fee_payer() {
    let (_server, helius) = setup_mock().await;

    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &Pubkey::new_unique(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![],
        lookup_tables: None,
        fee_payer: None,
        priority_fee_cap: None,
        cu_buffer_multiplier: None,
    };

    let result = helius.create_smart_transaction_without_signers(&config).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        HeliusError::InvalidInput(msg) => {
            assert!(
                msg.contains("Fee payer") || msg.contains("fee payer"),
                "Expected fee payer error, got: {}",
                msg
            );
        }
        other => panic!("Expected InvalidInput, got: {:?}", other),
    }
}
