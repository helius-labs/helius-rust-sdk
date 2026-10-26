use futures_util::{SinkExt, StreamExt};
use helius::error::HeliusError;
use helius::parsed_streams::{ParsedStreamsClient, ParsedStreamsOptions};
use helius::types::parsed_streams::{
    ParsedStreamDetails, ParsedStreamTransactionStatus, ParsedStreamsCommitment, ParsedTransactionAccountsFilter,
    ParsedTransactionFilter, ParsedTransactionSubscribeOptions,
};
use helius::types::Cluster;
use helius::{Helius, HeliusBuilder};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::{accept_async, tungstenite::Message};

const JUPITER: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

/// The `details: "full"` notification from the Parsed Streams quickstart
fn docs_notification(subscription: u64) -> Value {
    let summary = json!({
        "type": "swap",
        "description": "6jduWNCTQzG91JGBchfGGxd55Vi5FxJCCJEV18RkXzJX swapped 0.001 SOL for 0.183985 EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v via Jupiter",
        "parsedData": {
            "type": "swap",
            "protocol": "jupiter",
            "kind": "swap",
            "in_amount": "1000000",
            "actual_out_amount": "183985",
            "input_mint": "So11111111111111111111111111111111111111112",
            "output_mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        }
    });

    json!({
        "jsonrpc": "2.0",
        "method": "parsedTransactionNotification",
        "params": {
            "subscription": subscription,
            "result": {
                "context": { "slot": 430172053 },
                "value": {
                    "transaction": {
                        "signature": "3riSYL4HTRxgQjLayt6L2JPaDR3oaEQg1H4v3fnjUxNU",
                        "slot": 430172053,
                        "blockTime": null,
                        "feePayer": "6jduWNCTQzG91JGBchfGGxd55Vi5FxJCCJEV18RkXzJX",
                        "fee": 5000,
                        "accountKeys": ["6jduWNCTQzG91JGBchfGGxd55Vi5FxJCCJEV18RkXzJX", "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh"],
                        "status": "ok",
                        "error": null,
                        "summary": summary,
                        "nativeTransfers": [
                            { "fromUserAccount": "6jduWNCTQzG91JGBchfGGxd55Vi5FxJCCJEV18RkXzJX", "toUserAccount": "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh", "amount": 1000000 }
                        ],
                        "tokenTransfers": [
                            {
                                "fromUserAccount": "6jduWNCTQzG91JGBchfGGxd55Vi5FxJCCJEV18RkXzJX",
                                "toUserAccount": "AeUfFU6L",
                                "fromTokenAccount": "HLaEoW1s",
                                "toTokenAccount": "G13P9kSY",
                                "rawTokenAmount": 183985,
                                "decimals": 6,
                                "tokenStandard": "Fungible",
                                "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
                            }
                        ]
                    },
                    "instructions": [
                        {
                            "instructionIndex": 4,
                            "innerInstructionIndex": null,
                            "stackHeight": 1,
                            "programId": JUPITER,
                            "programName": "jupiter",
                            "instructionName": "route",
                            "summary": summary,
                            "decoded": {
                                "args": { "in_amount": "1000000", "slippage_bps": 50 },
                                "accounts": [
                                    { "name": "user_transfer_authority", "pubkey": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", "isSigner": true, "isWritable": false }
                                ]
                            }
                        },
                        {
                            "instructionIndex": 4,
                            "innerInstructionIndex": 0,
                            "stackHeight": 2,
                            "programId": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
                            "programName": null,
                            "instructionName": null,
                            "summary": null,
                            "decoded": null,
                            "rawData": "3Bxs4h24hBtQy9rw",
                            "rawAccounts": []
                        }
                    ],
                    "matchedIndexes": [0]
                }
            }
        }
    })
}

/// Starts a local WebSocket server that acks subscribe with id 23, then sends `notification`,
/// and answers unsubscribe with `true`. Each request it receives is forwarded on the returned
/// channel
async fn start_server() -> (String, mpsc::UnboundedReceiver<Value>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    let (requests_sender, requests) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let requests_sender = requests_sender.clone();
            tokio::spawn(async move {
                let mut ws = accept_async(stream).await.unwrap();
                while let Some(Ok(message)) = ws.next().await {
                    let Message::Text(text) = message else { continue };
                    let request: Value = serde_json::from_str(text.as_str()).unwrap();
                    let _ = requests_sender.send(request.clone());
                    let id = request["id"].clone();

                    let replies: Vec<Value> = match request["method"].as_str().unwrap() {
                        "parsedTransactionSubscribe" => {
                            vec![json!({"jsonrpc": "2.0", "id": id, "result": 23}), docs_notification(23)]
                        }
                        "parsedTransactionUnsubscribe" => vec![json!({"jsonrpc": "2.0", "id": id, "result": true})],
                        other => panic!("unexpected method {other}"),
                    };
                    for reply in replies {
                        ws.send(Message::Text(reply.to_string().into())).await.unwrap();
                    }
                }
            });
        }
    });

    (url, requests)
}

