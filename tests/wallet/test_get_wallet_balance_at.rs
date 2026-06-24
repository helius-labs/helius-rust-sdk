use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{
    ApiKey, BalanceAtAsOf, BalanceAtQuery, BalanceAtRequested, BalanceAtResponse, Cluster, HeliusEndpoints,
};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

fn make_helius(url: &str) -> Helius {
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
    Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    }
}

#[tokio::test]
async fn test_get_wallet_balance_at_with_time() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: BalanceAtResponse = BalanceAtResponse {
        wallet: "5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9".to_string(),
        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        is_native: false,
        balance: "284961463.392936".to_string(),
        balance_raw: "284961463392936".to_string(),
        decimals: 6,
        requested: BalanceAtRequested {
            time: Some(1736536800),
            slot: None,
            datetime: None,
        },
        as_of: Some(BalanceAtAsOf {
            slot: 313000000,
            block_time: Some(1736536794),
            signature: "5Cyy7Mh9nVgFq3T8wJp2sKxR4dE6bA1uZoNcLrXmYqUpon".to_string(),
        }),
    };

    server
        .mock(
            "GET",
            "/v1/wallet/5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9/balance-at?api-key=fake_api_key&mint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&time=1736536800",
        )
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = make_helius(&url);

    let response: Result<BalanceAtResponse> = helius
        .get_wallet_balance_at(
            "5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            BalanceAtQuery::Time(1736536800),
        )
        .await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let balance: BalanceAtResponse = response.unwrap();
    assert_eq!(balance.balance, "284961463.392936");
    assert_eq!(balance.balance_raw, "284961463392936");
    assert_eq!(balance.decimals, 6);
    assert!(!balance.is_native);
    assert_eq!(balance.requested.time, Some(1736536800));
    let as_of = balance.as_of.expect("as_of should be present");
    assert_eq!(as_of.slot, 313000000);
    assert_eq!(as_of.block_time, Some(1736536794));
}

#[tokio::test]
async fn test_get_wallet_balance_at_with_slot() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: BalanceAtResponse = BalanceAtResponse {
        wallet: "5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9".to_string(),
        mint: "So11111111111111111111111111111111111111111".to_string(),
        is_native: true,
        balance: "1.5".to_string(),
        balance_raw: "1500000000".to_string(),
        decimals: 9,
        requested: BalanceAtRequested {
            time: None,
            slot: Some(313000000),
            datetime: None,
        },
        as_of: Some(BalanceAtAsOf {
            slot: 313000000,
            block_time: None,
            signature: "5Cyy7Mh9nVgFq3T8wJp2sKxR4dE6bA1uZoNcLrXmYqUpon".to_string(),
        }),
    };

    server
        .mock(
            "GET",
            "/v1/wallet/5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9/balance-at?api-key=fake_api_key&mint=So11111111111111111111111111111111111111111&slot=313000000",
        )
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = make_helius(&url);

    let response: Result<BalanceAtResponse> = helius
        .get_wallet_balance_at(
            "5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9",
            "So11111111111111111111111111111111111111111",
            BalanceAtQuery::Slot(313000000),
        )
        .await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let balance: BalanceAtResponse = response.unwrap();
    assert!(balance.is_native);
    assert_eq!(balance.decimals, 9);
    assert_eq!(balance.requested.slot, Some(313000000));
}

#[tokio::test]
async fn test_get_wallet_balance_at_with_datetime() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: BalanceAtResponse = BalanceAtResponse {
        wallet: "5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9".to_string(),
        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        is_native: false,
        balance: "100".to_string(),
        balance_raw: "100000000".to_string(),
        decimals: 6,
        requested: BalanceAtRequested {
            time: Some(1736536800),
            slot: None,
            datetime: Some("2025-01-10 19:20:00".to_string()),
        },
        as_of: None,
    };

    // The datetime value is form-encoded: space becomes `+`.
    server
        .mock(
            "GET",
            "/v1/wallet/5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9/balance-at?api-key=fake_api_key&mint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&datetime=2025-01-10+19%3A20%3A00",
        )
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius: Helius = make_helius(&url);

    let response: Result<BalanceAtResponse> = helius
        .get_wallet_balance_at(
            "5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            BalanceAtQuery::Datetime("2025-01-10 19:20:00".to_string()),
        )
        .await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let balance: BalanceAtResponse = response.unwrap();
    assert_eq!(balance.requested.datetime, Some("2025-01-10 19:20:00".to_string()));
    assert!(balance.as_of.is_none());
    assert_eq!(balance.balance, "100");
}
