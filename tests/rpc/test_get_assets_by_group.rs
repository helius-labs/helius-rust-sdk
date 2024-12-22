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
async fn test_get_assets_by_group_success() {
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
            items: vec![
                Asset {
                    interface: Interface::ProgrammableNFT,
                    id: "Obi-Wan Kenobi".to_string(),
                    content: Some(Content {
                        schema: "https://schema.metaplex.com/nft1.0.json".to_string(),
                        json_uri: "https://example.com/nft.json".to_string(),
                        files: Some(vec![File {
                            uri: Some("https://example.com/image.png".to_string()),
                            mime: Some("image/png".to_string()),
                            cdn_uri: Some("https://cdn.example.com/image.png".to_string()),
                            quality: None,
                            contexts: None,
                        }]),
                        metadata: Metadata {
                            attributes: Some(vec![Attribute {
                                value: Value::String("Jedi Master".to_string()),
                                trait_type: "Rank".to_string(),
                            }, Attribute {
                                value: Value::String("Galactic Republic".to_string()),
                                trait_type: "Affiliation".to_string(),
                            }, Attribute {
                                value: Value::String("Jedi Order".to_string()),
                                trait_type: "Affiliation".to_string(),
                            }]),
                            description: Some("Obi-Wan Kenobi was a legendary Force-sensitive human male Jedi Master who served on the Jedi High Council during the final years of the Republic Era".to_string()),
                            name: Some("Obi-Wan Kenobi".to_string()),
                            symbol: Some("Guiding Light".to_string()),
                        },
                        links: Some(Links {
                            external_url: Some("https://example.com".to_string()),
                            image: Some("https://example.com/image.png".to_string()),
                            animation_url: None,
                        }),
                    }),
                    authorities: Some(vec![Authorities {
                        address: "JediCouncil".to_string(),
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
                    grouping: Some(vec![Group {
                        group_key: "collection".to_string(),
                        group_value: Some("JediCouncil".to_string()),
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
                    creators: Some(vec![Creator {
                        address: "TheForce".to_string(),
                        share: 100,
                        verified: true,
                    }]),
                    ownership: Ownership {
                        frozen: true,
                        delegated: true,
                        delegate: Some("DelegateAddress".to_string()),
                        ownership_model: OwnershipModel::Single,
                        owner: "OwnerAddress".to_string(),
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

    let sorting: AssetSorting = AssetSorting {
        sort_by: AssetSortBy::Created,
        sort_direction: Some(AssetSortDirection::Asc),
    };

    let request: GetAssetsByGroup = GetAssetsByGroup {
        group_key: "collection".to_string(),
        group_value: "J1S9H3QjnRtBbbuD4HjPV6RpRhwuk4zKbxsnCHuTgh9w".to_string(),
        page: Some(1),
        limit: Some(10),
        sort_by: Some(sorting),
        before: None,
        after: None,
        options: None,
        cursor: None,
    };

    let response: Result<AssetList> = helius.rpc().get_assets_by_group(request).await;
    assert!(response.is_ok(), "Expected an error due to server failure");

    let asset: AssetList = response.unwrap();
    assert_eq!(asset.total, 1);
    assert_eq!(
        asset.items[0].content.as_ref().unwrap().metadata.name,
        Some("Obi-Wan Kenobi".to_string())
    );
}

#[tokio::test]
async fn test_get_assets_by_group_failure() {
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

    let sorting: AssetSorting = AssetSorting {
        sort_by: AssetSortBy::Created,
        sort_direction: Some(AssetSortDirection::Asc),
    };

    let request: GetAssetsByGroup = GetAssetsByGroup {
        group_key: "collection".to_string(),
        group_value: "Bad Request".to_string(),
        page: Some(1),
        limit: Some(10),
        sort_by: Some(sorting),
        ..Default::default()
    };

    let response: Result<AssetList> = helius.rpc().get_assets_by_group(request).await;
    assert!(response.is_err(), "Expected an error due to server failure");
}
