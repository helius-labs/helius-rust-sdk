use crate::error::Result;
use crate::types::{
    BalancesResponse, BatchIdentityRequest, FundingSource, HistoryResponse, Identity, TransfersResponse,
};
use crate::Helius;

use reqwest::{Method, Url};

impl Helius {
    /// Get the appropriate base URL for wallet API calls
    ///
    /// Uses the config's API endpoint for testing/custom URLs, otherwise uses the
    /// production wallet API endpoint
    fn get_wallet_api_base_url(&self) -> String {
        // If using a custom URL (e.g., for testing with mockito), use the config's endpoint
        // Check if it's a local/test URL or custom URL
        let api_url = &self.config.endpoints.api;
        if self.config.custom_url.is_some()
            || api_url.starts_with("http://localhost")
            || api_url.starts_with("http://127.0.0.1")
        {
            self.config.endpoints.api.clone()
        } else {
            "https://api.helius.xyz/".to_string()
        }
    }

    /// Retrieves identity information for a known wallet address
    ///
    /// Returns identity information such as name, type, category, and tags for known
    /// wallet addresses like exchanges, protocols, etc.
    ///
    /// # Arguments
    /// * `wallet` - The Solana wallet address (base58 encoded) to lookup
    ///
    /// # Returns
    /// A `Result` wrapping an `Identity` if the wallet has known identity information
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing
    /// - The wallet address is invalid
    /// - No identity information is available (404)
    /// - The API request fails
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let identity = helius
    ///         .get_wallet_identity("HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664")
    ///         .await
    ///         .unwrap();
    ///     println!("Wallet name: {}", identity.name);
    /// }
    /// ```
    pub async fn get_wallet_identity(&self, wallet: &str) -> Result<Identity> {
        let api_key = self.config.require_api_key("wallet identity")?;
        let base_url = self.get_wallet_api_base_url();
        let url: String = format!("{}v1/wallet/{}/identity?api-key={}", base_url, wallet, api_key.as_str());
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client.handler.send(Method::GET, parsed_url, None::<&()>).await
    }

