use solana_sdk::pubkey::Pubkey;

use super::helpers::setup_mock;

fn rent_exempt_response() -> String {
    r#"{"jsonrpc":"2.0","result":2282880,"id":1}"#.to_string()
}

fn latest_blockhash_response() -> String {
    r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":{"blockhash":"EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N","lastValidBlockHeight":100}},"id":1}"#.to_string()
}

fn mock_rent_exempt(server: &mut mockito::Server) {
    server
        .mock("POST", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Regex("getMinimumBalanceForRentExemption".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(rent_exempt_response())
        .create();
}

fn mock_latest_blockhash(server: &mut mockito::Server) {
    server
        .mock("POST", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Regex("getLatestBlockhash".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(latest_blockhash_response())
        .create();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_stake_transaction_success() {
    let (mut server, helius) = setup_mock().await;
    mock_rent_exempt(&mut server);
    mock_latest_blockhash(&mut server);

    let owner = Pubkey::new_unique();
    let result = helius.create_stake_transaction(owner, 1.0).await;

    assert!(result.is_ok(), "create_stake_transaction failed: {:?}", result.err());
    let (encoded_tx, stake_pubkey) = result.unwrap();

    assert!(!encoded_tx.is_empty(), "Encoded transaction should not be empty");
    assert_ne!(stake_pubkey, Pubkey::default(), "Stake pubkey should not be default");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_stake_transaction_rejects_zero_amount() {
    let (mut server, helius) = setup_mock().await;
    mock_rent_exempt(&mut server);

    let owner = Pubkey::new_unique();
    let result = helius.create_stake_transaction(owner, 0.0).await;

    assert!(result.is_err(), "Expected error for zero stake amount");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_stake_transaction_rejects_negative_amount() {
    let (mut server, helius) = setup_mock().await;
    mock_rent_exempt(&mut server);

    let owner = Pubkey::new_unique();
    let result = helius.create_stake_transaction(owner, -1.0).await;

    assert!(result.is_err(), "Expected error for negative stake amount");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_stake_transaction_rejects_nan() {
    let (mut server, helius) = setup_mock().await;
    mock_rent_exempt(&mut server);

    let owner = Pubkey::new_unique();
    let result = helius.create_stake_transaction(owner, f64::NAN).await;

    assert!(result.is_err(), "Expected error for NaN stake amount");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_create_stake_transaction_rejects_infinity() {
    let (mut server, helius) = setup_mock().await;
    mock_rent_exempt(&mut server);

    let owner = Pubkey::new_unique();
    let result = helius.create_stake_transaction(owner, f64::INFINITY).await;

    assert!(result.is_err(), "Expected error for infinite stake amount");
}
