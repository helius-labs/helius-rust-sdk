use helius::client::Helius;
use helius::error::Result;
use helius::types::Cluster;

#[test]
fn test_creating_new_client_success() {
    let api_key: &str = "valid-api-key";
    let cluster: Cluster = Cluster::StakedMainnetBeta;

    let result: Result<Helius> = Helius::new(api_key, cluster);
    assert!(result.is_ok());

    let helius: Helius = result.unwrap();
    assert_eq!(helius.config.api_key, api_key);
}

#[test]
fn test_creating_new_async_client_success() {
    let api_key: &str = "valid-api-key";
    let cluster: Cluster = Cluster::StakedMainnetBeta;

    let result: Result<Helius> = Helius::new_with_async_solana(api_key, cluster);
    assert!(result.is_ok());

    let helius: Helius = result.unwrap();
    assert_eq!(helius.config.api_key, api_key);
    assert!(helius.async_rpc_client.is_some());
}
