use bytes::Bytes;
use helius::config::Config;
use helius::error::{HeliusError, Result};
use helius::request_handler::decode_response;
use helius::rpc_client::RpcClient;
use helius::types::parsed_events::{
    ComparisonBounds, GetTransactionHistoryRequest, GetTransactionHistoryResponse, ParseTransactionsRequest,
    ParsedEventsCommitment, ParsedEventsSortOrder, ParsedTransactionResult, ParsedTransactionStatus, ParserStatus,
};
use helius::types::{ApiKey, Cluster, HeliusEndpoints};
use helius::{Helius, HeliusBuilder};
use mockito::{Matcher, Server};
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;

const SIG: &str = "5xSKzM8bvpudE521jikHqASzMr23Ms4X4ieY3K8oFPFrJWCSSgYocJmHznrR8b12voDxKDH8ykdCLXSRrx6duVLH";
const ADDRESS: &str = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY";

fn helius_for(server: &Server, cluster: Cluster, api_key: Option<&str>) -> Helius {
    let url: String = format!("{}/", server.url());
    let config: Arc<Config> = Arc::new(Config {
        api_key: api_key.map(|key| ApiKey::new(key).unwrap()),
        cluster,
        endpoints: HeliusEndpoints {
            api: url.clone(),
            rpc: url,
        },
        custom_url: None,
    });

    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    }
}

fn parse_transactions_body() -> serde_json::Value {
    json!([
        {
            "signature": SIG,
            "parserStatus": "OK",
            "parsed": {
                "slot": 433950192,
                "blockTime": 1784487060,
                "fee": 2005000,
                "feePayer": "GV6UUmNxz2RpKxmNAPadYKb7uQpszwqQAu3qLJxVdC52",
                "transactionStatus": "OK",
                "error": null,
                "decodedError": null,
                "nativeTransfers": [],
                "tokenTransfers": [{
                    "fromUserAccount": "GV6UUmNxz2RpKxmNAPadYKb7uQpszwqQAu3qLJxVdC52",
                    "toUserAccount": "7iWnBRRhBCiNXXPhqiGzvvBkKrvFSWqqmxRyu9VyYBxE",
                    "fromTokenAccount": "teBPkZtsnaKDpGHTy1KUZeh1PbVtbrKXhzruYpRiUUs",
                    "toTokenAccount": "2oL6my4QDDCfpgJZX1bZV1NgbmuNptKdgcE8wJm6efgk",
                    "rawTokenAmount": 1498500000000u64,
                    "decimals": 9,
                    "tokenStandard": "Fungible",
                    "mint": "So11111111111111111111111111111111111111112"
                }],
                "summary": {
                    "type": "swap",
                    "description": "Swapped SOL for USDC",
                    "parsedData": { "protocol": "jupiter" }
                },
                "instructions": [{
                    "instructionIndex": 0,
                    "innerInstructionIndex": null,
                    "stackHeight": 1,
                    "programId": "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
                    "rawAccounts": ["teBPkZtsnaKDpGHTy1KUZeh1PbVtbrKXhzruYpRiUUs"],
                    "rawData": "3Bxs4h24hBtQy9rw",
                    "summary": null,
                    "programName": "token_2022",
                    "instructionName": "transfer_checked",
                    "decoded": {
                        "args": { "amount": "1498500000000", "decimals": 9 },
                        "accounts": [{
                            "name": "source",
                            "pubkey": "teBPkZtsnaKDpGHTy1KUZeh1PbVtbrKXhzruYpRiUUs",
                            "isSigner": false,
                            "isWritable": true
                        }]
                    }
                }]
            }
        },
        {
            "signature": "missing",
            "parserStatus": "ERROR",
            "parserError": { "code": "transaction_not_found", "message": "transaction not found" }
        }
    ])
}

#[tokio::test]
async fn test_parse_transactions_posts_signatures_and_returns_per_item_results() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server
        .mock("POST", "/v1/parsed-events/transactions?api-key=test-key")
        .match_body(Matcher::Json(json!({
            "transactions": [SIG, "missing"],
            "commitment": "finalized",
            "includeRawTransaction": true
        })))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(parse_transactions_body().to_string())
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let results: Vec<ParsedTransactionResult> = helius
        .parsed_events()
        .parse_transactions(ParseTransactionsRequest {
            transactions: vec![SIG.to_string(), "missing".to_string()],
            commitment: Some(ParsedEventsCommitment::Finalized),
            include_raw_transaction: Some(true),
        })
        .await
        .expect("parse_transactions should succeed");

    mock.assert();
    assert_eq!(results.len(), 2);

    let ok = &results[0];
    assert_eq!(ok.parser_status, ParserStatus::Ok);
    let parsed = ok.parsed.as_ref().expect("parsed should be set");
    assert_eq!(parsed.slot, 433950192);
    assert_eq!(parsed.transaction_status, ParsedTransactionStatus::Ok);
    assert_eq!(parsed.token_transfers[0].raw_token_amount, 1_498_500_000_000);
    let summary = parsed.summary.as_ref().unwrap();
    assert_eq!(summary.summary_type, "swap");
    assert_eq!(summary.parsed_data.as_ref().unwrap()["protocol"], "jupiter");
    let instruction = &parsed.instructions[0];
    assert_eq!(instruction.inner_instruction_index, None);
    assert_eq!(instruction.instruction_name.as_deref(), Some("transfer_checked"));
    let decoded = instruction.decoded.as_ref().unwrap();
    assert_eq!(decoded.args["amount"], "1498500000000");
    assert!(decoded.accounts[0].is_writable);

    let error = &results[1];
    assert_eq!(error.parser_status, ParserStatus::Error);
    assert!(error.parsed.is_none());
    assert_eq!(error.parser_error.as_ref().unwrap().code, "transaction_not_found");
}

