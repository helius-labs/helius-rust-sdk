#![allow(dead_code)]
use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;
use crate::rpc_client::RpcClient;
use crate::types::Cluster;

use reqwest::Client;

pub struct Helius {
    pub config: Config,
    pub client: Client,
    pub rpc_client: Arc<RpcClient>,
}

impl Helius {
    pub fn new(api_key: &str, cluster: Cluster) -> Result<Self> {
        let config: Config = Config::new(api_key, cluster)?;
        let client: Client = Client::new();
        let rpc_client: RpcClient = Arc::new(RpcClient::new(Arc::new(client.clone()), config.clone()));

        Ok(Helius {
            config,
            client,
            rpc_client,
        })
    }

    /// Provides a thread-safe way to access RPC functionalities
    pub fn rpc(&self) -> Arc<RpcClient> {
        self.rpc_client.clone()
    }
}
