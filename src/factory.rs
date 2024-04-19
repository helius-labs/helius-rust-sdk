use std::sync::Arc;

use crate::client::Helius;
use crate::config::Config;
use crate::error::Result;
use crate::rpc_client::RpcClient;
use crate::types::Cluster;

use reqwest::Client;

pub struct HeliusFactory {
    api_key: String,
}

impl HeliusFactory {
    pub fn new(api_key: &str) -> Self {
        HeliusFactory {
            api_key: api_key.to_string(),
        }
    }

    pub fn create(&self, cluster: Cluster) -> Result<Helius> {
        let config: Config = Config::new(&self.api_key, cluster)?;
        let client: Client = Client::new();
        let rpc_client = RpcClient::new(Arc::new(client), Arc::new(config.clone()))?;

        Ok(Helius {
            config,
            client,
            rpc_client,
        })
    }
}

// Example Usage

// #[tokio::main]
// async fn main() {
//     let api_key = "your_api_key_here";
//     let factory = HeliusFactory::new(api_key);

//     let helius_dev = factory.create(Cluster::Devnet).unwrap();
//     // helius_dev.request_name(...).await;

//     let helius_main = factory.create(Cluster::MainnetBeta).unwrap();
//     // helius_main.request_name(...).await;
// }
