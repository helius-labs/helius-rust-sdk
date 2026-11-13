use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{ApiKey, BatchIdentityResult, Cluster, HeliusEndpoints, Identity};
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

    let response: Result<Vec<BatchIdentityResult>> = helius.get_batch_wallet_identity(&addresses).await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());
    let identities: Vec<BatchIdentityResult> = response.unwrap();

    assert_eq!(identities.len(), 2);
    assert!(
        matches!(&identities[0], BatchIdentityResult::Resolved(entry) if entry.identity.name == "Binance 1" && entry.input_domain.is_none())
    );
    assert!(
        matches!(&identities[1], BatchIdentityResult::Resolved(entry) if entry.identity.name == "Coinbase 2" && entry.input_domain.is_none())
    );
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

    let response: Result<Vec<BatchIdentityResult>> = helius.get_batch_wallet_identity(&addresses).await;

    assert!(response.is_ok());

    let identities: Vec<BatchIdentityResult> = response.unwrap();
    assert_eq!(identities.len(), 0);
}

#[tokio::test]
async fn test_get_batch_wallet_identity_mixed_domains() {
    let mut server = Server::new_async().await;
    let body = serde_json::json!([
        {"address":"toly-wallet","type":"person","name":"Toly","category":"Individual","tags":[],
         "inputDomain":"toly.sol"},
        {"address":null,"type":"unknown","inputDomain":"nonexistent-xyz.sol","unresolved":true}
    ]);
    let mock = server
        .mock("POST", "/v1/wallet/batch-identity?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(body.to_string())
        .create_async()
        .await;
    let url = format!("{}/", server.url());
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

    let result = helius
        .get_batch_wallet_identity(&["toly.sol".to_string(), "nonexistent-xyz.sol".to_string()])
        .await;
    assert!(result.is_ok(), "The API call failed: {result:?}");
    let entries = result.unwrap();
    assert_eq!(entries.len(), 2);
    assert!(matches!(&entries[0], BatchIdentityResult::Resolved(entry)
        if entry.identity.name == "Toly" && entry.input_domain.as_deref() == Some("toly.sol")));
    assert!(matches!(&entries[1], BatchIdentityResult::Unresolved(entry)
        if entry.address.is_none() && entry.entity_type == "unknown"
            && entry.input_domain == "nonexistent-xyz.sol" && entry.unresolved));
    assert_eq!(serde_json::to_value(&entries).unwrap(), body);
    mock.assert_async().await;
}
