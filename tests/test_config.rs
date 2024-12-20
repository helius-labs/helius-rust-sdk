use helius::config::Config;
use helius::error::{HeliusError, Result};
use helius::types::Cluster;
use helius::Helius;

#[test]
fn test_config_new_with_empty_api_key() {
    let result: Result<Config> = Config::new("", Cluster::Devnet);
    assert!(matches!(result, Err(HeliusError::InvalidInput(_))));
}

#[test]
fn test_config_new_with_valid_api_key() {
    let result: Result<Config> = Config::new("valid-api-key", Cluster::Devnet);
    assert!(result.is_ok());

    let config: Config = result.unwrap();
    assert_eq!(config.api_key, "valid-api-key");
    assert_eq!(config.endpoints.api, "https://api-devnet.helius-rpc.com/");
    assert_eq!(config.endpoints.rpc, "https://devnet.helius-rpc.com/");
}

#[test]
fn test_create_basic_client() {
    let config: Config = Config::new("valid-api-key", Cluster::Devnet).unwrap();
    let result: Result<Helius> = config.create_client();
    assert!(result.is_ok());

    let client: Helius = result.unwrap();
    assert!(client.async_rpc_client.is_none());
    assert!(client.ws_client.is_none());
}

#[test]
fn test_create_async_client() {
    let config: Config = Config::new("valid-api-key", Cluster::Devnet).unwrap();
    let result: Result<Helius> = config.create_client_with_async();
    assert!(result.is_ok());

    let client: Helius = result.unwrap();
    assert!(client.async_rpc_client.is_some());
    assert!(client.ws_client.is_none());
}
