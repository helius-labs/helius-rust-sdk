use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{
    ApiResponse, Asset, Attribute, Authorities, Cluster, CollectionMetadata, Compression, Content, Creator, File,
    GetAssetBatch, GetAssetOptions, Group, HeliusEndpoints, Interface, Links, Metadata, Ownership, OwnershipModel,
    Royalty, RoyaltyModel, Scope, Supply,
};

use mockito::{self, Server};
use reqwest::Client;
use serde_json::Value;

#[tokio::test]
async fn test_get_asset_batch_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<Vec<Option<Asset>>> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: vec![
            Some(Asset {
                interface: Interface::V1NFT,
                id: "81bxPqYCE8j34nQm7Rooqi8Vt3iMHLzgZJ71rUVbQQuz".to_string(),
                content: Some(Content {
                    schema: "https://schema.metaplex.com/nft1.0.json".to_string(),
                    json_uri: "https://entities.nft.helium.io/112vepK9Upx7juv4pCdwqzm7reBo6VYEQHKRNmU6PEaKX1ese5NE".to_string(),
                    files: Some(vec![File {
                        uri: Some("https://shdw-drive.genesysgo.net/CsDkETHRRR1EcueeN346MJoqzymkkr7RFjMqGpZMzAib/hotspot.png".to_string()),
                        mime: Some("image/png".to_string()),
                        cdn_uri: Some("https://cdn.helius-rpc.com/cdn-cgi/image//https://shdw-drive.genesysgo.net/CsDkETHRRR1EcueeN346MJoqzymkkr7RFjMqGpZMzAib/hotspot.png".to_string()),
                        quality: None,
                        contexts: None,
                    }]),
                    metadata: Metadata {
                        attributes: Some(vec![
                            Attribute {
                                value: Value::String("112vepK9Upx7juv4pCdwqzm7reBo6VYEQHKRNmU6PEaKX1ese5NE".to_string()),
                                trait_type: "ecc_compact".to_string(),
                            },
                            Attribute {
                                value: Value::Bool(true),
                                trait_type: "rewardable".to_string(),
                            }
                        ]),
                        description: Some("A hotspot NFT on Helium".to_string()),
                        name: "gentle-mandarin-ferret".to_string(),
                        symbol: "HOTSPOT".to_string(),
                    },
                    links: Some(Links {
                        external_url: None,
                        image: Some("https://shdw-drive.genesysgo.net/CsDkETHRRR1EcueeN346MJoqzymkkr7RFjMqGpZMzAib/hotspot.png".to_string()),
                        animation_url: None,
                    }),
                }),
                authorities: Some(vec![Authorities {
                    address: "AngjrikiSwNAqed9GEZrXW9eyyptqiwkWtXgw5hkJ1jM".to_string(),
                    scopes: vec![Scope::Full],
                }]),
                compression: Some(Compression {
                    eligible: false,
                    compressed: true,
                    data_hash: "D4yd9ePcnVX2uKujrC4NSCFNVsNTUB1gKUq2bnouR8Ba".to_string(),
                    creator_hash: "GuBHQ5zhrHdkADX8ttpGvSYPQB2M6K5Zcj2qq5HogPgQ".to_string(),
                    asset_hash: "DE2bb82qfupE4Wcv28brKXCKztnEKC2XnwxAmN8LzqPe".to_string(),
                    tree: "E7JbdYSZLvVmqvw6XfEwNQCEAybTF6E1mnrHUXw8hXDV".to_string(),
                    seq: 29627,
                    leaf_id: 29584,
                }),
                grouping: Some(vec![Group {
                    group_key: "collection".to_string(),
                    group_value: Some("FC2daoG1bnBPHGBypxPLb8hU8dtQdx7cYYsU9RR4eYmf".to_string()),
                    verified: None,
                    collection_metadata: Some(CollectionMetadata {
                        name: Some("RAKwireless".to_string()),
                        symbol: Some("MAKER".to_string()),
                        image: Some("".to_string()),
                        description: Some("".to_string()),
                        external_url: Some("".to_string()),
                    }),
                }]),
                royalty: Some(Royalty {
                    royalty_model: RoyaltyModel::Creators,
                    target: None,
                    percent: 0.0,
                    basis_points: 0,
                    primary_sale_happened: true,
                    locked: false,
                }),
                creators: Some(vec![Creator {
                    address: "Fv5hf1Fg58htfC7YEXKNEfkpuogUUQDDTLgjGWxxv48H".to_string(),
                    share: 100,
                    verified: true,
                }]),
                ownership: Ownership {
                    frozen: false,
                    delegated: false,
                    delegate: None,
                    ownership_model: OwnershipModel::Single,
                    owner: "9gGP3HLonuAiE5Xqb87sEua6SY3uHmTPuobbxPsLUV4u".to_string(),
                },
                supply: Some(Supply {
                    print_max_supply: None,
                    print_current_supply: None,
                    edition_nonce: None,
                    edition_number: None,
                    master_edition_mint: None,
                }),
                uses: None,
                mutable: true,
                burnt: false,
                mint_extensions: None,
                token_info: None,
                group_definition: None,
                plugins: None,
                unknown_plugins: None,
                mpl_core_info: None,
            }), Some(Asset {
                interface: Interface::V1NFT,
                id: "CWHuz6GPjWYdwt7rTfRHKaorMwZP58Spyd7aqGK7xFbn".to_string(),
                content: Some(Content {
                    schema: "https://schema.metaplex.com/nft1.0.json".to_string(),
                    json_uri: "https://arweave.net/n_sgSwRGFxnrJTbKDonkUeppFMXIaUrijrh-Q81xKG4".to_string(),
                    files: Some(vec![
                        File {
                            uri: Some("https://arweave.net/SCv3eCEk1aXZ0vK8M2GBAL7_BMcRoBh9Gqvoep-5hGE?ext=jpg".to_string()),
                            mime: Some("image/jpeg".to_string()),
                            cdn_uri: Some("https://cdn.helius-rpc.com/cdn-cgi/image//https://arweave.net/SCv3eCEk1aXZ0vK8M2GBAL7_BMcRoBh9Gqvoep-5hGE?ext=jpg".to_string()),
                            quality: None,
                            contexts: None,
                        },
                        File {
                            uri: Some("".to_string()),
                            mime: Some("image/png".to_string()),
                            cdn_uri: Some("https://cdn.helius-rpc.com/cdn-cgi/image//".to_string()),
                            quality: None,
                            contexts: None,
                        },
                    ]),
                    metadata: Metadata {
                        attributes: Some(vec![
                            Attribute {
                                value: serde_json::Value::String("1".to_string()),
                                trait_type: "Season".to_string(),
                            },
                            Attribute {
                                value: serde_json::Value::String("6".to_string()),
                                trait_type: "Drop".to_string(),
                            },
                            Attribute {
                                value: serde_json::Value::String("Legendary".to_string()),
                                trait_type: "Rarity".to_string(),
                            }
                        ]),
                        description: Some("Aerial photograph of a parking structure in LA depicting the photographer, Andrew Mason, \"sliding\" through the image.".to_string()),
                        name: "Slide".to_string(),
                        symbol: "".to_string(),
                    },
                    links: Some(Links {
                        external_url: Some("".to_string()),
                        image: Some("https://arweave.net/SCv3eCEk1aXZ0vK8M2GBAL7_BMcRoBh9Gqvoep-5hGE?ext=jpg".to_string()),
                        animation_url: None,
                    }),
                }),
                authorities: Some(vec![Authorities {
                    address: "FUtErNuVuWwsgN5fMY5v9pT33KbL4qqRvY5mwAxt3kSc".to_string(),
                    scopes: vec![Scope::Full],
                }]),
                compression: Some(Compression {
                    eligible: false,
                    compressed: true,
                    data_hash: "7a4RoSG6tTeMuyKhVAZF9Qc9kAxaL4ux3HkB7KMvjbfZ".to_string(),
                    creator_hash: "HCuVLuwsjo7FRNZhX8gBRrMaPHo9JPoirvTB8ZJwL8XQ".to_string(),
                    asset_hash: "HjF2mFb4fWFKQwuSUNxivq1BUvuLo824MxCygW58S9uk".to_string(),
                    tree: "DRSdumv6r6KpxbGtnJaRSJL99WnBEi7rphY5euu6NGEu".to_string(),
                    seq: 604021,
                    leaf_id: 8,
                }),
                grouping: Some(vec![Group {
                    group_key: "collection".to_string(),
                    group_value: Some("AMSNskm2RZqPXCZ6P2z6JLyHWMQF6pQ8RA8Q6x42Xufq".to_string()),
                    verified: None,
                    collection_metadata: Some(CollectionMetadata {
                        name: Some("The Art of Flight".to_string()),
                        symbol: Some("".to_string()),
                        image: Some("https://arweave.net/yaXmwoRlmKRb12Fb8vjfpDhgo8qDYn87unpLqCclOvo?ext=jpeg".to_string()),
                        description: Some("A collection of aerial photography by Andrew Mason.".to_string()),
                        external_url: Some("".to_string()),
                    }),
                }]),
                royalty: Some(Royalty {
                    royalty_model: RoyaltyModel::Creators,
                    target: None,
                    percent: 0.065,
                    basis_points: 650,
                    primary_sale_happened: false,
                    locked: false,
                }),
                creators: Some(vec![
                    Creator {
                        address: "AMSNnsE3jVjDbKg15pG13RgJYf6JCFmCVXdj4j22h4Pn".to_string(),
                        share: 0,
                        verified: true,
                    },
                    Creator {
                        address: "WoMbXFtdfH8crq2Zi7bQhfGx2Gv8EN4saP13gcdUGog".to_string(),
                        share: 23,
                        verified: false,
                    },
                    Creator {
                        address: "ERpxtwaZd6Ee2WD2vRxzhVXj1kaYdPA4xmwEvavX5iMy".to_string(),
                        share: 77,
                        verified: false,
                    }
                ]),
                ownership: Ownership {
                    frozen: false,
                    delegated: false,
                    delegate: None,
                    ownership_model: OwnershipModel::Single,
                    owner: "EwjZNHVVcKdu2Dkvp2fpYdfFLmE9E9TierKefdGBxyS".to_string(),
                },
                supply: Some(Supply {
                    print_max_supply: None,
                    print_current_supply: None,
                    edition_nonce: None,
                    edition_number: None,
                    master_edition_mint: None,
                }),
                uses: None,
                mutable: true,
                burnt: false,
                mint_extensions: None,
                token_info: None,
                group_definition: None,
                plugins: None,
                unknown_plugins: None,
                mpl_core_info: None,
            })
        ],
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

    let request: GetAssetBatch = GetAssetBatch {
        ids: vec![
            "81bxPqYCE8j34nQm7Rooqi8Vt3iMHLzgZJ71rUVbQQuz".to_string(),
            "CWHuz6GPjWYdwt7rTfRHKaorMwZP58Spyd7aqGK7xFbn".to_string(),
        ],
        display_options: Some(GetAssetOptions {
            show_collection_metadata: true,
            ..Default::default()
        }),
    };

    let response: Result<Vec<Option<Asset>>> = helius.rpc().get_asset_batch(request).await;
    assert!(response.is_ok(), "API call failed with the error: {:?}", response.err());

    let assets: Vec<Option<Asset>> = response.unwrap();
    assert_eq!(assets.len(), 2, "Expected two assets to be returned");
}

#[tokio::test]
async fn test_get_asset_batch_failure() {
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

    let request: GetAssetBatch = GetAssetBatch {
        ids: vec!["invalid_id".to_string(), "!@#$%^&*()_+".to_string()],
        display_options: Some(GetAssetOptions {
            show_collection_metadata: true,
            ..Default::default()
        }),
    };

    let response: Result<Vec<Option<Asset>>> = helius.rpc().get_asset_batch(request).await;
    assert!(response.is_err(), "Expected an error but got a successful response");
}
