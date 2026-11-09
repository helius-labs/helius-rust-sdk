use helius::optimized_transaction::MAX_LOADED_ACCOUNTS_DATA_SIZE_BYTES;
use helius::types::{
    CreateSmartTransactionConfig, CreateSmartTransactionSeedConfig, SmartTransaction, TransactionVersion,
};
use solana_sdk::{
    message::VersionedMessage,
    pubkey::Pubkey,
    signature::{keypair_from_seed, Keypair, Signer},
};
use solana_system_interface::instruction as system_instruction;
use std::sync::Arc;

use solana_commitment_config::CommitmentConfig;

use super::helpers::{
    mock_latest_blockhash, mock_priority_fee_estimate, mock_simulate_transaction, mock_simulate_transaction_failed,
    setup_mock, setup_mock_with_commitment,
};

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_legacy_success() {
    let (mut server, helius) = setup_mock().await;

    // Set up all required RPC mocks
    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server);
    mock_simulate_transaction(&mut server, 50_000);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![payer_signer],
        lookup_tables: None, // No LUTs = legacy transaction
        fee_payer: None,
        priority_fee_cap: None,
        cu_buffer_multiplier: None,
        ..Default::default()
    };

    let result = helius.create_smart_transaction(&config).await;
    assert!(result.is_ok(), "create_smart_transaction failed: {:?}", result.err());

    let (transaction, last_valid_block_height) = result.unwrap();

    // Should be a legacy transaction (no lookup tables)
    assert!(
        matches!(transaction, SmartTransaction::Legacy(_)),
        "Expected Legacy transaction without lookup tables"
    );

    assert!(last_valid_block_height > 0, "last_valid_block_height should be > 0");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_with_priority_fee_cap() {
    let (mut server, helius) = setup_mock().await;

    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server); // Returns 1000
    mock_simulate_transaction(&mut server, 50_000);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![payer_signer],
        lookup_tables: None,
        fee_payer: None,
        priority_fee_cap: Some(500), // Cap below the 1000 estimate
        cu_buffer_multiplier: None,
        ..Default::default()
    };

    let result = helius.create_smart_transaction(&config).await;
    assert!(
        result.is_ok(),
        "create_smart_transaction with fee cap failed: {:?}",
        result.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_with_custom_cu_multiplier() {
    let (mut server, helius) = setup_mock().await;

    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server);
    mock_simulate_transaction(&mut server, 50_000);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![payer_signer],
        lookup_tables: None,
        fee_payer: None,
        priority_fee_cap: None,
        cu_buffer_multiplier: Some(1.5), // Custom 50% buffer
        ..Default::default()
    };

    let result = helius.create_smart_transaction(&config).await;
    assert!(
        result.is_ok(),
        "create_smart_transaction with custom CU multiplier failed: {:?}",
        result.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_with_separate_fee_payer() {
    let (mut server, helius) = setup_mock().await;

    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server);
    mock_simulate_transaction(&mut server, 50_000);

    let signer = Keypair::new();
    let fee_payer = Keypair::new();
    let signer_arc: Arc<dyn Signer> = Arc::new(signer.insecure_clone());
    let fee_payer_arc: Arc<dyn Signer> = Arc::new(fee_payer.insecure_clone());

    // The signer is the sender of the transfer, fee_payer just pays fees
    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &signer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![signer_arc],
        lookup_tables: None,
        fee_payer: Some(fee_payer_arc),
        priority_fee_cap: None,
        cu_buffer_multiplier: None,
        ..Default::default()
    };

    let result = helius.create_smart_transaction(&config).await;
    assert!(
        result.is_ok(),
        "create_smart_transaction with separate fee payer failed: {:?}",
        result.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_low_compute_units_gets_minimum() {
    let (mut server, helius) = setup_mock().await;

    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server);
    // Simulate very low compute units (below the 1000 minimum)
    mock_simulate_transaction(&mut server, 500);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![payer_signer],
        lookup_tables: None,
        fee_payer: None,
        priority_fee_cap: None,
        cu_buffer_multiplier: None,
        ..Default::default()
    };

    // Should succeed — compute units below 1000 get clamped to 1000
    let result = helius.create_smart_transaction(&config).await;
    assert!(
        result.is_ok(),
        "create_smart_transaction with low CU failed: {:?}",
        result.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_v1_success() {
    let (mut server, helius) = setup_mock().await;

    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server);
    mock_simulate_transaction(&mut server, 50_000);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![payer_signer],
        version: TransactionVersion::V1,
        ..Default::default()
    };

    let (transaction, last_valid_block_height) = helius
        .create_smart_transaction(&config)
        .await
        .expect("v1 smart tx should build");

    assert!(last_valid_block_height > 0);
    match transaction {
        SmartTransaction::Versioned(vtx) => match vtx.message {
            VersionedMessage::V1(m) => {
                // Fee and CU limit live in the v1 header config, not in ComputeBudget instructions.
                assert!(
                    m.config.compute_unit_limit.is_some(),
                    "v1 CU limit should be set in the header config"
                );
                assert!(
                    m.config.priority_fee.is_some(),
                    "v1 priority fee should be set in the header config"
                );
                assert_eq!(m.instructions.len(), 1, "v1 must not add ComputeBudget instructions");
            }
            other => panic!("expected VersionedMessage::V1, got {other:?}"),
        },
        other => panic!("expected a versioned transaction, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_v1_rejects_lookup_tables() {
    let (mut server, helius) = setup_mock().await;
    mock_latest_blockhash(&mut server);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![payer_signer],
        version: TransactionVersion::V1,
        lookup_tables: Some(vec![]),
        ..Default::default()
    };

    let result = helius.create_smart_transaction(&config).await;
    assert!(
        result.is_err(),
        "v1 with lookup tables should be rejected, got {result:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_with_seeds_v1_success() {
    let (mut server, helius) = setup_mock().await;

    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server);
    mock_simulate_transaction(&mut server, 50_000);

    let seed = [7u8; 32];
    let signer = keypair_from_seed(&seed).unwrap();

    let config = CreateSmartTransactionSeedConfig::new(
        vec![system_instruction::transfer(
            &signer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        vec![seed],
    )
    .with_v1();

    let (transaction, _) = helius
        .create_smart_transaction_with_seeds(&config)
        .await
        .expect("v1 seed smart tx should build");

    match transaction {
        SmartTransaction::Versioned(vtx) => assert!(
            matches!(vtx.message, VersionedMessage::V1(_)),
            "expected a V1 message from the seed path"
        ),
        other => panic!("expected a versioned transaction, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_without_signers_rejects_v1() {
    let (mut server, helius) = setup_mock().await;
    mock_latest_blockhash(&mut server);

    let fee_payer = Keypair::new();
    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &fee_payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        fee_payer: Some(Arc::new(fee_payer.insecure_clone())),
        version: TransactionVersion::V1,
        ..Default::default()
    };

    let result = helius.create_smart_transaction_without_signers(&config).await;
    assert!(result.is_err(), "the unsigned path must reject v1, got {result:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_errors_on_failed_simulation() {
    let (mut server, helius) = setup_mock().await;

    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server);
    // The simulation fails (err set, unitsConsumed: 0) — the SDK must not proceed with bogus units.
    mock_simulate_transaction_failed(&mut server);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());
    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![payer_signer],
        ..Default::default()
    };

    let result = helius.create_smart_transaction(&config).await;
    assert!(
        result.is_err(),
        "a failed simulation must surface as an error, got {result:?}"
    );
}

/// Drives `loaded_accounts_data_size_limit` end to end and asserts it reaches the v1 header, rather
/// than only testing the resolver in isolation.
#[tokio::test(flavor = "multi_thread")]
async fn test_v1_carries_the_caller_supplied_data_size_limit() {
    let (mut server, helius) = setup_mock().await;

    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server);
    mock_simulate_transaction(&mut server, 50_000);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig::new(
        vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        vec![payer_signer],
    )
    .with_v1()
    .with_loaded_accounts_data_size_limit(2 * 1024 * 1024);

    let (transaction, _) = helius
        .create_smart_transaction(&config)
        .await
        .expect("v1 smart tx should build");

    match transaction {
        SmartTransaction::Versioned(vtx) => match vtx.message {
            VersionedMessage::V1(m) => assert_eq!(
                m.config.loaded_accounts_data_size_limit,
                Some(2 * 1024 * 1024),
                "the caller's budget must reach the header, not the 64 MiB default"
            ),
            other => panic!("expected a v1 message, got {other:?}"),
        },
        other => panic!("expected a versioned transaction, got {other:?}"),
    }
}

