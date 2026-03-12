use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{ApiKey, Cluster, HeliusEndpoints, Identity};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

#[tokio::test]
async fn test_get_wallet_identity_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Identity = Identity {
        address: "HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664".to_string(),
        entity_type: "exchange".to_string(),
        name: "Binance 1".to_string(),
        category: "Centralized Exchange".to_string(),
        tags: vec!["Centralized Exchange".to_string()],
    };

    server
        .mock(
            "GET",
            "/v1/wallet/HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664/identity?api-key=fake_api_key",
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

    let response: Result<Identity> = helius
        .get_wallet_identity("HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664")
        .await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let identity: Identity = response.unwrap();
    assert_eq!(identity.address, "HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664");
    assert_eq!(identity.name, "Binance 1");
    assert_eq!(identity.entity_type, "exchange");
    assert_eq!(identity.category, "Centralized Exchange");
}

#[tokio::test]
async fn test_get_wallet_identity_not_found() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock("GET", "/v1/wallet/UnknownAddress123/identity?api-key=fake_api_key")
        .with_status(404)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error":"No identity information available for this address","code":404}"#)
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

    let response: Result<Identity> = helius.get_wallet_identity("UnknownAddress123").await;
    assert!(response.is_err(), "Expected an error for unknown wallet");
}
