use bytes::Bytes;
use helius::config::Config;
use helius::error::{HeliusError, Result};
use helius::request_handler::decode_response;
use helius::rpc_client::RpcClient;
use helius::types::parsed_events::{
    ComparisonBounds, GetTransactionHistoryRequest, GetTransactionHistoryResponse, ParseTransactionsRequest,
    ParsedEventsCommitment, ParsedEventsSortOrder, ParsedEventsStatusFilter, ParsedTransactionResult,
    ParsedTransactionStatus, ParserStatus, RawTransactionEncoding, ScanBudget, ScanCountResponse, ScanMatch,
    ScanProgramMatch, ScanRequest, ScanResponse, ScanSignature, ScanSignaturesResponse, ScanStopReason,
    TokenAccountsFilter,
};
use helius::types::{ApiKey, Cluster, HeliusEndpoints};
use helius::{Helius, HeliusBuilder};
use mockito::{Matcher, Server};
use reqwest::{Client, StatusCode};
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
            "rawTransaction": "jsonParsed"
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
            raw_transaction: Some(RawTransactionEncoding::JsonParsed),
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

#[test]
fn test_raw_transaction_encodings_use_the_api_spelling() {
    for (encoding, wire) in [
        (RawTransactionEncoding::None, "none"),
        (RawTransactionEncoding::Json, "json"),
        (RawTransactionEncoding::JsonParsed, "jsonParsed"),
        (RawTransactionEncoding::Base64, "base64"),
    ] {
        assert_eq!(serde_json::to_value(encoding).unwrap(), wire);
    }
    for (filter, wire) in [
        (TokenAccountsFilter::None, "none"),
        (TokenAccountsFilter::BalanceChanged, "balanceChanged"),
        (TokenAccountsFilter::All, "all"),
    ] {
        assert_eq!(serde_json::to_value(filter).unwrap(), wire);
    }
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
            "status": "any",
            "tokenAccounts": "balanceChanged",
            "minContextSlot": 433000000,
            "rawTransaction": "base64",
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
            status: Some(ParsedEventsStatusFilter::Any),
            token_accounts: Some(TokenAccountsFilter::BalanceChanged),
            min_context_slot: Some(433_000_000),
            raw_transaction: Some(RawTransactionEncoding::Base64),
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

const PUMP_FUN: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

fn pump_fun_scan(instructions: &[&str]) -> ScanRequest {
    ScanRequest {
        address: PUMP_FUN.to_string(),
        criteria: ScanMatch {
            programs: Some(vec![ScanProgramMatch {
                program_id: PUMP_FUN.to_string(),
                instructions: Some(instructions.iter().map(|name| name.to_string()).collect()),
                ..Default::default()
            }]),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[tokio::test]
async fn test_scan_posts_match_criteria_and_returns_matches_with_a_summary() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server
        .mock("POST", "/v1/parsed-events/scan?api-key=test-key")
        .match_body(Matcher::Json(json!({
            "address": PUMP_FUN,
            "match": {
                "programs": [{ "programId": PUMP_FUN, "instructions": ["buy", "sell"] }],
                "mints": [USDC]
            },
            "limit": 1,
            "budget": { "maxScannedTransactions": 2000 },
            "sortOrder": "asc",
            "commitment": "finalized",
            "status": "any",
            "tokenAccounts": "all",
            "minContextSlot": 460000000,
            "rawTransaction": "json",
            "time": { "gte": 1735689600 },
            "paginationToken": "460869102:203"
        })))
        .with_status(200)
        .with_body(
            json!({
                "data": parse_transactions_body().as_array().unwrap()[..1],
                "paginationToken": "460869102:204",
                "scan": {
                    "stoppedOn": "limit",
                    "scannedTransactions": 1,
                    "matchedTransactions": 1,
                    "scannedThrough": { "slot": 460869102, "blockTime": 1792848886 },
                    "creditsCharged": 11
                }
            })
            .to_string(),
        )
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let mut request: ScanRequest = pump_fun_scan(&["buy", "sell"]);
    request.criteria.mints = Some(vec![USDC.to_string()]);
    let page: ScanResponse = helius
        .parsed_events()
        .scan(ScanRequest {
            limit: Some(1),
            budget: Some(ScanBudget {
                max_scanned_transactions: Some(2000),
            }),
            sort_order: Some(ParsedEventsSortOrder::Asc),
            commitment: Some(ParsedEventsCommitment::Finalized),
            status: Some(ParsedEventsStatusFilter::Any),
            token_accounts: Some(TokenAccountsFilter::All),
            min_context_slot: Some(460_000_000),
            raw_transaction: Some(RawTransactionEncoding::Json),
            time: Some(ComparisonBounds {
                gte: Some(1_735_689_600),
                ..Default::default()
            }),
            pagination_token: Some("460869102:203".to_string()),
            ..request
        })
        .await
        .expect("scan should succeed");

    mock.assert();
    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].signature, SIG);
    assert_eq!(page.data[0].parser_status, ParserStatus::Ok);
    assert_eq!(page.pagination_token.as_deref(), Some("460869102:204"));
    assert_eq!(page.scan.stopped_on, ScanStopReason::Limit);
    assert_eq!(page.scan.scanned_transactions, 1);
    assert_eq!(page.scan.matched_transactions, 1);
    assert_eq!(page.scan.credits_charged, 11);
    let through = page.scan.scanned_through.expect("scanned_through");
    assert_eq!((through.slot, through.block_time), (460_869_102, Some(1_792_848_886)));
}

/// A budget stop with no matches is still a page to continue from, not the end
#[tokio::test]
async fn test_scan_budget_stop_returns_an_empty_page_with_a_token() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/scan?api-key=test-key")
        .with_status(200)
        .with_body(
            json!({
                "data": [],
                "paginationToken": "460903064:422",
                "scan": {
                    "stoppedOn": "budget",
                    "scannedTransactions": 2000,
                    "matchedTransactions": 0,
                    "scannedThrough": { "slot": 460903064, "blockTime": 1792856105 },
                    "creditsCharged": 12
                }
            })
            .to_string(),
        )
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let page: ScanResponse = helius
        .parsed_events()
        .scan(pump_fun_scan(&["set_params"]))
        .await
        .unwrap();

    assert!(page.data.is_empty());
    assert_eq!(page.pagination_token.as_deref(), Some("460903064:422"));
    assert_eq!(page.scan.stopped_on, ScanStopReason::Budget);
    assert_eq!(page.scan.scanned_transactions, 2000);
}

