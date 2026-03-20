use crate::config::Config;
use crate::error::{HeliusError, Result};
use crate::types::{validate_rpc_url, validate_ws_url, ApiKey, Cluster, HeliusEndpoints};
use crate::websocket::EnhancedWebsocket;
use crate::Helius;
use reqwest::Client;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncSolanaRpcClient;
use solana_commitment_config::CommitmentConfig;
use std::sync::Arc;
use url::Url;

/// Builder for creating Helius client instances with flexible configuration.
///
/// The builder pattern provides a clean, type-safe way to configure the Helius
/// client with various options including custom RPC endpoints, optional API keys,
/// async Solana clients, WebSocket support, and more.
///
/// # Examples
///
/// ## Standard Helius Endpoint
/// ```ignore
/// use helius::{HeliusBuilder, Cluster};
///
/// let helius = HeliusBuilder::new()
///     .with_api_key("your-api-key")?
///     .with_cluster(Cluster::MainnetBeta)
///     .build()
///     .await?;
/// ```
///
/// ## Custom RPC Endpoint
/// ```ignore
/// let helius = HeliusBuilder::new()
///     .with_custom_url("https://my-rpc-provider.com/")?
///     .with_api_key("optional-key")?
///     .build()
///     .await?;
/// ```
///
/// ## Development with Localhost
/// ```ignore
/// let helius = HeliusBuilder::new()
///     .with_custom_url("http://localhost:8899")?
///     .build()
///     .await?;
/// ```
///
/// ## Full-Featured Setup
/// ```ignore
/// let helius = HeliusBuilder::new()
///     .with_api_key("your-api-key")?
///     .with_cluster(Cluster::MainnetBeta)
///     .with_async_solana()
///     .with_websocket(None, None)
///     .with_commitment(CommitmentConfig::confirmed())
///     .build()
///     .await?;
/// ```
#[derive(Default)]
pub struct HeliusBuilder {
    api_key: Option<ApiKey>,
    cluster: Option<Cluster>,
    custom_rpc_url: Option<Url>,
    custom_api_url: Option<Url>,
    custom_ws_url: Option<Url>,
    commitment: Option<CommitmentConfig>,
    http_client: Option<Client>,
    enable_async: bool,
    ws_config: Option<(Option<u64>, Option<u64>)>, // (ping_interval, pong_timeout)
}

