use helius::error::Result;
use helius::types::{BalancesResponse, Cluster};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Example wallet address
    let wallet: &str = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY";

    // Get balances with options
    let response: Result<BalancesResponse> = helius
        .get_wallet_balances(
            wallet,
            Some(1),     // page
            Some(50),    // limit
            Some(false), // showZeroBalance
            Some(true),  // showNative (include SOL)
            Some(true),  // showNfts
        )
        .await;

    match response {
        Ok(balances) => {
            println!("Wallet Balances:");
            println!("Total USD Value: ${:.2}", balances.total_usd_value);
            println!("\nTokens ({}):", balances.balances.len());

            for balance in balances.balances {
                println!(
                    "  {}: {} (${:.2})",
                    balance.symbol.as_ref().unwrap_or(&"Unknown".to_string()),
                    balance.balance,
                    balance.usd_value.unwrap_or(0.0)
                );
            }

            if let Some(nfts) = balances.nfts {
                println!("\nNFTs ({}):", nfts.len());
                for nft in nfts.iter().take(5) {
                    println!(
                        "  {} ({})",
                        nft.name.as_ref().unwrap_or(&"Unnamed".to_string()),
                        if nft.compressed { "Compressed" } else { "Standard" }
                    );
                }
            }

            println!(
                "\nPagination: Page {} of {} (has more: {})",
                balances.pagination.page, balances.pagination.limit, balances.pagination.has_more
            );
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
