use helius::error::Result;
use helius::types::inner::{TransactionDetails, TransactionEntry};
use helius::types::{Cluster, GetTransactionsForAddressOptions};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Get transactions for a specific address
    let address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // SPL Token Program

    let options = GetTransactionsForAddressOptions {
        limit: Some(5),                                            // Get the last 5 transactions
        transaction_details: Some(TransactionDetails::Signatures), // Just get signatures
        ..Default::default()
    };

    println!("Fetching transactions for address: {}", address);
    println!(
        "Options: limit={:?}, transaction_details={:?}",
        options.limit, options.transaction_details
    );
    println!();

    let response = helius
        .rpc()
        .get_transactions_for_address(address.to_string(), options)
        .await;

    match response {
        Ok(result) => {
            println!("Successfully fetched {} transactions", result.data.len());
            println!();

            for (i, entry) in result.data.iter().enumerate() {
                match entry {
                    TransactionEntry::Signature(sig) => {
                        println!("Transaction #{}", i + 1);
                        println!("  Signature: {}", sig.signature);
                        println!("  Slot: {}", sig.slot);
                        println!("  Block time: {:?}", sig.block_time);
                        println!("  Status: {:?}", sig.confirmation_status);
                    }
                    TransactionEntry::Full(tx) => {
                        println!("Transaction #{}: {:?}", i + 1, tx);
                    }
                    TransactionEntry::Unknown(val) => {
                        println!("Transaction #{} (unknown format): {}", i + 1, val);
                    }
                }
            }

            if let Some(token) = result.pagination_token {
                println!();
                println!("Pagination token for next page: {}", token);
            } else {
                println!();
                println!("No more pages available");
            }
        }
        Err(e) => {
            println!("Error fetching transactions: {:?}", e);
        }
    }

    Ok(())
}
