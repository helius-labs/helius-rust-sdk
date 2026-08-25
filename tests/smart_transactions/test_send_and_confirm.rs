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

/// A send that keeps failing must surface the underlying error rather than a generic timeout.
/// The retry arm previously discarded it (`Err(_) => continue`), so a permanently-failing
/// transaction — a malformed one, say — was retried until the deadline and reported only as
/// "failed to confirm", with the real reason gone.
#[tokio::test(flavor = "multi_thread")]
async fn test_surfaces_send_error_instead_of_timeout() {
    let (mut server, helius) = setup_mock().await;

    // Blockhash is still valid, so the loop runs and the send is actually attempted.
    mock_block_height(&mut server, 50);

    // A 2s budget with a 500ms pause between attempts fits roughly four sends. Without the
    // pause the loop re-sends as fast as the network allows and blows straight through this.
    let send_mock = server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("sendTransaction".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Transaction signature verification failure"},"id":1}"#,
        )
        .expect_at_most(6)
        .create();

    let tx = dummy_transaction();
    let result = helius
        .send_and_confirm_transaction(
            &tx,
            RpcSendTransactionConfig::default(),
            100,
            Some(Duration::from_secs(2)),
        )
        .await;

    let err = result.expect_err("a failing send should not return Ok");
    assert!(
        !matches!(err, HeliusError::Timeout { .. }),
        "the send error should replace the generic timeout, got {err:?}"
    );
    assert!(
        err.to_string().contains("signature verification failure"),
        "the underlying send error should be preserved, got {err:?}"
    );
    // Also guards the backoff: a hot retry loop trips the request cap above.
    send_mock.assert();
}

/// The retained send error must not outlive the attempt it came from. If an early send fails but
/// a later one succeeds, the run ended in a confirmation timeout — reporting the stale send error
/// would misattribute it, and would claim the transaction never left when it may well have landed.
#[tokio::test(flavor = "multi_thread")]
async fn test_stale_send_error_does_not_mask_timeout() {
    let (mut server, helius) = setup_mock().await;
    let tx = dummy_transaction();

    mock_block_height(&mut server, 50);

    // First send fails...
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("sendTransaction".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"jsonrpc":"2.0","error":{"code":-32002,"message":"Node is behind by 42 slots"},"id":1}"#)
        .expect(1)
        .create();

    // ...then subsequent sends succeed, but the transaction never confirms. The RPC client checks
    // the returned signature against the transaction's own, so the mock must echo the real one.
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("sendTransaction".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(format!(r#"{{"jsonrpc":"2.0","result":"{}","id":1}}"#, tx.signatures[0]))
        .create();

    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getSignatureStatuses".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":[null]},"id":1}"#)
        .create();

    let result = helius
        .send_and_confirm_transaction(
            &tx,
            RpcSendTransactionConfig::default(),
            100,
            Some(Duration::from_secs(2)),
        )
        .await;

    let err = result.expect_err("an unconfirmed transaction should not return Ok");
    assert!(
        matches!(err, HeliusError::Timeout { .. }),
        "the run ended in a confirmation timeout, not the earlier send failure, got {err:?}"
    );
}
