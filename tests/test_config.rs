use helius_rust_sdk::config::Config;
use helius_rust_sdk::types::Cluster;
use helius_rust_sdk::error::HeliusError;

#[test]
fn test_config_new_with_empty_api_key() {
    let result: Result<Config, HeliusError> = Config::new("", Cluster::Devnet);
    assert!(matches!(result, Err(HeliusError::InvalidInput(_))));
}

#[test]
fn test_config_new_with_valid_api_key() {
    let result: Result<Config, HeliusError> = Config::new("valid-api-key", Cluster::Devnet);
    assert!(result.is_ok());

    let config = result.unwrap();
    assert_eq!(config.api_key, "valid-api-key");
    assert_eq!(config.endpoints.api, "https://api-devnet.helius-rpc.com");
    assert_eq!(config.endpoints.rpc, "https://devnet.helius-rpc.com");
}