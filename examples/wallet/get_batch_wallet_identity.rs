use helius::error::Result;
use helius::types::{BatchIdentityResult, Cluster};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Example: Check identities of known exchange wallets and SNS domains
    let addresses: Vec<String> = vec![
        "HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664".to_string(), // Binance
        "2ojv9BAiHUrvsm9gxDe7fJSzbNZSJcxZvf8dqmWGHG8S".to_string(), // Coinbase
        "toly.sol".to_string(),                                     // SNS domain, resolved to its owner
        "nonexistent-xyz.sol".to_string(),                          // Domain that doesn't resolve
    ];

    let response: Result<Vec<BatchIdentityResult>> = helius.get_batch_wallet_identity(&addresses).await;

    match response {
        Ok(identities) => {
            println!("Got {} entries", identities.len());
            for identity in identities {
                match identity {
                    // Domains that can't be resolved come back with no identity
                    BatchIdentityResult::Unresolved(entry) => println!("{} (unresolved)", entry.input_domain),
                    BatchIdentityResult::Resolved(entry) => {
                        if let Some(domain) = &entry.input_domain {
                            println!("Domain: {}", domain);
                        }
                        println!("{}: {}", entry.identity.address, entry.identity.name)
                    }
                }
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
