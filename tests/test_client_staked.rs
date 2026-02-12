use helius::client::Helius;
use helius::error::Result;
use helius::types::Cluster;

#[tokio::test]
async fn test_creating_new_client_staked_success() {
    let api_key: &str = "valid-api-key";
    let cluster: Cluster = Cluster::StakedMainnetBeta;

    let result: Result<Helius> = Helius::new(api_key, cluster);
    assert!(result.is_ok());

    let helius: Helius = result.unwrap();
    assert!(helius.config.api_key.is_some());
    assert_eq!(helius.config.api_key.as_ref().unwrap().as_str(), api_key);
}

#[tokio::test]
async fn test_creating_new_async_client_staked_fails_without_real_ws() {
    // new_async() includes WebSocket, which requires a real connection.
    // With a fake API key, the WS connection will fail.
    let api_key: &str = "valid-api-key";
    let cluster: Cluster = Cluster::StakedMainnetBeta;

    let result: Result<Helius> = Helius::new_async(api_key, cluster).await;
    assert!(result.is_err());
}
