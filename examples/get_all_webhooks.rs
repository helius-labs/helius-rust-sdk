use helius::error::Result;
use helius::types::{Cluster, Webhook};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let response: Result<Vec<Webhook>> = helius.get_all_webhooks().await;
    println!("Webhooks  {:?}", response);
    Ok(())
}
