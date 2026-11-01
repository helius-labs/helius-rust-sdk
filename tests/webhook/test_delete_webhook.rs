use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{ApiKey, Cluster, HeliusEndpoints};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

#[tokio::test]
async fn test_delete_webhook_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock("DELETE", "/v0/webhooks/0e8250a1-ceec-4757-ad69?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .create();

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
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };

    let response: Result<()> = helius.delete_webhook("0e8250a1-ceec-4757-ad69").await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());
}

#[tokio::test]
async fn test_delete_webhook_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock("DELETE", "/v0/webhooks/0e8250a1-ceec-4757-ad69?api-key=fake_api_key")
        .with_status(500)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error":"Internal Server Error"}"#)
        .create();

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
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };
    let response: Result<()> = helius.delete_webhook("0e8250a1-ceec-4757-ad69").await;
    assert!(response.is_err(), "Expected an error due to server failure");
}

#[tokio::test]
async fn test_webhook_methods_reject_empty_or_invalid_webhook_id() {
    let helius = Helius::new("fake_api_key", Cluster::Devnet).unwrap();

    for invalid_id in ["", "   ", "id/with/slash", "id?query=1"] {
        let get_res = helius.get_webhook_by_id(invalid_id).await;
        assert!(
            matches!(get_res, Err(helius::error::HeliusError::InvalidInput(ref text)) if text.contains("webhook_id")),
            "Expected InvalidInput for get_webhook_by_id({:?}), got: {:?}",
            invalid_id,
            get_res
        );

        let del_res = helius.delete_webhook(invalid_id).await;
        assert!(
            matches!(del_res, Err(helius::error::HeliusError::InvalidInput(ref text)) if text.contains("webhook_id")),
            "Expected InvalidInput for delete_webhook({:?}), got: {:?}",
            invalid_id,
            del_res
        );
    }
}
