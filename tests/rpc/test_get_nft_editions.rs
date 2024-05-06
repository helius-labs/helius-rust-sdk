use std::sync::Arc;

use helius_sdk::client::Helius;
use helius_sdk::config::Config;
use helius_sdk::error::HeliusError;
use helius_sdk::rpc_client::RpcClient;
use helius_sdk::types::*;

use mockito::{self, Server};
use reqwest::Client;

#[tokio::test]
async fn test_get_nft_editions_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<EditionsList> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: EditionsList {
            total: 1,
            limit: 1,
            page: Some(1),
            master_edition_address: "8SHfqzJYABeGfiG1apwiEYt6TvfGQiL1pdwEjvTKsyiZ".to_string(),
            supply: 65,
            max_supply: Some(69),
            editions: vec![Edition {
                mint: "GJvFDcBWf6aDncd1TBzx2ou1rgLFYaMBdbYLBa9oTAEw".to_string(),
                edition_address: "AoxgzXKEsJmUyF5pBb3djn9cJFA26zh2SQHvd9EYijZV".to_string(),
                edition: Some(1),
            }],
        },
        id: "1".to_string(),
    };

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });

    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
    };

    let request = GetNftEditions {
        mint: Some("Ey2Qb8kLctbchQsMnhZs5DjY32To2QtPuXNwWvk4NosL".to_string()),
        page: Some(1),
        limit: Some(1),
    };

    let response: Result<EditionsList, HeliusError> = helius.rpc().get_nft_editions(request).await;
    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let editions_list: EditionsList = response.unwrap();
    assert_eq!(
        editions_list.editions.len(),
        1,
        "No token account returned when one was expected"
    );

    assert_eq!(
        editions_list.editions[0].mint, "GJvFDcBWf6aDncd1TBzx2ou1rgLFYaMBdbYLBa9oTAEw",
        "Edition mint does not match expected value"
    );
}

#[tokio::test]
async fn test_get_nft_editions_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });

    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
    };

    let request = GetNftEditions {
        mint: Some("Ey2Qb8kLctbchQsMnhZs5DjY32To2QtPuXNwWvk4NosL".to_string()),
        page: Some(1),
        limit: Some(1),
    };

    let response: Result<EditionsList, HeliusError> = helius.rpc().get_nft_editions(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
