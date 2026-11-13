use super::helpers::mock_helius;

use helius::error::{HeliusError, Result};
use helius::types::inner::RpcRequest;
use helius::types::*;

use mockito::{self, Matcher, Mock, Server};
use serde_json::{json, Value};
use solana_commitment_config::CommitmentLevel;

const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const OWNER: &str = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY";
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_ACCOUNT: &str = "EKyGnXReqmmXyqBHmMjV2CKCLafJS47xyhXXZ5WuFMxD";

fn params_of<R: serde::Serialize + std::fmt::Debug>(method: &str, request: R) -> Value {
    serde_json::to_value(RpcRequest::new(method.to_string(), request)).unwrap()["params"].clone()
}

fn account_json(lamports: u64, last_slot: u64, last_tx: Option<u32>) -> Value {
    json!({
        "lamports": lamports,
        "data": ["", "base64"],
        "owner": TOKEN_PROGRAM,
        "executable": false,
        "rentEpoch": 18446744073709551615u64,
        "space": 82,
        "lastUpdatedAt": { "slot": last_slot, "transactionIndex": last_tx }
    })
}

fn rpc_result(result: Value) -> String {
    json!({ "jsonrpc": "2.0", "id": "helius-rust-sdk", "result": result }).to_string()
}

fn token_page(pubkeys: &[&str], pagination_key: Option<&str>) -> String {
    let accounts: Vec<Value> = pubkeys
        .iter()
        .map(|pubkey| json!({ "pubkey": pubkey, "account": account_json(2039280, 444999837, Some(700)) }))
        .collect();
    rpc_result(json!({
        "context": { "slot": 468129571 },
        "value": { "accounts": accounts, "paginationKey": pagination_key }
    }))
}

async fn mock_rpc(server: &mut Server, body: String, hits: usize) -> Mock {
    server
        .mock("POST", "/?api-key=fake_api_key")
        .expect(hits)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create_async()
        .await
}

#[test]
fn test_historical_account_info_request_serialization() {
    let config = GetHistoricalAccountInfoConfig {
        encoding: Some(Encoding::JsonParsed),
        data_slice: Some(DataSlice { offset: 0, length: 8 }),
        commitment: Some(CommitmentLevel::Confirmed),
        min_context_slot: Some(445_000_000),
    };
    let request = GetHistoricalAccountInfoRequest::new(
        USDC_MINT.to_string(),
        HistoricalAccountQuery::after_transaction(445_000_000, 10),
        Some(config),
    );

    assert_eq!(
        params_of("getHistoricalAccountInfo", request),
        json!([
            USDC_MINT,
            { "slot": 445000000, "transactionIndex": 10 },
            {
                "encoding": "jsonParsed",
                "dataSlice": { "length": 8, "offset": 0 },
                "commitment": "confirmed",
                "minContextSlot": 445000000
            }
        ])
    );
}

#[test]
fn test_historical_account_info_request_omits_optional_config() {
    let latest = GetHistoricalAccountInfoRequest::new(USDC_MINT.to_string(), HistoricalAccountQuery::latest(), None);
    assert_eq!(params_of("getHistoricalAccountInfo", latest), json!([USDC_MINT, {}]));

    let at_slot = GetHistoricalAccountInfoRequest::new(
        USDC_MINT.to_string(),
        HistoricalAccountQuery::at_slot(445_000_000),
        None,
    );
    assert_eq!(
        params_of("getHistoricalAccountInfo", at_slot),
        json!([USDC_MINT, { "slot": 445000000 }])
    );
}

#[test]
fn test_historical_token_accounts_request_serialization() {
    let pagination_key = format!("{}:{}", USDC_MINT, TOKEN_ACCOUNT);
    let config = GetHistoricalTokenAccountsByOwnerConfig {
        filter: Some(TokenAccountsOwnerFilter::Mint {
            mint: USDC_MINT.to_string(),
        }),
        encoding: Some(Encoding::Base64Zstd),
        limit: Some(1000),
        pagination_key: Some(pagination_key.clone()),
        commitment: Some(CommitmentLevel::Finalized),
        max_pages: Some(3),
        ..Default::default()
    };
    let request = GetHistoricalTokenAccountsByOwnerRequest::new(
        OWNER.to_string(),
        HistoricalAccountQuery::at_slot(445_000_000),
        Some(config),
    );

    assert_eq!(
        params_of("getHistoricalTokenAccountsByOwner", request),
        json!([
            OWNER,
            { "slot": 445000000 },
            {
                "mint": USDC_MINT,
                "encoding": "base64+zstd",
                "limit": 1000,
                "paginationKey": pagination_key,
                "commitment": "finalized"
            }
        ])
    );
}

