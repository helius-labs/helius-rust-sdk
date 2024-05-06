use std::sync::Arc;

use helius_sdk::client::Helius;
use helius_sdk::config::Config;
use helius_sdk::error::HeliusError;
use helius_sdk::rpc_client::RpcClient;
use helius_sdk::types::*;

use mockito::{self, Server};
use reqwest::Client;

#[tokio::test]
async fn test_get_token_accounts_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<TokenAccountsList> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: TokenAccountsList {
            total: 1,
            limit: 1,
            page: Some(1),
            cursor: None,
            before: None,
            after: None,
            token_accounts: vec![TokenAccount {
                address: "FDxksmT4hRCpS2Wr5NF2i3uuYGeTz6pSyychb44gDzL".to_string(),
                mint: Some("2v5byxxWVeAcrN39fznVrhuWZuoPkjpzGuqJHemyqP1x".to_string()),
                owner: Some("GdNh12yVy5Lsew9WXVCV5ErgK5SpmsBJkcti5jVtPB7o".to_string()),
                amount: Some(1),
                delegate: None,
                delegated_amount: Some(0),
                token_extensions: None,
                frozen: true,
            }],
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
    };

    let request: GetTokenAccounts = GetTokenAccounts {
        owner: Some("GdNh12yVy5Lsew9WXVCV5ErgK5SpmsBJkcti5jVtPB7o".to_string()),
        page: Some(1),
        limit: Some(1),
        ..Default::default()
    };

    let response: Result<TokenAccountsList, HeliusError> = helius.rpc().get_token_accounts(request).await;
    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let token_accounts: TokenAccountsList = response.unwrap();
    assert!(
        token_accounts.total > 0,
        "No token account returned when one was expected"
    );

    assert_eq!(
        token_accounts.token_accounts[0].address, "FDxksmT4hRCpS2Wr5NF2i3uuYGeTz6pSyychb44gDzL",
        "signature does not match expected value"
    );
}

#[tokio::test]
async fn test_get_token_accounts_failure() {
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

    let request = GetTokenAccounts {
        owner: Some("GdNh12yVy5Lsew9WXVCV5ErgK5SpmsBJkcti5jVtPB7o".to_string()),
        page: Some(1),
        limit: Some(1),
        ..Default::default()
    };

    let response: Result<TokenAccountsList, HeliusError> = helius.rpc().get_token_accounts(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
