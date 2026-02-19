use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{ApiKey, Cluster, HeliusEndpoints, Pagination, Transfer, TransferDirection, TransfersResponse};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

#[tokio::test]
async fn test_get_wallet_transfers_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: TransfersResponse = TransfersResponse {
        data: vec![
            Transfer {
                signature: "5wHu1qwD7Jsj3xqWjdSEJmYr3Q5f5RjXqjqQJ7jqEj7jqEj7jqEj7jqEj7jqEj7jqE".to_string(),
                timestamp: 1704067200,
                direction: TransferDirection::In,
                counterparty: "HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664".to_string(),
                mint: "So11111111111111111111111111111111111111112".to_string(),
                symbol: Some("SOL".to_string()),
                amount: 1.5,
                amount_raw: "1500000000".to_string(),
                decimals: 9,
            },
            Transfer {
                signature: "6xIv2qwD7Jsj3xqWjdSEJmYr3Q5f5RjXqjqQJ7jqEj7jqEj7jqEj7jqEj7jqEj7jqE".to_string(),
                timestamp: 1704066200,
                direction: TransferDirection::Out,
                counterparty: "2ojv9BAiHUrvsm9gxDe7fJSzbNZSJcxZvf8dqmWGHG8S".to_string(),
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                symbol: Some("USDC".to_string()),
                amount: 100.0,
                amount_raw: "100000000".to_string(),
                decimals: 6,
            },
        ],
        pagination: Pagination {
            has_more: true,
            next_cursor: Some("next_cursor".to_string()),
        },
    };

    server
        .mock(
            "GET",
            "/v1/wallet/GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz/transfers?api-key=fake_api_key&limit=50",
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

    let response: Result<TransfersResponse> = helius
        .get_wallet_transfers("GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz", Some(50), None)
        .await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let transfers: TransfersResponse = response.unwrap();
    assert_eq!(transfers.data.len(), 2);
    assert_eq!(transfers.data[0].direction, TransferDirection::In);
    assert_eq!(transfers.data[1].direction, TransferDirection::Out);
    assert_eq!(transfers.pagination.has_more, true);
}

#[tokio::test]
async fn test_get_wallet_transfers_with_cursor() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: TransfersResponse = TransfersResponse {
        data: vec![],
        pagination: Pagination {
            has_more: false,
            next_cursor: None,
        },
    };

    server
        .mock(
            "GET",
            "/v1/wallet/GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz/transfers?api-key=fake_api_key&cursor=page2cursor",
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

    let response: Result<TransfersResponse> = helius
        .get_wallet_transfers(
            "GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz",
            None,
            Some("page2cursor"),
        )
        .await;

    assert!(response.is_ok());

    let transfers: TransfersResponse = response.unwrap();
    assert_eq!(transfers.pagination.has_more, false);
}
