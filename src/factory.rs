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
    /// Initializes a new HeliusFactory instance
    pub fn new(api_key: &str) -> Self {
        HeliusFactory {
            api_key: api_key.to_string(),
        }
    }

    /// Provides a way to create multiple Helius instances in a thread-safe manner
    pub fn create(&self, cluster: Cluster) -> Result<Helius> {
        let config: Arc<Config> = Arc::new(Config::new(&self.api_key, cluster)?);
        let client: Client = Client::new();
        let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), config.clone())?);

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
