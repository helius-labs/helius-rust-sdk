use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::{self, Server};
use reqwest::Client;

#[tokio::test]
async fn test_get_asset_proof_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<AssetProof> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: AssetProof {
            root: "FcyXjiB62Jd7pidqvJGaqxoY9CQxDn65tFM69iW8vaji".to_string(),
            proof: vec![
                "EmJXiXEAhEN3FfNQtBa5hwR8LC5kHvdLsaGCoERosZjK".to_string(),
                "7NEfhcNPAwbw3L87fjsPqTz2fQdd1CjoLE138SD58FDQ".to_string(),
                "6dM3VyeQoYkRFZ74G53EwvUPbQC6LsMZge6c7S1Ds4ks".to_string(),
                "A9AACJ5m7UtaVz4HxzhDxGjYaY88rc2XPoFvnoTvgYBj".to_string(),
                "2VG5cKeBZdqozwhHGGzs13b9tzy9TXt9kPfN8MzSJ1Sm".to_string(),
                "3E1uFze4pi6BnTZXMsQbeh3jQCeDi966Zax9aMbYgg2D".to_string(),
                "EZWcjuxCvSj2megG1zXKfXkUF2MKdSEaYfKFGeYSoPrQ".to_string(),
                "HSbJ8quT4vuXFgf5FnjzeUuFfAtLKsq6W1Frj8y1qrif".to_string(),
                "GJMLzL4F4hY9yFHY1EY6XRmW4wpuNGeBZTiv7vM2mYra".to_string(),
                "FYPtEiqmRx6JprHQvWeEWEuVp3WA7DPRCE4VbhFRVuAj".to_string(),
                "6MJKrpnK1GbYsnEzwMRWStNGkTjAZF23NhzTQSQVXsD3".to_string(),
                "HjnrJn5vBUUzpCxzjjM9ZnCPuXei2cXKJjX468B9yWD7".to_string(),
                "4YCF1CSyTXm1Yi9W9JeYevawupkomdgy2dLxEBHL9euq".to_string(),
                "E3oMtCuPEauftdZLX8EZ8YX7BbFzpBCVRYEiLxwPJLY2".to_string(),
            ],
            node_index: 16384,
            leaf: "6YdZXw49M97mfFTwgQb6kxM2c6eqZkHSaW9XhhoZXtzv".to_string(),
            tree_id: "2kuTFCcjbV22wvUmtmgsFR7cas7eZUzAu96jzJUvUcb7".to_string(),
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
        async_rpc_client: None,
        ws_client: None,
    };

    let request: GetAssetProof = GetAssetProof {
        id: "Bu1DEKeawy7txbnCEJE4BU3BKLXaNAKCYcHR4XhndGss".to_string(),
    };

    let response: Result<Option<AssetProof>> = helius.rpc().get_asset_proof(request).await;
    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let asset_response: Option<AssetProof> = response.unwrap();
    assert!(asset_response.is_some(), "No asset returned when one was expected");

    let asset_proof: AssetProof = asset_response.unwrap();

    assert_eq!(
        asset_proof.leaf, "6YdZXw49M97mfFTwgQb6kxM2c6eqZkHSaW9XhhoZXtzv",
        "Asset Proof Leaf does not match expected value"
    );
}

#[tokio::test]
async fn test_get_asset_proof_failure() {
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
        async_rpc_client: None,
        ws_client: None,
    };

    let request: GetAssetProof = GetAssetProof {
        id: "invalid-id-helius-is-the-best".to_string(),
    };

    let response: Result<Option<AssetProof>> = helius.rpc().get_asset_proof(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
