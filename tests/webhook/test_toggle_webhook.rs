use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{ApiKey, Cluster, HeliusEndpoints, ToggleWebhookRequest, TransactionType, Webhook, WebhookType};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

#[tokio::test]
async fn test_toggle_webhook_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Webhook = Webhook {
        webhook_url: "https://webhook.site/0e8250a1-ceec-4757-ad69-cc6473085bfc".to_string(),
        transaction_types: vec![TransactionType::Any],
        account_addresses: vec![],
        webhook_type: WebhookType::Enhanced,
        auth_header: None,
        webhook_id: "0e8250a1-ceec-4757-ad69".to_string(),
        wallet: "9Jt8mC9HXvh2g5s3PbTsNU71RS9MXUbhEMEmLTixYirb".to_string(),
        project: "Mockito".to_string(),
        active: false,
        ..Default::default()
    };

    server
        .mock("PATCH", "/v0/webhooks/0e8250a1-ceec-4757-ad69?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
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

    let request: ToggleWebhookRequest = ToggleWebhookRequest {
        webhook_id: "0e8250a1-ceec-4757-ad69".to_string(),
        active: false,
    };
    let response: Result<Webhook> = helius.toggle_webhook(request).await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());
    let webhook_response: Webhook = response.unwrap();

    assert_eq!(webhook_response.webhook_id, "0e8250a1-ceec-4757-ad69");
    assert!(!webhook_response.active);
}

#[tokio::test]
async fn test_toggle_webhook_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock("PATCH", "/v0/webhooks/0e8250a1-ceec-4757-ad69?api-key=fake_api_key")
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

    let request: ToggleWebhookRequest = ToggleWebhookRequest {
        webhook_id: "0e8250a1-ceec-4757-ad69".to_string(),
        active: true,
    };

    let response: Result<Webhook> = helius.toggle_webhook(request).await;
    assert!(response.is_err(), "Expected an error due to server failure");
}
