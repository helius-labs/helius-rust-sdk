use std::time::Duration;

use helius::error::HeliusError;

use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_commitment_config::CommitmentLevel;
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

/// Mocks `sendTransaction` the way a node answers a transaction built on a `confirmed` blockhash
/// (as every smart transaction is): preflight at `confirmed` accepts it, while preflight at
/// `finalized` runs on the rooted bank, which does not know the blockhash yet. Returned uncreated so
/// each test can set its expectations.
fn mock_send_by_preflight_commitment(server: &mut Server, tx: &Transaction) -> (mockito::Mock, mockito::Mock) {
    let send_with = |commitment: &str| {
        Matcher::AllOf(vec![
            Matcher::Regex("sendTransaction".to_string()),
            Matcher::Regex(format!(r#""preflightCommitment":"{commitment}""#)),
        ])
    };

    let confirmed = server
        .mock("POST", Matcher::Any)
        .match_body(send_with("confirmed"))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(format!(r#"{{"jsonrpc":"2.0","result":"{}","id":1}}"#, tx.signatures[0]));
    let finalized = server
        .mock("POST", Matcher::Any)
        .match_body(send_with("finalized"))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","error":{"code":-32002,"message":"Transaction simulation failed: Blockhash not found","data":{"err":"BlockhashNotFound","logs":[],"accounts":null,"unitsConsumed":0,"returnData":null}},"id":1}"#,
        );

    (confirmed, finalized)
}

fn mock_confirmed_status(server: &mut Server) {
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getSignatureStatuses".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":[{"slot":1,"confirmations":null,"err":null,"status":{"Ok":null},"confirmationStatus":"confirmed"}]},"id":1}"#,
        )
        .create();
}

/// The smart-transaction builders fetch their blockhash at `confirmed`, and `SmartTransactionConfig::new`
/// sends with `RpcSendTransactionConfig::default()`, which leaves the preflight commitment unset.
/// Unset, the RPC client fills in `finalized`, so every send failed preflight with "Blockhash not
/// found" until the root caught up with the blockhash, or until the timeout ran out first.
#[tokio::test(flavor = "multi_thread")]
async fn test_unset_preflight_commitment_defaults_to_confirmed() {
    let (mut server, helius) = setup_mock().await;
    let tx = dummy_transaction();

    mock_block_height(&mut server, 50);
    let (confirmed, finalized) = mock_send_by_preflight_commitment(&mut server, &tx);
    let confirmed = confirmed.expect(1).create();
    let finalized = finalized.expect(0).create();
    mock_confirmed_status(&mut server);

    let result = helius
        .send_and_confirm_transaction(
            &tx,
            RpcSendTransactionConfig::default(),
            100,
            Some(Duration::from_secs(2)),
        )
        .await;

    assert_eq!(result.ok(), Some(tx.signatures[0]), "the first send should go through");
    confirmed.assert();
    finalized.assert();
}

/// A preflight commitment the caller sets is sent as is.
#[tokio::test(flavor = "multi_thread")]
async fn test_explicit_preflight_commitment_is_kept() {
    let (mut server, helius) = setup_mock().await;
    let tx = dummy_transaction();

    mock_block_height(&mut server, 50);
    let (confirmed, finalized) = mock_send_by_preflight_commitment(&mut server, &tx);
    let confirmed = confirmed.expect(0).create();
    let finalized = finalized.expect_at_least(1).create();

    let result = helius
        .send_and_confirm_transaction(
            &tx,
            RpcSendTransactionConfig {
                preflight_commitment: Some(CommitmentLevel::Finalized),
                ..RpcSendTransactionConfig::default()
            },
            100,
            Some(Duration::from_secs(1)),
        )
        .await;

    let err = result.expect_err("the finalized preflight rejects the confirmed blockhash");
    assert!(err.to_string().contains("Blockhash not found"), "got {err:?}");
    confirmed.assert();
    finalized.assert();
}

/// Mocks `getSignatureStatuses` to report an on-chain failure.
fn mock_failed_status(server: &mut Server) {
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getSignatureStatuses".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":[{"slot":1,"confirmations":null,"err":{"InstructionError":[0,{"Custom":6001}]},"status":{"Err":{"InstructionError":[0,{"Custom":6001}]}},"confirmationStatus":"confirmed"}]},"id":1}"#,
        )
        .create();
}

