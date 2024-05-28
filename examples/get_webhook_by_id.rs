use helius::types::Cluster;
use helius::Helius;

#[tokio::main]
async fn main() {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let webhook_id = "your_webhook_id";
    let response = helius.get_webhook_by_id(webhook_id).await;
    println!("Webhook  {:?}", response);
}
