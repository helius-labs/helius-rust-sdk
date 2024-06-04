use helius::error::Result;
use helius::types::*;
use helius::Helius;

use solana_sdk::hash::Hash;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new_with_async_solana(api_key, cluster).expect("Failed to create a Helius client");

    let latest_blockhash: Hash = helius.async_connection()?.get_latest_blockhash().await?;
    println!("{:?}", latest_blockhash);

    Ok(())
}
