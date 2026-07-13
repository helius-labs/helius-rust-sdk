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

    // Long enough to make the RPC call and enter the retry sleep, but shorter than the 5s
    // retry interval so the poll is still in-flight (not yet timed out).
    tokio::time::sleep(Duration::from_secs(2)).await;

    assert!(
        !handle.is_finished(),
        "poll should still be retrying after an empty status response, not panicked or returned"
    );

    handle.abort();
}
