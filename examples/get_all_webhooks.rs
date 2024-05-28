use helius::types::Cluster;
use helius::Helius;

#[tokio::main]
async fn main() {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let response = helius.get_all_webhooks().await;
    println!("Webhooks  {:?}", response);
}
