use std::{ops::Deref, sync::Arc};

use crate::config::Config;
use crate::error::{HeliusError, Result};
use crate::rpc_client::RpcClient;
use crate::types::{validate_rpc_url, ApiKey, Cluster, HeliusEndpoints};
use crate::websocket::EnhancedWebsocket;

use reqwest::Client;
use solana_client::nonblocking::rpc_client::RpcClient as AsyncSolanaRpcClient;
use solana_client::rpc_client::RpcClient as SolanaRpcClient;
use solana_commitment_config::CommitmentConfig;

/// The `Helius` struct is the main entry point to interacting with the SDK
///
/// This client is responsible for setting up the network and configuration settings used to interact with the various provided methods.
/// It also provides methods to access RPC client functionalities. The client ensures thread-safe access to the underlying RPC client
pub struct Helius {
    /// The configuration which specifies an `api_key`, `cluster`, and the requisite `endpoints`
    pub config: Arc<Config>,
    /// An HTTP client used for making API requests. The client is reused for all requests made through this instance of `Helius`
    pub client: Client,
    /// A reference-counted RPC client tailored for making requests in a thread-safe manner
    pub rpc_client: Arc<RpcClient>,
    /// An optional asynchronous Solana client for async operations
    pub async_rpc_client: Option<Arc<AsyncSolanaRpcClient>>,
    /// A reference-counted enhanced (geyser) websocket client
    pub ws_client: Option<Arc<EnhancedWebsocket>>,
}

impl Helius {
    /// Creates a basic Helius client for standard RPC operations
    ///
    /// This is the simplest way to create a Helius client. It is **synchronous** and does not
    /// require `.await`. For async Solana operations or WebSocket support, use `new_async()`
    /// or `HeliusBuilder`.
    ///
    /// # Arguments
    /// * `api_key` - The API key required for authenticating the requests made
    /// * `cluster` - The Solana cluster (Devnet or MainnetBeta) that defines the given network environment
    ///
    /// # Returns
    /// An instance of `Helius` if successful. A `HeliusError` is returned if an error occurs during configuration or initialization
    ///
    /// # Example
    /// ```rust
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// let helius = Helius::new("your_api_key", Cluster::Devnet)
    ///     .expect("Failed to create a Helius client");
    /// ```
    pub fn new(api_key: &str, cluster: Cluster) -> Result<Self> {
        let api_key = ApiKey::new(api_key)?;
        let endpoints = HeliusEndpoints::for_cluster(&cluster);
        let config = Arc::new(Config {
            api_key: Some(api_key),
            cluster,
            endpoints,
            custom_url: None,
        });

        let client = Client::builder().build().map_err(HeliusError::ReqwestError)?;
        let rpc_client = Arc::new(RpcClient::new(Arc::new(client.clone()), config.clone())?);

        Ok(Helius {
            config,
            client,
            rpc_client,
            async_rpc_client: None,
            ws_client: None,
        })
    }

    /// Creates a full-featured async Helius client with WebSocket support
    ///
    /// This is the recommended constructor for production applications that need:
    /// - Async Solana RPC operations
    /// - Enhanced WebSocket transaction streaming
    /// - Confirmed commitment level
    ///
    /// This constructor is **async** because it establishes a WebSocket connection.
    /// For basic sync operations, use `new()`. For custom configuration, use `HeliusBuilder`.
    ///
    /// # Arguments
    /// * `api_key` - The API key required for authenticating the requests made
    /// * `cluster` - The Solana cluster (Devnet or MainnetBeta) that defines the given network environment
    ///
    /// # Returns
    /// An instance of `Helius` if successful. A `HeliusError` is returned if an error occurs during configuration or initialization
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new_async("your_api_key", Cluster::MainnetBeta)
    ///         .await
    ///         .expect("Failed to create a Helius client");
    ///
    ///     // Access async client
    ///     let async_client = helius.async_connection().expect("Async client available");
    ///
    ///     // Access WebSocket
    ///     let ws = helius.ws().expect("WebSocket available");
    /// }
    /// ```
    pub async fn new_async(api_key: &str, cluster: Cluster) -> Result<Self> {
        crate::HeliusBuilder::new()
            .with_api_key(api_key)?
            .with_cluster(cluster)
            .with_async_solana()
            .with_websocket(None, None)
            .with_commitment(CommitmentConfig::confirmed())
            .build()
            .await
    }

