use helius::error::Result;
use helius::types::{Cluster, GetCompressedAccountRequest};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Look up a compressed account by its address
    let request = GetCompressedAccountRequest {
        address: Some("11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3".to_string()),
        ..Default::default()
    };

    let response = helius.get_compressed_account(request).await;

    match response {
        Ok(result) => {
            println!("Slot: {}", result.context.slot);
            match result.value {
                Some(account) => {
                    println!("Owner: {}", account.owner);
                    println!("Lamports: {}", account.lamports);
                    println!("Tree: {}", account.tree);
                    println!("Leaf index: {}", account.leaf_index);
                    println!("Slot created: {}", account.slot_created);
                }
                None => println!("No account found at this address"),
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
