use helius::error::Result;
use helius::types::{Cluster, GetValidityProofRequest};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Request a validity proof for compressed accounts by their hashes
    let request = GetValidityProofRequest {
        hashes: Some(vec!["11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string()]),
        ..Default::default()
    };

    let response = helius.get_validity_proof(request).await;

    match response {
        Ok(result) => {
            println!("Slot: {}", result.context.slot);

            let proof = &result.value;
            println!("Proof components:");
            println!("  a: {}", proof.compressed_proof.a);
            println!("  b: {}", proof.compressed_proof.b);
            println!("  c: {}", proof.compressed_proof.c);
            println!("Roots: {:?}", proof.roots);
            println!("Merkle trees: {:?}", proof.merkle_trees);
            println!("Leaf indices: {:?}", proof.leaf_indices);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
