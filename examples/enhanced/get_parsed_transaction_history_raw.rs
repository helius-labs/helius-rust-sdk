//! Fetches a page of parsed transaction history and decodes it off the async runtime.
//!
//! `parsed_transaction_history` deserializes the response on the calling task. A full page
//! (100 enhanced transactions) is the largest payload the enhanced transactions API returns,
//! and parsing it is CPU work that parks a tokio worker for its duration. In a pipeline that
//! pulls many pages concurrently, that shows up as latency on every other task scheduled on
//! that worker.
//!
//! The `_raw` variant returns the body as `Bytes` instead, which is `Send + 'static` and cheap
//! to move, so decoding can run on tokio's blocking pool with `decode_response` — the same
//! simd-json/serde_json parser the typed method uses, so the result is identical.

use helius::error::Result;
use helius::request_handler::decode_response;
use helius::types::*;
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
        address: "2k5AXX4guW9XwRQ1AKCpAuUqgWDpQpwFfpVFh3hnm2Ha".to_string(),
        before: None,
        until: None,
        transaction_type: None,
        commitment: None,
        limit: Some(100),
        source: None,
    };

    // Network I/O stays on the runtime; only the bytes come back.
    let body = helius.parsed_transaction_history_raw(request).await?;
    println!("Fetched {} bytes of parsed transaction history", body.len());

    // Decoding moves to the blocking pool so it cannot stall other async tasks. `HeliusError`
    // converts from `JoinError`, so both the join and the decode are handled by `?`.
    let transactions: Vec<EnhancedTransaction> = tokio::task::spawn_blocking(move || decode_response(&body)).await??;

    println!("Decoded {} transactions", transactions.len());
    for transaction in transactions.iter().take(5) {
        println!(
            "{} — {:?}: {}",
            transaction.signature, transaction.transaction_type, transaction.description
        );
    }

    Ok(())
}
