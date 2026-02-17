use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{ApiKey, Cluster, HeliusEndpoints, Identity};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

#[tokio::test]
async fn test_get_batch_wallet_identity_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<Identity> = vec![
        Identity {
            address: "HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664".to_string(),
            entity_type: "exchange".to_string(),
            name: "Binance 1".to_string(),
            category: "Centralized Exchange".to_string(),
            tags: vec!["Centralized Exchange".to_string()],
        },
        Identity {
            address: "2ojv9BAiHUrvsm9gxDe7fJSzbNZSJcxZvf8dqmWGHG8S".to_string(),
            entity_type: "exchange".to_string(),
            name: "Coinbase 2".to_string(),
            category: "Centralized Exchange".to_string(),
            tags: vec!["Centralized Exchange".to_string()],
        },
    ];

    server
        .mock("POST", "/v1/wallet/batch-identity?api-key=fake_api_key")
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

    let addresses = vec![
        "HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664".to_string(),
        "2ojv9BAiHUrvsm9gxDe7fJSzbNZSJcxZvf8dqmWGHG8S".to_string(),
    ];

    let response: Result<Vec<Identity>> = helius.get_batch_wallet_identity(&addresses).await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());
    let identities: Vec<Identity> = response.unwrap();

    assert_eq!(identities.len(), 2);
    assert_eq!(identities[0].name, "Binance 1");
    assert_eq!(identities[1].name, "Coinbase 2");
}

#[tokio::test]
async fn test_get_batch_wallet_identity_empty_array() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<Identity> = vec![];

    server
        .mock("POST", "/v1/wallet/batch-identity?api-key=fake_api_key")
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

    let addresses = vec!["UnknownAddress1".to_string(), "UnknownAddress2".to_string()];

    let response: Result<Vec<Identity>> = helius.get_batch_wallet_identity(&addresses).await;

    assert!(response.is_ok());

    let identities: Vec<Identity> = response.unwrap();
    assert_eq!(identities.len(), 0);
}
