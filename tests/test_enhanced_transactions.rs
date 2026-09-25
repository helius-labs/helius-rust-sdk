use bytes::Bytes;
use helius::config::Config;
use helius::error::{HeliusError, Result};
use helius::request_handler::decode_response;
use helius::rpc_client::RpcClient;
use helius::types::{
    AccountData, ApiKey, Cluster, EnhancedTransaction, HeliusEndpoints, InnerInstruction, Instruction, NativeTransfer,
    ParseTransactionsRequest, ParsedTransactionHistoryRequest, Source, TokenStandard, TokenTransfer, TransactionEvent,
    TransactionType, TransferUserAccounts,
};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use serde_json::Number;
use solana_commitment_config::CommitmentLevel;
use std::sync::Arc;

fn create_test_helius(url: &str) -> Helius {
    let config: Arc<Config> = Arc::new(Config {
        api_key: Some(ApiKey::new("fake_api_key").unwrap()),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
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

fn create_mock_transaction(signature: &str, tx_type: TransactionType, source: Source) -> EnhancedTransaction {
    EnhancedTransaction {
        account_data: vec![AccountData {
            account: "".to_string(),
            native_token_balance: Some(Number::from(10)),
            token_balance_changes: None,
        }],
        description: "Human readable interpretation of the transaction".to_string(),
        transaction_type: tx_type,
        source,
        fee: 5000,
        fee_payer: "8cRrU1NzNpjL3k2BwjW3VixAcX6VFc29KHr4KZg8cs2Y".to_string(),
        signature: signature.to_string(),
        slot: 148277128,
        native_transfers: None,
        token_transfers: Some(vec![TokenTransfer {
            user_accounts: TransferUserAccounts {
                from_user_account: Some("2iK5FbRZcJHAfhUNYmYdKTzSLnZE9NGAECurPoxDA3o7".to_string()),
                to_user_account: Some("DCAKxn5PFNN1mBREPWGdk1RXg5aVH9rPErLfBFEi2Emb".to_string()),
            },
            from_token_account: Some("DCAKxn5PFNN1mBREPWGdk1RXg5aVH9rPErLfBFEi2Emb".to_string()),
            to_token_account: Some("Bbnr95sKEcgWHHdD6UEU7MDek419KgMP1tWYUPP61fJk".to_string()),
            token_amount: Number::from(32),
            token_standard: TokenStandard::Fungible,
            mint: "6oCioNHNTh4Xoz33mQSoTW4mxxmnyWVBNgv7zHjUuBkK".to_string(),
        }]),
        transaction_error: None,
        instructions: vec![Instruction {
            accounts: vec![],
            data: "kdL8HQJrbbvQRGXmoadaja1Qvs".to_string(),
            program_id: "MEisE1HzehtrDpAAT8PnLHjpSSkRYakotTuJRPjTpo8".to_string(),
            inner_instructions: vec![InnerInstruction {
                accounts: vec![],
                data: "Dd1k91cWt84qJoQr3F".to_string(),
                program_id: "Dd1k91cWt84qJoQr3FT7EXQpSaMtZtwPwdho7RbMWtEV".to_string(),
            }],
        }],
        events: TransactionEvent::default(),
        timestamp: 1656442333,
    }
}

#[tokio::test]
async fn test_parse_transactions_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![create_mock_transaction(
        "yy5BT9benHhx8fGCvhcAfTtLEHAtRJ3hRTzVL16bdrTCWm63t2vapfrZQZLJC3RcuagekaXjSs2zUGQvbcto8DK",
        TransactionType::Any,
        Source::Jupiter,
    )];

    server
        .mock("POST", "/v0/transactions?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParseTransactionsRequest = ParseTransactionsRequest {
        transactions: vec![
            "DiG7v24AXRQqGagDx8pcVxRgVrgFoXUpJgp7xb62ycG9".to_string(),
            "46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi".to_string(),
        ],
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parse_transactions(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let tx_response: Vec<EnhancedTransaction> = response.unwrap();
    assert_eq!(
        tx_response[0].signature,
        "yy5BT9benHhx8fGCvhcAfTtLEHAtRJ3hRTzVL16bdrTCWm63t2vapfrZQZLJC3RcuagekaXjSs2zUGQvbcto8DK"
    );
    assert_eq!(tx_response[0].transaction_type, TransactionType::Any);
}

#[tokio::test]
async fn test_parse_transactions_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock("POST", "/v0/transactions?api-key=fake_api_key")
        .with_status(500)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error":"Internal Server Error"}"#)
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParseTransactionsRequest = ParseTransactionsRequest {
        transactions: vec![
            "DiG7v24AXRQqGagDx8pcVxRgVrgFoXUpJgp7xb62ycG9".to_string(),
            "46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi".to_string(),
        ],
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parse_transactions(request).await;
    assert!(response.is_err(), "Expected an error due to server failure");
}

#[tokio::test]
async fn test_parse_transactions_empty_response() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![];

    server
        .mock("POST", "/v0/transactions?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParseTransactionsRequest = ParseTransactionsRequest {
        transactions: vec!["nonexistent_signature".to_string()],
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parse_transactions(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());
    assert!(response.unwrap().is_empty(), "Expected empty response");
}

#[tokio::test]
async fn test_parse_transactions_multiple_results() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![
        create_mock_transaction(
            "sig1aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            TransactionType::Transfer,
            Source::SystemProgram,
        ),
        create_mock_transaction(
            "sig2aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            TransactionType::Swap,
            Source::Jupiter,
        ),
    ];

    server
        .mock("POST", "/v0/transactions?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParseTransactionsRequest = ParseTransactionsRequest {
        transactions: vec![
            "sig1aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            "sig2aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        ],
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parse_transactions(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let txs: Vec<EnhancedTransaction> = response.unwrap();
    assert_eq!(txs.len(), 2, "Expected two transactions");
    assert_eq!(txs[0].signature, "sig1aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert_eq!(txs[0].transaction_type, TransactionType::Transfer);
    assert_eq!(txs[0].source, Source::SystemProgram);
    assert_eq!(txs[1].signature, "sig2aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    assert_eq!(txs[1].transaction_type, TransactionType::Swap);
    assert_eq!(txs[1].source, Source::Jupiter);
}

#[tokio::test]
async fn test_parse_transactions_with_native_transfers() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mut tx = create_mock_transaction(
        "sig_native_aaaaaaaaaaaaaaaaaaaaaaaaaaa",
        TransactionType::Transfer,
        Source::SystemProgram,
    );
    tx.native_transfers = Some(vec![NativeTransfer {
        user_accounts: TransferUserAccounts {
            from_user_account: Some("8cRrU1NzNpjL3k2BwjW3VixAcX6VFc29KHr4KZg8cs2Y".to_string()),
            to_user_account: Some("DCAKxn5PFNN1mBREPWGdk1RXg5aVH9rPErLfBFEi2Emb".to_string()),
        },
        amount: Number::from(1_000_000_000),
    }]);
    tx.token_transfers = None;

    let mock_response: Vec<EnhancedTransaction> = vec![tx];

    server
        .mock("POST", "/v0/transactions?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParseTransactionsRequest = ParseTransactionsRequest {
        transactions: vec!["sig_native_aaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()],
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parse_transactions(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let txs: Vec<EnhancedTransaction> = response.unwrap();
    assert!(txs[0].token_transfers.is_none());
    let native = txs[0].native_transfers.as_ref().unwrap();
    assert_eq!(native.len(), 1);
    assert_eq!(native[0].amount, Number::from(1_000_000_000));
    assert_eq!(
        native[0].user_accounts.from_user_account.as_deref(),
        Some("8cRrU1NzNpjL3k2BwjW3VixAcX6VFc29KHr4KZg8cs2Y")
    );
}

#[tokio::test]
async fn test_parse_transactions_detailed_field_assertions() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![create_mock_transaction(
        "sig_detail_aaaaaaaaaaaaaaaaaaaaaaaaaaa",
        TransactionType::Any,
        Source::Jupiter,
    )];

    server
        .mock("POST", "/v0/transactions?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParseTransactionsRequest = ParseTransactionsRequest {
        transactions: vec!["sig_detail_aaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()],
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parse_transactions(request).await;
    let txs: Vec<EnhancedTransaction> = response.unwrap();
    let tx = &txs[0];

    assert_eq!(tx.fee, 5000);
    assert_eq!(tx.fee_payer, "8cRrU1NzNpjL3k2BwjW3VixAcX6VFc29KHr4KZg8cs2Y");
    assert_eq!(tx.slot, 148277128);
    assert_eq!(tx.timestamp, 1656442333);
    assert!(tx.transaction_error.is_none());

    assert_eq!(tx.account_data.len(), 1);
    assert_eq!(tx.account_data[0].native_token_balance, Some(Number::from(10)));

    let token_transfer = &tx.token_transfers.as_ref().unwrap()[0];
    assert_eq!(token_transfer.token_amount, Number::from(32));
    assert_eq!(token_transfer.token_standard, TokenStandard::Fungible);
    assert_eq!(token_transfer.mint, "6oCioNHNTh4Xoz33mQSoTW4mxxmnyWVBNgv7zHjUuBkK");

    assert_eq!(tx.instructions.len(), 1);
    assert_eq!(tx.instructions[0].inner_instructions.len(), 1);
    assert_eq!(
        tx.instructions[0].inner_instructions[0].program_id,
        "Dd1k91cWt84qJoQr3FT7EXQpSaMtZtwPwdho7RbMWtEV"
    );
}

#[tokio::test]
async fn test_parse_transaction_history_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![create_mock_transaction(
        "yy5BT9benHhx8fGCvhcAfTtLEHAtRJ3hRTzVL16bdrTCWm63t2vapfrZQZLJC3RcuagekaXjSs2zUGQvbcto8DK",
        TransactionType::Any,
        Source::Jupiter,
    )];

    server
        .mock(
            "GET",
            "/v0/addresses/46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi/transactions?api-key=fake_api_key",
        )
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
        address: "46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi".to_string(),
        before: None,
        until: None,
        transaction_type: None,
        commitment: None,
        limit: None,
        source: None,
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parsed_transaction_history(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let tx_response: Vec<EnhancedTransaction> = response.unwrap();
    assert_eq!(
        tx_response[0].signature,
        "yy5BT9benHhx8fGCvhcAfTtLEHAtRJ3hRTzVL16bdrTCWm63t2vapfrZQZLJC3RcuagekaXjSs2zUGQvbcto8DK"
    );
    assert_eq!(tx_response[0].transaction_type, TransactionType::Any);
}

#[tokio::test]
async fn test_parse_transaction_history_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock(
            "GET",
            "/v0/addresses/46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi/transactions?api-key=fake_api_key",
        )
        .with_status(500)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error":"Internal Server Error"}"#)
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
        address: "46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi".to_string(),
        before: None,
        until: None,
        transaction_type: None,
        commitment: None,
        limit: None,
        source: None,
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parsed_transaction_history(request).await;
    assert!(response.is_err(), "Expected an error due to server failure");
}

#[tokio::test]
async fn test_parse_transaction_history_with_query_params() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![create_mock_transaction(
        "sig_filtered_aaaaaaaaaaaaaaaaaaaaaaaaaaa",
        TransactionType::Swap,
        Source::Jupiter,
    )];

    server
        .mock("GET", mockito::Matcher::Regex(
            r"/v0/addresses/46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi/transactions\?api-key=fake_api_key&before=.*&until=.*&commitment=.*&source=.*&type=.*&limit=.*".to_string()
        ))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
        address: "46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi".to_string(),
        before: Some("before_sig_aaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()),
        until: Some("until_sig_aaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()),
        commitment: Some(CommitmentLevel::Finalized),
        source: Some(Source::Jupiter),
        transaction_type: Some(TransactionType::Swap),
        limit: Some(10),
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parsed_transaction_history(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let txs: Vec<EnhancedTransaction> = response.unwrap();
    assert_eq!(txs.len(), 1);
    assert_eq!(txs[0].transaction_type, TransactionType::Swap);
}

#[tokio::test]
async fn test_parse_transaction_history_empty_response() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![];

    server
        .mock(
            "GET",
            "/v0/addresses/EmptyWallet1111111111111111111111111111111/transactions?api-key=fake_api_key",
        )
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
        address: "EmptyWallet1111111111111111111111111111111".to_string(),
        before: None,
        until: None,
        transaction_type: None,
        commitment: None,
        limit: None,
        source: None,
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parsed_transaction_history(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());
    assert!(response.unwrap().is_empty(), "Expected empty transaction history");
}

/// The raw variant hits the same URL as the typed one and returns the body untouched;
/// `decode_response` then yields exactly what `parsed_transaction_history` would have, and can
/// do so on the blocking pool.
#[tokio::test]
async fn test_parsed_transaction_history_raw_returns_body_and_decodes_off_runtime() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![create_mock_transaction(
        "yy5BT9benHhx8fGCvhcAfTtLEHAtRJ3hRTzVL16bdrTCWm63t2vapfrZQZLJC3RcuagekaXjSs2zUGQvbcto8DK",
        TransactionType::Swap,
        Source::Jupiter,
    )];
    let body: String = serde_json::to_string(&mock_response).unwrap();

    server
        .mock("GET", mockito::Matcher::Regex(
            r"/v0/addresses/46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi/transactions\?api-key=fake_api_key&type=SWAP&limit=100".to_string()
        ))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(&body)
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
        address: "46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi".to_string(),
        before: None,
        until: None,
        transaction_type: Some(TransactionType::Swap),
        commitment: None,
        limit: Some(100),
        source: None,
    };

    let raw: Bytes = helius
        .parsed_transaction_history_raw(request)
        .await
        .expect("raw history call failed");
    assert_eq!(raw.as_ref(), body.as_bytes(), "raw body must be returned unmodified");

    let decoded: Vec<EnhancedTransaction> =
        tokio::task::spawn_blocking(move || decode_response::<Vec<EnhancedTransaction>>(&raw))
            .await
            .expect("blocking decode task should complete")
            .expect("body should decode");

    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].signature, mock_response[0].signature);
    assert_eq!(decoded[0].transaction_type, TransactionType::Swap);
    assert_eq!(decoded[0].source, Source::Jupiter);
}

/// A failure status on the raw history path is still mapped to the matching error.
#[tokio::test]
async fn test_parsed_transaction_history_raw_maps_failure_status() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock(
            "GET",
            "/v0/addresses/46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi/transactions?api-key=fake_api_key",
        )
        .with_status(429)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error":"Too many requests"}"#)
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
        address: "46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi".to_string(),
        before: None,
        until: None,
        transaction_type: None,
        commitment: None,
        limit: None,
        source: None,
    };

    let response: Result<Bytes> = helius.parsed_transaction_history_raw(request).await;
    assert!(
        matches!(response, Err(HeliusError::RateLimitExceeded { .. })),
        "expected RateLimitExceeded, got {response:?}"
    );
}

/// The raw parse variant posts the same request to the same URL and returns the body untouched.
#[tokio::test]
async fn test_parse_transactions_raw_returns_body() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![create_mock_transaction(
        "2sShYqqcWAcJiGc3oK74iFsYKgLCNiY2DsivMbaJGQT8pRzR8z5iBcdmTMXRobH8cZNZgeV9Ur9VjvLsykfFE2Li",
        TransactionType::Transfer,
        Source::SystemProgram,
    )];
    let body: String = serde_json::to_string(&mock_response).unwrap();

    server
        .mock("POST", "/v0/transactions?api-key=fake_api_key")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "transactions": ["2sShYqqcWAcJiGc3oK74iFsYKgLCNiY2DsivMbaJGQT8pRzR8z5iBcdmTMXRobH8cZNZgeV9Ur9VjvLsykfFE2Li"]
        })))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(&body)
        .create();

    let helius: Helius = create_test_helius(&url);
    let request: ParseTransactionsRequest = ParseTransactionsRequest {
        transactions: vec![
            "2sShYqqcWAcJiGc3oK74iFsYKgLCNiY2DsivMbaJGQT8pRzR8z5iBcdmTMXRobH8cZNZgeV9Ur9VjvLsykfFE2Li".to_string(),
        ],
    };

    let raw: Bytes = helius
        .parse_transactions_raw(request)
        .await
        .expect("raw parse call failed");
    assert_eq!(raw.as_ref(), body.as_bytes(), "raw body must be returned unmodified");

    let decoded: Vec<EnhancedTransaction> = decode_response(&raw).expect("body should decode");
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].signature, mock_response[0].signature);
    assert_eq!(decoded[0].transaction_type, TransactionType::Transfer);
    assert_eq!(decoded[0].source, Source::SystemProgram);
}