#[test]
fn test_historical_token_accounts_program_filter_and_empty_config() {
    let program = GetHistoricalTokenAccountsByOwnerConfig {
        filter: Some(TokenAccountsOwnerFilter::Program {
            program_id: TOKEN_PROGRAM.to_string(),
        }),
        ..Default::default()
    };
    let request = GetHistoricalTokenAccountsByOwnerRequest::new(
        OWNER.to_string(),
        HistoricalAccountQuery::latest(),
        Some(program),
    );
    assert_eq!(
        params_of("getHistoricalTokenAccountsByOwner", request),
        json!([OWNER, {}, { "programId": TOKEN_PROGRAM }])
    );

    let empty = GetHistoricalTokenAccountsByOwnerRequest::new(
        OWNER.to_string(),
        HistoricalAccountQuery::latest(),
        Some(GetHistoricalTokenAccountsByOwnerConfig::default()),
    );
    assert_eq!(
        params_of("getHistoricalTokenAccountsByOwner", empty),
        json!([OWNER, {}, {}])
    );
}

#[test]
fn test_historical_token_accounts_config_deserializes_filter() {
    let config: GetHistoricalTokenAccountsByOwnerConfig =
        serde_json::from_value(json!({ "mint": USDC_MINT, "limit": 5 })).unwrap();
    assert!(matches!(config.filter, Some(TokenAccountsOwnerFilter::Mint { ref mint }) if mint == USDC_MINT));
    assert_eq!(config.limit, Some(5));

    let unfiltered: GetHistoricalTokenAccountsByOwnerConfig = serde_json::from_value(json!({ "limit": 5 })).unwrap();
    assert!(unfiltered.filter.is_none());
}

#[test]
fn test_historical_account_query_positions() {
    assert_eq!(
        HistoricalAccountQuery::before_transaction(445_000_000, 10),
        HistoricalAccountQuery::after_transaction(445_000_000, 9)
    );
    assert_eq!(
        HistoricalAccountQuery::before_transaction(445_000_000, 0),
        HistoricalAccountQuery::at_slot(444_999_999)
    );

    let transaction_write = AccountWritePosition {
        slot: 444_999_837,
        transaction_index: Some(700),
    };
    assert_eq!(
        HistoricalAccountQuery::before(&transaction_write),
        HistoricalAccountQuery::after_transaction(444_999_837, 699)
    );

    let reward_write = AccountWritePosition {
        slot: 445_000_000,
        transaction_index: None,
    };
    assert_eq!(
        HistoricalAccountQuery::before(&reward_write),
        HistoricalAccountQuery::at_slot(444_999_999)
    );
}

#[tokio::test]
async fn test_get_historical_account_info_success() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    // The response the docs captured from the archival router
    let mock = server
        .mock("POST", "/?api-key=fake_api_key")
        .match_body(Matcher::Json(json!({
            "jsonrpc": "2.0",
            "id": "helius-rust-sdk",
            "method": "getHistoricalAccountInfo",
            "params": [USDC_MINT, { "slot": 445000000 }, { "dataSlice": { "offset": 0, "length": 0 } }]
        })))
        .with_status(200)
        .with_body(rpc_result(json!({
            "context": { "slot": 468129574 },
            "value": account_json(531589475273, 444999837, Some(700))
        })))
        .create_async()
        .await;

    let helius = mock_helius(&server.url());
    let config = GetHistoricalAccountInfoConfig {
        data_slice: Some(DataSlice { offset: 0, length: 0 }),
        ..Default::default()
    };
    let response = helius
        .rpc()
        .get_historical_account_info(
            USDC_MINT.to_string(),
            HistoricalAccountQuery::at_slot(445_000_000),
            Some(config),
        )
        .await
        .unwrap();

    mock.assert_async().await;
    assert_eq!(response.context.slot, 468129574);
    let account = response.value.unwrap();
    assert_eq!(account.lamports, 531589475273);
    assert_eq!(account.owner, TOKEN_PROGRAM);
    assert_eq!(account.data, json!(["", "base64"]));
    assert!(!account.executable);
    assert_eq!(account.rent_epoch, u64::MAX);
    assert_eq!(account.space, 82);
    assert_eq!(
        account.last_updated_at,
        AccountWritePosition {
            slot: 444999837,
            transaction_index: Some(700)
        }
    );
}

