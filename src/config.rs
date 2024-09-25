use std::sync::Arc;

use reqwest::Client;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncSolanaRpcClient;
use url::Url;

use crate::error::{HeliusError, Result};
use crate::rpc_client::RpcClient;
use crate::types::{Cluster, HeliusEndpoints};
use crate::Helius;

/// Configuration settings for the Helius client
///
/// `Config` contains all the necessary parameters needed to configure and authenticate the `Helius` client to interact with a specific Solana cluster
#[derive(Clone)]
pub struct Config {
    /// The API key used for authenticating requests
    pub api_key: String,
    /// The target Solana cluster the client will interact with
    pub cluster: Cluster,
    /// The endpoints associated with the specified `cluster`. Note these endpoints are automatically determined based on the cluster to ensure requests
    /// are made to the correct cluster
    pub endpoints: HeliusEndpoints,
}

impl Config {
    /// Creates a new configuration for the `Helius` client
    ///
    /// # Arguments
    /// * `api_key` - A string slice that holds the API key necessary for authenticating the client
    /// * `cluster` - The Solana cluster to interact with
    ///
    /// # Returns
    /// An instance of `Config` if successful
    ///
    /// # Errors
    /// Returns `HeliusError::InvalidInput` if the `api_key` is empty
    pub fn new(api_key: &str, cluster: Cluster) -> Result<Self> {
        if api_key.is_empty() {
            return Err(HeliusError::InvalidInput("API key cannot be empty".to_string()));
        }

        let endpoints: HeliusEndpoints = HeliusEndpoints::for_cluster(&cluster);

        Ok(Config {
            api_key: api_key.to_string(),
            cluster,
            endpoints,
        })
    }

    pub fn rpc_client_with_reqwest_client(&self, client: Client) -> crate::error::Result<RpcClient> {
        RpcClient::new(Arc::new(client), Arc::new(self.clone()))
    }

    pub fn client_with_async_solana(self) -> crate::error::Result<Helius> {
        let mut rpc_url_with_api_key: Url = self.endpoints.rpc.parse()?;
        rpc_url_with_api_key
            .query_pairs_mut()
            .append_pair("api_key", &self.api_key)
            .finish();

        let reqwest_client = Client::new();

        // DAS client
        let rpc_client = self.rpc_client_with_reqwest_client(reqwest_client.clone())?;

        let async_rpc_client = Arc::new(AsyncSolanaRpcClient::new(rpc_url_with_api_key.to_string()));

        Ok(Helius {
            config: Arc::new(self.clone()),
            client: reqwest_client,
            rpc_client: Arc::new(rpc_client),
            async_rpc_client: Some(async_rpc_client),
            ws_client: None,
        })
    }
}
