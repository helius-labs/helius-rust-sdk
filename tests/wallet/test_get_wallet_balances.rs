use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{ApiKey, BalancesPagination, BalancesResponse, Cluster, HeliusEndpoints, TokenBalance};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

#[tokio::test]
async fn test_get_wallet_balances_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: BalancesResponse = BalancesResponse {
        balances: vec![
            TokenBalance {
                mint: "So11111111111111111111111111111111111111112".to_string(),
                symbol: Some("SOL".to_string()),
                name: Some("Solana".to_string()),
                balance: 204.316691779,
                decimals: 9,
                price_per_token: Some(83.32),
                usd_value: Some(17025.80),
                logo_uri: None,
                token_program: "spl-token".to_string(),
            },
            TokenBalance {
                mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
                symbol: Some("USDC".to_string()),
                name: Some("USD Coin".to_string()),
                balance: 1000.0,
                decimals: 6,
                price_per_token: Some(1.0),
                usd_value: Some(1000.0),
                logo_uri: None,
                token_program: "spl-token".to_string(),
            },
        ],
        nfts: None,
        total_usd_value: 18025.80,
        pagination: BalancesPagination {
            page: 1,
            limit: 100,
            has_more: false,
        },
    };

    server
        .mock(
            "GET",
            "/v1/wallet/GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz/balances?api-key=fake_api_key&page=1&limit=100&showZeroBalance=false&showNative=true&showNfts=false",
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

    let response: Result<BalancesResponse> = helius
        .get_wallet_balances(
            "GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz",
            Some(1),
            Some(100),
            Some(false),
            Some(true),
            Some(false),
        )
        .await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());
    let balances: BalancesResponse = response.unwrap();

    assert_eq!(balances.balances.len(), 2);
    assert_eq!(balances.total_usd_value, 18025.80);
    assert_eq!(balances.pagination.page, 1);
    assert!(!balances.pagination.has_more);
    assert_eq!(balances.balances[0].symbol, Some("SOL".to_string()));
}

#[tokio::test]
async fn test_get_wallet_balances_with_pagination() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: BalancesResponse = BalancesResponse {
        balances: vec![],
        nfts: None,
        total_usd_value: 0.0,
        pagination: BalancesPagination {
            page: 2,
            limit: 50,
            has_more: true,
        },
    };

    server
        .mock(
            "GET",
            "/v1/wallet/GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz/balances?api-key=fake_api_key&page=2&limit=50",
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

    let response: Result<BalancesResponse> = helius
        .get_wallet_balances(
            "GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz",
            Some(2),
            Some(50),
            None,
            None,
            None,
        )
        .await;

    assert!(response.is_ok());

    let balances: BalancesResponse = response.unwrap();
    assert_eq!(balances.pagination.page, 2);
    assert!(balances.pagination.has_more);
}