#[tokio::test]
async fn test_get_historical_account_info_null_value() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    mock_rpc(
        &mut server,
        rpc_result(json!({ "context": { "slot": 1 }, "value": null })),
        1,
    )
    .await;

    let response = mock_helius(&server.url())
        .rpc()
        .get_historical_account_info(USDC_MINT.to_string(), HistoricalAccountQuery::latest(), None)
        .await
        .unwrap();

    assert!(response.value.is_none());
}

#[tokio::test]
async fn test_get_historical_account_info_unchanged_since_coverage() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    mock_rpc(
        &mut server,
        rpc_result(json!({
            "context": { "slot": 1 },
            "value": account_json(1, HISTORICAL_ACCOUNT_COVERAGE_START_SLOT, None)
        })),
        1,
    )
    .await;

    let account = mock_helius(&server.url())
        .rpc()
        .get_historical_account_info(USDC_MINT.to_string(), HistoricalAccountQuery::latest(), None)
        .await
        .unwrap()
        .value
        .unwrap();

    assert_eq!(
        account.last_updated_at,
        AccountWritePosition {
            slot: HISTORICAL_ACCOUNT_COVERAGE_START_SLOT,
            transaction_index: None
        }
    );
}

#[tokio::test]
async fn test_get_historical_account_info_before_coverage_rpc_error() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    mock_rpc(
        &mut server,
        json!({
            "jsonrpc": "2.0",
            "id": "helius-rust-sdk",
            "error": { "code": -32018, "message": "Account history not available" }
        })
        .to_string(),
        1,
    )
    .await;

    let result: Result<GetHistoricalAccountInfoResponse> = mock_helius(&server.url())
        .rpc()
        .get_historical_account_info(
            USDC_MINT.to_string(),
            HistoricalAccountQuery::at_slot(444_000_000),
            None,
        )
        .await;

    match result {
        Err(HeliusError::RpcError { code, message }) => {
            assert_eq!(code, -32018);
            assert_eq!(message, "Account history not available");
        }
        other => panic!("expected RpcError -32018, got {:?}", other),
    }
}

#[tokio::test]
async fn test_get_historical_token_accounts_by_owner_success() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let next_key = format!("{}:{}", USDC_MINT, TOKEN_ACCOUNT);
    let mock = server
        .mock("POST", "/?api-key=fake_api_key")
        .match_body(Matcher::Json(json!({
            "jsonrpc": "2.0",
            "id": "helius-rust-sdk",
            "method": "getHistoricalTokenAccountsByOwner",
            "params": [OWNER, { "slot": 445000000 }, { "encoding": "jsonParsed", "limit": 2 }]
        })))
        .with_status(200)
        .with_body(token_page(&[TOKEN_ACCOUNT], Some(&next_key)))
        .create_async()
        .await;

    let config = GetHistoricalTokenAccountsByOwnerConfig {
        encoding: Some(Encoding::JsonParsed),
        limit: Some(2),
        ..Default::default()
    };
    let response = mock_helius(&server.url())
        .rpc()
        .get_historical_token_accounts_by_owner(
            OWNER.to_string(),
            HistoricalAccountQuery::at_slot(445_000_000),
            Some(config),
        )
        .await
        .unwrap();

    mock.assert_async().await;
    assert_eq!(response.context.slot, 468129571);
    assert_eq!(response.value.accounts.len(), 1);
    assert_eq!(response.value.accounts[0].pubkey, TOKEN_ACCOUNT);
    assert_eq!(response.value.accounts[0].account.lamports, 2039280);
    assert_eq!(response.value.pagination_key, Some(next_key));
}

#[tokio::test]
async fn test_get_all_historical_token_accounts_follows_cursor_through_empty_page() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;

    // Every page must carry the same slot and filter; the first must drop the caller's stale cursor
    let mut page_mock = |pagination_key: Option<&str>, body: String| {
        let mut config = json!({ "programId": TOKEN_PROGRAM });
        if let Some(key) = pagination_key {
            config["paginationKey"] = json!(key);
        }
        server
            .mock("POST", "/?api-key=fake_api_key")
            .match_body(Matcher::Json(json!({
                "jsonrpc": "2.0",
                "id": "helius-rust-sdk",
                "method": "getHistoricalTokenAccountsByOwner",
                "params": [OWNER, { "slot": 445000000 }, config]
            })))
            .with_status(200)
            .with_body(body)
            .expect(1)
            .create()
    };
    let pages = [
        page_mock(None, token_page(&["Account1"], Some("MintA:Account1"))),
        page_mock(Some("MintA:Account1"), token_page(&[], Some("MintB:Account9"))),
        page_mock(Some("MintB:Account9"), token_page(&["Account2"], None)),
    ];

    let config = GetHistoricalTokenAccountsByOwnerConfig {
        filter: Some(TokenAccountsOwnerFilter::Program {
            program_id: TOKEN_PROGRAM.to_string(),
        }),
        pagination_key: Some("stale".to_string()),
        ..Default::default()
    };
    let accounts = mock_helius(&server.url())
        .rpc()
        .get_all_historical_token_accounts_by_owner(
            OWNER.to_string(),
            HistoricalAccountQuery::at_slot(445_000_000),
            config,
        )
        .await
        .unwrap();

    for page in pages {
        page.assert_async().await;
    }
    let pubkeys: Vec<&str> = accounts.iter().map(|account| account.pubkey.as_str()).collect();
    assert_eq!(pubkeys, vec!["Account1", "Account2"]);
}

