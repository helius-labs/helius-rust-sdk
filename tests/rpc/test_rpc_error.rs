use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::error::{HeliusError, Result};
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::{self, Server};
use reqwest::Client;

/// Builds a `Helius` client wired to the given mock server URL.
fn mock_helius(url: &str) -> Helius {
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

    Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    }
}

/// A JSON-RPC error object returned with an HTTP 200 status (as Solana/Helius do for
/// method-level failures) must surface as `HeliusError::RpcError`, not a deserialization error.
#[tokio::test]
async fn test_rpc_error_is_surfaced() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params: missing field `id`"},"id":"1"}"#,
        )
        .create();

    let helius: Helius = mock_helius(&url);

    let request: GetAsset = GetAsset {
        id: "JE9mLqmTRZnUYoMJofSmZp8nZT4pzgARtAJS8crtgVWV".to_string(),
        display_options: None,
    };

    let response: Result<Option<Asset>> = helius.rpc().get_asset(request).await;

    match response {
        Err(HeliusError::RpcError { code, message }) => {
            assert_eq!(code, -32602, "Unexpected JSON-RPC error code");
            assert!(
                message.contains("Invalid params"),
                "Error message did not carry the server description: {message}"
            );
        }
        other => panic!("Expected HeliusError::RpcError, got {other:?}"),
    }
}

/// When the JSON-RPC error object includes a `data` field, its contents are appended to the
/// surfaced error message so callers retain the server's additional context.
#[tokio::test]
async fn test_rpc_error_includes_data() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","error":{"code":-32002,"message":"Transaction simulation failed","data":{"logs":["Program failed"]}},"id":"1"}"#,
        )
        .create();

    let helius: Helius = mock_helius(&url);

    let request: GetAsset = GetAsset {
        id: "JE9mLqmTRZnUYoMJofSmZp8nZT4pzgARtAJS8crtgVWV".to_string(),
        display_options: None,
    };

    let response: Result<Option<Asset>> = helius.rpc().get_asset(request).await;

    match response {
        Err(HeliusError::RpcError { code, message }) => {
            assert_eq!(code, -32002, "Unexpected JSON-RPC error code");
            assert!(
                message.contains("Transaction simulation failed") && message.contains("logs"),
                "Error message did not include the server-supplied data: {message}"
            );
        }
        other => panic!("Expected HeliusError::RpcError, got {other:?}"),
    }
}

/// A `"result": null` success response must still deserialize to `Ok(None)` for methods
/// returning `Option<T>`, rather than being mistaken for a missing result.
#[tokio::test]
async fn test_null_result_is_ok_none() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"jsonrpc":"2.0","result":null,"id":"1"}"#)
        .create();

    let helius: Helius = mock_helius(&url);

    let request: GetAsset = GetAsset {
        id: "JE9mLqmTRZnUYoMJofSmZp8nZT4pzgARtAJS8crtgVWV".to_string(),
        display_options: None,
    };

    let response: Result<Option<Asset>> = helius.rpc().get_asset(request).await;

    assert!(
        matches!(response, Ok(None)),
        "Expected Ok(None) for a null result, got {response:?}"
    );
}

/// Per the JSON-RPC spec, parse errors and invalid requests (`-32700` / `-32600`) are
/// returned with `"id": null`. This must still surface as `HeliusError::RpcError` rather
/// than a cryptic deserialization error over the `id` field.
#[tokio::test]
async fn test_rpc_error_with_null_id() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"jsonrpc":"2.0","error":{"code":-32700,"message":"Parse error"},"id":null}"#)
        .create();

    let helius: Helius = mock_helius(&url);

    let request: GetAsset = GetAsset {
        id: "JE9mLqmTRZnUYoMJofSmZp8nZT4pzgARtAJS8crtgVWV".to_string(),
        display_options: None,
    };

    let response: Result<Option<Asset>> = helius.rpc().get_asset(request).await;

    match response {
        Err(HeliusError::RpcError { code, message }) => {
            assert_eq!(code, -32700, "Unexpected JSON-RPC error code");
            assert!(
                message.contains("Parse error"),
                "Error message did not carry the server description: {message}"
            );
        }
        other => panic!("Expected HeliusError::RpcError, got {other:?}"),
    }
}
