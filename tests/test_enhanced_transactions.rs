use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{
    AccountData, Cluster, EnhancedTransaction, HeliusEndpoints, InnerInstruction, Instruction,
    ParseTransactionsRequest, ParsedTransactionHistoryRequest, Source, TokenStandard, TokenTransfer, TransactionEvent,
    TransactionType, TransferUserAccounts,
};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use serde_json::Number;
use std::sync::Arc;

#[tokio::test]
async fn test_parse_transactions_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![EnhancedTransaction {
        #[allow(deprecated)]
        account_data: vec![AccountData {
            account: "".to_string(),
            native_balance_change: Some(10),
            native_token_balance: Some(Number::from(10)),
            token_balance_changes: None,
        }],
        description: "Human readable interpretation of the transaction".to_string(),
        transaction_type: TransactionType::Any,
        source: Source::Jupiter,
        fee: 5000,
        fee_payer: "8cRrU1NzNpjL3k2BwjW3VixAcX6VFc29KHr4KZg8cs2Y".to_string(),
        signature: "yy5BT9benHhx8fGCvhcAfTtLEHAtRJ3hRTzVL16bdrTCWm63t2vapfrZQZLJC3RcuagekaXjSs2zUGQvbcto8DK"
            .to_string(),
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
    }];

    server
        .mock("POST", "/v0/transactions?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });

    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };

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
        "yy5BT9benHhx8fGCvhcAfTtLEHAtRJ3hRTzVL16bdrTCWm63t2vapfrZQZLJC3RcuagekaXjSs2zUGQvbcto8DK".to_string()
    );
    assert_eq!(tx_response[0].transaction_type, TransactionType::Any);
}

#[tokio::test]
async fn test_parse_transactions_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });
    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };
    let request: ParseTransactionsRequest = ParseTransactionsRequest {
        transactions: vec![
            "DiG7v24AXRQqGagDx8pcVxRgVrgFoXUpJgp7xb62ycG9".to_string(),
            "46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi".to_string(),
        ],
    };
    server
        .mock("POST", "/v0/transactions?api-key=fake_api_key")
        .with_status(500)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error":"Internal Server Error"}"#)
        .create();

    let response: Result<Vec<EnhancedTransaction>> = helius.parse_transactions(request).await;
    assert!(response.is_err(), "Expected an error due to server failure");
}
#[tokio::test]
async fn test_parse_transaction_history_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<EnhancedTransaction> = vec![EnhancedTransaction {
        #[allow(deprecated)]
        account_data: vec![AccountData {
            account: "".to_string(),
            native_balance_change: Some(10),
            native_token_balance: Some(Number::from(10)),
            token_balance_changes: None,
        }],
        description: "Human readable interpretation of the transaction".to_string(),
        transaction_type: TransactionType::Any,
        source: Source::Jupiter,
        fee: 5000,
        fee_payer: "8cRrU1NzNpjL3k2BwjW3VixAcX6VFc29KHr4KZg8cs2Y".to_string(),
        signature: "yy5BT9benHhx8fGCvhcAfTtLEHAtRJ3hRTzVL16bdrTCWm63t2vapfrZQZLJC3RcuagekaXjSs2zUGQvbcto8DK"
            .to_string(),
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
    }];

    server
        .mock(
            "GET",
            "/v0/addresses/46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi/transactions?api-key=fake_api_key",
        )
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });

    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };

    let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
        address: "46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi".to_string(),
        before: None,
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parsed_transaction_history(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let tx_response: Vec<EnhancedTransaction> = response.unwrap();
    assert_eq!(
        tx_response[0].signature,
        "yy5BT9benHhx8fGCvhcAfTtLEHAtRJ3hRTzVL16bdrTCWm63t2vapfrZQZLJC3RcuagekaXjSs2zUGQvbcto8DK".to_string()
    );
    assert_eq!(tx_response[0].transaction_type, TransactionType::Any);
}

#[tokio::test]
async fn test_parse_transaction_history_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });
    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };
    let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
        address: "46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi".to_string(),
        before: None,
    };
    server
        .mock(
            "GET",
            "/v0/addresses/46tC8n6GyWvUjFxpTE9juG5WZ72RXADpPhY4S1d6wvTi/transactions?api-key=fake_api_key",
        )
        .with_status(500)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error":"Internal Server Error"}"#)
        .create();

    let response: Result<Vec<EnhancedTransaction>> = helius.parsed_transaction_history(request).await;
    assert!(response.is_err(), "Expected an error due to server failure");
}