#[tokio::test]
async fn test_get_all_historical_token_accounts_respects_max_pages() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = mock_rpc(&mut server, token_page(&["Account1"], Some("MintA:Account1")), 1).await;

    let config = GetHistoricalTokenAccountsByOwnerConfig {
        max_pages: Some(1),
        ..Default::default()
    };
    let accounts = mock_helius(&server.url())
        .rpc()
        .get_all_historical_token_accounts_by_owner(
            OWNER.to_string(),
            HistoricalAccountQuery::at_slot(445_000_000),
            config,
        )
        .await
        .unwrap();

    mock.assert_async().await;
    assert_eq!(accounts.len(), 1);
}

#[tokio::test]
async fn test_get_all_historical_token_accounts_stops_on_non_advancing_cursor() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = mock_rpc(&mut server, token_page(&["Account1"], Some("MintA:Account1")), 2).await;

    let accounts = mock_helius(&server.url())
        .rpc()
        .get_all_historical_token_accounts_by_owner(
            OWNER.to_string(),
            HistoricalAccountQuery::at_slot(445_000_000),
            GetHistoricalTokenAccountsByOwnerConfig::default(),
        )
        .await
        .unwrap();

    mock.assert_async().await;
    assert_eq!(accounts.len(), 2);
}

#[tokio::test]
async fn test_get_historical_token_accounts_by_owner_failure() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create_async()
        .await;

    let response: Result<GetHistoricalTokenAccountsByOwnerResponse> = mock_helius(&server.url())
        .rpc()
        .get_historical_token_accounts_by_owner(OWNER.to_string(), HistoricalAccountQuery::latest(), None)
        .await;

    assert!(matches!(response, Err(HeliusError::InternalError { .. })));
}

#[tokio::test]
async fn test_historical_token_accounts_rejects_out_of_range_limit_without_a_request() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = mock_rpc(&mut server, token_page(&["Account1"], None), 0).await;
    let helius = mock_helius(&server.url());

    for limit in [0, HISTORICAL_TOKEN_ACCOUNTS_MAX_LIMIT + 1, 100_000] {
        let config = GetHistoricalTokenAccountsByOwnerConfig {
            limit: Some(limit),
            ..Default::default()
        };
        let page = helius
            .rpc()
            .get_historical_token_accounts_by_owner(
                OWNER.to_string(),
                HistoricalAccountQuery::at_slot(445_000_000),
                Some(config.clone()),
            )
            .await;
        assert!(
            matches!(page, Err(HeliusError::InvalidInput(_))),
            "limit {limit}: {page:?}"
        );

        let all = helius
            .rpc()
            .get_all_historical_token_accounts_by_owner(
                OWNER.to_string(),
                HistoricalAccountQuery::at_slot(445_000_000),
                config,
            )
            .await;
        assert!(
            matches!(all, Err(HeliusError::InvalidInput(_))),
            "limit {limit}: {all:?}"
        );
    }

    mock.assert_async().await;
}

#[tokio::test]
async fn test_historical_token_accounts_accepts_max_limit() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let mock = server
        .mock("POST", "/?api-key=fake_api_key")
        .match_body(Matcher::PartialJson(json!({
            "params": [OWNER, { "slot": 445000000 }, { "limit": 25000 }]
        })))
        .with_status(200)
        .with_body(token_page(&["Account1"], None))
        .create_async()
        .await;

    let config = GetHistoricalTokenAccountsByOwnerConfig {
        limit: Some(HISTORICAL_TOKEN_ACCOUNTS_MAX_LIMIT),
        ..Default::default()
    };
    let response = mock_helius(&server.url())
        .rpc()
        .get_historical_token_accounts_by_owner(
            OWNER.to_string(),
            HistoricalAccountQuery::at_slot(445_000_000),
            Some(config),
        )
        .await
        .unwrap();

    mock.assert_async().await;
    assert_eq!(response.value.accounts.len(), 1);
}
