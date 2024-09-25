use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{
    ApiResponse, Asset, AssetList, Attribute, Authorities, Cluster, Compression, Content, Creator, File,
    GetAssetsByOwner, Group, HeliusEndpoints, Interface, Links, Metadata, Ownership, OwnershipModel, Royalty,
    RoyaltyModel, Scope, Supply,
};
use helius::Helius;

use mockito::{self, Server};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

#[tokio::test]
async fn test_get_assets_by_owner_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<AssetList> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: AssetList {
            grand_total: None,
            total: 1,
            limit: 1,
            page: Some(1),
            before: None,
            after: None,
            cursor: None,
            items: vec![Asset {
                interface: Interface::V1NFT,
                id: "HcaSe8RfGASYdPws2NFu7y84C1mWAhsdnhDDSWAQGfmh".to_string(),
                content: Some(Content {
                    schema: "https://schema.metaplex.com/nft1.0.json".to_string(),
                    json_uri: "https://www.hi-hi.vip/json/3000jup.json".to_string(),
                    files: Some(vec![File {
                        uri: Some("https://img.hi-hi.vip/json/img/3000jup.png".to_string()),
                        mime: Some("image/png".to_string()),
                        cdn_uri: Some(
                            "https://cdn.helius-rpc.com/cdn-cgi/image//https://img.hi-hi.vip/json/img/3000jup.png"
                                .to_string(),
                        ),
                        quality: None,
                        contexts: None,
                    }]),
                    metadata: Metadata {
                        attributes: Some(vec![
                            Attribute {
                                value: Value::String("https://3000jup.com".to_string()),
                                trait_type: "Website".to_string(),
                            },
                            Attribute {
                                value: Value::String("True".to_string()),
                                trait_type: "Verified".to_string(),
                            },
                            Attribute {
                                value: Value::String("3,000+ JUP ($1800+)".to_string()),
                                trait_type: "Amount".to_string(),
                            },
                            Attribute {
                                value: Value::String("35 minutes!".to_string()),
                                trait_type: "Time Left".to_string(),
                            },
                        ]),
                        description: Some(
                            "Visit the domain shown in the picture and claim your exclusive voucher 3000jup.com"
                                .to_string(),
                        ),
                        name: Some("3000Jup For You 3000Jup.com".to_string()),
                        symbol: Some("JFY".to_string()),
                    },
                    links: Some(Links {
                        external_url: Some("https://3000jup.com".to_string()),
                        image: Some("https://img.hi-hi.vip/json/img/3000jup.png".to_string()),
                        animation_url: None,
                    }),
                }),
                authorities: Some(vec![Authorities {
                    address: "EobuvydowsixNH5YR3fXKJSQGn4S3z8sA11hzkwnET9j".to_string(),
                    scopes: vec![Scope::Full],
                }]),
                compression: Some(Compression {
                    eligible: false,
                    compressed: true,
                    data_hash: "AXcVRZ11inm3rnG2JhRyKfMZCqaZo1aAURhsx1hPC9ch".to_string(),
                    creator_hash: "4CYWKsNbuGh3dsDeWhHghchpiNTNiuL8ttex8FQJ18x4".to_string(),
                    asset_hash: "JD1U2XhhUzDakmDLnYKLSXQTDQB6sbTdjLVSFkKS9ZHL".to_string(),
                    tree: "DBfiwSuJ5Zd5EisfECLmcr55DdF74zxrzqKvjHMU31Je".to_string(),
                    seq: 48371,
                    leaf_id: 48338,
                }),
                grouping: Some(vec![Group {
                    group_key: "collection".to_string(),
                    group_value: Some("DvvGHph6KMGKfEfXvPAdGaxAxhwimFymp5NLjfhaNXfF".to_string()),
                    verified: None,
                    collection_metadata: None,
                }]),
                royalty: Some(Royalty {
                    royalty_model: RoyaltyModel::Creators,
                    target: None,
                    percent: 0.0,
                    basis_points: 0,
                    primary_sale_happened: false,
                    locked: false,
                }),
                creators: Some(vec![Creator {
                    address: "Gs2oeyBi3SMgovFiSWVPds7HUKUhVDMpAjNJxY1k4zW7".to_string(),
                    share: 100,
                    verified: true,
                }]),
                ownership: Ownership {
                    frozen: false,
                    delegated: true,
                    delegate: Some("Gs2oeyBi3SMgovFiSWVPds7HUKUhVDMpAjNJxY1k4zW7".to_string()),
                    ownership_model: OwnershipModel::Single,
                    owner: "GNPwr9fk9RJbfy9nSKbNiz5NPfc69KVwnizverx6fNze".to_string(),
                },
                uses: None,
                supply: Some(Supply {
                    print_max_supply: None,
                    print_current_supply: None,
                    edition_nonce: None,
                    edition_number: None,
                    master_edition_mint: None,
                }),
                mutable: true,
                burnt: false,
                mint_extensions: None,
                token_info: None,
                group_definition: None,
                plugins: None,
                unknown_plugins: None,
                mpl_core_info: None,
            }],
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

    let request: GetAssetsByOwner = GetAssetsByOwner {
        owner_address: "GNPwr9fk9RJbfy9nSKbNiz5NPfc69KVwnizverx6fNze".to_string(),
        page: 1,
        limit: None,
        before: None,
        after: None,
        display_options: None,
        sort_by: None,
        ..Default::default()
    };

    let response: Result<AssetList> = helius.rpc().get_assets_by_owner(request).await;
    assert!(response.is_ok(), "The API call failed: {:?}", response.err());

    let api_response: AssetList = response.unwrap();
    assert_eq!(api_response.total, 1, "Total does not match");
    assert_eq!(api_response.items.len(), 1, "Items count does not match");
}

#[tokio::test]
async fn test_get_assets_by_owner_failure() {
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

    let request: GetAssetsByOwner = GetAssetsByOwner {
        owner_address: "GNPwr9fk9RJbfy9nSKbNiz5NPfc69KVwnizverx6fNze".to_string(),
        page: 1,
        limit: None,
        before: None,
        after: None,
        display_options: None,
        sort_by: None,
        ..Default::default()
    };

    let response: Result<AssetList> = helius.rpc().get_assets_by_owner(request).await;
    assert!(response.is_err(), "Expected an error due to server failure");
}