#[tokio::test]
async fn test_parse_transactions_omits_unset_options() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server
        .mock("POST", "/v1/parsed-events/transactions?api-key=test-key")
        .match_body(Matcher::Json(json!({ "transactions": [SIG] })))
        .with_status(200)
        .with_body("[]")
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let results = helius
        .parsed_events()
        .parse_transactions(ParseTransactionsRequest {
            transactions: vec![SIG.to_string()],
            ..Default::default()
        })
        .await
        .unwrap();

    mock.assert();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_parse_transactions_tolerates_an_unknown_status() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/transactions?api-key=test-key")
        .with_status(200)
        .with_body(json!([{ "signature": SIG, "parserStatus": "PENDING" }]).to_string())
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let results = helius
        .parsed_events()
        .parse_transactions(ParseTransactionsRequest {
            transactions: vec![SIG.to_string()],
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(results[0].parser_status, ParserStatus::Unknown);
    assert_eq!(serde_json::to_value(ParserStatus::Unknown).unwrap(), "UNKNOWN");
}

#[tokio::test]
async fn test_get_transaction_history_posts_address_and_filters() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server
        .mock("POST", "/v1/parsed-events/transaction-history?api-key=test-key")
        .match_body(Matcher::Json(json!({
            "address": ADDRESS,
            "limit": 50,
            "sortOrder": "asc",
            "slot": { "gte": 433000000 },
            "time": { "lt": 1784487060 },
            "paginationToken": "433000000:0"
        })))
        .with_status(200)
        .with_body(
            json!({
                "data": [{
                    "signature": "sig1",
                    "parserStatus": "ERROR",
                    "parserError": { "code": "transaction_not_found", "message": "not found" }
                }],
                "paginationToken": "433950192:12"
            })
            .to_string(),
        )
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let page: GetTransactionHistoryResponse = helius
        .parsed_events()
        .get_transaction_history(GetTransactionHistoryRequest {
            address: ADDRESS.to_string(),
            limit: Some(50),
            sort_order: Some(ParsedEventsSortOrder::Asc),
            slot: Some(ComparisonBounds {
                gte: Some(433_000_000),
                ..Default::default()
            }),
            time: Some(ComparisonBounds {
                lt: Some(1_784_487_060),
                ..Default::default()
            }),
            pagination_token: Some("433000000:0".to_string()),
            ..Default::default()
        })
        .await
        .expect("get_transaction_history should succeed");

    mock.assert();
    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].signature, "sig1");
    assert_eq!(page.pagination_token.as_deref(), Some("433950192:12"));
}

