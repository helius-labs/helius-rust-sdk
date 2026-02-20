use helius::error::Result;
use helius::types::{Cluster, TransfersResponse};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Example wallet address
    let wallet: &str = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY";

    // Get transfer history
    let response: Result<TransfersResponse> = helius
        .get_wallet_transfers(
            wallet,
            Some(20), // limit
            None,     // cursor for pagination
        )
        .await;

    match response {
        Ok(transfers) => {
            println!("Transfer History ({} transfers):", transfers.data.len());

            for transfer in transfers.data.iter().take(10) {
                println!("\n  Signature: {}", transfer.signature);
                println!("  Direction: {:?}", transfer.direction);
                println!("  Counterparty: {}", transfer.counterparty);
                println!(
                    "  Amount: {} {}",
                    transfer.amount,
                    transfer.symbol.as_ref().unwrap_or(&"Unknown".to_string())
                );
                println!("  Mint: {}", transfer.mint);
                println!("  Timestamp: {}", transfer.timestamp);
            }

            println!("\nHas more transfers: {}", transfers.pagination.has_more);
            if let Some(cursor) = transfers.pagination.next_cursor {
                println!("Next cursor: {}", cursor);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
