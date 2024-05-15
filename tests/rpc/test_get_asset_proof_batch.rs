use std::collections::HashMap;
use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::error::HeliusError;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::{self, Server};
use reqwest::Client;

#[tokio::test]
async fn test_get_asset_proof_batch_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<HashMap<String, Option<AssetProof>>> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: {
            let mut proofs: HashMap<String, Option<AssetProof>> = HashMap::new();
            proofs.insert(
                "81bxPqYCE8j34nQm7Rooqi8Vt3iMHLzgZJ71rUVbQQuz".to_string(),
                Some(AssetProof {
                    root: "root_hash_1".to_string(),
                    proof: vec!["proof1".to_string(), "proof2".to_string()],
                    node_index: 123,
                    leaf: "leaf_hash_1".to_string(),
                    tree_id: "tree_id_1".to_string(),
                }),
            );
            proofs.insert(
                "CWHuz6GPjWYdwt7rTfRHKaorMwZP58Spyd7aqGK7xFbn".to_string(),
                Some(AssetProof {
                    root: "root_hash_2".to_string(),
                    proof: vec!["proof3".to_string(), "proof4".to_string()],
                    node_index: 456,
                    leaf: "leaf_hash_2".to_string(),
                    tree_id: "tree_id_2".to_string(),
                }),
            );
            proofs
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

    let request: GetAssetProofBatch = GetAssetProofBatch {
        ids: vec![
            "81bxPqYCE8j34nQm7Rooqi8Vt3iMHLzgZJ71rUVbQQuz".to_string(),
            "CWHuz6GPjWYdwt7rTfRHKaorMwZP58Spyd7aqGK7xFbn".to_string(),
        ],
    };

    let response: Result<HashMap<String, Option<AssetProof>>, HeliusError> =
        helius.rpc().get_asset_proof_batch(request).await;
    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let proofs: HashMap<String, Option<AssetProof>> = response.unwrap();
    assert_eq!(proofs.len(), 2);
    assert!(proofs.contains_key("81bxPqYCE8j34nQm7Rooqi8Vt3iMHLzgZJ71rUVbQQuz"));
    assert!(proofs.contains_key("CWHuz6GPjWYdwt7rTfRHKaorMwZP58Spyd7aqGK7xFbn"));

    let proof_81: &AssetProof = proofs
        .get("81bxPqYCE8j34nQm7Rooqi8Vt3iMHLzgZJ71rUVbQQuz")
        .and_then(Option::as_ref)
        .unwrap();
    assert_eq!(
        proof_81.leaf, "leaf_hash_1",
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
        ws_client: None,
    };

    let request: GetAssetProofBatch = GetAssetProofBatch {
        ids: vec!["Hello there".to_string(), "General Kenobi".to_string()],
    };

    let response: Result<HashMap<String, Option<AssetProof>>, HeliusError> =
        helius.rpc().get_asset_proof_batch(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