#[tokio::test]
async fn test_get_transaction_history_final_page_has_no_token() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/transaction-history?api-key=test-key")
        .with_status(200)
        .with_body(r#"{"data":[]}"#)
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let page = helius
        .parsed_events()
        .get_transaction_history(GetTransactionHistoryRequest {
            address: ADDRESS.to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    assert!(page.data.is_empty());
    assert_eq!(page.pagination_token, None);
}

/// An empty cursor means "first page", same as no cursor, so it is left out of the body
#[tokio::test]
async fn test_get_transaction_history_omits_an_empty_pagination_token() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server
        .mock("POST", "/v1/parsed-events/transaction-history?api-key=test-key")
        .match_body(Matcher::Json(json!({ "address": ADDRESS })))
        .with_status(200)
        .with_body(r#"{"data":[],"paginationToken":null}"#)
        .expect(2)
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    for pagination_token in [None, Some(String::new())] {
        helius
            .parsed_events()
            .get_transaction_history(GetTransactionHistoryRequest {
                address: ADDRESS.to_string(),
                pagination_token,
                ..Default::default()
            })
            .await
            .unwrap();
    }

    mock.assert();
}

#[tokio::test]
async fn test_get_transaction_history_raw_decodes_off_runtime() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/transaction-history?api-key=test-key")
        .with_status(200)
        .with_body(json!({ "data": parse_transactions_body(), "paginationToken": "next" }).to_string())
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let body: Bytes = helius
        .parsed_events()
        .get_transaction_history_raw(GetTransactionHistoryRequest {
            address: ADDRESS.to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let page: GetTransactionHistoryResponse = tokio::task::spawn_blocking(move || decode_response(&body))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(page.data.len(), 2);
    assert_eq!(page.pagination_token.as_deref(), Some("next"));
}

#[tokio::test]
async fn test_parse_transactions_raw_returns_body() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/transactions?api-key=test-key")
        .with_status(200)
        .with_body(parse_transactions_body().to_string())
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let body: Bytes = helius
        .parsed_events()
        .parse_transactions_raw(ParseTransactionsRequest {
            transactions: vec![SIG.to_string()],
            ..Default::default()
        })
        .await
        .unwrap();

    let results: Vec<ParsedTransactionResult> = decode_response(&body).unwrap();
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_parse_transactions_maps_api_error() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/transactions?api-key=test-key")
        .with_status(400)
        .with_body(r#"{"error":"invalid request payload"}"#)
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let err: HeliusError = helius
        .parsed_events()
        .parse_transactions(ParseTransactionsRequest::default())
        .await
        .unwrap_err();

    match err {
        HeliusError::BadRequest { path, text } => {
            assert_eq!(path, "/v1/parsed-events/transactions");
            assert_eq!(text, "invalid request payload");
        }
        other => panic!("expected BadRequest, got {other:?}"),
    }
}

#[tokio::test]
async fn test_parse_transactions_maps_gateway_auth_error() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/transactions?api-key=test-key")
        .with_status(401)
        .with_body(r#"{"jsonrpc":"2.0","error":{"code":-32401,"message":"invalid api key provided"}}"#)
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let err: HeliusError = helius
        .parsed_events()
        .parse_transactions(ParseTransactionsRequest::default())
        .await
        .unwrap_err();

    match err {
        HeliusError::Unauthorized { text, .. } => assert!(text.contains("invalid api key provided"), "{text}"),
        other => panic!("expected Unauthorized, got {other:?}"),
    }
}

#[tokio::test]
async fn test_parsed_events_requires_an_api_key() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server.mock("POST", Matcher::Any).expect(0).create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, None);
    let result: Result<Vec<ParsedTransactionResult>> = helius
        .parsed_events()
        .parse_transactions(ParseTransactionsRequest::default())
        .await;

    match result {
        Err(HeliusError::InvalidInput(msg)) => assert!(msg.contains("Parsed Events"), "{msg}"),
        other => panic!("expected InvalidInput, got {other:?}"),
    }
    mock.assert();
}

#[tokio::test]
async fn test_parsed_events_rejects_devnet_cluster_without_a_request() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server.mock("POST", Matcher::Any).expect(0).create();

    let helius: Helius = helius_for(&server, Cluster::Devnet, Some("test-key"));
    let parse = helius
        .parsed_events()
        .parse_transactions(ParseTransactionsRequest::default())
        .await;
    let history = helius
        .parsed_events()
        .get_transaction_history(GetTransactionHistoryRequest {
            address: ADDRESS.to_string(),
            ..Default::default()
        })
        .await;

    for err in [parse.unwrap_err(), history.unwrap_err()] {
        match err {
            HeliusError::InvalidInput(msg) => assert_eq!(msg, "Parsed Events is only available on mainnet"),
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }
    mock.assert();
}

#[tokio::test]
async fn test_parsed_events_rejects_a_devnet_custom_url() {
    for url in [
        "https://devnet.helius-rpc.com/",
        "https://abc-fast-devnet.helius-rpc.com/",
        "https://api.devnet.solana.com/",
    ] {
        let helius: Helius = HeliusBuilder::new()
            .with_custom_url(url)
            .unwrap()
            .with_api_key("test-key")
            .unwrap()
            .with_cluster(Cluster::MainnetBeta)
            .build()
            .await
            .unwrap();

        let result = helius
            .parsed_events()
            .parse_transactions(ParseTransactionsRequest::default())
            .await;
        assert!(
            matches!(result, Err(HeliusError::InvalidInput(ref msg)) if msg.contains("only available on mainnet")),
            "{url}: {result:?}"
        );
    }
}

/// A custom URL defaults the cluster to Devnet, so a mainnet one must opt in with `with_cluster`
#[tokio::test]
async fn test_parsed_events_custom_url_needs_an_explicit_mainnet_cluster() {
    let helius: Helius = HeliusBuilder::new()
        .with_custom_url("https://rpc.example.com/")
        .unwrap()
        .with_api_key("test-key")
        .unwrap()
        .build()
        .await
        .unwrap();

    match helius
        .parsed_events()
        .parse_transactions(ParseTransactionsRequest::default())
        .await
    {
        Err(HeliusError::InvalidInput(msg)) => assert!(msg.contains("with_cluster(Cluster::MainnetBeta)"), "{msg}"),
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}
