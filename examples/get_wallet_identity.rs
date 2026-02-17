use helius::error::Result;
use helius::types::{Cluster, Identity};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Example: Get identity for a known exchange wallet
    let wallet: &str = "HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664";
    let response: Result<Identity> = helius.get_wallet_identity(wallet).await;

    match response {
        Ok(identity) => {
            println!("Wallet Identity:");
            println!("  Address: {}", identity.address);
            println!("  Name: {}", identity.name);
            println!("  Type: {}", identity.entity_type);
            println!("  Category: {}", identity.category);
            println!("  Tags: {:?}", identity.tags);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
