use crate::error::{HeliusError, Result};
use crate::types::{Cluster, HeliusEndpoints};

pub struct Config {
    pub api_key: String,
    pub cluster: Cluster,
    pub endpoints: HeliusEndpoints,
}

impl Config {
    pub fn new(api_key: &str, cluster: Cluster) -> Result<Self> {
        if api_key.is_empty() {
            return Err(HeliusError::InvalidInput("API key must not be empty".to_string()));
        }

        let endpoints: HeliusEndpoints = match cluster {
            Cluster::Devnet => HeliusEndpoints {
                api: "https://api-devnet.helius-rpc.com".to_string(),
                rpc: "https://devnet.helius-rpc.com".to_string(),
            },
            Cluster::MainnetBeta => HeliusEndpoints {
                api: "https://api-mainnet.helius-rpc.com".to_string(),
                rpc: "https://mainnet.helius-rpc.com".to_string(),
            },
        };

        Ok(Config {
            api_key: api_key.to_string(),
            cluster,
            endpoints,
        })
    }
}
