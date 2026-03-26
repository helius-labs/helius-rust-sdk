use solana_sdk::pubkey::Pubkey;

use super::helpers::{mock_latest_blockhash, setup_mock};

#[tokio::test(flavor = "multi_thread")]
async fn test_create_unstake_transaction_success() {
    let (mut server, helius) = setup_mock().await;
    mock_latest_blockhash(&mut server);

    let owner = Pubkey::new_unique();
    let stake_account = Pubkey::new_unique();

    let result = helius.create_unstake_transaction(owner, stake_account).await;
    assert!(result.is_ok(), "create_unstake_transaction failed: {:?}", result.err());

    let encoded_tx = result.unwrap();
    assert!(!encoded_tx.is_empty(), "Encoded transaction should not be empty");

    // Verify it's valid base58 by decoding
    let decoded = solana_sdk::bs58::decode(&encoded_tx).into_vec();
    assert!(decoded.is_ok(), "Transaction should be valid base58");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_unstake_transaction_rpc_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Regex("getLatestBlockhash".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error"},"id":1}"#)
        .create();

    let owner = Pubkey::new_unique();
    let stake_account = Pubkey::new_unique();

    let result = helius.create_unstake_transaction(owner, stake_account).await;
    assert!(result.is_err(), "Expected error when RPC fails");
}
