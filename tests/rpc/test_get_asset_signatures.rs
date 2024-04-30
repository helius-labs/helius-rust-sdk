use std::sync::Arc;

use helius_sdk::client::Helius;
use helius_sdk::config::Config;
use helius_sdk::error::HeliusError;
use helius_sdk::rpc_client::RpcClient;
use helius_sdk::types::*;

use mockito::{self, Server};
use reqwest::Client;

#[tokio::test]
async fn test_get_asset_signatures_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<TransactionSignatureList> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: TransactionSignatureList {
            total: 1,
            limit: 1000,
            page: Some(1),
            before: None,
            after: None,
            items: vec![(
                "3uLpAGykcJmC4cvPoURqAKKktLLFeZBXid6SeXji6Pnd7YAtDxEG3PRXLXpUBdt1N6W18nUGeKv6eNUPb7Po7u3v".to_string(),
                "MintToCollectionV1".to_string(),
            )],
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

    let request: GetAssetSignatures = GetAssetSignatures {
        id: Some("8qjkHtHsqww1rac6Uctj4V7Z5yHoTyQj3iJ5vc4Aka8".to_string()),
        page: Some(1),
        ..Default::default()
    };

    let response: Result<TransactionSignatureList, HeliusError> = helius.rpc().get_asset_signatures(request).await;
    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let signatures: TransactionSignatureList = response.unwrap();
    assert!(signatures.total > 0, "No signature returned when one was expected");

    assert_eq!(
        signatures.items[0].0,
        "3uLpAGykcJmC4cvPoURqAKKktLLFeZBXid6SeXji6Pnd7YAtDxEG3PRXLXpUBdt1N6W18nUGeKv6eNUPb7Po7u3v".to_string(),
        "Signature does not match expected value"
    );
}

#[tokio::test]
async fn test_get_asset_signatures_failure() {
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

    let request: GetAssetSignatures = GetAssetSignatures {
        id: Some("8qjkHtHsqww1rac6Uctj4V7Z5yHoTyQj3iJ5vc4Aka8".to_string()),
        page: Some(1),
        ..Default::default()
    };

    let response: Result<TransactionSignatureList, HeliusError> = helius.rpc().get_asset_signatures(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
