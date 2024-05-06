use helius_sdk::config::Config;
use helius_sdk::error::HeliusError;
use helius_sdk::rpc_client::RpcClient;
use helius_sdk::types::{Asset, Cluster, GetAssetBatch, GetAssetOptions};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let config: Config = Config::new(api_key, cluster)?;
    let client: reqwest::Client = reqwest::Client::new();
    let rpc_client: RpcClient = RpcClient::new(Arc::new(client), Arc::new(config))?;

    let request: GetAssetBatch = GetAssetBatch {
        ids: vec![
            "81bxPqYCE8j34nQm7Rooqi8Vt3iMHLzgZJ71rUVbQQuz".to_string(),
            "CWHuz6GPjWYdwt7rTfRHKaorMwZP58Spyd7aqGK7xFbn".to_string(),
        ],
        display_options: Some(GetAssetOptions {
            show_collection_metadata: true,
            ..Default::default()
        }),
    };

    let response: Result<Vec<Option<Asset>>, HeliusError> = rpc_client.get_asset_batch(request).await;
    println!("Assets: {:?}", response);

    Ok(())
}
