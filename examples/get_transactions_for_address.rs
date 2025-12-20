use helius::error::Result;
use helius::types::inner::TransactionDetails;
use helius::types::{Cluster, GetTransactionsForAddressOptions, GetTransactionsForAddressResponse};
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

    let response: Result<GetTransactionsForAddressResponse> = helius
        .rpc()
        .get_transactions_for_address(address.to_string(), options)
        .await;

    match response {
        Ok(result) => {
            println!("Successfully fetched {} transactions", result.data.len());
            println!();

            for (i, tx) in result.data.iter().enumerate() {
                println!("Transaction #{}: {}", i + 1, serde_json::to_string_pretty(tx).unwrap());
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
