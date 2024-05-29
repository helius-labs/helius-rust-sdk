use helius::error::HeliusError;
use helius::types::{Cluster, Webhook};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let response: Result<Vec<Webhook>, HeliusError> = helius.get_all_webhooks().await;
    println!("Webhooks  {:?}", response);
    Ok(())
}