#[tokio::test]
async fn test_scan_window_end_has_no_token_and_tolerates_unknown_values() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/scan?api-key=test-key")
        .with_status(200)
        .with_body(
            json!({
                "data": [],
                "scan": { "stoppedOn": "windowEnd", "scannedTransactions": 0, "matchedTransactions": 0, "creditsCharged": 10 }
            })
            .to_string(),
        )
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let page: ScanResponse = helius.parsed_events().scan(pump_fun_scan(&["buy"])).await.unwrap();
    assert_eq!(page.scan.stopped_on, ScanStopReason::WindowEnd);
    assert_eq!(page.pagination_token, None);
    assert_eq!(page.scan.scanned_through, None);

    // A stop reason added to the API later does not fail the page
    let page: ScanResponse = decode_response(
        json!({ "data": [], "scan": { "stoppedOn": "somethingNew", "creditsCharged": 10 } })
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    assert_eq!(page.scan.stopped_on, ScanStopReason::Unknown);
}

/// Unset options and an empty cursor stay out of the body, since the server rejects unknown or
/// null fields
#[tokio::test]
async fn test_scan_omits_unset_options_and_an_empty_pagination_token() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server
        .mock("POST", "/v1/parsed-events/scan?api-key=test-key")
        .match_body(Matcher::Json(json!({
            "address": ADDRESS,
            "match": { "mints": [USDC] }
        })))
        .with_status(200)
        .with_body(r#"{"data":[],"scan":{"stoppedOn":"windowEnd"}}"#)
        .expect(2)
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    for pagination_token in [None, Some(String::new())] {
        helius
            .parsed_events()
            .scan(ScanRequest {
                address: ADDRESS.to_string(),
                criteria: ScanMatch {
                    mints: Some(vec![USDC.to_string()]),
                    ..Default::default()
                },
                pagination_token,
                ..Default::default()
            })
            .await
            .unwrap();
    }

    mock.assert();
}

