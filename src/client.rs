use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;
use crate::rpc_client::RpcClient;
use crate::types::Cluster;

use reqwest::Client;

/// The `Helius` struct is the main entry point to interacting with the SDK
///
/// This client is responsible for setting up the network and configuration settins used to interact with the various provided methods.
/// It also provides methods to access RPC client functionalities. The client ensures thread-safe access to the underlying RPC client
pub struct Helius {
    /// The configuration which specifies an `api_key`, `cluster`, and the requisite `endpoints`
    pub config: Arc<Config>,
    /// An HTTP client used for making API requests. The client is reused for all requests made through this instance of `Helius`
    pub client: Client,
    /// A reference-counted RPC client tailored for making requests in a thread-safe manner
    pub rpc_client: Arc<RpcClient>,
}

impl Helius {
    /// Creates a new instance of `Helius` configured with a specific API key and a target cluster
    ///
    /// # Arguments
    /// * `api_key` - The API key required for authenticating requests made
    /// * `cluster` - The Solana cluster (Devnet or MainnetBeta) that defines the given network environment
    ///
    /// # Returns
    /// An instance of `Helius` if successful. A `HeliusError` is returned if an error occurs during configuration or initialization of the HTTP or RPC client
    ///
    /// # Example
    /// ```rust
    /// let helius = Helius::new("your_api_key", Cluster::Devnet).expect("Failed to create a Helius client")
    /// ```
    pub fn new(api_key: &str, cluster: Cluster) -> Result<Self> {
        let config: Arc<Config> = Arc::new(Config::new(api_key, cluster)?);
        let client: Client = Client::new();
        let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), config.clone())?);

        Ok(Helius {
            config,
            client,
            rpc_client,
        })
    }

    /// Provides a thread-safe way to access RPC functionalities
    ///
    /// # Returns
    /// A cloned `Arc<RpcClient>` that can be safely shared across threads
    pub fn rpc(&self) -> Arc<RpcClient> {
        self.rpc_client.clone()
    }
}
