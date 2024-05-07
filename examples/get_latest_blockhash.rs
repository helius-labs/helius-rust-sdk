use helius_sdk::config::Config;
use helius_sdk::error::HeliusError;
use helius_sdk::rpc_client::RpcClient;
use helius_sdk::types::*;
use helius_sdk::Helius;

use reqwest::Client;
use solana_client::client_error::ClientError;
use solana_sdk::hash::Hash;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let config: Arc<Config> = Arc::new(Config::new(api_key, cluster)?);
    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());

    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
    };

    let result: Result<Hash, ClientError> = helius.connection().get_latest_blockhash();
    println!("{:?}", result);

    Ok(())
}
