use helius::config::Config;
use helius::error::{HeliusError, Result};
use helius::rpc_client::RpcClient;
use helius::types::{
    AdminBillingCycle, AdminSubscriptionDetails, AdminUsageBreakdown, ApiKey, Cluster, HeliusEndpoints, ProjectUsage,
};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

fn create_test_helius(url: &str) -> Helius {
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
async fn test_get_project_usage_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response = ProjectUsage {
        credits_remaining: 999_080,
        credits_used: 920,
        prepaid_credits_remaining: 5_000,
        prepaid_credits_used: 0,
        subscription_details: AdminSubscriptionDetails {
            billing_cycle: AdminBillingCycle {
                start: "2026-04-01".to_string(),
                end: "2026-05-01".to_string(),
            },
            credits_limit: 1_000_000,
            plan: "Professional".to_string(),
        },
        usage: AdminUsageBreakdown {
            api: 0,
            archival: 0,
            das: 200,
            grpc: 0,
            grpc_geyser: 0,
            photon: 0,
            rpc: 720,
            stream: 0,
            webhook: 0,
            websocket: 0,
        },
    };

    server
        .mock("GET", "/v0/admin/projects/proj-123/usage?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let helius = create_test_helius(&url);
    let response: Result<ProjectUsage> = helius.get_project_usage("proj-123").await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let usage: ProjectUsage = response.unwrap();
    assert_eq!(usage.credits_remaining, 999_080);
    assert_eq!(usage.credits_used, 920);
    assert_eq!(usage.subscription_details.plan, "Professional");
    assert_eq!(usage.subscription_details.billing_cycle.start, "2026-04-01");
    assert_eq!(usage.usage.rpc, 720);
    assert_eq!(usage.usage.das, 200);
}

#[tokio::test]
async fn test_get_project_usage_requires_api_key() {
    let helius: Helius = Helius::new_with_url("http://localhost:8899").unwrap();

    let response: Result<ProjectUsage> = helius.get_project_usage("proj-123").await;
    assert!(
        matches!(response, Err(HeliusError::InvalidInput(ref text)) if text.contains("admin project usage")),
        "Expected missing-api-key error, got: {:?}",
        response
    );
}
