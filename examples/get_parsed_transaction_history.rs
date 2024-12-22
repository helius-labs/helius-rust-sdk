use helius::error::Result;
use helius::types::*;
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
        address: "2k5AXX4guW9XwRQ1AKCpAuUqgWDpQpwFfpVFh3hnm2Ha".to_string(),
        before: None,
        until: None,
        transaction_type: None,
        commitment: None,
        limit: None,
        source: None,
    };

    let response: Result<Vec<EnhancedTransaction>> = helius.parsed_transaction_history(request).await;

    println!("Assets: {:?}", response);

    Ok(())
}
