use helius_sdk::client::Helius;
use helius_sdk::error::HeliusError;
use helius_sdk::types::Cluster;

#[test]
fn test_creating_new_client_success() {
    let api_key: &str = "valid-api-key";
    let cluster: Cluster = Cluster::Devnet;

    let result: Result<Helius, HeliusError> = Helius::new(api_key, cluster);
    assert!(result.is_ok());

    let helius: Helius = result.unwrap();
    assert_eq!(helius.config.api_key, api_key);
}
