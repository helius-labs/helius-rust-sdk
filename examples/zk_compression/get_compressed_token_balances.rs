use helius::error::Result;
use helius::types::{Cluster, GetCompressedTokenBalancesByOwnerRequest};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    // Get all compressed token balances for a wallet, grouped by mint
    let request = GetCompressedTokenBalancesByOwnerRequest {
        owner: "11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_token_balances_by_owner(request).await;

    match response {
        Ok(result) => {
            println!("Slot: {}", result.context.slot);
            println!("Token balances ({}):", result.value.token_balances.len());

            for token in &result.value.token_balances {
                println!("  Mint: {} — Balance: {}", token.mint, token.balance);
            }

            if let Some(cursor) = &result.value.cursor {
                println!("\nMore results available (cursor: {})", cursor);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