    /// Creates a Helius client with a custom RPC URL
    ///
    /// Use this when you want to:
    /// - Connect to your own RPC node
    /// - Use a third-party RPC provider
    /// - Point to localhost for development
    /// - Route through a proxy
    ///
    /// This constructor is **synchronous** and does not require `.await`.
    /// API key is optional when using custom URLs. You can add one via `HeliusBuilder`
    /// if your custom endpoint requires authentication.
    ///
    /// # Arguments
    /// * `url` - The custom RPC endpoint URL (http:// or https://)
    ///
    /// # Returns
    /// An instance of `Helius` if successful. A `HeliusError` is returned if the URL is invalid
    ///
    /// # Example
    /// ```rust
    /// use helius::Helius;
    ///
    /// // Production custom RPC
    /// let helius = Helius::new_with_url("https://my-rpc-provider.com/")
    ///     .expect("Failed to create client");
    ///
    /// // Local development
    /// let local_helius = Helius::new_with_url("http://localhost:8899")
    ///     .expect("Failed to create client");
    /// ```
    pub fn new_with_url(url: &str) -> Result<Self> {
        let validated = validate_rpc_url(url)?;
        let url_string = validated.to_string();
        let config = Arc::new(Config {
            api_key: None,
            cluster: Cluster::Devnet, // Default for custom URLs
            endpoints: HeliusEndpoints {
                api: url_string.clone(),
                rpc: url_string.clone(),
            },
            custom_url: Some(url_string),
        });

        let client = Client::builder().build().map_err(HeliusError::ReqwestError)?;
        let rpc_client = Arc::new(RpcClient::new(Arc::new(client.clone()), config.clone())?);

        Ok(Helius {
            config,
            client,
            rpc_client,
            async_rpc_client: None,
            ws_client: None,
        })
    }

    /// Provides a thread-safe way to access RPC functionalities
    ///
    /// # Returns
    /// A cloned `Arc<RpcClient>` that can be safely shared across threads
    pub fn rpc(&self) -> Arc<RpcClient> {
        self.rpc_client.clone()
    }

    /// Provides a thread-safe way to access asynchronous Solana client functionalities
    ///
    /// # Returns
    /// A `Result` containing a `HeliusAsyncSolanaClient` if an `async_rpc_client` exists, otherwise a `HeliusError`
    pub fn async_connection(&self) -> Result<HeliusAsyncSolanaClient> {
        match &self.async_rpc_client {
            Some(client) => Ok(HeliusAsyncSolanaClient::new(client.clone())),
            None => Err(HeliusError::ClientNotInitialized {
                text: "An asynchronous Solana RPC client must be initialized before trying to access async_connection"
                    .to_string(),
            }),
        }
    }

    /// Provides a thread-safe way to access synchronous Solana client functionalities
    ///
    /// # Returns
    /// A cloned `Arc<SolanaRpcClient>` that can be safely shared across threads
    pub fn connection(&self) -> Arc<SolanaRpcClient> {
        self.rpc_client.solana_client.clone()
    }

    /// Returns the enhanced (Geyser) WebSocket client, if one was initialized.
    ///
    /// The WebSocket client is only available when the `Helius` instance was created with
    /// `new_async()` or via `HeliusBuilder::with_websocket()`.
    ///
    /// # Returns
    /// `Some(Arc<EnhancedWebsocket>)` if a WebSocket client is available, `None` otherwise
    pub fn ws(&self) -> Option<Arc<EnhancedWebsocket>> {
        self.ws_client.clone()
    }

    /// Returns the client configuration.
    ///
    /// # Returns
    /// A cloned `Arc<Config>` containing the API key, cluster, and endpoint settings
    pub fn config(&self) -> Arc<Config> {
        self.config.clone()
    }
}

/// A wrapper around the asynchronous Solana RPC client that provides thread-safe access
pub struct HeliusAsyncSolanaClient {
    client: Arc<AsyncSolanaRpcClient>,
}

impl HeliusAsyncSolanaClient {
    /// Creates a new instance of `HeliusAsyncSolanaClient`
    ///
    /// # Arguments
    /// * `client` - The asynchronous Solana RPC client to wrap
    ///
    /// # Returns
    /// An instance of `HeliusAsyncSolanaClient`
    pub fn new(client: Arc<AsyncSolanaRpcClient>) -> Self {
        Self { client }
    }
}

impl Deref for HeliusAsyncSolanaClient {
    type Target = AsyncSolanaRpcClient;

    /// Dereferences the wrapper to provide access to the underlying asynchronous Solana RPC client
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
