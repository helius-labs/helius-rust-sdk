use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::{self, Server};
use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn test_get_program_accounts_v2_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response = RpcResponse {
        jsonrpc: "2.0".to_string(),
        id: "1".to_string(),
        result: GetProgramAccountsV2Response {
            context: None,
            accounts: vec![GpaAccount {
                pubkey: "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string(),
                account: AccountInfo {
                    lamports: 23357760,
                    owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
                    data: json!(["base64encodeddata", "base64"]),
                    executable: false,
                    rent_epoch: 361,
                    space: Some(165),
                },
            }],
            pagination_key: Some("abc123".to_string()),
            total_results: Some(42),
        },
    };

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: Some(ApiKey::new("fake_api_key").unwrap()),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
        custom_url: None,
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

    let response: Result<GetProgramAccountsV2Response> = helius
        .rpc()
        .get_program_accounts_v2(
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            GetProgramAccountsV2Config::default(),
        )
        .await;
    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let result: GetProgramAccountsV2Response = response.unwrap();
    assert!(result.context.is_none());
    assert_eq!(result.accounts.len(), 1);
    assert_eq!(result.accounts[0].pubkey, "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");
    assert_eq!(result.accounts[0].account.lamports, 23357760);
    assert_eq!(result.pagination_key, Some("abc123".to_string()));
    assert_eq!(result.total_results, Some(42));
}

#[tokio::test]
async fn test_get_program_accounts_v2_with_context() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response = RpcResponse {
        jsonrpc: "2.0".to_string(),
        id: "1".to_string(),
        result: GetProgramAccountsV2Response {
            context: Some(RpcContext {
                slot: 308_150_001,
                api_version: Some("2.2.1".to_string()),
            }),
            accounts: vec![
                GpaAccount {
                    pubkey: "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin".to_string(),
                    account: AccountInfo {
                        lamports: 23357760,
                        owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
                        data: json!(["base64encodeddata", "base64"]),
                        executable: false,
                        rent_epoch: 361,
                        space: Some(165),
                    },
                },
                GpaAccount {
                    pubkey: "3PmRSx5oRLkPdt5P2RjFDHNExgkX1PgHSjaKyjCo8tYE".to_string(),
                    account: AccountInfo {
                        lamports: 2039280,
                        owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
                        data: json!(["aW5pdGlhbGl6ZWQ=", "base64"]),
                        executable: false,
                        rent_epoch: 361,
                        space: Some(165),
                    },
                },
            ],
            pagination_key: None,
            total_results: Some(2),
        },
    };

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: Some(ApiKey::new("fake_api_key").unwrap()),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
        custom_url: None,
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

    let response: Result<GetProgramAccountsV2Response> = helius
        .rpc()
        .get_program_accounts_v2(
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            GetProgramAccountsV2Config {
                with_context: Some(true),
                ..Default::default()
            },
        )
        .await;
    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let result: GetProgramAccountsV2Response = response.unwrap();

    // Verify context is present and correct
    let context = result.context.expect("context should be present when with_context is true");
    assert_eq!(context.slot, 308_150_001);
    assert_eq!(context.api_version, Some("2.2.1".to_string()));

    // Verify accounts
    assert_eq!(result.accounts.len(), 2);
    assert_eq!(result.accounts[0].pubkey, "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");
    assert_eq!(result.accounts[1].pubkey, "3PmRSx5oRLkPdt5P2RjFDHNExgkX1PgHSjaKyjCo8tYE");

    // Verify pagination indicates no more pages
    assert!(result.pagination_key.is_none());
    assert_eq!(result.total_results, Some(2));
}

#[tokio::test]
async fn test_get_program_accounts_v2_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: Some(ApiKey::new("fake_api_key").unwrap()),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
        custom_url: None,
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

    let response: Result<GetProgramAccountsV2Response> = helius
        .rpc()
        .get_program_accounts_v2(
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            GetProgramAccountsV2Config::default(),
        )
        .await;
    assert!(response.is_err(), "Expected an error but got success");
}
