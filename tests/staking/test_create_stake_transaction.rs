use solana_sdk::pubkey::Pubkey;

use super::helpers::{mock_latest_blockhash, mock_rent_exempt, setup_mock};

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
