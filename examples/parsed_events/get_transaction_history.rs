use helius::error::Result;
use helius::types::parsed_events::{GetTransactionHistoryRequest, GetTransactionHistoryResponse};
use helius::types::Cluster;
use helius::Helius;

/// Pages through an address's parsed transaction history, newest first.
///
/// Pagination is manual: pass each page's `pagination_token` back to fetch the next one. It is
/// `None` once the end of the range is reached. Parsed Events is mainnet only.
#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let helius: Helius = Helius::new(api_key, Cluster::MainnetBeta)?;

    let mut pagination_token: Option<String> = None;

    // Fetch up to three pages (100 transactions each, the default)
    for _ in 0..3 {
        let page: GetTransactionHistoryResponse = helius
            .parsed_events()
            .get_transaction_history(GetTransactionHistoryRequest {
                address: "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY".to_string(),
                pagination_token,
                ..Default::default()
            })
            .await?;

        for result in &page.data {
            let summary_type = result
                .parsed
                .as_ref()
                .and_then(|parsed| parsed.summary.as_ref())
                .map_or("unknown", |summary| summary.summary_type.as_str());
            println!("{} {}", result.signature, summary_type);
        }

        pagination_token = page.pagination_token;
        if pagination_token.is_none() {
            break;
        }
    }

    Ok(())
}
