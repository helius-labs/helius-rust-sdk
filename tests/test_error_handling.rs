use std::sync::Arc;

use helius::config::Config;
use helius::error::HeliusError;
use helius::rpc_client::RpcClient;
use helius::types::*;
use helius::Helius;

use mockito::Server;
use reqwest::Client;

fn create_test_helius(url: &str) -> Helius {
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

// ---------------------------------------------------------------------------
// RPC endpoint error tests (POST /?api-key=...)
// ---------------------------------------------------------------------------

async fn rpc_error_test(status: u16, body: &str) -> HeliusError {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(status.into())
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create();

    let helius = create_test_helius(&url);
    let request = GetAsset {
        id: "test_asset_id".to_string(),
        display_options: None,
    };

    helius.rpc().get_asset(request).await.unwrap_err()
}

#[tokio::test]
async fn test_rpc_bad_request_400() {
    let err = rpc_error_test(400, r#"{"error":"Invalid asset ID format"}"#).await;
    assert!(
        matches!(err, HeliusError::BadRequest { ref text, .. } if text.contains("Invalid asset ID format")),
        "Expected BadRequest, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_rpc_unauthorized_401() {
    let err = rpc_error_test(401, r#"{"error":"Invalid API key"}"#).await;
    assert!(
        matches!(err, HeliusError::Unauthorized { .. }),
        "Expected Unauthorized, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_rpc_forbidden_403() {
    let err = rpc_error_test(403, r#"{"error":"Forbidden"}"#).await;
    assert!(
        matches!(err, HeliusError::Unauthorized { .. }),
        "Expected Unauthorized (403 maps to Unauthorized), got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_rpc_not_found_404() {
    let err = rpc_error_test(404, r#"{"error":"Resource not found"}"#).await;
    assert!(
        matches!(err, HeliusError::NotFound { ref text } if text.contains("Resource not found")),
        "Expected NotFound, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_rpc_rate_limit_429() {
    let err = rpc_error_test(429, r#"{"error":"Rate limit exceeded"}"#).await;
    assert!(
        matches!(err, HeliusError::RateLimitExceeded { .. }),
        "Expected RateLimitExceeded, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_rpc_internal_error_500() {
    let err = rpc_error_test(500, r#"{"error":"Internal Server Error"}"#).await;
    assert!(
        matches!(err, HeliusError::InternalError { ref text, .. } if text.contains("Internal Server Error")),
        "Expected InternalError, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_rpc_unknown_status_503() {
    let err = rpc_error_test(503, r#"{"error":"Service Unavailable"}"#).await;
    assert!(
        matches!(err, HeliusError::Unknown { .. }),
        "Expected Unknown for 503, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_rpc_error_with_object_body() {
    let err = rpc_error_test(
        400,
        r#"{"error":{"code":"INVALID_PARAM","message":"Invalid parameter value"}}"#,
    )
    .await;
    assert!(
        matches!(err, HeliusError::BadRequest { ref text, .. } if text.contains("INVALID_PARAM") && text.contains("Invalid parameter value")),
        "Expected BadRequest with structured error, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_rpc_error_with_non_json_body() {
    let err = rpc_error_test(500, "Gateway Timeout").await;
    assert!(
        matches!(err, HeliusError::InternalError { ref text, .. } if text.contains("Gateway Timeout")),
        "Expected InternalError with raw text, got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Wallet endpoint error tests (GET /v1/wallet/...)
// ---------------------------------------------------------------------------

async fn wallet_identity_error_test(status: u16, body: &str) -> HeliusError {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock(
            "GET",
            "/v1/wallet/TestAddr111111111111111111111111111111111/identity?api-key=fake_api_key",
        )
        .with_status(status.into())
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create();

    let helius = create_test_helius(&url);
    helius
        .get_wallet_identity("TestAddr111111111111111111111111111111111")
        .await
        .unwrap_err()
}

#[tokio::test]
async fn test_wallet_bad_request_400() {
    let err = wallet_identity_error_test(400, r#"{"error":"Invalid address"}"#).await;
    assert!(
        matches!(err, HeliusError::BadRequest { ref text, .. } if text.contains("Invalid address")),
        "Expected BadRequest, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_wallet_unauthorized_401() {
    let err = wallet_identity_error_test(401, r#"{"error":"Invalid API key"}"#).await;
    assert!(
        matches!(err, HeliusError::Unauthorized { .. }),
        "Expected Unauthorized, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_wallet_not_found_404() {
    let err = wallet_identity_error_test(404, r#"{"error":"Wallet not found"}"#).await;
    assert!(
        matches!(err, HeliusError::NotFound { ref text } if text.contains("Wallet not found")),
        "Expected NotFound, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_wallet_rate_limit_429() {
    let err = wallet_identity_error_test(429, r#"{"error":"Too many requests"}"#).await;
    assert!(
        matches!(err, HeliusError::RateLimitExceeded { .. }),
        "Expected RateLimitExceeded, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_wallet_internal_error_500() {
    let err = wallet_identity_error_test(500, r#"{"error":"Internal Server Error"}"#).await;
    assert!(
        matches!(err, HeliusError::InternalError { .. }),
        "Expected InternalError, got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Enhanced transactions endpoint error tests (POST /v0/transactions?...)
// ---------------------------------------------------------------------------

async fn parse_transactions_error_test(status: u16, body: &str) -> HeliusError {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock("POST", "/v0/transactions?api-key=fake_api_key")
        .with_status(status.into())
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create();

    let helius = create_test_helius(&url);
    let request = ParseTransactionsRequest {
        transactions: vec!["test_sig".to_string()],
    };

    helius.parse_transactions(request).await.unwrap_err()
}

#[tokio::test]
async fn test_enhanced_tx_bad_request_400() {
    let err = parse_transactions_error_test(400, r#"{"error":"Invalid transaction signature"}"#).await;
    assert!(
        matches!(err, HeliusError::BadRequest { ref text, .. } if text.contains("Invalid transaction signature")),
        "Expected BadRequest, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_enhanced_tx_unauthorized_401() {
    let err = parse_transactions_error_test(401, r#"{"error":"Invalid API key"}"#).await;
    assert!(
        matches!(err, HeliusError::Unauthorized { .. }),
        "Expected Unauthorized, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_enhanced_tx_rate_limit_429() {
    let err = parse_transactions_error_test(429, r#"{"error":"Rate limit exceeded"}"#).await;
    assert!(
        matches!(err, HeliusError::RateLimitExceeded { .. }),
        "Expected RateLimitExceeded, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_enhanced_tx_internal_error_500() {
    let err = parse_transactions_error_test(500, r#"{"error":"Internal Server Error"}"#).await;
    assert!(
        matches!(err, HeliusError::InternalError { .. }),
        "Expected InternalError, got: {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Webhook endpoint error tests (POST /v0/webhooks?...)
// ---------------------------------------------------------------------------

async fn webhook_get_all_error_test(status: u16, body: &str) -> HeliusError {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock("GET", "/v0/webhooks?api-key=fake_api_key")
        .with_status(status.into())
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create();

    let helius = create_test_helius(&url);
    helius.get_all_webhooks().await.unwrap_err()
}

#[tokio::test]
async fn test_webhook_bad_request_400() {
    let err = webhook_get_all_error_test(400, r#"{"error":"Bad request"}"#).await;
    assert!(
        matches!(err, HeliusError::BadRequest { .. }),
        "Expected BadRequest, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_webhook_unauthorized_401() {
    let err = webhook_get_all_error_test(401, r#"{"error":"Invalid API key"}"#).await;
    assert!(
        matches!(err, HeliusError::Unauthorized { .. }),
        "Expected Unauthorized, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_webhook_rate_limit_429() {
    let err = webhook_get_all_error_test(429, r#"{"error":"Rate limit exceeded"}"#).await;
    assert!(
        matches!(err, HeliusError::RateLimitExceeded { .. }),
        "Expected RateLimitExceeded, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_webhook_internal_error_500() {
    let err = webhook_get_all_error_test(500, r#"{"error":"Internal Server Error"}"#).await;
    assert!(
        matches!(err, HeliusError::InternalError { .. }),
        "Expected InternalError, got: {:?}",
        err
    );
}
