use helius::error::Result;
use helius::types::{BalanceAtQuery, BalanceAtResponse, Cluster};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Example wallet and the USDC mint
    let wallet: &str = "5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9";
    let mint: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    // Query the balance at a past point in time. Provide exactly one of:
    //   BalanceAtQuery::Time(unix_seconds)
    //   BalanceAtQuery::Datetime("2025-01-10 19:20:00".to_string())  // UTC unless a timezone is given
    //   BalanceAtQuery::Slot(313_000_000)                            // exact and deterministic
    let response: Result<BalanceAtResponse> = helius
        .get_wallet_balance_at(wallet, mint, BalanceAtQuery::Time(1736536800))
        .await;

    match response {
        Ok(balance) => {
            println!("Wallet: {}", balance.wallet);
            println!("Mint: {} (native SOL: {})", balance.mint, balance.is_native);
            println!(
                "Balance: {} ({} raw, {} decimals)",
                balance.balance, balance.balance_raw, balance.decimals
            );

            match balance.as_of {
                Some(as_of) => {
                    println!("Read from transaction {} at slot {}", as_of.signature, as_of.slot);
                    if let Some(block_time) = as_of.block_time {
                        println!("Block time: {}", block_time);
                    }
                }
                None => println!("No matching activity at or before the requested point — balance is genuinely 0"),
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
