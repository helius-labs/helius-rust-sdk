use helius::error::HeliusError;
use helius::types::{Asset, Cluster, GetAssetBatch, GetAssetOptions};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

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

    let response: Result<Vec<Option<Asset>>, HeliusError> = helius.rpc().get_asset_batch(request).await;
    println!("Assets: {:?}", response);

    Ok(())
}
