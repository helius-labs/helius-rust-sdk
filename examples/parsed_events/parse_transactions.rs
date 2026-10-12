use helius::error::Result;
use helius::types::parsed_events::{ParseTransactionsRequest, ParsedTransactionResult};
use helius::types::Cluster;
use helius::Helius;

/// Parses transaction signatures into decoded instructions, transfers, and summaries.
///
/// Each result carries its own status: a missing transaction comes back as an item with a
/// `parser_error` instead of failing the whole request. Parsed Events is mainnet only.
#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let helius: Helius = Helius::new(api_key, Cluster::MainnetBeta)?;

    let request: ParseTransactionsRequest = ParseTransactionsRequest {
        transactions: vec![
            "5xSKzM8bvpudE521jikHqASzMr23Ms4X4ieY3K8oFPFrJWCSSgYocJmHznrR8b12voDxKDH8ykdCLXSRrx6duVLH".to_string(),
        ],
        ..Default::default()
    };

    let results: Vec<ParsedTransactionResult> = helius.parsed_events().parse_transactions(request).await?;

    for result in results {
        match (result.parsed, result.parser_error) {
            (Some(parsed), _) => println!(
                "{}: {}",
                result.signature,
                parsed.summary.map_or("unrecognized".to_string(), |s| s.description)
            ),
            (None, Some(error)) => eprintln!("{}: {} ({})", result.signature, error.code, error.message),
            (None, None) => eprintln!("{}: {:?}", result.signature, result.parser_status),
        }
    }

    Ok(())
}