/// The default is the 64 MiB maximum, matching the implicit budget legacy and v0 get.
#[tokio::test(flavor = "multi_thread")]
async fn test_v1_defaults_the_data_size_limit_to_the_maximum() {
    let (mut server, helius) = setup_mock().await;

    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server);
    mock_simulate_transaction(&mut server, 50_000);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig::new(
        vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        vec![payer_signer],
    )
    .with_v1();

    let (transaction, _) = helius
        .create_smart_transaction(&config)
        .await
        .expect("v1 smart tx should build");

    match transaction {
        SmartTransaction::Versioned(vtx) => match vtx.message {
            VersionedMessage::V1(m) => {
                assert_eq!(
                    m.config.loaded_accounts_data_size_limit,
                    Some(MAX_LOADED_ACCOUNTS_DATA_SIZE_BYTES)
                );
                // The runtime reads an unset compute-unit limit as 0, so it must always be present.
                assert!(
                    m.config.compute_unit_limit.is_some(),
                    "an unset v1 compute-unit limit means a zero compute budget"
                );
            }
            other => panic!("expected a v1 message, got {other:?}"),
        },
        other => panic!("expected a versioned transaction, got {other:?}"),
    }
}

/// Legacy and v0 have nowhere to carry the limit, so setting it there is rejected rather than
/// silently dropped.
#[tokio::test(flavor = "multi_thread")]
async fn test_data_size_limit_is_rejected_without_v1() {
    let (mut server, helius) = setup_mock().await;
    mock_latest_blockhash(&mut server);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());

    let config = CreateSmartTransactionConfig::new(
        vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        vec![payer_signer],
    )
    .with_loaded_accounts_data_size_limit(1024);

    let result = helius.create_smart_transaction(&config).await;

    match result {
        Ok(_) => panic!("expected the non-v1 config to be rejected"),
        Err(err) => assert!(
            err.to_string().contains("requires TransactionVersion::V1"),
            "expected a version-mismatch error, got {err}"
        ),
    }
}

