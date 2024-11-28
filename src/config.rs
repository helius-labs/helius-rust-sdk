use crate::error::{HeliusError, Result};
use crate::types::{Cluster, HeliusEndpoints, MintApiAuthority};
use crate::websocket::{ EnhancedWebsocket, ENHANCED_WEBSOCKET_URL };
use crate::Helius;
use crate::rpc_client::RpcClient;
use std::sync::Arc;
use reqwest::Client;
use url::{ParseError, Url};
use solana_client::nonblocking::rpc_client::RpcClient as AsyncSolanaRpcClient;

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

    pub fn rpc_client_with_reqwest_client(&self, client: Client) -> Result<RpcClient> {
        RpcClient::new(Arc::new(client), Arc::new(self.clone()))
    }

    /// Creates a basic Helius client from this configuration
    ///
    /// # Returns
    /// A `Result` containing a Helius client with basic RPC capabilities
    pub fn create_client(self) -> Result<Helius> {
        let client: Client = Client::builder().build().map_err(HeliusError::ReqwestError)?;
        let rpc_client: Arc<RpcClient> = Arc::new(self.rpc_client_with_reqwest_client(client.clone())?);

        Ok(Helius {
            config: Arc::new(self),
            client,
            rpc_client,
            async_rpc_client: None,
            ws_client: None,
        })
    }

    /// Creates a Helius client with async Solana capabilities
    ///
    /// # Returns
    /// A `Result` containing a Helius client with both RPC and async Solana capabilities
    pub fn create_client_with_async(self) -> Result<Helius> {
        let client: Client = Client::builder().build().map_err(HeliusError::ReqwestError)?;
        let mut rpc_url: Url = Url::parse(&self.endpoints.rpc)
            .map_err(|e: ParseError| HeliusError::InvalidInput(format!("Invalid RPC URL: {}", e)))?;
        
        rpc_url.query_pairs_mut()
            .append_pair("api-key", &self.api_key);

        let async_solana_client: Arc<AsyncSolanaRpcClient> = Arc::new(AsyncSolanaRpcClient::new(rpc_url.to_string()));
        let rpc_client: Arc<RpcClient> = Arc::new(self.rpc_client_with_reqwest_client(client.clone())?);

        Ok(Helius {
            config: Arc::new(self),
            client,
            rpc_client,
            async_rpc_client: Some(async_solana_client),
            ws_client: None,
        })
    }

    /// Creates a Helius client with websocket support
    ///
    /// # Arguments
    /// * `ping_interval_secs` - Optional duration in seconds between ping messages
    /// * `pong_timeout_secs` - Optional duration in seconds to wait for pong response
    ///
    /// # Returns
    /// A `Result` containing a Helius client with websocket support
    pub async fn create_client_with_ws(
        self,
        ping_interval_secs: Option<u64>,
        pong_timeout_secs: Option<u64>,
    ) -> Result<Helius> {
        let client: Client = Client::builder().build().map_err(HeliusError::ReqwestError)?;
        let rpc_client: Arc<RpcClient> = Arc::new(self.rpc_client_with_reqwest_client(client.clone())?);
        
        let wss: String = format!("{}{}", ENHANCED_WEBSOCKET_URL, self.api_key);
        let ws_client: Arc<EnhancedWebsocket> = Arc::new(
            EnhancedWebsocket::new(&wss, ping_interval_secs, pong_timeout_secs).await?
        );

        Ok(Helius {
            config: Arc::new(self),
            client,
            rpc_client,
            async_rpc_client: None,
            ws_client: Some(ws_client),
        })
    }

    /// Creates a full-featured Helius client with both async and websocket support
    ///
    /// # Arguments
    /// * `ping_interval_secs` - Optional duration in seconds between ping messages
    /// * `pong_timeout_secs` - Optional duration in seconds to wait for pong response
    ///
    /// # Returns
    /// A `Result` containing a fully-featured Helius client
    pub async fn create_full_client(
        self,
        ping_interval_secs: Option<u64>,
        pong_timeout_secs: Option<u64>,
    ) -> Result<Helius> {
        let client: Client = Client::builder().build().map_err(HeliusError::ReqwestError)?;
        let rpc_client: Arc<RpcClient> = Arc::new(self.rpc_client_with_reqwest_client(client.clone())?);
        
        // Setup async client
        let mut rpc_url: Url = Url::parse(&self.endpoints.rpc)
            .map_err(|e: ParseError| HeliusError::InvalidInput(format!("Invalid RPC URL: {}", e)))?;
        rpc_url.query_pairs_mut()
            .append_pair("api-key", &self.api_key);
        let async_solana_client = Arc::new(AsyncSolanaRpcClient::new(rpc_url.to_string()));

        // Setup websocket
        let wss: String = format!("{}{}", ENHANCED_WEBSOCKET_URL, self.api_key);
        let ws_client: Arc<EnhancedWebsocket> = Arc::new(
            EnhancedWebsocket::new(&wss, ping_interval_secs, pong_timeout_secs).await?
        );

        Ok(Helius {
            config: Arc::new(self),
            client,
            rpc_client,
            async_rpc_client: Some(async_solana_client),
            ws_client: Some(ws_client),
        })
    }

    pub fn mint_api_authority(&self) -> MintApiAuthority {
        MintApiAuthority::from_cluster(&self.cluster)
    }
}
