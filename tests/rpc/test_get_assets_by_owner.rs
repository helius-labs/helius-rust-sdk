use helius_sdk::config::Config;
use helius_sdk::error::HeliusError;
use helius_sdk::rpc_client::RpcClient;
use helius_sdk::types::{ApiResponse, AssetsByOwnerRequest, Cluster, HeliusEndpoints, ResponseType};
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
    print!("{}", url);

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "jsonrpc": "2.0",
            "result": {
                "total": 2,
                "limit": 10,
                "page": 1,
                "items": [
                    {
                        "interface": "V1NFT",
                        "id": "123",
                        "content": {
                            "schema": "http://example.com/schema",
                            "json_uri": "http://example.com/json",
                            "files": [
                                {
                                    "uri": "http://example.com/file1",
                                    "mime": "image/png"
                                }
                            ],
                            "metadata": {
                                "attributes": [
                                    {
                                        "value": "blue",
                                        "trait_type": "color"
                                    }
                                ],
                                "description": "A description",
                                "name": "Item1",
                                "symbol": "SYM"
                            }
                        },
                        "authorities": [],
                        "compression": {
                            "eligible": true,
                            "compressed": false,
                            "data_hash": "hash1",
                            "creator_hash": "hash2",
                            "asset_hash": "hash3",
                            "tree": "tree1",
                            "seq": 1,
                            "leaf_id": 1
                        },
                        "grouping": [],
                        "royalty": {
                            "royalty_model": "Creators",
                            "percent": 5.0,
                            "basis_points": 500
                        },
                        "ownership": {
                            "frozen": false,
                            "delegated": false,
                            "owner": "OwnerAddress1",
                            "ownership_model": "Single"
                        },
                        "creators": [],
                        "uses": {
                            "use_method": "Single",
                            "remaining": 5,
                            "total": 10
                        },
                        "supply": {
                            "print_max_supply": 100,
                            "print_current_supply": 50
                        },
                        "mutable": false,
                        "burnt": false
                    }
                ]
            },
            "id": 1
        }"#,
        )
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
        assert_eq!(list.total, Some(2), "Total does not match");
        assert!(list.items.is_some(), "Items are missing");
        assert_eq!(list.items.unwrap().len(), 2, "Items count does not match");
    } else {
        panic!("Expected GetAssetResponseList");
    }
}
