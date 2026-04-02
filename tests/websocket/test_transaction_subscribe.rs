use futures_util::{SinkExt, StreamExt};
use helius::types::{
    enhanced_websocket::TransactionDetails, RpcTransactionsConfig, TransactionCommitment, TransactionNotification,
    TransactionSubscribeFilter, TransactionSubscribeOptions, UiEnhancedTransactionEncoding,
};
use helius::websocket::EnhancedWebsocket;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::{accept_async, tungstenite::Message};

async fn start_mock_ws_server(transaction_notification: Value) -> String {
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

                        match method {
                            "transactionSubscribe" => {
                                let ack = json!({
                                    "jsonrpc": "2.0",
                                    "result": 1,
                                    "id": id
                                });
                                ws.send(Message::Text(ack.to_string().into())).await.unwrap();

                                let notification = json!({
                                    "jsonrpc": "2.0",
                                    "method": "transactionNotification",
                                    "params": {
                                        "result": transaction_notification,
                                        "subscription": 1
                                    }
                                });
                                ws.send(Message::Text(notification.to_string().into())).await.unwrap();
                            }
                            "transactionUnsubscribe" => {
                                let response = json!({
                                    "jsonrpc": "2.0",
                                    "result": true,
                                    "id": id
                                });
                                ws.send(Message::Text(response.to_string().into())).await.unwrap();
                                let _ = ws.close(None).await;
                                break;
                            }
                            _ => {}
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

fn subscription_config(details: TransactionDetails) -> RpcTransactionsConfig {
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
            transaction_details: Some(details),
            show_rewards: Some(false),
            max_supported_transaction_version: Some(0),
        },
    }
}

#[tokio::test]
async fn transaction_subscribe_decodes_signature_notifications() {
    let url = start_mock_ws_server(json!({
        "signature": "3Tf7QH4w9mK3rT6uV8xZ1cD5eF7gH9jK2LmN4pQ6sTuW8XaB2cDe4Fg6Hj8Km9Np2Qr4St6Uv8Wx7Yz2aBc4DeF6",
        "slot": 123456u64,
        "transactionIndex": 17u64
    }))
    .await;

    let ws = EnhancedWebsocket::new(&url, Some(30), None).await.unwrap();
    let (mut stream, _unsub) = ws
        .transaction_subscribe(subscription_config(TransactionDetails::Signatures))
        .await
        .unwrap();

    let event = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("timed out waiting for signature notification")
        .expect("stream ended before signature notification");

    match event {
        TransactionNotification::Signature(entry) => {
            assert_eq!(entry.slot, 123456);
            assert_eq!(entry.transaction_index, 17);
            assert_eq!(
                entry.signature,
                "3Tf7QH4w9mK3rT6uV8xZ1cD5eF7gH9jK2LmN4pQ6sTuW8XaB2cDe4Fg6Hj8Km9Np2Qr4St6Uv8Wx7Yz2aBc4DeF6"
            );
        }
        other => panic!("expected signature notification, got {other:?}"),
    }
}

#[tokio::test]
async fn transaction_subscribe_decodes_full_notifications() {
    let url = start_mock_ws_server(json!({
        "signature": "5Gh8Jk2LmN4pQ6sTuW8XaB2cDe4Fg6Hj8Km9Np2Qr4St6Uv8Wx7Yz2aBc4DeF6Gh7Jk9Lm2Np4Qr6St8Uv2Wx4Yz",
        "slot": 123457u64,
        "transactionIndex": 18u64,
        "transaction": {
            "meta": {
                "err": null,
                "fee": 1005000u64,
                "preBalances": [1, 2, 3],
                "postBalances": [1, 2, 3],
                "status": { "Ok": null }
            },
            "transaction": [
                "AQ==",
                "base64"
            ],
            "version": 0
        }
    }))
    .await;

    let ws = EnhancedWebsocket::new(&url, Some(30), None).await.unwrap();
    let (mut stream, _unsub) = ws
        .transaction_subscribe(subscription_config(TransactionDetails::Full))
        .await
        .unwrap();

    let event = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("timed out waiting for full notification")
        .expect("stream ended before full notification");

    match event {
        TransactionNotification::Full(entry) => {
            assert_eq!(entry.slot, 123457);
            assert_eq!(entry.transaction_index, 18);
            assert_eq!(
                entry.signature,
                "5Gh8Jk2LmN4pQ6sTuW8XaB2cDe4Fg6Hj8Km9Np2Qr4St6Uv8Wx7Yz2aBc4DeF6Gh7Jk9Lm2Np4Qr6St8Uv2Wx4Yz"
            );
        }
        other => panic!("expected full notification, got {other:?}"),
    }
}

#[tokio::test]
async fn transaction_subscribe_decodes_accounts_notifications() {
    let url = start_mock_ws_server(json!({
        "signature": "7Jk9Lm2Np4Qr6St8Uv2Wx4Yz6Bc8De1Fg3Hj5Km7Np9Qr2St4Uv6Wx8Yz1Bc3De5Fg7Hj9Km2Np4Qr6St8Uv2Wx",
        "slot": 123458u64,
        "transactionIndex": 19u64,
        "transaction": {
            "meta": {
                "err": null,
                "fee": 2005000u64,
                "preBalances": [10, 20, 30],
                "postBalances": [11, 21, 31],
                "status": { "Ok": null }
            },
            "transaction": {
                "accountKeys": [
                    {
                        "pubkey": "9Ab3Cd5Ef7Gh9Jk2Lm4Np6Qr8St1Uv3Wx5Yz7bCe9DfG",
                        "signer": true,
                        "source": "transaction",
                        "writable": true
                    },
                    {
                        "pubkey": "2Qr4St6Uv8Wx1Yz3Bc5De7Fg9Hj2Km4Np6Qr8St1Uv3W",
                        "signer": false,
                        "source": "lookupTable",
                        "writable": false
                    }
                ],
                "signatures": [
                    "7Jk9Lm2Np4Qr6St8Uv2Wx4Yz6Bc8De1Fg3Hj5Km7Np9Qr2St4Uv6Wx8Yz1Bc3De5Fg7Hj9Km2Np4Qr6St8Uv2Wx"
                ]
            },
            "version": 0
        }
    }))
    .await;

    let ws = EnhancedWebsocket::new(&url, Some(30), None).await.unwrap();
    let (mut stream, _unsub) = ws
        .transaction_subscribe(subscription_config(TransactionDetails::Accounts))
        .await
        .unwrap();

    let event = timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("timed out waiting for accounts notification")
        .expect("stream ended before accounts notification");

    match event {
        TransactionNotification::Full(entry) => {
            assert_eq!(entry.slot, 123458);
            assert_eq!(entry.transaction_index, 19);
            assert_eq!(
                entry.signature,
                "7Jk9Lm2Np4Qr6St8Uv2Wx4Yz6Bc8De1Fg3Hj5Km7Np9Qr2St4Uv6Wx8Yz1Bc3De5Fg7Hj9Km2Np4Qr6St8Uv2Wx"
            );
            assert!(matches!(
                entry.transaction.transaction,
                solana_transaction_status::EncodedTransaction::Accounts(_)
            ));
        }
        other => panic!("expected full notification carrying accounts payload, got {other:?}"),
    }
}
