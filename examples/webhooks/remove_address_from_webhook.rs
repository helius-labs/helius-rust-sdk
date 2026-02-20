use helius::error::Result;
use helius::types::{Cluster, Webhook};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let webhook_id = "your_webhook_id";
    let addresses_to_remove = ["your_address1".to_string(), "your_address2".to_string()];
    let response: Result<Webhook> = helius
        .remove_addresses_from_webhook(webhook_id, &addresses_to_remove)
        .await;
    println!("Addresses removed  {:?}", response);

    Ok(())
}
