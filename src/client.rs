use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;
use crate::rpc_client::RpcClient;
use crate::types::Cluster;
use crate::websocket::{EnhancedWebsocket, ENHANCED_WEBSOCKET_URL};

use reqwest::Client;
use solana_client::rpc_client::RpcClient as SolanaRpcClient;

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
    /// A reference-counted enhanced (geyser) websocket client
    pub ws_client: Option<Arc<EnhancedWebsocket>>,
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
    /// use helius::client::Helius;
    /// use helius::types::Cluster;
    ///
    /// let helius = Helius::new("your_api_key", Cluster::Devnet).expect("Failed to create a Helius client");
    /// ```
    pub fn new(api_key: &str, cluster: Cluster) -> Result<Self> {
        let config: Arc<Config> = Arc::new(Config::new(api_key, cluster)?);
        let client: Client = Client::new();
        let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), config.clone())?);
        Ok(Helius {
            config,
            client,
            rpc_client,
            ws_client: None,
        })
    }

    /// The enhanced websocket is optional, and this method is used to create a new instance of `Helius` with an enhanced websocket client.
    /// Upon calling this method, the websocket will connect hence the asynchronous function definition omission from the default `new` method.
    ///
    /// # Example
    /// ```rust
    /// use helius::Helius;
    /// use helius::error::Result;
    /// use helius::types::{Cluster, RpcTransactionsConfig, TransactionSubscribeFilter, TransactionSubscribeOptions};
    /// use solana_sdk::pubkey;
    /// use tokio_stream::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///   let helius = Helius::new("your_api_key", Cluster::MainnetBeta).expect("Failed to create a Helius client");
    ///   // you may monitor transactions for any pubkey, this is just an example.
    ///   let key = pubkey!("BtsmiEEvnSuUnKxqXj2PZRYpPJAc7C34mGz8gtJ1DAaH");
    ///   let config = RpcTransactionsConfig {
    ///     filter: TransactionSubscribeFilter::standard(&key),
    ///     options: TransactionSubscribeOptions::default(),
    ///   };
    ///   if let Some(ws) = helius.ws() {
    ///     let (mut stream, _unsub) = ws.transaction_subscribe(config).await?;
    ///     while let Some(event) = stream.next().await {
    ///       println!("{:#?}", event);
    ///     }
    ///   }
    ///   Ok(())
    /// }
    /// ```
    pub async fn new_with_ws(api_key: &str, cluster: Cluster) -> Result<Self> {
        let config: Arc<Config> = Arc::new(Config::new(api_key, cluster)?);
        let client: Client = Client::new();
        let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), config.clone())?);
        let wss = format!("{}{}", ENHANCED_WEBSOCKET_URL, api_key);
        let ws_client = Arc::new(EnhancedWebsocket::new(&wss).await?);
        Ok(Helius {
            config,
            client,
            rpc_client,
            ws_client: Some(ws_client),
        })
    }

    /// Provides a thread-safe way to access RPC functionalities
    ///
    /// # Returns
    /// A cloned `Arc<RpcClient>` that can be safely shared across threads
    pub fn rpc(&self) -> Arc<RpcClient> {
        self.rpc_client.clone()
    }

    pub fn connection(&self) -> Arc<SolanaRpcClient> {
        self.rpc_client.solana_client.clone()
    }

    pub fn ws(&self) -> Option<Arc<EnhancedWebsocket>> {
        self.ws_client.clone()
    }
}
