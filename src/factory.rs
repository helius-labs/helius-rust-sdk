use crate::client::Helius;
use crate::config::Config;
use crate::types::Cluster;
use crate::error::Result;

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
        let config = Config::new(&self.api_key, cluster)?;
        let client = Client::new();

        Ok(Helius { config, client })
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