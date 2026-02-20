use helius::error::Result;
use helius::types::{AssetProof, Cluster, GetAssetProofBatch};
use helius::Helius;

use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let request: GetAssetProofBatch = GetAssetProofBatch {
        ids: vec![
            "81bxPqYCE8j34nQm7Rooqi8Vt3iMHLzgZJ71rUVbQQuz".to_string(),
            "CWHuz6GPjWYdwt7rTfRHKaorMwZP58Spyd7aqGK7xFbn".to_string(),
        ],
    };

    let response: Result<HashMap<String, Option<AssetProof>>> = helius.rpc().get_asset_proof_batch(request).await;
    println!("Assets: {:?}", response);

    Ok(())
}
