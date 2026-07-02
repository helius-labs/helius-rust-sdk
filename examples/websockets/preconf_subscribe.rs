use futures_util::StreamExt;
use helius::error::Result;
use helius::preconf::PreconfClient;

/// Subscribe to Helius **Pre Confirmations** (`preconfSubscribe`).
///
/// Pre Confirmations are the lowest-latency transaction stream: scheduled
/// transactions are delivered *before* they are shredded. A pre-confirmation is
/// an **early signal, not a guarantee** — a streamed transaction may still fail
/// to land.
///
/// NOTE: Pre Confirmations are served from the Gatekeeper endpoint
/// (`wss://beta.helius-rpc.com`). Despite the `beta` host name this is not a beta
/// product — it is where Pre Confirmations launch during the Gatekeeper migration.
#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";

    // `preconfSubscribe` takes no filters; it streams ALL scheduled transactions.
    let (client, mut stream) = PreconfClient::connect_with_api_key(api_key).await?;

    let mut count = 0;
    while let Some(event) = stream.next().await {
        println!(
            "preconf: v{} slot={} index={} status={:?} sig={:?} ({} raw bytes)",
            event.version,
            event.slot,
            event.transaction_index,
            event.status,
            event.transaction.signatures.first(),
            event.transaction_bytes.len(),
        );

        count += 1;
        if count >= 10 {
            break;
        }
    }

    // Tidy shutdown (sends a close frame and joins the background task).
    client.shutdown().await?;
    Ok(())
}