/// Mocks `simulateTransaction` the way a node answers it at each commitment for a draft built on
/// a `confirmed` blockhash: the confirmed bank knows the blockhash and runs the draft, while the
/// rooted (`finalized`) bank does not know it yet and rejects it with `BlockhashNotFound`.
fn mock_simulation_by_commitment(server: &mut mockito::Server) -> (mockito::Mock, mockito::Mock) {
    let simulate_at = |commitment: &str| {
        mockito::Matcher::AllOf(vec![
            mockito::Matcher::Regex("simulateTransaction".to_string()),
            mockito::Matcher::Regex(format!(r#""commitment":"{commitment}""#)),
        ])
    };

    let confirmed = server
        .mock("POST", mockito::Matcher::Any)
        .match_body(simulate_at("confirmed"))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":{"err":null,"logs":[],"accounts":null,"unitsConsumed":50000,"returnData":null}},"id":1}"#,
        )
        .expect_at_least(1)
        .create();
    let finalized = server
        .mock("POST", mockito::Matcher::Any)
        .match_body(simulate_at("finalized"))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":{"err":"BlockhashNotFound","logs":[],"accounts":null,"unitsConsumed":0,"returnData":null}},"id":1}"#,
        )
        .expect(0)
        .create();

    (confirmed, finalized)
}

/// The draft's blockhash is fetched at the client's commitment, so it must be simulated there too.
/// At the RPC default (`finalized`) a confirmed-commitment client, such as `Helius::new_async`
/// builds, has every draft rejected with `BlockhashNotFound`, failing every smart transaction.
#[tokio::test(flavor = "multi_thread")]
async fn test_create_smart_transaction_simulates_at_the_client_commitment() {
    let (mut server, helius) = setup_mock_with_commitment(Some(CommitmentConfig::confirmed())).await;

    mock_latest_blockhash(&mut server);
    mock_priority_fee_estimate(&mut server);
    let (confirmed, finalized) = mock_simulation_by_commitment(&mut server);

    let payer = Keypair::new();
    let payer_signer: Arc<dyn Signer> = Arc::new(payer.insecure_clone());
    let config = CreateSmartTransactionConfig {
        instructions: vec![system_instruction::transfer(
            &payer.pubkey(),
            &Pubkey::new_unique(),
            1000,
        )],
        signers: vec![payer_signer],
        ..Default::default()
    };

    let result = helius.create_smart_transaction(&config).await;
    assert!(result.is_ok(), "create_smart_transaction failed: {:?}", result.err());

    confirmed.assert();
    finalized.assert();
}

/// The seed-based path reports `units_consumed` as is, so a rejected simulation there does not
/// fail: it yields 0 units and the transaction is built with the 1,000 CU floor instead of the
/// real estimate.
#[tokio::test(flavor = "multi_thread")]
async fn test_thread_safe_compute_units_are_simulated_at_the_client_commitment() {
    let (mut server, helius) = setup_mock_with_commitment(Some(CommitmentConfig::confirmed())).await;

    mock_latest_blockhash(&mut server);
    let (confirmed, finalized) = mock_simulation_by_commitment(&mut server);

    let payer = Keypair::new();
    let units = helius
        .get_compute_units_thread_safe(
            vec![system_instruction::transfer(
                &payer.pubkey(),
                &Pubkey::new_unique(),
                1000,
            )],
            payer.pubkey(),
            vec![],
            Some(&[&payer]),
            TransactionVersion::Auto,
        )
        .await
        .unwrap();

    assert_eq!(units, Some(50_000), "the estimate should come from the confirmed bank");
    confirmed.assert();
    finalized.assert();
}
