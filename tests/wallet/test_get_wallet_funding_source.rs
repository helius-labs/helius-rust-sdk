use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{ApiKey, Cluster, FundingSource, HeliusEndpoints};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

#[tokio::test]
async fn test_get_wallet_funding_source_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: FundingSource = FundingSource {
        funder: "2ojv9BAiHUrvsm9gxDe7fJSzbNZSJcxZvf8dqmWGHG8S".to_string(),
        funder_name: Some("Coinbase 2".to_string()),
        funder_type: Some("exchange".to_string()),
        mint: "So11111111111111111111111111111111111111112".to_string(),
        symbol: "SOL".to_string(),
        amount: 0.05,
        amount_raw: "50000000".to_string(),
        decimals: 9,
        signature: "5wHu1qwD7Jsj3xqWjdSEJmYr3Q5f5RjXqjqQJ7jqEj7jqEj7jqEj7jqEj7jqEj7jqE".to_string(),
        timestamp: 1704067200,
        date: "2024-01-01T00:00:00.000Z".to_string(),
        slot: 250000000,
        explorer_url: "https://orbmarkets.io/tx/5wHu1qwD7Jsj3xqWjdSEJmYr3Q5f5RjXqjqQJ7jqEj7jqEj7jqEj7jqEj7jqEj7jqE"
            .to_string(),
    };

    server
        .mock(
            "GET",
            "/v1/wallet/GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz/funded-by?api-key=fake_api_key",
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

    let response: Result<FundingSource> = helius
        .get_wallet_funding_source("GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz")
        .await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let funding: FundingSource = response.unwrap();
    assert_eq!(funding.funder, "2ojv9BAiHUrvsm9gxDe7fJSzbNZSJcxZvf8dqmWGHG8S");
    assert_eq!(funding.funder_name, Some("Coinbase 2".to_string()));
    assert_eq!(funding.funder_type, Some("exchange".to_string()));
    assert_eq!(funding.amount, 0.05);
    assert_eq!(funding.symbol, "SOL");
}

#[tokio::test]
async fn test_get_wallet_funding_source_not_found() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock("GET", "/v1/wallet/UnfundedWallet123/funded-by?api-key=fake_api_key")
        .with_status(404)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error":"No funding transaction found","code":404}"#)
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

    let response: Result<FundingSource> = helius.get_wallet_funding_source("UnfundedWallet123").await;
    assert!(
        response.is_err(),
        "Expected an error for wallet with no funding transaction"
    );
}
