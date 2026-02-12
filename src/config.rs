use crate::error::{HeliusError, Result};
use crate::rpc_client::RpcClient;
use crate::types::{ApiKey, Cluster, HeliusEndpoints, MintApiAuthority};
use crate::websocket::EnhancedWebsocket;
use crate::Helius;
use reqwest::Client;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncSolanaRpcClient;
use std::sync::Arc;
use url::Url;

/// Configuration settings for the Helius client
///
/// `Config` contains all the necessary parameters needed to configure and authenticate the `Helius` client to interact with a specific Solana cluster
#[derive(Clone)]
pub struct Config {
    /// Optional API key for authentication.
    /// Required for webhooks, enhanced transactions, and Helius-hosted endpoints.
    pub api_key: Option<ApiKey>,
    /// The target Solana cluster the client will interact with
    pub cluster: Cluster,
    /// The endpoints associated with the specified `cluster`. Note these endpoints are automatically determined based on the cluster to ensure requests
    /// are made to the correct cluster
    pub endpoints: HeliusEndpoints,
    /// Custom RPC URL if provided (for debugging/logging)
    pub custom_url: Option<String>,
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
    ///
    /// # Deprecated
    /// Use `HeliusBuilder` for more flexible configuration:
    /// ```ignore
    /// use helius::HeliusBuilder;
    /// use helius::types::Cluster;
    ///
    /// let helius = HeliusBuilder::new()
    ///     .with_api_key("key")?
    ///     .with_cluster(Cluster::MainnetBeta)
    ///     .build()
    ///     .await?;
    /// ```
    pub fn new(api_key: &str, cluster: Cluster) -> Result<Self> {
        let endpoints: HeliusEndpoints = HeliusEndpoints::for_cluster(&cluster);

        Ok(Config {
            api_key: Some(ApiKey::new(api_key)?),
            cluster,
            endpoints,
            custom_url: None,
        })
    }

    /// Checks if an API key is available.
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Gets the API key or returns an error with helpful guidance.
    ///
    /// Use this in methods that require authentication.
    ///
    /// # Errors
    /// Returns `HeliusError::InvalidInput` if no API key is configured.
    pub fn require_api_key(&self, feature: &str) -> Result<&ApiKey> {
        self.api_key.as_ref().ok_or_else(|| {
            HeliusError::InvalidInput(format!(
                "API key is required for {}. \
                 Initialize with HeliusBuilder::new().with_api_key(\"your-key\")",
                feature
            ))
        })
    }

    /// Builds an RPC URL with authentication.
    ///
    /// Appends api-key query parameter only if key is present.
    pub fn build_rpc_url(&self) -> String {
        self.build_url(&self.endpoints.rpc)
    }

    /// Builds an API URL with authentication.
    pub fn build_api_url(&self) -> String {
        self.build_url(&self.endpoints.api)
    }

    /// Internal: Builds a URL with optional API key parameter.
    fn build_url(&self, base: &str) -> String {
        let mut url = Url::parse(base)
            .expect("Config endpoints should always be valid URLs");

        if let Some(ref key) = self.api_key {
            url.query_pairs_mut().append_pair("api-key", key.as_str());
        }

        url.to_string()
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
        let rpc_url = self.build_rpc_url();

        let async_solana_client: Arc<AsyncSolanaRpcClient> = Arc::new(AsyncSolanaRpcClient::new(rpc_url));
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

        let api_key = self.require_api_key("WebSocket connections")?;
        let wss: String = EnhancedWebsocket::get_url(&self.cluster, api_key.as_str())?;
        let ws_client: Arc<EnhancedWebsocket> =
            Arc::new(EnhancedWebsocket::new(&wss, ping_interval_secs, pong_timeout_secs).await?);

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
        let rpc_url = self.build_rpc_url();
        let async_solana_client = Arc::new(AsyncSolanaRpcClient::new(rpc_url));

        // Setup websocket
        let api_key = self.require_api_key("WebSocket connections")?;
        let wss: String = EnhancedWebsocket::get_url(&self.cluster, api_key.as_str())?;
        let ws_client: Arc<EnhancedWebsocket> =
            Arc::new(EnhancedWebsocket::new(&wss, ping_interval_secs, pong_timeout_secs).await?);

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