/// Several programs, each with names, discriminators, both, or neither
#[tokio::test]
async fn test_scan_serializes_several_programs() {
    const JUPITER: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server
        .mock("POST", "/v1/parsed-events/scan?api-key=test-key")
        .match_body(Matcher::Json(json!({
            "address": ADDRESS,
            "match": {
                "programs": [
                    { "programId": PUMP_FUN, "instructions": ["buy"], "discriminators": ["0x66063d1201daebea"] },
                    { "programId": JUPITER }
                ]
            }
        })))
        .with_status(200)
        .with_body(r#"{"data":[],"scan":{"stoppedOn":"windowEnd"}}"#)
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    helius
        .parsed_events()
        .scan(ScanRequest {
            address: ADDRESS.to_string(),
            criteria: ScanMatch {
                programs: Some(vec![
                    ScanProgramMatch {
                        program_id: PUMP_FUN.to_string(),
                        instructions: Some(vec!["buy".to_string()]),
                        discriminators: Some(vec!["0x66063d1201daebea".to_string()]),
                    },
                    ScanProgramMatch {
                        program_id: JUPITER.to_string(),
                        ..Default::default()
                    },
                ]),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();

    mock.assert();
}

/// The signatures shape sends the same request plus `returnShape`, and returns signature rows
#[tokio::test]
async fn test_scan_signatures_sends_the_shape_and_returns_signature_rows() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server
        .mock("POST", "/v1/parsed-events/scan?api-key=test-key")
        .match_body(Matcher::Json(json!({
            "address": PUMP_FUN,
            "match": { "programs": [{ "programId": PUMP_FUN, "instructions": ["buy"] }] },
            "tokenAccounts": "balanceChanged",
            "returnShape": "signatures"
        })))
        .with_status(200)
        .with_body(
            json!({
                "data": [
                    { "signature": "sig-a", "slot": 460869102, "transactionIndex": 203, "blockTime": 1792848886 },
                    { "signature": "sig-b", "slot": 460869090, "transactionIndex": 7, "blockTime": null }
                ],
                "paginationToken": "460868000:0",
                "scan": {
                    "stoppedOn": "budget",
                    "scannedTransactions": 10000,
                    "matchedTransactions": 2,
                    "scannedThrough": { "slot": 460868000, "blockTime": 1792848000 },
                    "creditsCharged": 20
                }
            })
            .to_string(),
        )
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let page: ScanSignaturesResponse = helius
        .parsed_events()
        .scan_signatures(ScanRequest {
            token_accounts: Some(TokenAccountsFilter::BalanceChanged),
            ..pump_fun_scan(&["buy"])
        })
        .await
        .unwrap();

    mock.assert();
    assert_eq!(
        page.data,
        vec![
            ScanSignature {
                signature: "sig-a".to_string(),
                slot: 460_869_102,
                transaction_index: 203,
                block_time: Some(1_792_848_886),
            },
            ScanSignature {
                signature: "sig-b".to_string(),
                slot: 460_869_090,
                transaction_index: 7,
                block_time: None,
            },
        ]
    );
    assert_eq!(page.pagination_token.as_deref(), Some("460868000:0"));
    assert_eq!(page.scan.stopped_on, ScanStopReason::Budget);
    assert_eq!(page.scan.matched_transactions, 2);
}

/// The count shape has no `data`; the count is `scan.matched_transactions`
#[tokio::test]
async fn test_scan_count_sends_the_shape_and_reads_the_count() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server
        .mock("POST", "/v1/parsed-events/scan?api-key=test-key")
        .match_body(Matcher::Json(json!({
            "address": ADDRESS,
            "match": { "mints": [USDC] },
            "returnShape": "count"
        })))
        .with_status(200)
        .with_body(
            json!({
                "scan": {
                    "stoppedOn": "windowEnd",
                    "scannedTransactions": 812,
                    "matchedTransactions": 37,
                    "scannedThrough": { "slot": 460000000, "blockTime": 1792000000 },
                    "creditsCharged": 11
                }
            })
            .to_string(),
        )
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let page: ScanCountResponse = helius
        .parsed_events()
        .scan_count(ScanRequest {
            address: ADDRESS.to_string(),
            criteria: ScanMatch {
                mints: Some(vec![USDC.to_string()]),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();

    mock.assert();
    assert_eq!(page.pagination_token, None);
    assert_eq!(page.scan.stopped_on, ScanStopReason::WindowEnd);
    assert_eq!(page.scan.matched_transactions, 37);
    assert_eq!(page.scan.scanned_transactions, 812);
}

#[tokio::test]
async fn test_scan_maps_a_rejected_request_to_bad_request() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/scan?api-key=test-key")
        .with_status(400)
        .with_body(
            r#"{"error":"match.program.instructions not found in the IDLs of 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P: not_a_real_ix"}"#,
        )
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let err: HeliusError = helius
        .parsed_events()
        .scan(pump_fun_scan(&["not_a_real_ix"]))
        .await
        .unwrap_err();

    match err {
        HeliusError::BadRequest { path, text } => {
            assert_eq!(path, "/v1/parsed-events/scan");
            assert!(text.contains("not_a_real_ix"), "{text}");
        }
        other => panic!("expected BadRequest, got {other:?}"),
    }
}

#[tokio::test]
async fn test_scan_surfaces_a_retryable_503() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/scan?api-key=test-key")
        .with_status(503)
        .with_body(r#"{"error":"service unavailable"}"#)
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let err: HeliusError = helius.parsed_events().scan(pump_fun_scan(&["buy"])).await.unwrap_err();

    match err {
        HeliusError::Unknown { code, text } => {
            assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
            assert_eq!(text, "service unavailable");
        }
        other => panic!("expected Unknown, got {other:?}"),
    }
}

#[tokio::test]
async fn test_scan_raw_decodes_off_runtime() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/v1/parsed-events/scan?api-key=test-key")
        .with_status(200)
        .with_body(
            json!({
                "data": parse_transactions_body(),
                "paginationToken": "next",
                "scan": { "stoppedOn": "deadline", "scannedTransactions": 3000, "matchedTransactions": 2, "creditsCharged": 13 }
            })
            .to_string(),
        )
        .create();

    let helius: Helius = helius_for(&server, Cluster::MainnetBeta, Some("test-key"));
    let body: Bytes = helius.parsed_events().scan_raw(pump_fun_scan(&["buy"])).await.unwrap();

    let page: ScanResponse = tokio::task::spawn_blocking(move || decode_response(&body))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(page.data.len(), 2);
    assert_eq!(page.scan.stopped_on, ScanStopReason::Deadline);
    assert_eq!(page.scan.credits_charged, 13);
}

#[tokio::test]
async fn test_scan_rejects_devnet_without_a_request() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server.mock("POST", Matcher::Any).expect(0).create();

    let helius: Helius = helius_for(&server, Cluster::Devnet, Some("test-key"));
    match helius.parsed_events().scan(pump_fun_scan(&["buy"])).await {
        Err(HeliusError::InvalidInput(msg)) => assert_eq!(msg, "Parsed Events is only available on mainnet"),
        other => panic!("expected InvalidInput, got {other:?}"),
    }
    mock.assert();
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
