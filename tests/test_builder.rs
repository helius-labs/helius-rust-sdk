use helius::error::HeliusError;
use helius::types::{ApiKey, Cluster};
use helius::{Helius, HeliusBuilder};
use solana_commitment_config::CommitmentConfig;

// ===== ApiKey Validation =====

#[test]
fn test_api_key_valid() {
    let key = ApiKey::new("my-api-key").unwrap();
    assert_eq!(key.as_str(), "my-api-key");
}

#[test]
fn test_api_key_empty_rejected() {
    assert!(ApiKey::new("").is_err());
}

#[test]
fn test_api_key_whitespace_only_rejected() {
    assert!(ApiKey::new("   ").is_err());
}

#[test]
fn test_api_key_into_string() {
    let key = ApiKey::new("test-key").unwrap();
    let s: String = key.into();
    assert_eq!(s, "test-key");
}

// ===== URL Validation =====

#[test]
fn test_validate_rpc_url_https() {
    assert!(helius::types::validate_rpc_url("https://api.mainnet-beta.solana.com").is_ok());
}

#[test]
fn test_validate_rpc_url_http_localhost() {
    assert!(helius::types::validate_rpc_url("http://localhost:8899").is_ok());
}

#[test]
fn test_validate_rpc_url_rejects_file_scheme() {
    assert!(helius::types::validate_rpc_url("file:///etc/passwd").is_err());
}

#[test]
fn test_validate_rpc_url_rejects_javascript_scheme() {
    assert!(helius::types::validate_rpc_url("javascript:alert(1)").is_err());
}

#[test]
fn test_validate_rpc_url_rejects_credentials() {
    assert!(helius::types::validate_rpc_url("https://user:pass@api.example.com").is_err());
}

#[test]
fn test_validate_rpc_url_rejects_username_only() {
    assert!(helius::types::validate_rpc_url("https://user@api.example.com").is_err());
}

#[test]
fn test_validate_rpc_url_rejects_invalid_url() {
    assert!(helius::types::validate_rpc_url("not a url").is_err());
}

#[test]
fn test_validate_ws_url_wss() {
    assert!(helius::types::validate_ws_url("wss://atlas-mainnet.helius-rpc.com").is_ok());
}

#[test]
fn test_validate_ws_url_ws_localhost() {
    assert!(helius::types::validate_ws_url("ws://localhost:8900").is_ok());
}

#[test]
fn test_validate_ws_url_rejects_http() {
    assert!(helius::types::validate_ws_url("http://example.com").is_err());
}

#[test]
fn test_validate_ws_url_rejects_credentials() {
    assert!(helius::types::validate_ws_url("wss://user:pass@example.com").is_err());
}

// ===== Builder: Basic Construction =====

#[tokio::test]
async fn test_builder_basic_with_api_key_and_cluster() {
    let result = HeliusBuilder::new()
        .with_api_key("test-key")
        .unwrap()
        .with_cluster(Cluster::Devnet)
        .build()
        .await;

    assert!(result.is_ok());
    let helius = result.unwrap();
    assert!(helius.config.api_key.is_some());
    assert_eq!(helius.config.api_key.as_ref().unwrap().as_str(), "test-key");
    assert!(helius.config.custom_url.is_none());
    assert!(helius.async_rpc_client.is_none());
    assert!(helius.ws_client.is_none());
}

#[tokio::test]
async fn test_builder_fails_without_cluster_or_url() {
    let result = HeliusBuilder::new()
        .with_api_key("test-key")
        .unwrap()
        .build()
        .await;

    assert!(result.is_err());
    if let Err(HeliusError::InvalidInput(msg)) = result {
        assert!(msg.contains("cluster") || msg.contains("custom URL"));
    } else {
        panic!("Expected InvalidInput error");
    }
}

#[test]
fn test_builder_fails_with_empty_api_key() {
    let result = HeliusBuilder::new().with_api_key("");
    assert!(result.is_err());
}

// ===== Builder: Custom URL =====

#[tokio::test]
async fn test_builder_custom_url_no_api_key() {
    let result = HeliusBuilder::new()
        .with_custom_url("http://localhost:8899")
        .unwrap()
        .build()
        .await;

    assert!(result.is_ok());
    let helius = result.unwrap();
    assert!(helius.config.api_key.is_none());
    assert!(helius.config.custom_url.is_some());
    assert_eq!(helius.config.custom_url.as_ref().unwrap(), "http://localhost:8899/");
}

#[tokio::test]
async fn test_builder_custom_url_with_api_key() {
    let result = HeliusBuilder::new()
        .with_custom_url("https://my-rpc.example.com/")
        .unwrap()
        .with_api_key("my-key")
        .unwrap()
        .build()
        .await;

    assert!(result.is_ok());
    let helius = result.unwrap();
    assert!(helius.config.api_key.is_some());
    assert!(helius.config.custom_url.is_some());
}

