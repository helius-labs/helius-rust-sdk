use helius::config::Config;
use helius::error::Result;
use helius::types::Cluster;
use helius::Helius;

/// Demonstrates creating a Helius client with async Solana capabilities using the config-based approach
#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let config: Config = Config::new(api_key, cluster)?;
    let async_client: Helius = config.create_client_with_async()?;

    if let Ok(async_conn) = async_client.async_connection() {
        println!(
            "Async client - Get Block Height: {:?}",
            async_conn.get_block_height().await
        );
    }

    Ok(())
}
