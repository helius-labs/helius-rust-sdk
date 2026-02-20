use helius::error::Result;
use helius::types::{Cluster, HistoryResponse};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Example wallet address
    let wallet: &str = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY";

    // Get transaction history with filters
    let response: Result<HistoryResponse> = helius
        .get_wallet_history(
            wallet,
            Some(20),                           // limit
            None,                               // before cursor
            None,                               // after cursor
            Some("SWAP".to_string()),           // filter by SWAP transactions
            Some("balanceChanged".to_string()), // only show txs that changed balances
        )
        .await;

    match response {
        Ok(history) => {
            println!("Transaction History ({} transactions):", history.data.len());

            for tx in history.data.iter().take(5) {
                println!("\n  Signature: {}", tx.signature);
                println!("  Slot: {}", tx.slot);
                println!("  Fee: {} SOL", tx.fee);
                println!("  Fee Payer: {}", tx.fee_payer);

                if let Some(timestamp) = tx.timestamp {
                    println!("  Timestamp: {}", timestamp);
                }

                if let Some(error) = &tx.error {
                    println!("  Error: {}", error);
                }

                println!("  Balance Changes:");
                for change in &tx.balance_changes {
                    println!(
                        "    {} {} (mint: {})",
                        if change.amount >= 0.0 { "+" } else { "" },
                        change.amount,
                        change.mint
                    );
                }
            }

            println!("\nHas more transactions: {}", history.pagination.has_more);
            if let Some(cursor) = history.pagination.next_cursor {
                println!("Next cursor: {}", cursor);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