#[test]
fn test_builder_custom_url_rejects_invalid() {
    let result = HeliusBuilder::new()
        .with_custom_url("not-a-url");
    assert!(result.is_err());
}

#[test]
fn test_builder_custom_url_rejects_ftp() {
    let result = HeliusBuilder::new()
        .with_custom_url("ftp://files.example.com");
    assert!(result.is_err());
}

#[test]
fn test_builder_custom_url_rejects_credentials() {
    let result = HeliusBuilder::new()
        .with_custom_url("https://user:pass@rpc.example.com");
    assert!(result.is_err());
}

// ===== Builder: Custom API URL =====

#[tokio::test]
async fn test_builder_separate_api_and_rpc_urls() {
    let result = HeliusBuilder::new()
        .with_custom_url("https://rpc.example.com/")
        .unwrap()
        .with_custom_api_url("https://api.example.com/")
        .unwrap()
        .build()
        .await;

    assert!(result.is_ok());
    let helius = result.unwrap();
    assert_eq!(helius.config.endpoints.rpc, "https://rpc.example.com/");
    assert_eq!(helius.config.endpoints.api, "https://api.example.com/");
}

#[tokio::test]
async fn test_builder_custom_url_defaults_api_to_rpc() {
    let result = HeliusBuilder::new()
        .with_custom_url("https://my-rpc.example.com/")
        .unwrap()
        .build()
        .await;

    assert!(result.is_ok());
    let helius = result.unwrap();
    // When only custom_url is set, API URL defaults to the same as RPC
    assert_eq!(helius.config.endpoints.rpc, "https://my-rpc.example.com/");
    assert_eq!(helius.config.endpoints.api, "https://my-rpc.example.com/");
}

// ===== Builder: Async Solana Client =====

#[tokio::test]
async fn test_builder_with_async_solana() {
    let result = HeliusBuilder::new()
        .with_api_key("test-key")
        .unwrap()
        .with_cluster(Cluster::Devnet)
        .with_async_solana()
        .build()
        .await;

    assert!(result.is_ok());
    let helius = result.unwrap();
    assert!(helius.async_rpc_client.is_some());
    assert!(helius.ws_client.is_none());
}

// ===== Builder: Commitment =====

