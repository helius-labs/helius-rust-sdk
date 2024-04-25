use helius_sdk::config::Config;
use helius_sdk::error::HeliusError;
use helius_sdk::rpc_client::RpcClient;
use helius_sdk::types::{
    ApiResponse, AssetsByOwnerRequest, Attribute, Cluster, Content, File, GetAssetResponse, GetAssetResponseList,
    HeliusEndpoints, Interface, Metadata, Ownership, OwnershipModel, ResponseType,
};
use helius_sdk::Helius;

use mockito::{self, Server};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Default)]
struct MockAssetResponse {
    total: u32,
    assets: Vec<String>,
}

#[tokio::test]
async fn test_get_assets_by_owner_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: ResponseType::GetAssetResponseList(GetAssetResponseList {
            total: Some(1),
            limit: Some(10),
            page: Some(1),
            items: Some(vec![GetAssetResponse {
                interface: Interface::V1NFT, 
                id: "123".to_string(),
                content: Some(Content {
                    schema: "http://example.com/schema".to_string(),
                    json_uri: "http://example.com/json".to_string(),
                    files: Some(vec![File {
                        uri: Some("http://example.com/file1".to_string()),
                        mime: Some("image/png".to_string()),
                        cdn_uri: None,
                        quality: None,
                        contexts: None,
                    }]),
                    metadata: Metadata {
                        attributes: Some(vec![Attribute {
                            value: "blue".to_string(),
                            trait_type: "color".to_string(),
                        }]),
                        description: Some("A description".to_string()),
                        name: "Item1".to_string(),
                        symbol: "SYM".to_string(),
                    },
                    links: None,
                }),
                authorities: None,
                compression: None,
                grouping: None,
                royalty: None,
                ownership: Ownership {
                    frozen: false,
                    delegated: false,
                    delegate: None,
                    ownership_model: OwnershipModel::Single,
                    owner: "OwnerAddress1".to_string(),
                },
                creators: None,
                uses: None,
                supply: None,
                mutable: false,
                burnt: false,
            }]),
        }),
        id: 1,
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

    let request: AssetsByOwnerRequest = AssetsByOwnerRequest {
        owner_address: "GNPwr9fk9RJbfy9nSKbNiz5NPfc69KVwnizverx6fNze".to_string(),
        page: Some(1),
        ..Default::default()
    };

    let response: Result<ApiResponse, HeliusError> = helius.rpc().get_assets_by_owner(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let api_response: ApiResponse = response.unwrap();
    if let ResponseType::GetAssetResponseList(list) = api_response.result {
        assert_eq!(list.total, Some(1), "Total does not match");
        assert!(list.items.is_some(), "Items are missing");
        assert_eq!(list.items.unwrap().len(), 1, "Items count does not match");
    } else {
        panic!("Expected GetAssetResponseList");
    }
}
