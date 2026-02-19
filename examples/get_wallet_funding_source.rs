use helius::error::Result;
use helius::types::{Cluster, FundingSource};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Example wallet address
    let wallet: &str = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY";

    // Get the original funding source
    let response: Result<FundingSource> = helius.get_wallet_funding_source(wallet).await;

    match response {
        Ok(funding) => {
            println!("Wallet Funding Source:");
            println!("  Funder: {}", funding.funder);

            if let Some(name) = funding.funder_name {
                println!("  Funder Name: {}", name);
            }

            if let Some(funder_type) = funding.funder_type {
                println!("  Funder Type: {}", funder_type);
            }

            println!("  Amount: {} {}", funding.amount, funding.symbol);
            println!("  Raw Amount: {} lamports", funding.amount_raw);
            println!("  Signature: {}", funding.signature);
            println!("  Timestamp: {}", funding.timestamp);
            println!("  Date: {}", funding.date);
            println!("  Slot: {}", funding.slot);
            println!("  Explorer: {}", funding.explorer_url);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