#[tokio::test]
async fn test_builder_with_commitment() {
    let result = HeliusBuilder::new()
        .with_api_key("test-key")
        .unwrap()
        .with_cluster(Cluster::Devnet)
        .with_commitment(CommitmentConfig::confirmed())
        .build()
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_builder_with_async_and_commitment() {
    let result = HeliusBuilder::new()
        .with_api_key("test-key")
        .unwrap()
        .with_cluster(Cluster::MainnetBeta)
        .with_async_solana()
        .with_commitment(CommitmentConfig::finalized())
        .build()
        .await;

    assert!(result.is_ok());
    let helius = result.unwrap();
    assert!(helius.async_rpc_client.is_some());
}

// ===== Builder: Custom HTTP Client =====

#[tokio::test]
async fn test_builder_with_custom_http_client() {
    let custom_client = reqwest::Client::builder()
        .user_agent("helius-test")
        .build()
        .unwrap();

    let result = HeliusBuilder::new()
        .with_api_key("test-key")
        .unwrap()
        .with_cluster(Cluster::Devnet)
        .with_http_client(custom_client)
        .build()
        .await;

    assert!(result.is_ok());
}

// ===== Builder: WebSocket (without real connection) =====

#[tokio::test]
async fn test_builder_websocket_requires_api_key() {
    // WebSocket on Helius endpoints requires an API key
    let result = HeliusBuilder::new()
        .with_custom_url("http://localhost:8899")
        .unwrap()
        .with_websocket(None, None)
        .build()
        .await;

    // Should fail because WS needs an API key (no custom WS URL provided)
    assert!(result.is_err());
}

// ===== Builder: Custom WebSocket URL =====

#[test]
fn test_builder_custom_ws_url_valid() {
    let builder = HeliusBuilder::new()
        .with_custom_ws_url("wss://ws.example.com/");
    assert!(builder.is_ok());
}

#[test]
fn test_builder_custom_ws_url_rejects_http() {
    let result = HeliusBuilder::new()
        .with_custom_ws_url("http://ws.example.com/");
    assert!(result.is_err());
}

// ===== Builder: Cluster Endpoints =====

#[tokio::test]
async fn test_builder_devnet_endpoints() {
    let helius = HeliusBuilder::new()
        .with_api_key("test-key")
        .unwrap()
        .with_cluster(Cluster::Devnet)
        .build()
        .await
        .unwrap();

    assert_eq!(helius.config.endpoints.rpc, "https://devnet.helius-rpc.com/");
    assert_eq!(helius.config.endpoints.api, "https://api-devnet.helius-rpc.com/");
}

#[tokio::test]
async fn test_builder_mainnet_endpoints() {
    let helius = HeliusBuilder::new()
        .with_api_key("test-key")
        .unwrap()
        .with_cluster(Cluster::MainnetBeta)
        .build()
        .await
        .unwrap();

    assert_eq!(helius.config.endpoints.rpc, "https://mainnet.helius-rpc.com/");
    assert_eq!(helius.config.endpoints.api, "https://api-mainnet.helius-rpc.com/");
}

#[tokio::test]
async fn test_builder_staked_mainnet_endpoints() {
    let helius = HeliusBuilder::new()
        .with_api_key("test-key")
        .unwrap()
        .with_cluster(Cluster::StakedMainnetBeta)
        .build()
        .await
        .unwrap();

    assert_eq!(helius.config.endpoints.rpc, "https://staked.helius-rpc.com/");
    assert_eq!(helius.config.endpoints.api, "https://api-mainnet.helius-rpc.com/");
}

// ===== Config: URL Building =====

#[test]
fn test_config_build_rpc_url_with_api_key() {
    let config = helius::config::Config::new("my-key", Cluster::Devnet).unwrap();
    let url = config.build_rpc_url();
    assert!(url.contains("api-key=my-key"));
    assert!(url.starts_with("https://devnet.helius-rpc.com/"));
}

#[test]
fn test_config_build_api_url_with_api_key() {
    let config = helius::config::Config::new("my-key", Cluster::Devnet).unwrap();
    let url = config.build_api_url();
    assert!(url.contains("api-key=my-key"));
    assert!(url.starts_with("https://api-devnet.helius-rpc.com/"));
}

#[test]
fn test_config_require_api_key_when_present() {
    let config = helius::config::Config::new("my-key", Cluster::Devnet).unwrap();
    let key = config.require_api_key("test feature");
    assert!(key.is_ok());
    assert_eq!(key.unwrap().as_str(), "my-key");
}

#[test]
fn test_config_require_api_key_when_absent() {
    use helius::config::Config;
    use helius::types::HeliusEndpoints;

    let config = Config {
        api_key: None,
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: "https://api-devnet.helius-rpc.com/".to_string(),
            rpc: "https://devnet.helius-rpc.com/".to_string(),
        },
        custom_url: None,
    };

    let result = config.require_api_key("webhooks");
    assert!(result.is_err());
    if let Err(HeliusError::InvalidInput(msg)) = result {
        assert!(msg.contains("webhooks"));
    } else {
        panic!("Expected InvalidInput error");
    }
}

#[test]
fn test_config_has_api_key() {
    let config = helius::config::Config::new("my-key", Cluster::Devnet).unwrap();
    assert!(config.has_api_key());
}

// ===== Simplified Constructors =====

#[tokio::test]
async fn test_helius_new_creates_basic_client() {
    let helius = Helius::new("test-key", Cluster::Devnet).unwrap();

    assert!(helius.config.api_key.is_some());
    assert_eq!(helius.config.api_key.as_ref().unwrap().as_str(), "test-key");
    assert!(helius.async_rpc_client.is_none());
    assert!(helius.ws_client.is_none());
}

#[tokio::test]
async fn test_helius_new_rejects_empty_key() {
    let result = Helius::new("", Cluster::Devnet);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_helius_new_async_fails_without_real_connection() {
    // new_async() includes WebSocket which requires a real Helius server
    let result = Helius::new_async("fake-key", Cluster::Devnet).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_helius_new_with_url_localhost() {
    let helius = Helius::new_with_url("http://localhost:8899").unwrap();

    assert!(helius.config.api_key.is_none());
    assert!(helius.config.custom_url.is_some());
    assert!(helius.async_rpc_client.is_none());
    assert!(helius.ws_client.is_none());
}

#[tokio::test]
async fn test_helius_new_with_url_https() {
    let helius = Helius::new_with_url("https://my-rpc.example.com/").unwrap();

    assert!(helius.config.api_key.is_none());
    assert!(helius.config.custom_url.is_some());
}

#[tokio::test]
async fn test_helius_new_with_url_rejects_invalid() {
    let result = Helius::new_with_url("not-a-url");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_helius_new_with_url_rejects_credentials() {
    let result = Helius::new_with_url("https://user:pass@rpc.example.com");
    assert!(result.is_err());
}
