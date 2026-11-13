use helius::error::Result;
use helius::types::{BatchIdentityResult, Cluster};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Example: Batch lookup for multiple known wallet addresses
    let addresses: Vec<String> = vec![
        "HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664".to_string(), // Binance
        "2ojv9BAiHUrvsm9gxDe7fJSzbNZSJcxZvf8dqmWGHG8S".to_string(), // Coinbase
    ];

    let response: Result<Vec<BatchIdentityResult>> = helius.get_batch_wallet_identity(&addresses).await;

    match response {
        Ok(identities) => {
            println!("Found {} wallet identities:", identities.len());
            for identity in identities {
                match identity {
                    BatchIdentityResult::Resolved(entry) => {
                        println!("{}: {}", entry.identity.address, entry.identity.name)
                    }
                    BatchIdentityResult::Unresolved(entry) => println!("Unresolved: {}", entry.input_domain),
                }
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
