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
async fn test_mint_compressed_nft() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<MintResponse> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: MintResponse {
            signature: "rqs2XsREo7Q15dmPgWmneYo9NUqA2z1RrvYYMoynJWtr3rUfqBy9gZhXorWwLGowZ63Sodnciwt62Y79F7CSnSu"
                .to_string(),
            minted: true,
            asset_id: Some("FhYMMur2tMTWXyWEvHKX1zqzMkGSJk4sXATgrcnqAdGL".to_string()),
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

    let request: MintCompressedNftRequest = MintCompressedNftRequest {
        name: "Qui-Gon Jinn".to_string(),
        symbol: "QGJ".to_string(),
        owner: "DCQnfUH6mHA333mzkU22b4hMvyqcejUBociodq8bB5HF".to_string(),
        description: "Qui-Gon Jinn, a Force-sensitive human male, was a venerable if maverick Jedi Master who lived during the last years of the Republic Era".to_string(),
        attributes: vec![Attribute {
                trait_type: "Rank".to_string(),
                value: Value::String("Jedi Master".to_string()),
            }, Attribute {
                trait_type: "Masters".to_string(),
                value: Value::String("Dooku".to_string()),
            }, Attribute {
                trait_type: "Homeworld".to_string(),
                value: Value::String("Coruscant".to_string()),
            }, Attribute {
                trait_type: "Quote".to_string(),
                value: Value::String("There's always a bigger fish".to_string()),
            },
        ],
        image_url: Some("https://static.wikia.nocookie.net/starwars/images/f/f6/Qui-Gon_Jinn_Headshot_TPM.jpg/revision/latest?cb=20180430174809".to_string()),
        external_url: Some("https://starwars.fandom.com/wiki/Qui-Gon_Jinn".to_string()),
        seller_fee_basis_points: Some(6900),
        delegate: None,
        collection: None,
        uri: None,
        creators: None,
        confirm_transaction: Some(true),
    };

    let result: Result<MintResponse> = helius.mint_compressed_nft(request).await;
    assert!(result.is_ok(), "API call failed with error: {:?}", result.err());

    let mint_response: MintResponse = result.unwrap();
    assert_eq!(
        mint_response.signature,
        "rqs2XsREo7Q15dmPgWmneYo9NUqA2z1RrvYYMoynJWtr3rUfqBy9gZhXorWwLGowZ63Sodnciwt62Y79F7CSnSu"
    );
    assert!(mint_response.minted);
    assert_eq!(
        mint_response.asset_id,
        Some("FhYMMur2tMTWXyWEvHKX1zqzMkGSJk4sXATgrcnqAdGL".to_string())
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
        async_rpc_client: None,
        ws_client: None,
    };

    let request: MintCompressedNftRequest = MintCompressedNftRequest {
        name: "Darth Bane".to_string(),
        symbol: "DB".to_string(),
        owner: "Invalid Owner".to_string(),
        description: "Darth Bane was a legendary human male Dark Lord of the Sith and the sole survivor of the destruction of the Brotherhood of Darkness at the hands of the Jedi Order during the Jedi-Sith War a thousand years before the Clone Wars".to_string(),
        attributes: vec![Attribute {
                trait_type: "Affiliation".to_string(),
                value: Value::String("Sith Order".to_string()),
            }, Attribute {
                trait_type: "Apprentice".to_string(),
                value: Value::String("Darth Zannah".to_string()),
            }, Attribute {
                trait_type: "Homeworld".to_string(),
                value: Value::String("Moraband".to_string()),
            }, Attribute {
                trait_type: "Eyes".to_string(),
                value: Value::String("Yellow".to_string()),
            },
        ],
        image_url: Some("https://static.wikia.nocookie.net/starwars/images/5/5e/DarthBane-GoH-cropped.png/revision/latest?cb=20231227033316".to_string()),
        external_url: Some("https://starwars.fandom.com/wiki/Darth_Bane".to_string()),
        seller_fee_basis_points: Some(6900),
        delegate: Some("".to_string()),
        collection: Some("................".to_string()),
        uri: None,
        creators: None,
        confirm_transaction: Some(true),
    };

    let result: Result<MintResponse> = helius.mint_compressed_nft(request).await;
    assert!(result.is_err(), "Expected an error but got success");
}

#[tokio::test]
async fn test_mint_api_authority_from_cluster_success() {
    let devnet_cluster: Cluster = Cluster::Devnet;
    let mainnet_cluster: Cluster = Cluster::MainnetBeta;

    let devnet_authority: std::result::Result<MintApiAuthority, &str> = MintApiAuthority::from_cluster(devnet_cluster);
    let mainnet_authority: std::result::Result<MintApiAuthority, &str> =
        MintApiAuthority::from_cluster(mainnet_cluster);

    assert_eq!(
        devnet_authority.unwrap(),
        MintApiAuthority::Devnet("2LbAtCJSaHqTnP9M5QSjvAMXk79RNLusFspFN5Ew67TC"),
        "Devnet authority did not match expected value"
    );
    assert_eq!(
        mainnet_authority.unwrap(),
        MintApiAuthority::Mainnet("HnT5KVAywGgQDhmh6Usk4bxRg4RwKxCK4jmECyaDth5R"),
        "Mainnet authority did not match expected value"
    );
}

#[tokio::test]
async fn test_mint_api_authority_from_cluster_failure() {
    let devnet_cluster: Cluster = Cluster::Devnet;
    let mainnet_cluster: Cluster = Cluster::MainnetBeta;

    let devnet_authority: std::result::Result<MintApiAuthority, &str> = MintApiAuthority::from_cluster(devnet_cluster);
    let mainnet_authority: std::result::Result<MintApiAuthority, &str> =
        MintApiAuthority::from_cluster(mainnet_cluster);

    assert_ne!(
        devnet_authority.unwrap(),
        MintApiAuthority::Devnet("Blade"),
        "Devnet authority did not match expected value"
    );
    assert_ne!(
        mainnet_authority.unwrap(),
        MintApiAuthority::Mainnet("Deacon Frost"),
        "Mainnet authority did not match expected value"
    );
}