impl HeliusBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the API key for authentication.
    ///
    /// Required for:
    /// - Helius-hosted endpoints
    /// - Webhook operations
    /// - Enhanced transaction parsing
    ///
    /// Optional for custom RPC endpoints.
    ///
    /// # Errors
    /// Returns error if the API key is empty or whitespace-only.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// HeliusBuilder::new()
    ///     .with_api_key("your-api-key")?
    /// ```
    pub fn with_api_key(mut self, key: impl Into<String>) -> Result<Self> {
        self.api_key = Some(ApiKey::new(key)?);
        Ok(self)
    }

    /// Sets the Solana cluster (Devnet, MainnetBeta, or StakedMainnetBeta).
    ///
    /// Only used when connecting to Helius-hosted endpoints.
    /// Ignored when using custom URLs.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use helius::types::Cluster;
    ///
    /// HeliusBuilder::new()
    ///     .with_cluster(Cluster::MainnetBeta)
    /// ```
    pub fn with_cluster(mut self, cluster: Cluster) -> Self {
        self.cluster = Some(cluster);
        self
    }

    /// Sets a custom RPC URL instead of using Helius-hosted endpoints.
    ///
    /// This allows you to:
    /// - Use your own RPC node
    /// - Connect through a proxy
    /// - Point to localhost for development
    /// - Use third-party RPC providers
    ///
    /// # Security
    /// - Only http:// and https:// schemes are allowed
    /// - Credentials in URLs are rejected (use `with_api_key()` instead)
    /// - Localhost is allowed (useful for development)
    ///
    /// # Arguments
    /// * `url` - The custom RPC endpoint URL
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Production custom RPC
    /// builder.with_custom_url("https://my-rpc-provider.com/")?;
    ///
    /// // Local development
    /// builder.with_custom_url("http://localhost:8899")?;
    /// ```
    pub fn with_custom_url(mut self, url: impl AsRef<str>) -> Result<Self> {
        let validated = validate_rpc_url(url.as_ref())?;
        self.custom_rpc_url = Some(validated.clone());

        // Default API URL to same as RPC URL
        if self.custom_api_url.is_none() {
            self.custom_api_url = Some(validated);
        }

        Ok(self)
    }

    /// Sets a custom API endpoint URL (separate from RPC).
    ///
    /// Use this when your API endpoint differs from your RPC endpoint.
    /// This is where webhook and enhanced transaction requests are sent.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// builder
    ///     .with_custom_url("https://rpc.example.com/")?
    ///     .with_custom_api_url("https://api.example.com/")?;
    /// ```
    pub fn with_custom_api_url(mut self, url: impl AsRef<str>) -> Result<Self> {
        self.custom_api_url = Some(validate_rpc_url(url.as_ref())?);
        Ok(self)
    }

    /// Sets a custom WebSocket URL for enhanced transactions.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// builder.with_custom_ws_url("wss://ws.example.com/")?;
    /// ```
    pub fn with_custom_ws_url(mut self, url: impl AsRef<str>) -> Result<Self> {
        self.custom_ws_url = Some(validate_ws_url(url.as_ref())?);
        Ok(self)
    }

    /// Sets the commitment level for Solana transactions.
    ///
    /// Common values:
    /// - `CommitmentConfig::processed()` - Fastest, least safe
    /// - `CommitmentConfig::confirmed()` - Balanced (recommended)
    /// - `CommitmentConfig::finalized()` - Slowest, most safe
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use solana_commitment_config::CommitmentConfig;
    ///
    /// builder.with_commitment(CommitmentConfig::confirmed());
    /// ```
    pub fn with_commitment(mut self, commitment: CommitmentConfig) -> Self {
        self.commitment = Some(commitment);
        self
    }

    /// Enables async Solana client support.
    ///
    /// Call this to add `async_rpc_client` to the Helius instance.
    /// Access it via `helius.async_connection()?`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let helius = HeliusBuilder::new()
    ///     .with_api_key("key")?
    ///     .with_cluster(Cluster::MainnetBeta)
    ///     .with_async_solana()
    ///     .build()
    ///     .await?;
    ///
    /// let async_client = helius.async_connection()?;
    /// ```
    pub fn with_async_solana(mut self) -> Self {
        self.enable_async = true;
        self
    }

    /// Enables WebSocket support for enhanced transaction streaming.
    ///
    /// # Arguments
    /// * `ping_interval_secs` - Seconds between ping messages (default: 10)
    /// * `pong_timeout_secs` - Seconds to wait for pong before disconnect (default: 30)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Default timeouts
    /// builder.with_websocket(None, None);
    ///
    /// // Custom timeouts
    /// builder.with_websocket(Some(5), Some(15));
    /// ```
    pub fn with_websocket(mut self, ping_interval_secs: Option<u64>, pong_timeout_secs: Option<u64>) -> Self {
        self.ws_config = Some((ping_interval_secs, pong_timeout_secs));
        self
    }

    /// Uses a custom reqwest HTTP client.
    ///
    /// Useful for configuring:
    /// - Custom timeouts
    /// - Proxy settings
    /// - TLS configuration
    /// - Connection pooling
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::time::Duration;
    /// use reqwest::Client;
    ///
    /// let client = Client::builder()
    ///     .timeout(Duration::from_secs(30))
    ///     .build()?;
    ///
    /// builder.with_http_client(client);
    /// ```
    pub fn with_http_client(mut self, client: Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Builds the Helius client with the configured settings.
    ///
    /// # Errors
    /// - `InvalidInput` if configuration is incomplete or invalid
    /// - `ReqwestError` if HTTP client creation fails
    /// - Various errors if WebSocket connection fails (when enabled)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let helius = HeliusBuilder::new()
    ///     .with_api_key("key")?
    ///     .with_cluster(Cluster::MainnetBeta)
    ///     .build()
    ///     .await?;
    /// ```
    pub async fn build(self) -> Result<Helius> {
        // Build config (doesn't move self anymore since build_config takes &self)
        let config = self.build_config()?;

        // Now extract fields
        let custom_ws_url = self.custom_ws_url;
        let ws_config = self.ws_config;
        let commitment = self.commitment;
        let enable_async = self.enable_async;
        let http_client = self.http_client;
        let client = http_client.unwrap_or_else(|| Client::builder().build().expect("Failed to build HTTP client"));

        // Build RPC client
        let rpc_client = if let Some(commitment) = commitment {
            Arc::new(crate::rpc_client::RpcClient::new_with_commitment(
                Arc::new(client.clone()),
                config.clone(),
                commitment,
            )?)
        } else {
            Arc::new(crate::rpc_client::RpcClient::new(
                Arc::new(client.clone()),
                config.clone(),
            )?)
        };

        // Build async Solana client if requested
        let async_rpc_client = if enable_async {
            let url = config.build_rpc_url();
            let async_client = if let Some(commitment) = commitment {
                AsyncSolanaRpcClient::new_with_commitment(url, commitment)
            } else {
                AsyncSolanaRpcClient::new(url)
            };
            Some(Arc::new(async_client))
        } else {
            None
        };

        // Build WebSocket client if requested
        let ws_client = if let Some((ping_interval, pong_timeout)) = ws_config {
            let ws_url = if let Some(custom_ws) = custom_ws_url {
                custom_ws.to_string()
            } else {
                // Requires API key for Helius WebSocket
                let api_key = config.require_api_key("WebSocket connections")?;
                EnhancedWebsocket::get_url(&config.cluster, api_key.as_str())?
            };

            Some(Arc::new(
                EnhancedWebsocket::new(&ws_url, ping_interval, pong_timeout).await?,
            ))
        } else {
            None
        };

        Ok(Helius {
            config,
            client,
            rpc_client,
            async_rpc_client,
            ws_client,
        })
    }

    /// Internal: Builds the Config from builder settings.
    fn build_config(&self) -> Result<Arc<Config>> {
        // Case 1: Custom URL provided
        if let Some(ref rpc_url) = self.custom_rpc_url {
            return Ok(Arc::new(Config {
                api_key: self.api_key.clone(),
                cluster: self.cluster.clone().unwrap_or(Cluster::Devnet), // Default for custom URLs
                endpoints: HeliusEndpoints {
                    api: self
                        .custom_api_url
                        .as_ref()
                        .map(|u| u.to_string())
                        .unwrap_or_else(|| rpc_url.to_string()),
                    rpc: rpc_url.to_string(),
                },
                custom_url: Some(rpc_url.to_string()),
            }));
        }

        // Case 2: Standard Helius endpoints
        let cluster = self.cluster.clone().ok_or_else(|| {
            HeliusError::InvalidInput(
                "Either cluster or custom URL must be specified. \
                 Use .with_cluster(Cluster::MainnetBeta) or .with_custom_url(\"...\")"
                    .to_string(),
            )
        })?;

        // Warn if no API key for Helius endpoints
        if self.api_key.is_none() {
            log::warn!(
                "No API key provided for Helius endpoint. \
                 Most features require authentication. Use .with_api_key(\"your-key\")"
            );
        }

        let endpoints = HeliusEndpoints::for_cluster(&cluster);

        Ok(Arc::new(Config {
            api_key: self.api_key.clone(),
            cluster,
            endpoints,
            custom_url: None,
        }))
    }
}
