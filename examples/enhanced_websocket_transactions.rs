use helius::error::Result;
use helius::types::{Cluster, RpcTransactionsConfig, TransactionSubscribeFilter, TransactionSubscribeOptions};
use helius::Helius;
use solana_sdk::pubkey;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    // Uses custom ping-pong timeouts to ping every 15s and timeout after 45s of no pong
    let helius: Helius = Helius::new_with_ws_with_timeouts(api_key, cluster, Some(15), Some(45)).await?;

    let key: pubkey::Pubkey = pubkey!("BtsmiEEvnSuUnKxqXj2PZRYpPJAc7C34mGz8gtJ1DAaH");

    let config: RpcTransactionsConfig = RpcTransactionsConfig {
        filter: TransactionSubscribeFilter::standard(&key),
        options: TransactionSubscribeOptions::default(),
    };

    if let Some(ws) = helius.ws() {
        let (mut stream, _unsub) = ws.transaction_subscribe(config).await?;
        while let Some(event) = stream.next().await {
            println!("{:#?}", event);
        }
    }

    Ok(())
}
