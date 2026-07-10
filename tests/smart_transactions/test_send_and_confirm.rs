use std::time::Duration;

use helius::error::HeliusError;

use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::{
    hash::Hash,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_system_interface::instruction as system_instruction;

use mockito::{Matcher, Server};

use super::helpers::setup_mock;

/// Mocks `getBlockHeight` to return a fixed height.
fn mock_block_height(server: &mut Server, height: u64) {
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getBlockHeight".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(format!(r#"{{"jsonrpc":"2.0","result":{height},"id":1}}"#))
        .create();
}

fn dummy_transaction() -> Transaction {
    let payer = Keypair::new();
    let ix = system_instruction::transfer(&payer.pubkey(), &Pubkey::new_unique(), 1);
    Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], Hash::default())
}

/// Once the current block height exceeds `last_valid_block_height`, the blockhash has expired
/// and `send_and_confirm_transaction` must stop immediately without attempting a send. This
/// guards the retry-loop condition: with `&&` the loop exits as soon as either the timeout
/// elapses or the blockhash expires, whereas the previous `||` kept looping (and sending)
/// until both elapsed.
#[tokio::test(flavor = "multi_thread")]
async fn test_stops_when_blockhash_expired() {
    let (mut server, helius) = setup_mock().await;

    // Current height (200) is already past the last valid height (100) we pass below.
    mock_block_height(&mut server, 200);

    // If the loop body ever runs, it will POST a sendTransaction — which must not happen.
    let send_mock = server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("sendTransaction".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":"1111111111111111111111111111111111111111111111111111111111111111","id":1}"#,
        )
        .expect(0)
        .create();

    let tx = dummy_transaction();
    let result = helius
        .send_and_confirm_transaction(
            &tx,
            RpcSendTransactionConfig::default(),
            100,
            Some(Duration::from_secs(1)),
        )
        .await;

    assert!(
        matches!(result, Err(HeliusError::Timeout { .. })),
        "Expected a Timeout error once the blockhash expired, got {result:?}"
    );
    send_mock.assert();
}
