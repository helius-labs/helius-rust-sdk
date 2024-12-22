use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{
    ApiResponse, Asset, AssetList, Attribute, Authorities, Cluster, Compression, Content, Creator, File,
    GetAssetsByCreator, Group, HeliusEndpoints, Interface, Links, Metadata, Ownership, OwnershipModel, Royalty,
    RoyaltyModel, Scope, Supply,
};
use helius::Helius;

use mockito::{self, Server};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

#[tokio::test]
async fn test_get_assets_by_creator_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<AssetList> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: AssetList {
    grand_total: None,
    total: 1,
    limit: 1,
    page: Some(
        1,
    ),
    before: None,
    after: None,
    cursor: None,
    items: vec![
        Asset {
            interface: Interface::ProgrammableNFT,
            id: "JEGruwYE13mhX2wi2MGrPmeLiVyZtbBptmVy9vG3pXRC".to_string(),
            content: Some(
                Content {
                    schema: "https://schema.metaplex.com/nft1.0.json".to_string(),
                    json_uri: "https://madlads.s3.us-west-2.amazonaws.com/json/6867.json".to_string(),
                    files: Some(
                        vec![
                            File {
                                uri: Some(
                                    "https://madlads.s3.us-west-2.amazonaws.com/images/6867.png".to_string(),
                                ),
                                mime: Some(
                                    "image/png".to_string(),
                                ),
                                cdn_uri: Some(
                                    "https://cdn.helius-rpc.com/cdn-cgi/image//https://madlads.s3.us-west-2.amazonaws.com/images/6867.png".to_string(),
                                ),
                                quality: None,
                                contexts: None,
                            },
                            File {
                                uri: Some(
                                    "https://arweave.net/qJ5B6fx5hEt4P7XbicbJQRyTcbyLaV-OQNA1KjzdqOQ/6867.png".to_string(),
                                ),
                                mime: Some(
                                    "image/png".to_string(),
                                ),
                                cdn_uri: Some(
                                    "https://cdn.helius-rpc.com/cdn-cgi/image//https://arweave.net/qJ5B6fx5hEt4P7XbicbJQRyTcbyLaV-OQNA1KjzdqOQ/6867.png".to_string(),
                                ),
                                quality: None,
                                contexts: None,
                            },
                        ],
                    ),
                    metadata: Metadata {
                        attributes: Some(
                            vec![
                                Attribute {
                                    value: Value::String("Male".to_string()),
                                    trait_type: "Gender".to_string(),
                                },
                                Attribute {
                                    value: Value::String("Galaxy".to_string()),
                                    trait_type: "Type".to_string(),
                                },
                                Attribute {
                                    value: Value::String("Galaxy".to_string()),
                                    trait_type: "Expression".to_string(),
                                },
                                Attribute {
                                    value: Value::String("Galaxy".to_string()),
                                    trait_type: "Eyes".to_string(),
                                },
                                Attribute {
                                    value: Value::String("Warlock Robe".to_string()),
                                    trait_type: "Clothing".to_string(),
                                },
                                Attribute {
                                    value: Value::String("Purple".to_string()),
                                    trait_type: "Background".to_string(),
                                },
                            ],
                        ),
                        description: Some(
                            "Fock it.".to_string(),
                        ),
                        name: Some("Mad Lads #6867".to_string()),
                        symbol: Some("MAD".to_string()),
                    },
                    links: Some(
                        Links {
                            external_url: Some(
                                "https://madlads.com".to_string(),
                            ),
                            image: Some(
                                "https://madlads.s3.us-west-2.amazonaws.com/images/6867.png".to_string(),
                            ),
                            animation_url: None,
                        },
                    ),
                },
            ),
            authorities: Some(
                vec![
                    Authorities {
                        address: "2RtGg6fsFiiF1EQzHqbd66AhW7R5bWeQGpTbv2UMkCdW".to_string(),
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
                            "J1S9H3QjnRtBbbuD4HjPV6RpRhwuk4zKbxsnCHuTgh9w".to_string(),
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
                    percent: 0.042,
                    basis_points: 420,
                    primary_sale_happened: true,
                    locked: false,
                },
            ),
            creators: Some(
                vec![
                    Creator {
                        address: "5XvhfmRjwXkGp3jHGmaKpqeerNYjkuZZBYLVQYdeVcRv".to_string(),
                        share: 0,
                        verified: true,
                    },
                    Creator {
                        address: "2RtGg6fsFiiF1EQzHqbd66AhW7R5bWeQGpTbv2UMkCdW".to_string(),
                        share: 100,
                        verified: true,
                    },
                ],
            ),
            ownership: Ownership {
                frozen: true,
                delegated: true,
                delegate: Some(
                    "FARqKAafAbgT25QcgiX5d1g6xpadgG7xymu5N6gSmp4x".to_string(),
                ),
                ownership_model: OwnershipModel::Single,
                owner: "3F21SJs4FMpsakrxmd8GjgfQZG6BN6MVsvXcm5Yc6Jcf".to_string(),
            },
            uses: None,
            supply: Some(
                Supply {
                    print_max_supply: None,
                    print_current_supply: None,
                    edition_nonce: None,
                    edition_number: None,
                    master_edition_mint: None,
                },
            ),
            mutable: true,
            burnt: false,
            mint_extensions: None,
            token_info: None,
            group_definition: None,
            plugins: None,
            unknown_plugins: None,
            mpl_core_info: None,
        },
    ],
    errors: None,
    native_balance: None,
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

    let request: GetAssetsByCreator = GetAssetsByCreator {
        creator_address: "2RtGg6fsFiiF1EQzHqbd66AhW7R5bWeQGpTbv2UMkCdW".to_string(),
        page: Some(1),
        limit: Some(1),
        ..Default::default()
    };

    let response: Result<AssetList> = helius.rpc().get_assets_by_creator(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let api_response: AssetList = response.unwrap();
    assert_eq!(api_response.total, 1, "Total does not match");
    assert_eq!(api_response.items.len(), 1, "Items count does not match");
}

#[tokio::test]
async fn test_get_assets_by_creator_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    // Simulate an API failure with status code 500
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

    let request: GetAssetsByCreator = GetAssetsByCreator {
        creator_address: "2RtGg6fsFiiF1EQzHqbd66AhW7R5bWeQGpTbv2UMkCdW".to_string(),
        page: Some(1),
        ..Default::default()
    };

    let response: Result<AssetList> = helius.rpc().get_assets_by_creator(request).await;
    assert!(response.is_err(), "Expected an error due to server failure");
}