#[tokio::test]
async fn test_parsed_streams_over_a_real_websocket() {
    let (url, mut requests) = start_server().await;
    let streams = ParsedStreamsClient::with_options("test-key", ParsedStreamsOptions::default().with_url(url)).unwrap();

    let mut subscription = streams
        .parsed_transaction_subscribe(
            ParsedTransactionFilter {
                programs: Some(vec![JUPITER.to_string()]),
                instruction_names: Some(vec!["route".to_string()]),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();
    assert_eq!(subscription.subscription_id(), Some(23));
    let request = requests.recv().await.unwrap();
    assert_eq!(
        request["params"],
        json!([{ "programs": [JUPITER], "instructionNames": ["route"] }])
    );

    let notification = timeout(Duration::from_secs(5), subscription.next())
        .await
        .expect("no notification")
        .expect("stream ended")
        .expect("notification did not decode");

    assert_eq!(notification.context.slot, 430172053);
    let value = notification.value;
    assert_eq!(value.transaction.status, ParsedStreamTransactionStatus::Ok);
    assert_eq!(value.transaction.fee, 5000);
    assert_eq!(value.transaction.account_keys.len(), 2);
    assert_eq!(value.transaction.summary.as_ref().unwrap().summary_type, "swap");
    assert_eq!(value.transaction.native_transfers[0].amount, 1000000);
    assert_eq!(value.transaction.token_transfers[0].raw_token_amount, 183985);
    assert_eq!(value.matched_indexes, Some(vec![0]));

    let matched = &value.instructions[0];
    assert_eq!(matched.instruction_name.as_deref(), Some("route"));
    assert_eq!(matched.inner_instruction_index, None);
    let decoded = matched.decoded.as_ref().unwrap();
    assert_eq!(decoded.args["in_amount"], "1000000");
    assert_eq!(decoded.accounts[0].name, "user_transfer_authority");

    let undecoded = &value.instructions[1];
    assert!(undecoded.decoded.is_none());
    assert_eq!(undecoded.inner_instruction_index, Some(0));
    assert_eq!(undecoded.raw_data.as_deref(), Some("3Bxs4h24hBtQy9rw"));

    assert!(subscription.unsubscribe().await);
    let request = requests.recv().await.unwrap();
    assert_eq!(request["method"], "parsedTransactionUnsubscribe");
    assert_eq!(request["params"], json!([23]));

    streams.close();
}

#[tokio::test]
async fn test_parsed_streams_reports_a_failed_connection() {
    let streams =
        ParsedStreamsClient::with_options("test-key", ParsedStreamsOptions::default().with_url("ws://127.0.0.1:1"))
            .unwrap();

    let err = streams
        .parsed_transaction_subscribe(ParsedTransactionFilter::default(), None)
        .await
        .unwrap_err();
    assert!(
        matches!(err, HeliusError::WebsocketClosed(ref msg) if msg.contains("connection failed")),
        "{err:?}"
    );
}

#[test]
fn test_parsed_streams_filter_and_options_serialize_to_camel_case() {
    let filter = ParsedTransactionFilter {
        programs: Some(vec![JUPITER.to_string()]),
        instruction_names: Some(vec!["route".to_string(), "shared_accounts_route".to_string()]),
        accounts: Some(ParsedTransactionAccountsFilter {
            include: Some(vec!["So11111111111111111111111111111111111111112".to_string()]),
            roles: Some(BTreeMap::from([(
                "user_transfer_authority".to_string(),
                "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string(),
            )])),
        }),
        include_failed: Some(false),
        include_cpi: Some(true),
    };
    let options = ParsedTransactionSubscribeOptions {
        commitment: Some(ParsedStreamsCommitment::Confirmed),
        details: Some(ParsedStreamDetails::Full),
    };

    // The subscribe request from the quickstart
    assert_eq!(
        json!([filter, options]),
        json!([
            {
                "programs": [JUPITER],
                "instructionNames": ["route", "shared_accounts_route"],
                "accounts": {
                    "include": ["So11111111111111111111111111111111111111112"],
                    "roles": { "user_transfer_authority": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin" }
                },
                "includeFailed": false,
                "includeCpi": true
            },
            { "commitment": "confirmed", "details": "full" }
        ])
    );

    // Unset fields are omitted, since the server rejects unknown or null ones
    assert_eq!(json!(ParsedTransactionFilter::default()), json!({}));
    assert_eq!(json!(ParsedTransactionSubscribeOptions::default()), json!({}));
}

#[test]
fn test_parsed_streams_status_tolerates_unknown_values() {
    let status: ParsedStreamTransactionStatus = serde_json::from_value(json!("pending")).unwrap();
    assert_eq!(status, ParsedStreamTransactionStatus::Unknown);
    let status: ParsedStreamTransactionStatus = serde_json::from_value(json!("error")).unwrap();
    assert_eq!(status, ParsedStreamTransactionStatus::Error);
}

#[tokio::test]
async fn test_helius_parsed_streams_requires_mainnet_and_an_api_key() {
    let mainnet = Helius::new("test-key", Cluster::MainnetBeta).unwrap();
    assert!(mainnet.parsed_streams().is_ok());

    let devnet = Helius::new("test-key", Cluster::Devnet).unwrap();
    match devnet.parsed_streams() {
        Err(HeliusError::InvalidInput(msg)) => assert_eq!(msg, "Parsed Streams is only available on mainnet"),
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let no_key = Helius::new_with_url("https://rpc.example.com/").unwrap();
    match no_key.parsed_streams() {
        Err(HeliusError::InvalidInput(msg)) => assert!(msg.contains("Parsed Streams"), "{msg}"),
        other => panic!("expected InvalidInput, got {other:?}"),
    }

    let devnet_host = HeliusBuilder::new()
        .with_custom_url("https://devnet.helius-rpc.com/")
        .unwrap()
        .with_api_key("test-key")
        .unwrap()
        .with_cluster(Cluster::MainnetBeta)
        .build()
        .await
        .unwrap();
    assert!(matches!(
        devnet_host.parsed_streams(),
        Err(HeliusError::InvalidInput(ref msg)) if msg.contains("only available on mainnet")
    ));
}
