use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::{self, Server};
use reqwest::Client;

#[tokio::test]
async fn test_get_rwa_asset_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<GetRwaAssetResponse> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetRwaAssetResponse {
            items: FullRwaAccount {
                asset_controller: Some(AssetControllerAccount {
                    address: "JeffAlbertson".to_string(),
                    mint: "RadioactiveMan#1".to_string(),
                    authority: "TheAndroidsDungeonandBaseballCardShop".to_string(),
                    delegate: "JeffAlbertson".to_string(),
                    version: 1,
                    closed: false,
                }),
                data_registry: Some(DataRegistryAccount {
                    address: "CGC".to_string(),
                    mint: "CertifiedGuarantyCompany".to_string(),
                    version: 10,
                    closed: false,
                }),
                identity_registry: None,
                policy_engine: None,
            },
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
        ws_client: None,
    };

    let request: GetRwaAssetRequest = GetRwaAssetRequest {
        id: "RadioactiveMan#1".to_string(),
    };

    let response: Result<GetRwaAssetResponse> = helius.rpc().get_rwa_asset(request).await;
    assert!(response.is_ok());
    assert_eq!(
        response.unwrap().items.asset_controller.unwrap().address,
        "JeffAlbertson"
    );
}

#[tokio::test]
async fn test_get_rwa_asset_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Asset not found"}"#)
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
        ws_client: None,
    };

    let request: GetRwaAssetRequest = GetRwaAssetRequest {
        id: "Flanders'BookofFaith".to_string(),
    };

    let response: Result<GetRwaAssetResponse> = helius.rpc().get_rwa_asset(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