    /// Retrieves identity information for multiple wallet addresses in a single request
    ///
    /// Performs a batch lookup of identity information for up to 100 wallet addresses.
    ///
    /// # Arguments
    /// * `addresses` - A slice of Solana wallet addresses (1-100 addresses)
    ///
    /// # Returns
    /// A `Result` wrapping a vector of `Identity` objects for addresses with known identity information
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing
    /// - The addresses slice is empty or contains more than 100 addresses
    /// - Any wallet address is invalid
    /// - The API request fails
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let addresses = vec![
    ///         "HXsKP7wrBWaQ8T2Vtjry3Nj3oUgwYcqq9vrHDM12G664".to_string(),
    ///         "2ojv9BAiHUrvsm9gxDe7fJSzbNZSJcxZvf8dqmWGHG8S".to_string(),
    ///     ];
    ///     let identities = helius.get_batch_wallet_identity(&addresses).await.unwrap();
    ///     for identity in identities {
    ///         println!("Wallet: {} - {}", identity.address, identity.name);
    ///     }
    /// }
    /// ```
    pub async fn get_batch_wallet_identity(&self, addresses: &[String]) -> Result<Vec<Identity>> {
        let api_key = self.config.require_api_key("batch wallet identity")?;
        let base_url = self.get_wallet_api_base_url();
        let url: String = format!("{}v1/wallet/batch-identity?api-key={}", base_url, api_key.as_str());
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        let request = BatchIdentityRequest {
            addresses: addresses.to_vec(),
        };

        self.rpc_client
            .handler
            .send(Method::POST, parsed_url, Some(&request))
            .await
    }

    /// Retrieves token and NFT balances for a wallet
    ///
    /// Returns up to 100 tokens per request, sorted by USD value in descending order.
    /// Tokens with pricing data appear first, followed by tokens without prices.
    ///
    /// # Arguments
    /// * `wallet` - The Solana wallet address
    /// * `page` - Page number for pagination (1-indexed, default: 1)
    /// * `limit` - Maximum number of tokens per page (1-100, default: 100)
    /// * `show_zero_balance` - Include tokens with zero balance (default: false)
    /// * `show_native` - Include native SOL in results (default: true)
    /// * `show_nfts` - Include NFTs in results (max 100, first page only, default: false)
    ///
    /// # Returns
    /// A `Result` wrapping a `BalancesResponse` containing token balances, optional NFTs,
    /// total USD value, and pagination metadata
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing
    /// - The wallet address is invalid
    /// - Invalid pagination parameters
    /// - The API request fails
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let balances = helius
    ///         .get_wallet_balances(
    ///             "GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz",
    ///             Some(1),
    ///             Some(50),
    ///             Some(false),
    ///             Some(true),
    ///             Some(true),
    ///         )
    ///         .await
    ///         .unwrap();
    ///     println!("Total portfolio value: ${}", balances.total_usd_value);
    ///     for balance in balances.balances {
    ///         println!("{}: {} (${:?})", balance.symbol.unwrap_or_default(), balance.balance, balance.usd_value);
    ///     }
    /// }
    /// ```
    pub async fn get_wallet_balances(
        &self,
        wallet: &str,
        page: Option<u32>,
        limit: Option<u32>,
        show_zero_balance: Option<bool>,
        show_native: Option<bool>,
        show_nfts: Option<bool>,
    ) -> Result<BalancesResponse> {
        let api_key = self.config.require_api_key("wallet balances")?;
        let base_url = self.get_wallet_api_base_url();
        let mut url: String = format!("{}v1/wallet/{}/balances?api-key={}", base_url, wallet, api_key.as_str());

        if let Some(page) = page {
            url = format!("{}&page={}", url, page);
        }

        if let Some(limit) = limit {
            url = format!("{}&limit={}", url, limit);
        }

        if let Some(show_zero_balance) = show_zero_balance {
            url = format!("{}&showZeroBalance={}", url, show_zero_balance);
        }

        if let Some(show_native) = show_native {
            url = format!("{}&showNative={}", url, show_native);
        }

        if let Some(show_nfts) = show_nfts {
            url = format!("{}&showNfts={}", url, show_nfts);
        }

        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client.handler.send(Method::GET, parsed_url, None::<&()>).await
    }

    /// Retrieves transaction history for a wallet with balance changes
    ///
    /// Returns human-readable, parsed transactions with balance changes for each transaction.
    /// Results are in reverse chronological order (newest first).
    ///
    /// # Arguments
    /// * `wallet` - The Solana wallet address
    /// * `limit` - Maximum number of transactions per request (1-100, default: 100)
    /// * `before` - Fetch transactions before this signature (use `pagination.nextCursor` from previous response)
    /// * `after` - Fetch transactions after this signature (for ascending order pagination)
    /// * `transaction_type` - Filter by transaction type (e.g., "SWAP", "TRANSFER", "NFT_SALE")
    /// * `token_accounts` - Filter transactions involving token accounts ("none", "balanceChanged", "all")
    ///
    /// # Returns
    /// A `Result` wrapping a `HistoryResponse` containing transactions with balance changes
    /// and pagination information
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing
    /// - The wallet address is invalid
    /// - Invalid parameters
    /// - The API request fails
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let history = helius
    ///         .get_wallet_history(
    ///             "GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz",
    ///             Some(50),
    ///             None,
    ///             None,
    ///             Some("SWAP".to_string()),
    ///             Some("balanceChanged".to_string()),
    ///         )
    ///         .await
    ///         .unwrap();
    ///     for tx in history.data {
    ///         println!("Signature: {}", tx.signature);
    ///         for change in tx.balance_changes {
    ///             println!("  {} changed by {}", change.mint, change.amount);
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn get_wallet_history(
        &self,
        wallet: &str,
        limit: Option<u32>,
        before: Option<&str>,
        after: Option<&str>,
        transaction_type: Option<String>,
        token_accounts: Option<String>,
    ) -> Result<HistoryResponse> {
        let api_key = self.config.require_api_key("wallet history")?;
        let base_url = self.get_wallet_api_base_url();
        let mut url: String = format!("{}v1/wallet/{}/history?api-key={}", base_url, wallet, api_key.as_str());

        if let Some(limit) = limit {
            url = format!("{}&limit={}", url, limit);
        }

        if let Some(before) = before {
            url = format!("{}&before={}", url, before);
        }

        if let Some(after) = after {
            url = format!("{}&after={}", url, after);
        }

        if let Some(transaction_type) = transaction_type {
            url = format!("{}&type={}", url, transaction_type);
        }

        if let Some(token_accounts) = token_accounts {
            url = format!("{}&tokenAccounts={}", url, token_accounts);
        }

        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client.handler.send(Method::GET, parsed_url, None::<&()>).await
    }

    /// Retrieves all token transfer activity for a wallet
    ///
    /// Returns transfers in reverse chronological order (i.e., newest first) with sender/recipient information.
    ///
    /// # Arguments
    /// * `wallet` - The Solana wallet address
    /// * `limit` - Maximum number of transfers to return (1-100, default: 50)
    /// * `cursor` - Pagination cursor from previous response
    ///
    /// # Returns
    /// A `Result` wrapping a `TransfersResponse` containing transfer activity and pagination information
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing
    /// - The wallet address is invalid
    /// - Invalid parameters
    /// - The API request fails
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let transfers = helius
    ///         .get_wallet_transfers("GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz", Some(50), None)
    ///         .await
    ///         .unwrap();
    ///     for transfer in transfers.data {
    ///         println!(
    ///             "{} {} {} to/from {}",
    ///             transfer.direction,
    ///             transfer.amount,
    ///             transfer.symbol.unwrap_or_default(),
    ///             transfer.counterparty
    ///         );
    ///     }
    /// }
    /// ```
    pub async fn get_wallet_transfers(
        &self,
        wallet: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<TransfersResponse> {
        let api_key = self.config.require_api_key("wallet transfers")?;
        let base_url = self.get_wallet_api_base_url();
        let mut url: String = format!(
            "{}v1/wallet/{}/transfers?api-key={}",
            base_url,
            wallet,
            api_key.as_str()
        );

        if let Some(limit) = limit {
            url = format!("{}&limit={}", url, limit);
        }

        if let Some(cursor) = cursor {
            url = format!("{}&cursor={}", url, cursor);
        }

        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client.handler.send(Method::GET, parsed_url, None::<&()>).await
    }

    /// Discovers the original funding source of a wallet
    ///
    /// Analyzes the wallet's first incoming SOL transfer to identify if it was funded by
    /// an exchange, another wallet, etc.
    ///
    /// # Arguments
    /// * `wallet` - The Solana wallet address
    ///
    /// # Returns
    /// A `Result` wrapping a `FundingSource` containing information about the original funder,
    /// funding amount, transaction signature, and timestamp
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing
    /// - The wallet address is invalid
    /// - No funding transaction found (404)
    /// - The API request fails
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let funding = helius
    ///         .get_wallet_funding_source("GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz")
    ///         .await
    ///         .unwrap();
    ///     println!(
    ///         "Funded by: {} ({:?}) with {} SOL",
    ///         funding.funder_name.unwrap_or(funding.funder.clone()),
    ///         funding.funder_type,
    ///         funding.amount
    ///     );
    /// }
    /// ```
    pub async fn get_wallet_funding_source(&self, wallet: &str) -> Result<FundingSource> {
        let api_key = self.config.require_api_key("wallet funding source")?;
        let base_url = self.get_wallet_api_base_url();
        let url: String = format!(
            "{}v1/wallet/{}/funded-by?api-key={}",
            base_url,
            wallet,
            api_key.as_str()
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client.handler.send(Method::GET, parsed_url, None::<&()>).await
    }
}
