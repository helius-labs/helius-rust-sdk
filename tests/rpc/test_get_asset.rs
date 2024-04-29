use std::sync::Arc;

use helius_sdk::client::Helius;
use helius_sdk::config::Config;
use helius_sdk::error::HeliusError;
use helius_sdk::rpc_client::RpcClient;
use helius_sdk::types::{
    ApiResponse, Attribute, Authorities, Cluster, Compression, Content, Creators, DisplayOptions, File,
    GetAssetRequest, GetAssetResponseForAsset, Grouping, HeliusEndpoints, Interface, Links, Metadata, Ownership,
    OwnershipModel, ResponseType, Royalty, RoyaltyModel, Scope, Supply,
};

use mockito::{self, Server};
use reqwest::Client;

#[tokio::test]
async fn test_get_asset_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: ResponseType::GetAssetResponseForAsset(GetAssetResponseForAsset {
            interface: Interface::ProgrammableNFT,
            id: "F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk".to_string(),
            content: Some(Content {
                schema: "https://schema.metaplex.com/nft1.0.json".to_string(),
                json_uri: "https://madlads.s3.us-west-2.amazonaws.com/json/8420.json".to_string(),
                files: Some(vec![File {
                    uri: Some("https://madlads.s3.us-west-2.amazonaws.com/images/8420.png".to_string()),
                    mime: Some("image/png".to_string()),
                    cdn_uri: Some("https://cdn.helius-rpc.com/cdn-cgi/image//https://madlads.s3.us-west-2.amazonaws.com/images/8420.png".to_string()),
                    quality: None,
                    contexts: None,
                }, File {
                    uri: Some("https://arweave.net/qJ5B6fx5hEt4P7XbicbJQRyTcbyLaV-OQNA1KjzdqOQ/0.png".to_string()),
                    mime: Some("image/png".to_string()),
                    cdn_uri: Some("https://cdn.helius-rpc.com/cdn-cgi/image//https://arweave.net/qJ5B6fx5hEt4P7XbicbJQRyTcbyLaV-OQNA1KjzdqOQ/0.png".to_string()),
                    quality: None,
                    contexts: None,
                }]),
                metadata: Metadata {
                    attributes: Some(vec![Attribute {
                        value: "Male".to_string(),
                        trait_type: "Gender".to_string(),
                    }, Attribute {
                        value: "King".to_string(),
                        trait_type: "Type".to_string(),
                    }, Attribute {
                        value: "Royal".to_string(),
                        trait_type: "Expression".to_string(),
                    }, Attribute {
                        value: "Mad Crown".to_string(),
                        trait_type: "Hat".to_string(),
                    }, Attribute {
                        value: "Madness".to_string(),
                        trait_type: "Eyes".to_string(),
                    }, Attribute {
                        value: "Mad Armor".to_string(),
                        trait_type: "Clothing".to_string(),
                    }, Attribute {
                        value: "Royal Rug".to_string(),
                        trait_type: "Background".to_string()
                    }]),
                    description: Some("Fock it.".to_string()),
                    name: "Mad Lads #8420".to_string(),
                    symbol: "MAD".to_string()
                },
                links: Some(Links {
                    external_url: Some("https://madlads.com".to_string()),
                    image: Some("https://madlads.s3.us-west-2.amazonaws.com/images/8420.png".to_string()),
                    animation_url: None
                }),
            }),
            authorities: Some(vec![Authorities {
                address: "2RtGg6fsFiiF1EQzHqbd66AhW7R5bWeQGpTbv2UMkCdW".to_string(),
                scopes: vec![Scope::Full],
            }]),
            compression: Some(Compression {
                eligible: false,
                compressed: false,
                data_hash: "".to_string(),
                creator_hash: "".to_string(),
                asset_hash: "".to_string(),
                tree: "".to_string(),
                seq: 0,
                leaf_id: 0,
            }),
            grouping: Some(vec![Grouping {
                group_key: "collection".to_string(),
                group_value: "J1S9H3QjnRtBbbuD4HjPV6RpRhwuk4zKbxsnCHuTgh9w".to_string(),
                verified: None,
                collection_metadata: None,
            }]),
            royalty: Some(Royalty {
                royalty_model: RoyaltyModel::Creators,
                target: None,
                percent: 0.042,
                basis_points: 420,
                primary_sale_happened: true,
                locked: false,
            }),
            creators: Some(vec![Creators {
                address: "5XvhfmRjwXkGp3jHGmaKpqeerNYjkuZZBYLVQYdeVcRv".to_string(),
                share: 0,
                verified: true,
            }, Creators {
                address: "2RtGg6fsFiiF1EQzHqbd66AhW7R5bWeQGpTbv2UMkCdW".to_string(),
                share: 100,
                verified: true,
            }]),
            ownership: Ownership {
                frozen: true,
                delegated: false,
                delegate: None,
                ownership_model: OwnershipModel::Single,
                owner: "4zdNGgAtFsW1cQgHqkiWyRsxaAgxrSRRynnuunxzjxue".to_string(),
            },
            mint_extensions: None,
            supply: Some(Supply {
                print_max_supply: None,
                print_current_supply: None,
                edition_nonce: None,
                edition_number: None,
                master_edition_mint: None,
            }),
            token_info: None,
            inscription: None,
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
    };

    let request: GetAssetRequest = GetAssetRequest {
        id: "F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk".to_string(),
        display_options: Some(DisplayOptions {
            show_unverified_collections: false,
            show_collection_metadata: false,
            show_fungible: false,
            show_inscription: false,
        }),
    };

    let response: Result<Option<GetAssetResponseForAsset>, HeliusError> = helius.rpc().get_asset(request).await;
    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let asset_response: Option<GetAssetResponseForAsset> = response.unwrap();
    assert!(asset_response.is_some(), "No asset returned when one was expected");

    let asset: GetAssetResponseForAsset = asset_response.unwrap();
    assert_eq!(
        asset.id, "F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk",
        "Asset ID does not match expected value"
    );
    assert_eq!(
        asset.interface,
        Interface::ProgrammableNFT,
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
    };

    let request: GetAssetRequest = GetAssetRequest {
        id: "F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk".to_string(),
        display_options: Some(DisplayOptions {
            show_unverified_collections: false,
            show_collection_metadata: false,
            show_fungible: false,
            show_inscription: false,
        }),
    };

    let response: Result<Option<GetAssetResponseForAsset>, HeliusError> = helius.rpc().get_asset(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
