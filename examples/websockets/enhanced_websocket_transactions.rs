use helius::error::Result;
use helius::types::{Cluster, RpcTransactionsConfig, TransactionSubscribeFilter, TransactionSubscribeOptions};
use helius::{Helius, HeliusBuilder};
use solana_sdk::pubkey;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    // Uses custom ping-pong timeouts to ping every 15s and timeout after 45s of no pong
    let helius: Helius = HeliusBuilder::new()
        .with_api_key(api_key)?
        .with_cluster(cluster)
        .with_websocket(Some(15), Some(45))
        .build()
        .await?;

    let key: pubkey::Pubkey = pubkey!("BtsmiEEvnSuUnKxqXj2PZRYpPJAc7C34mGz8gtJ1DAaH");

    let config: RpcTransactionsConfig = RpcTransactionsConfig {
        filter: TransactionSubscribeFilter {
            account_include: Some(vec![key.to_string()]),
            vote: Some(false),
            failed: None,
            signature: None,
            account_exclude: None,
            account_required: None,
        },
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
