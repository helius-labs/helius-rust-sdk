use helius::error::HeliusError;
use helius::types::*;
use helius::Helius;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let response: Result<Vec<EnhancedTransaction>, HeliusError> = helius
        .parsed_transaction_history("2k5AXX4guW9XwRQ1AKCpAuUqgWDpQpwFfpVFh3hnm2Ha")
        .await;
    println!("Assets: {:?}", response);

    Ok(())
}
