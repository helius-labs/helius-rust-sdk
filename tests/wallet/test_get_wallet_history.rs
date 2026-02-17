use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{ApiKey, BalanceChange, Cluster, HeliusEndpoints, HistoryResponse, HistoryTransaction, Pagination};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

#[tokio::test]
async fn test_get_wallet_history_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: HistoryResponse = HistoryResponse {
        data: vec![HistoryTransaction {
            signature: "5wHu1qwD7Jsj3xqWjdSEJmYr3Q5f5RjXqjqQJ7jqEj7jqEj7jqEj7jqEj7jqEj7jqE".to_string(),
            timestamp: Some(1704067200),
            slot: 250000000,
            fee: 0.000005,
            fee_payer: "GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz".to_string(),
            error: None,
            balance_changes: vec![
                BalanceChange {
                    mint: "So11111111111111111111111111111111111111112".to_string(),
                    amount: -0.05,
                    decimals: 9,
                },
                BalanceChange {
                    mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                    amount: 100.0,
                    decimals: 6,
                },
            ],
        }],
        pagination: Pagination {
            has_more: true,
            next_cursor: Some("next_page_cursor".to_string()),
        },
    };

    server
        .mock(
            "GET",
            "/v1/wallet/GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz/history?api-key=fake_api_key&limit=50&type=SWAP&tokenAccounts=balanceChanged",
        )
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: Some(ApiKey::new("fake_api_key").unwrap()),
        cluster: Cluster::MainnetBeta,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
        custom_url: None,
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

    let response: Result<HistoryResponse> = helius
        .get_wallet_history(
            "GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz",
            Some(50),
            None,
            None,
            Some("SWAP".to_string()),
            Some("balanceChanged".to_string()),
        )
        .await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let history: HistoryResponse = response.unwrap();
    assert_eq!(history.data.len(), 1);
    assert_eq!(history.data[0].balance_changes.len(), 2);
    assert_eq!(history.pagination.has_more, true);
    assert!(history.pagination.next_cursor.is_some());
}

#[tokio::test]
async fn test_get_wallet_history_with_pagination() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: HistoryResponse = HistoryResponse {
        data: vec![],
        pagination: Pagination {
            has_more: false,
            next_cursor: None,
        },
    };

    server
        .mock(
            "GET",
            "/v1/wallet/GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz/history?api-key=fake_api_key&before=cursor123",
        )
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: Some(ApiKey::new("fake_api_key").unwrap()),
        cluster: Cluster::MainnetBeta,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
        custom_url: None,
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

    let response: Result<HistoryResponse> = helius
        .get_wallet_history(
            "GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz",
            None,
            Some("cursor123"),
            None,
            None,
            None,
        )
        .await;

    assert!(response.is_ok());
    
    let history: HistoryResponse = response.unwrap();
    assert_eq!(history.pagination.has_more, false);
}
