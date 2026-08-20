use std::time::Duration;

use solana_sdk::signature::Signature;

use mockito::{Matcher, Server};

use super::helpers::setup_mock;

/// Mocks `getSignatureStatuses` to return an empty `value` array, as a misbehaving provider
/// might. The previous `status.value[0]` index would panic on this; the fix treats a missing
/// entry as "not yet available" and keeps retrying.
fn mock_empty_signature_statuses(server: &mut Server) {
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getSignatureStatuses".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":[]},"id":1}"#)
        .create();
}

/// An empty `getSignatureStatuses` response must not panic the confirmation poll — it should
/// keep retrying until it times out. We assert the spawned poll is still running (retrying)
/// shortly after the response, rather than having panicked (which resolves the join handle).
#[tokio::test(flavor = "multi_thread")]
async fn test_poll_confirmation_empty_status_does_not_panic() {
    let (mut server, helius) = setup_mock().await;
    mock_empty_signature_statuses(&mut server);

    let handle = tokio::spawn(async move { helius.poll_transaction_confirmation(Signature::default()).await });

    // Long enough to make several RPC calls and enter a retry sleep, but well short of the 15s
    // timeout so the poll is still in-flight.
    tokio::time::sleep(Duration::from_secs(2)).await;

    assert!(
        !handle.is_finished(),
        "poll should still be retrying after an empty status response, not panicked or returned"
    );

    handle.abort();
}

/// Mocks `getSignatureStatuses` to report the transaction as `processed` — the normal state in
/// the moments right after submission, and the one that used to fall through the match with no
/// sleep. Returns the mock so the caller can count how many times it was hit.
fn mock_processed_signature_statuses(server: &mut Server) -> mockito::Mock {
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getSignatureStatuses".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":[{"slot":1,"confirmations":0,"err":null,"status":{"Ok":null},"confirmationStatus":"processed"}]},"id":1}"#,
        )
        .expect_at_most(8)
        .create()
}

/// A transaction stuck in `processed` must not spin. It previously matched neither the confirmed
/// branch nor the error branch and looped straight back around with no sleep, issuing blocking
/// RPC calls as fast as the network allowed for the full 15s timeout (and, because that path
/// contained no `.await`, pinning a tokio worker thread while doing it).
///
/// The poll now always sleeps between checks, so only a handful of requests fit in the window.
/// `expect_at_most(8)` is what gives this test teeth: a regression sends hundreds and trips it.
#[tokio::test(flavor = "multi_thread")]
async fn test_poll_confirmation_processed_status_does_not_spin() {
    let (mut server, helius) = setup_mock().await;
    let status_mock = mock_processed_signature_statuses(&mut server);

    let handle = tokio::spawn(async move { helius.poll_transaction_confirmation(Signature::default()).await });

    // Covers the first few backoff steps (400ms + 800ms + 1600ms), so a correct poll has made
    // roughly 4 requests by now and a spinning one has made far more.
    tokio::time::sleep(Duration::from_secs(3)).await;

    assert!(
        !handle.is_finished(),
        "poll should still be retrying while the transaction is processed, not returned"
    );

    handle.abort();
    status_mock.assert();
}

/// A `confirmed` status returns the signature rather than continuing to poll, and does so
/// promptly — the backoff starts near the slot time specifically so the common case stays fast.
#[tokio::test(flavor = "multi_thread")]
async fn test_poll_confirmation_returns_on_confirmed() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getSignatureStatuses".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":[{"slot":1,"confirmations":null,"err":null,"status":{"Ok":null},"confirmationStatus":"confirmed"}]},"id":1}"#,
        )
        .create();

    let started = std::time::Instant::now();
    let signature = helius
        .poll_transaction_confirmation(Signature::default())
        .await
        .expect("a confirmed status should resolve the poll");

    assert_eq!(signature, Signature::default());
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "a confirmed transaction should return on the first check, took {:?}",
        started.elapsed()
    );
}

/// An on-chain execution error is terminal: the poll must surface it immediately rather than
/// retrying until the timeout.
#[tokio::test(flavor = "multi_thread")]
async fn test_poll_confirmation_surfaces_transaction_error() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getSignatureStatuses".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":[{"slot":1,"confirmations":0,"err":{"InstructionError":[0,{"Custom":1}]},"status":{"Err":{"InstructionError":[0,{"Custom":1}]}},"confirmationStatus":"processed"}]},"id":1}"#,
        )
        .create();

    let result = helius.poll_transaction_confirmation(Signature::default()).await;

    assert!(
        matches!(result, Err(helius::error::HeliusError::TransactionError(_))),
        "expected the on-chain error to be surfaced, got {result:?}"
    );
}
