use helius::error::Result;
use helius::types::parsed_events::{ScanMatch, ScanProgramMatch, ScanRequest, ScanResponse};
use helius::types::Cluster;
use helius::Helius;

const PUMP_FUN: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";

/// Scans the Pump.fun program's history server-side for `buy` and `sell` instructions.
///
/// Each request scans up to 10,000 transactions and returns only the parsed matches. A request
/// that runs out of budget before finding a match returns an empty page with a
/// `pagination_token`, so keep calling while one is returned. Parsed Events is mainnet only.
#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let helius: Helius = Helius::new(api_key, Cluster::MainnetBeta)?;

    let mut pagination_token: Option<String> = None;

    // Run up to three scan requests
    for _ in 0..3 {
        let page: ScanResponse = helius
            .parsed_events()
            .scan(ScanRequest {
                address: PUMP_FUN.to_string(),
                criteria: ScanMatch {
                    program: Some(ScanProgramMatch {
                        program_id: PUMP_FUN.to_string(),
                        instructions: Some(vec!["buy".to_string(), "sell".to_string()]),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                limit: Some(100),
                pagination_token,
                ..Default::default()
            })
            .await?;

        for result in &page.data {
            let slot = result.parsed.as_ref().map(|parsed| parsed.slot);
            println!("{} {:?}", result.signature, slot);
        }

        println!(
            "stopped on {:?}: scanned {}, matched {}, charged {} credits",
            page.scan.stopped_on,
            page.scan.scanned_transactions,
            page.scan.matched_transactions,
            page.scan.credits_charged
        );

        // Keep going while a token is returned, even when this page had no matches
        pagination_token = page.pagination_token;
        if pagination_token.is_none() {
            break;
        }
    }

    Ok(())
}