/// The loop gives up on expiry using a status read from before the height read. A transaction
/// that landed at or below the last valid height while the tip moved past it is only visible to a
/// fresh read, and reporting it as expired invites a rebuild that executes it twice.
#[tokio::test(flavor = "multi_thread")]
async fn test_expired_blockhash_returns_a_landed_transaction() {
    let (mut server, helius) = setup_mock().await;
    let tx = dummy_transaction();

    mock_block_height(&mut server, 200);
    mock_confirmed_status(&mut server);

    let result = helius
        .send_and_confirm_transaction(
            &tx,
            RpcSendTransactionConfig::default(),
            100,
            Some(Duration::from_secs(1)),
        )
        .await;

    assert_eq!(result.ok(), Some(tx.signatures[0]));
}

/// A fresh read that finds the transaction failed on-chain reports that failure, not the expiry.
#[tokio::test(flavor = "multi_thread")]
async fn test_expired_blockhash_reports_an_on_chain_failure() {
    let (mut server, helius) = setup_mock().await;
    let tx = dummy_transaction();

    mock_block_height(&mut server, 200);
    mock_failed_status(&mut server);

    let result = helius
        .send_and_confirm_transaction(
            &tx,
            RpcSendTransactionConfig::default(),
            100,
            Some(Duration::from_secs(1)),
        )
        .await;

    assert!(
        matches!(result, Err(HeliusError::TransactionError(_))),
        "got {result:?}"
    );
}

/// Once a transaction lands, resending it fails preflight as already processed. The loop retries
/// that until the deadline and then reported the send error, although the transaction succeeded.
#[tokio::test(flavor = "multi_thread")]
async fn test_already_processed_resends_return_the_landed_transaction() {
    let (mut server, helius) = setup_mock().await;
    let tx = dummy_transaction();

    mock_block_height(&mut server, 50);
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("sendTransaction".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","error":{"code":-32002,"message":"Transaction simulation failed: This transaction has already been processed","data":{"err":"AlreadyProcessed","logs":[],"accounts":null,"unitsConsumed":0,"returnData":null}},"id":1}"#,
        )
        .create();
    mock_confirmed_status(&mut server);

    let result = helius
        .send_and_confirm_transaction(
            &tx,
            RpcSendTransactionConfig::default(),
            100,
            Some(Duration::from_secs(1)),
        )
        .await;

    assert_eq!(result.ok(), Some(tx.signatures[0]));
}

/// Mocks `sendTransaction` the way a node answers a resend of a transaction it already has: the
/// preflight simulation finds it in the status cache and fails with `AlreadyProcessed`.
fn mock_send_already_processed(server: &mut Server) -> mockito::Mock {
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("sendTransaction".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","error":{"code":-32002,"message":"Transaction simulation failed: This transaction has already been processed","data":{"err":"AlreadyProcessed","logs":[],"accounts":null,"unitsConsumed":0,"returnData":null}},"id":1}"#,
        )
}

/// An `AlreadyProcessed` send failure means the transaction is already in the status cache, so
/// its status is read right away instead of resending it every 500 ms until the deadline.
#[tokio::test(flavor = "multi_thread")]
async fn test_already_processed_send_returns_without_resending() {
    let (mut server, helius) = setup_mock().await;
    let tx = dummy_transaction();

    mock_block_height(&mut server, 50);
    let send = mock_send_already_processed(&mut server).expect(1).create();
    mock_confirmed_status(&mut server);

    let result = helius
        .send_and_confirm_transaction(
            &tx,
            RpcSendTransactionConfig::default(),
            100,
            Some(Duration::from_secs(3)),
        )
        .await;

    assert_eq!(result.ok(), Some(tx.signatures[0]));
    send.assert();
}

/// The same read reports a transaction that landed but failed on-chain with its own error.
#[tokio::test(flavor = "multi_thread")]
async fn test_already_processed_send_reports_the_on_chain_failure_without_resending() {
    let (mut server, helius) = setup_mock().await;
    let tx = dummy_transaction();

    mock_block_height(&mut server, 50);
    let send = mock_send_already_processed(&mut server).expect(1).create();
    mock_failed_status(&mut server);

    let result = helius
        .send_and_confirm_transaction(
            &tx,
            RpcSendTransactionConfig::default(),
            100,
            Some(Duration::from_secs(3)),
        )
        .await;

    assert!(
        matches!(result, Err(HeliusError::TransactionError(_))),
        "got {result:?}"
    );
    send.assert();
}
