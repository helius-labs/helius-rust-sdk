use helius::error::Result;
use helius::types::{Cluster, Webhook};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let webhook_id = "your_webhook_id";
    let new_addresses = ["your_address1".to_string(), "your_address2".to_string()];
    let response: Result<Webhook> = helius.append_addresses_to_webhook(webhook_id, &new_addresses).await;
    println!("Addresses appended  {:?}", response);

    Ok(())
}
