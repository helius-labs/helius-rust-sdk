use helius::client::Helius;
use helius::error::Result;
use helius::types::Cluster;

#[tokio::test]
async fn test_creating_new_client_success() {
    let api_key: &str = "valid-api-key";
    let cluster: Cluster = Cluster::Devnet;

    let result: Result<Helius> = Helius::new(api_key, cluster);
    assert!(result.is_ok());

    let helius: Helius = result.unwrap();
    assert!(helius.config.api_key.is_some());
    assert_eq!(helius.config.api_key.as_ref().unwrap().as_str(), api_key);
}

#[tokio::test]
async fn test_creating_new_async_client_fails_without_real_ws() {
    // new_async() includes WebSocket, which requires a real connection.
    // With a fake API key, the WS connection will fail.
    let api_key: &str = "valid-api-key";
    let cluster: Cluster = Cluster::Devnet;

    let result: Result<Helius> = Helius::new_async(api_key, cluster).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_creating_async_client_via_builder() {
    // Use the builder directly without WebSocket for testability
    use helius::HeliusBuilder;

    let api_key: &str = "valid-api-key";
    let cluster: Cluster = Cluster::Devnet;

    let result: Result<Helius> = HeliusBuilder::new()
        .with_api_key(api_key)
        .unwrap()
        .with_cluster(cluster)
        .with_async_solana()
        .build()
        .await;
    assert!(result.is_ok());

    let helius: Helius = result.unwrap();
    assert!(helius.config.api_key.is_some());
    assert_eq!(helius.config.api_key.as_ref().unwrap().as_str(), api_key);
    assert!(helius.async_rpc_client.is_some());
}

#[tokio::test]
async fn test_creating_new_client_with_custom_url_success() {
    let result: Result<Helius> = Helius::new_with_url("http://localhost:8899");
    assert!(result.is_ok());

    let helius: Helius = result.unwrap();
    assert!(helius.config.api_key.is_none());
    assert!(helius.config.custom_url.is_some());
}
