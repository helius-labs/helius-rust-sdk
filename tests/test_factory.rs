use helius_sdk::{ Helius, HeliusFactory };
use helius_sdk::types::Cluster;

#[test]
fn test_factory_create_devnet_instance() {
    let factory: HeliusFactory = HeliusFactory::new("valid_api_key");
    let helius: Helius = factory.create(Cluster::Devnet).unwrap();

    assert_eq!(helius.config.api_key, "valid_api_key");
    assert_eq!(helius.config.endpoints.api, "https://api-devnet.helius-rpc.com");
    assert_eq!(helius.config.endpoints.rpc, "https://devnet.helius-rpc.com");
}

#[test]
fn test_factory_create_mainnet_instance() {
    let factory: HeliusFactory = HeliusFactory::new("valid_api_key");
    let helius: Helius = factory.create(Cluster::MainnetBeta).unwrap();

    assert_eq!(helius.config.api_key, "valid_api_key");
    assert_eq!(helius.config.endpoints.api, "https://api-mainnet.helius-rpc.com");
    assert_eq!(helius.config.endpoints.rpc, "https://mainnet.helius-rpc.com");
}