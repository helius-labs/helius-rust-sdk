use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::{self, Server};
use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn test_get_asset_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<Option<Asset>> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(Asset {
                interface: Interface::MplCoreAsset,
                id: "JE9mLqmTRZnUYoMJofSmZp8nZT4pzgARtAJS8crtgVWV".to_string(),
                content: Some(
                    Content {
                        schema: "https://schema.metaplex.com/nft1.0.json".to_string(),
                        json_uri: "https://arweave.net/q4vZtyCtru24QaHWAzFtPotd6BJR2bgKW5bVNxFLlAo".to_string(),
                        files: Some(
                            vec![
                                File {
                                    uri: Some(
                                        "https://arweave.net/euLWELOqMcufuO38zxY8IDnaS1vMs9CwvFPshP6kb3o".to_string(),
                                    ),
                                    mime: Some(
                                        "image/png".to_string(),
                                    ),
                                    cdn_uri: Some(
                                        "https://cdn.helius-rpc.com/cdn-cgi/image//https://arweave.net/euLWELOqMcufuO38zxY8IDnaS1vMs9CwvFPshP6kb3o".to_string(),
                                    ),
                                    quality: None,
                                    contexts: None,
                                }
                            ],
                        ),
                        metadata: Metadata {
                            attributes: Some(
                                vec![
                                    Attribute {
                                        value: Value::String("Common".to_string()),
                                        trait_type: "rarity".to_string(),
                                    },
                                    Attribute {
                                        value: Value::String("false".to_string()),
                                        trait_type: "used".to_string(),
                                    },
                                    Attribute {
                                        value: Value::String("false".to_string()),
                                        trait_type: "signed".to_string(),
                                    },
                                ],
                            ),
                            description: Some(
                                "Apt323 the 36 page Collectors Edition.".to_string(),
                            ),
                            name: "Apt323 Collectors Edition #72".to_string(),
                            symbol: "".to_string(),
                        },
                        links: Some(
                            Links {
                                external_url: Some(
                                    "https://dreader.app".to_string(),
                                ),
                                image: Some(
                                    "https://arweave.net/euLWELOqMcufuO38zxY8IDnaS1vMs9CwvFPshP6kb3o".to_string(),
                                ),
                                animation_url: None,
                            },
                        ),
                    },
                ),
                authorities: Some(
                    vec![
                        Authorities {
                            address: "FXj8W4m33SgLB5ZAg35g8wsqFTvywc6fmJTXzoQQhrVf".to_string(),
                            scopes: vec![
                                Scope::Full,
                            ],
                        },
                    ],
                ),
                compression: Some(
                    Compression {
                        eligible: false,
                        compressed: false,
                        data_hash: "".to_string(),
                        creator_hash: "".to_string(),
                        asset_hash: "".to_string(),
                        tree: "".to_string(),
                        seq: 0,
                        leaf_id: 0,
                    },
                ),
                grouping: Some(
                    vec![
                        Group {
                            group_key: "collection".to_string(),
                            group_value: Some(
                                "FxZuKKWwRTzBuqpUtMeZkTAWwneA5cjdzDJQtNTjXF4C".to_string(),
                            ),
                            verified: None,
                            collection_metadata: None,
                        },
                    ],
                ),
                royalty: Some(
                    Royalty {
                        royalty_model: RoyaltyModel::Creators,
                        target: None,
                        percent: 0.0,
                        basis_points: 0,
                        primary_sale_happened: false,
                        locked: false,
                    },
                ),
                creators: None,
                ownership: Ownership {
                    frozen: false,
                    delegated: false,
                    delegate: None,
                    ownership_model: OwnershipModel::Single,
                    owner: "faraVvDajMs9FRwLvjRKdpfvDwetTBqgdwy95MsQ6VZ".to_string(),
                },
                uses: None,
                supply: None,
                mutable: true,
                burnt: false,
                mint_extensions: None,
                token_info: None,
                group_definition: None,
                plugins: None,
                unknown_plugins: None,
                mpl_core_info: Some(
                    MplCoreInfo {
                        num_minted: None,
                        current_size: None,
                        plugins_json_version: Some(
                            1,
                        ),
                    },
                ),
            }),
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

    let request: GetAsset = GetAsset {
        id: "JE9mLqmTRZnUYoMJofSmZp8nZT4pzgARtAJS8crtgVWV".to_string(),
        display_options: Some(GetAssetOptions {
            show_unverified_collections: true,
            ..Default::default()
        }),
    };

    let response: Result<Option<Asset>> = helius.rpc().get_asset(request).await;
    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let asset_response: Option<Asset> = response.unwrap();
    assert!(asset_response.is_some(), "No asset returned when one was expected");

    let asset: Asset = asset_response.unwrap();
    assert_eq!(
        asset.id, "JE9mLqmTRZnUYoMJofSmZp8nZT4pzgARtAJS8crtgVWV",
        "Asset ID does not match expected value"
    );
    assert_eq!(
        asset.interface,
        Interface::MplCoreAsset,
        "Interface does not match expected value"
    );
}

#[tokio::test]
async fn test_get_asset_failure() {
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

    let request: GetAsset = GetAsset {
        id: "F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk".to_string(),
        display_options: Some(GetAssetOptions {
            show_collection_metadata: true,
            ..Default::default()
        }),
    };

    let response: Result<Option<Asset>> = helius.rpc().get_asset(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
