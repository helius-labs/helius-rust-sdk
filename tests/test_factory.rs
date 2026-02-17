use helius::types::Cluster;
use helius::{Helius, HeliusFactory};

#[test]
fn test_factory_create_devnet_instance() {
    let factory: HeliusFactory = HeliusFactory::new("valid_api_key");
    let helius: Helius = factory.create(Cluster::Devnet).unwrap();

    assert!(helius.config.api_key.is_some());
    assert_eq!(helius.config.api_key.as_ref().unwrap().as_str(), "valid_api_key");
    assert_eq!(helius.config.endpoints.api, "https://api-devnet.helius-rpc.com/");
    assert_eq!(helius.config.endpoints.rpc, "https://devnet.helius-rpc.com/");
}

#[test]
fn test_factory_create_mainnet_instance() {
    let factory: HeliusFactory = HeliusFactory::new("valid_api_key");
    let helius: Helius = factory.create(Cluster::MainnetBeta).unwrap();

    assert!(helius.config.api_key.is_some());
    assert_eq!(helius.config.api_key.as_ref().unwrap().as_str(), "valid_api_key");
    assert_eq!(helius.config.endpoints.api, "https://api-mainnet.helius-rpc.com/");
    assert_eq!(helius.config.endpoints.rpc, "https://mainnet.helius-rpc.com/");
}

#[test]
fn test_factory_create_staked_mainnet_instance() {
    let factory: HeliusFactory = HeliusFactory::new("valid_api_key");
    let helius: Helius = factory.create(Cluster::StakedMainnetBeta).unwrap();

    assert!(helius.config.api_key.is_some());
    assert_eq!(helius.config.api_key.as_ref().unwrap().as_str(), "valid_api_key");
    assert_eq!(helius.config.endpoints.api, "https://api-mainnet.helius-rpc.com/");
    assert_eq!(helius.config.endpoints.rpc, "https://staked.helius-rpc.com/");
}

#[test]
fn test_factory_create_with_reqwest() {
    let mut factory = HeliusFactory::new("valid_api_key");
    let helius: Helius = factory
        .with_client(reqwest::Client::new())
        .create(Cluster::MainnetBeta)
        .unwrap();

    assert!(helius.config.api_key.is_some());
    assert_eq!(helius.config.api_key.as_ref().unwrap().as_str(), "valid_api_key");
    assert_eq!(helius.config.endpoints.api, "https://api-mainnet.helius-rpc.com/");
    assert_eq!(helius.config.endpoints.rpc, "https://mainnet.helius-rpc.com/");
}
