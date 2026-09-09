use std::error::Error;
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
// Admin endpoint error tests (GET /v0/admin/projects/.../usage)
// ---------------------------------------------------------------------------

async fn admin_project_usage_error_test(status: u16, body: &str) -> HeliusError {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock("GET", "/v0/admin/projects/proj-123/usage?api-key=fake_api_key")
        .with_status(status.into())
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create();

    let helius = create_test_helius(&url);
    helius.get_project_usage("proj-123").await.unwrap_err()
}

#[tokio::test]
async fn test_admin_bad_request_400() {
    let err = admin_project_usage_error_test(400, r#"{"error":"Invalid project ID"}"#).await;
    assert!(
        matches!(err, HeliusError::BadRequest { ref text, .. } if text.contains("Invalid project ID")),
        "Expected BadRequest, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_admin_unauthorized_401() {
    let err = admin_project_usage_error_test(401, r#"{"error":"Invalid API key"}"#).await;
    assert!(
        matches!(err, HeliusError::Unauthorized { .. }),
        "Expected Unauthorized, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_admin_forbidden_403() {
    let err = admin_project_usage_error_test(403, r#"{"error":"Admin API not enabled"}"#).await;
    assert!(
        matches!(err, HeliusError::Unauthorized { .. }),
        "Expected Unauthorized (403 maps to Unauthorized), got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_admin_not_found_404() {
    let err = admin_project_usage_error_test(404, r#"{"error":"Project not found"}"#).await;
    assert!(
        matches!(err, HeliusError::NotFound { ref text } if text.contains("Project not found")),
        "Expected NotFound, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_admin_rate_limit_429() {
    let err = admin_project_usage_error_test(429, r#"{"error":"Too many requests"}"#).await;
    assert!(
        matches!(err, HeliusError::RateLimitExceeded { .. }),
        "Expected RateLimitExceeded, got: {:?}",
        err
    );
}

#[tokio::test]
async fn test_admin_internal_error_500() {
    let err = admin_project_usage_error_test(500, r#"{"error":"Internal Server Error"}"#).await;
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

// Reserves a port by binding it, then releases it, so a connect to it is refused rather than
// left hanging. Deterministic where a hardcoded "probably closed" port is not.
fn closed_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    port
}

fn source_chain(err: &dyn Error) -> Vec<String> {
    let mut chain = Vec::new();
    let mut next = err.source();
    while let Some(cause) = next {
        chain.push(cause.to_string());
        next = cause.source();
    }
    chain
}

/// Walks the chain for an `io::Error` of the given kind. Asserting on the kind rather than on the
/// rendered message keeps the test independent of the OS error number (61 on macOS, 111 on Linux)
/// and of the locale, since `io::Error`'s `Display` comes from `strerror` on Unix.
fn chain_has_io_kind(err: &dyn Error, kind: std::io::ErrorKind) -> bool {
    let mut next = err.source();
    while let Some(cause) = next {
        if cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io| io.kind() == kind)
        {
            return true;
        }
        next = cause.source();
    }
    false
}

/// `reqwest` deliberately omits its own cause chain from `Display`, so a `HeliusError` that wraps a
/// `reqwest::Error` without marking it `#[source]` reports *that* a request failed and never why:
/// "Network error: error sending request for url (...)" with `source()` returning `None`. Every
/// chain-aware consumer (`anyhow`'s `{:#}`, `eyre`, `tracing`) then has nothing to print.
#[tokio::test]
async fn test_network_error_preserves_reqwest_source() {
    let url = format!("http://127.0.0.1:{}/", closed_port());
    let helius = create_test_helius(&url);

    let err = helius.get_all_webhooks().await.expect_err("connect must fail");
    assert!(
        matches!(err, HeliusError::Network(_)),
        "Expected Network, got: {:?}",
        err
    );

    let chain = source_chain(&err);
    assert!(!chain.is_empty(), "source() is None; the reqwest error was swallowed");

    // The chain has to reach past the `reqwest::Error` itself and into its causes, all the way to
    // the OS error that says *why* the request failed — that is the part callers could not see.
    assert!(
        chain_has_io_kind(&err, std::io::ErrorKind::ConnectionRefused),
        "underlying cause is unreachable from the chain: {:?}",
        chain
    );
}

#[tokio::test]
async fn test_network_error_classification_accessors() {
    let url = format!("http://127.0.0.1:{}/", closed_port());
    let helius = create_test_helius(&url);

    let err = helius.get_all_webhooks().await.expect_err("connect must fail");
    assert!(
        err.as_reqwest().is_some(),
        "as_reqwest() should expose the reqwest error"
    );
    assert!(err.is_connect(), "a refused connection should classify as is_connect()");
    assert!(!err.is_timeout(), "a refused connection is not a timeout");
}

#[test]
fn test_serde_json_error_preserves_source() {
    let serde_err = serde_json::from_str::<Vec<u8>>("{not json}").expect_err("must fail to parse");
    let err = HeliusError::SerdeJson(serde_err);
    assert!(
        !source_chain(&err).is_empty(),
        "source() is None; the serde_json error was swallowed"
    );
}

#[test]
fn test_url_parse_error_displays_the_cause() {
    let parse_err = "not a url".parse::<reqwest::Url>().expect_err("must fail to parse");
    // Captured before the move, so the assertion tracks whatever `url` calls this rather than
    // hardcoding its current wording.
    let expected = parse_err.to_string();

    let err = HeliusError::from(parse_err);
    assert!(
        matches!(err, HeliusError::UrlParseError(_)),
        "Expected UrlParseError, got: {:?}",
        err
    );
    // A bare "Url parse error" tells a caller nothing about why the URL was rejected.
    assert!(
        err.to_string().contains(&expected),
        "Display should name the parse failure ({}), got: {}",
        expected,
        err
    );
}
