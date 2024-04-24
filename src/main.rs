use helius_sdk::config::Config;
use helius_sdk::error::HeliusError;
use helius_sdk::rpc_client::RpcClient;
use helius_sdk::types::types::ApiResponse;
use helius_sdk::types::{AssetsByOwnerRequest, Cluster};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "0abc88ab-a583-4f9c-9265-b8f8f16c0719";
    let cluster: Cluster = Cluster::MainnetBeta;

    let config: Config = Config::new(api_key, cluster)?;
    let client: reqwest::Client = reqwest::Client::new();
    let rpc_client = RpcClient::new(Arc::new(client), Arc::new(config))?;

    let request = AssetsByOwnerRequest {
        owner_address: "GNPwr9fk9RJbfy9nSKbNiz5NPfc69KVwnizverx6fNze".to_string(),
        page: Some(1),
        ..Default::default()
    };

    let response: ApiResponse = rpc_client.get_assets_by_owner(request).await?;
    println!("Assets: {:?}", response);

    Ok(())
}
