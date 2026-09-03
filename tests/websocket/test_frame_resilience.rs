use futures_util::{SinkExt, StreamExt};
use helius::types::enhanced_websocket::TransactionDetails;
use helius::types::{
    RpcTransactionsConfig, TransactionCommitment, TransactionNotification, TransactionSubscribeFilter,
    TransactionSubscribeOptions, UiEnhancedTransactionEncoding,
};
use helius::websocket::EnhancedWebsocket;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::{accept_async, tungstenite::Message};

/// Starts a mock server that acks a `transactionSubscribe`, then sends `interference` verbatim,
/// then a well-formed notification on the live subscription.
///
/// The interference frames are the ones that used to end `run_ws` outright. If the client still
/// delivers the notification afterwards, the bad frame was scoped to itself.
async fn start_server_sending(interference: Vec<Message>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{}", addr);

    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let mut ws = accept_async(stream).await.unwrap();

            while let Some(Ok(msg)) = ws.next().await {
                match msg {
                    Message::Text(text) => {
                        let request: Value = serde_json::from_str(&text).unwrap();
                        let id = request.get("id").and_then(Value::as_u64).unwrap();
                        let method = request.get("method").and_then(Value::as_str).unwrap();

                        if method == "transactionSubscribe" {
                            let ack = json!({"jsonrpc": "2.0", "result": 1, "id": id});
                            ws.send(Message::Text(ack.to_string().into())).await.unwrap();

                            for frame in &interference {
                                ws.send(frame.clone()).await.unwrap();
                            }

                            let notification = json!({
                                "jsonrpc": "2.0",
                                "method": "transactionNotification",
                                "params": {
                                    "result": {
                                        "signature": "3Tf7QH4w9mK3rT6uV8xZ1cD5eF7gH9jK2LmN4pQ6sTuW8XaB2cDe4Fg6Hj8Km9Np2Qr4St6Uv8Wx7Yz2aBc4DeF6",
                                        "slot": 424242u64,
                                        "transactionIndex": 7u64
                                    },
                                    "subscription": 1
                                }
                            });
                            ws.send(Message::Text(notification.to_string().into())).await.unwrap();
                        }
                    }
                    Message::Close(_) => {
                        let _ = ws.close(None).await;
                        break;
                    }
                    Message::Ping(data) => {
                        let _ = ws.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    url
}

fn subscription_config() -> RpcTransactionsConfig {
    RpcTransactionsConfig {
        filter: TransactionSubscribeFilter {
            account_include: Some(vec!["7Yf2QH4w9mK3rT6uV8xZ1cD5eF7gH9jK2LmN4pQ6sTuW".to_string()]),
            vote: Some(false),
            failed: Some(false),
            signature: None,
            account_exclude: None,
            account_required: None,
        },
        options: TransactionSubscribeOptions {
            commitment: Some(TransactionCommitment::Confirmed),
            encoding: Some(UiEnhancedTransactionEncoding::Base64),
            transaction_details: Some(TransactionDetails::Signatures),
            show_rewards: Some(false),
            max_supported_transaction_version: Some(0),
        },
    }
}

/// Subscribes, lets the server send `interference`, and asserts the subscription still delivers.
async fn assert_subscription_survives(interference: Vec<Message>, case: &str) {
    let url = start_server_sending(interference).await;

    let ws = EnhancedWebsocket::new(&url, Some(30), None).await.unwrap();
    let (mut stream, _unsub) = ws.transaction_subscribe(subscription_config()).await.unwrap();

    let event = timeout(Duration::from_secs(2), stream.next())
        .await
        .unwrap_or_else(|_| panic!("{case}: timed out — the bad frame tore down the subscription"))
        .unwrap_or_else(|| panic!("{case}: stream ended — the bad frame tore down the subscription"));

    match event {
        TransactionNotification::Signature(entry) => {
            assert_eq!(entry.slot, 424242, "{case}: wrong notification delivered");
        }
        other => panic!("{case}: expected a signature notification, got {other:?}"),
    }
}

/// A text frame that is not JSON at all. Previously `serde_json::from_str(&text)?` returned from
/// `run_ws`, dropping every subscriber's channel.
#[tokio::test]
async fn malformed_json_frame_does_not_kill_subscriptions() {
    assert_subscription_survives(vec![Message::Text("this is not json".into())], "non-JSON text frame").await;
}

/// Valid JSON that is not a JSON-RPC object.
#[tokio::test]
async fn non_object_json_frame_does_not_kill_subscriptions() {
    assert_subscription_survives(vec![Message::Text("[1,2,3]".into())], "JSON array frame").await;
}

/// `"id": null` is what a server returns for a parse or invalid-request error — a documented
/// JSON-RPC response, and previously fatal to every subscription on the connection.
#[tokio::test]
async fn null_id_frame_does_not_kill_subscriptions() {
    assert_subscription_survives(
        vec![Message::Text(
            json!({"jsonrpc": "2.0", "error": {"code": -32700, "message": "Parse error"}, "id": null})
                .to_string()
                .into(),
        )],
        "null id",
    )
    .await;
}

/// A duplicate or late ack for a request that is no longer pending. Previously hit
/// `log::warn!("Unknown request id"); break;`.
#[tokio::test]
async fn stale_ack_does_not_kill_subscriptions() {
    assert_subscription_survives(
        vec![Message::Text(
            json!({"jsonrpc": "2.0", "result": 99, "id": 4242}).to_string().into(),
        )],
        "stale ack",
    )
    .await;
}

/// A response carrying neither `result` nor `error`.
#[tokio::test]
async fn response_without_result_does_not_kill_subscriptions() {
    assert_subscription_survives(
        vec![Message::Text(json!({"jsonrpc": "2.0", "id": 4243}).to_string().into())],
        "response with no result",
    )
    .await;
}

/// Several different bad frames back to back, still below the tolerance threshold.
#[tokio::test]
async fn several_bad_frames_do_not_kill_subscriptions() {
    assert_subscription_survives(
        vec![
            Message::Text("not json".into()),
            Message::Text(json!({"jsonrpc": "2.0", "id": null}).to_string().into()),
            Message::Text(json!({"jsonrpc": "2.0", "result": 1, "id": 9999}).to_string().into()),
            Message::Text("{".into()),
        ],
        "four consecutive bad frames",
    )
    .await;
}

/// The tolerance is not unlimited: a peer that only ever sends garbage is broken, and the client
/// should surface that rather than spin silently. The stream ends instead of hanging forever.
#[tokio::test]
async fn a_persistently_broken_peer_still_ends_the_stream() {
    let flood: Vec<Message> = (0..30).map(|_| Message::Text("not json".into())).collect();
    let url = start_server_sending(flood).await;

    let ws = EnhancedWebsocket::new(&url, Some(30), None).await.unwrap();
    let (mut stream, _unsub) = ws.transaction_subscribe(subscription_config()).await.unwrap();

    let next = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("the client should give up on a peer sending only garbage, not hang");

    assert!(
        next.is_none(),
        "expected the stream to end once the unusable-frame tolerance was exceeded"
    );
}

/// The tolerance counts *consecutive* unusable frames, not lifetime ones. Without the reset, any
/// long-lived connection would die once it had seen ten bad frames in total, however far apart.
///
/// Sends well past the threshold in total, but never more than one in a row.
#[tokio::test]
async fn bad_frames_interleaved_with_good_ones_never_trip_the_threshold() {
    let notification = |slot: u64| {
        Message::Text(
            json!({
                "jsonrpc": "2.0",
                "method": "transactionNotification",
                "params": {
                    "result": {
                        "signature": "3Tf7QH4w9mK3rT6uV8xZ1cD5eF7gH9jK2LmN4pQ6sTuW8XaB2cDe4Fg6Hj8Km9Np2Qr4St6Uv8Wx7Yz2aBc4DeF6",
                        "slot": slot,
                        "transactionIndex": 1u64
                    },
                    "subscription": 1
                }
            })
            .to_string()
            .into(),
        )
    };

    // 24 bad frames in total — more than double the tolerance — but never two in a row.
    let mut interference = Vec::new();
    for slot in 0..24u64 {
        interference.push(Message::Text("not json".into()));
        interference.push(notification(slot));
    }

    let url = start_server_sending(interference).await;
    let ws = EnhancedWebsocket::new(&url, Some(30), None).await.unwrap();
    let (mut stream, _unsub) = ws.transaction_subscribe(subscription_config()).await.unwrap();

    // Every interleaved notification, plus the trailing one the server always sends.
    for expected_slot in 0..24u64 {
        let event = timeout(Duration::from_secs(2), stream.next())
            .await
            .unwrap_or_else(|_| panic!("timed out at slot {expected_slot}: the counter did not reset"))
            .unwrap_or_else(|| panic!("stream ended at slot {expected_slot}: the counter did not reset"));

        match event {
            TransactionNotification::Signature(entry) => assert_eq!(entry.slot, expected_slot),
            other => panic!("expected a signature notification, got {other:?}"),
        }
    }
}
