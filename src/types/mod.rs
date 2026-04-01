pub mod enhanced_transaction_types;
pub mod enhanced_websocket;
pub mod enums;
pub mod inner;
pub mod options;
pub mod zk_compression_types;

pub use self::enhanced_transaction_types::*;
pub use self::enhanced_websocket::{
    FullTransactionNotification, RpcTransactionsConfig, TransactionCommitment, TransactionNotification,
    TransactionSubscribeFilter, TransactionSubscribeOptions, UiEnhancedTransactionEncoding,
};
pub use self::enums::*;
pub use self::inner::*;
pub use self::options::*;
pub use self::zk_compression_types::*;

// Re-export wallet types
pub use self::inner::{
    BalanceChange, BalancesPagination, BalancesResponse, BatchIdentityRequest, FundingSource, HistoryResponse,
    HistoryTransaction, Identity, Nft, Pagination, TokenAccountsOption, TokenBalance, Transfer, TransferDirection,
    TransfersResponse,
};

use crate::error::{HeliusError, Result};
use url::Url;

/// Type-safe wrapper for Helius API keys.
///
/// Validates that keys are non-empty and provides type safety to prevent
/// accidentally passing wrong strings as API keys.
///
/// # Examples
///
/// ```
/// use helius::types::ApiKey;
///
/// let key = ApiKey::new("your-api-key").unwrap();
/// assert_eq!(key.as_str(), "your-api-key");
/// ```
#[derive(Clone, Debug)]
pub struct ApiKey(String);

impl ApiKey {
    /// Creates a new API key.
    ///
    /// # Errors
    /// Returns `HeliusError::InvalidInput` if the key is empty or whitespace-only.
    ///
    /// # Examples
    ///
    /// ```
    /// use helius::types::ApiKey;
    ///
    /// let key = ApiKey::new("my-api-key").unwrap();
    /// assert_eq!(key.as_str(), "my-api-key");
    ///
    /// // Empty keys are rejected
    /// assert!(ApiKey::new("").is_err());
    /// assert!(ApiKey::new("   ").is_err());
    /// ```
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let key = key.into();
        let trimmed = key.trim();

        if trimmed.is_empty() {
            return Err(HeliusError::InvalidInput("API key cannot be empty".to_string()));
        }

        Ok(ApiKey(key))
    }

    /// Returns the API key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<ApiKey> for String {
    fn from(key: ApiKey) -> String {
        key.0
    }
}

/// Validates RPC URLs with essential security checks.
///
/// # Security
/// - Only allows http/https schemes (prevents file://, javascript:, etc.)
/// - Rejects embedded credentials (user:pass@ format)
/// - Validates URL structure
///
/// # Arguments
/// * `url` - The RPC endpoint URL to validate
///
/// # Examples
/// ```
/// use helius::types::validate_rpc_url;
///
/// // Valid URLs
/// assert!(validate_rpc_url("https://api.mainnet-beta.solana.com").is_ok());
/// assert!(validate_rpc_url("http://localhost:8899").is_ok());
///
/// // Invalid URLs
/// assert!(validate_rpc_url("file:///etc/passwd").is_err());
/// assert!(validate_rpc_url("https://user:pass@api.com").is_err());
/// ```
pub fn validate_rpc_url(url: &str) -> Result<Url> {
    let parsed = Url::parse(url)
        .map_err(|e| HeliusError::InvalidInput(format!("Invalid URL format: {}. Expected: https://example.com/", e)))?;

    // Only allow http/https schemes
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(HeliusError::InvalidInput(format!(
                "Invalid URL scheme '{}'. Only 'http' and 'https' are allowed for RPC endpoints.",
                scheme
            )))
        }
    }

    // Reject credentials in URL (security risk - logged by proxies)
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(HeliusError::InvalidInput(
            "URL contains embedded credentials. Remove them and use .with_api_key() instead.".to_string(),
        ));
    }

    Ok(parsed)
}

/// Validates WebSocket URLs (wss:// or ws://).
///
/// # Security
/// - Only allows ws/wss schemes
/// - Rejects embedded credentials
/// - Validates URL structure
///
/// # Arguments
/// * `url` - The WebSocket endpoint URL to validate
///
/// # Examples
/// ```
/// use helius::types::validate_ws_url;
///
/// // Valid URLs
/// assert!(validate_ws_url("wss://atlas-mainnet.helius-rpc.com").is_ok());
/// assert!(validate_ws_url("ws://localhost:8900").is_ok());
///
/// // Invalid URLs
/// assert!(validate_ws_url("http://example.com").is_err());
/// assert!(validate_ws_url("wss://user:pass@example.com").is_err());
/// ```
pub fn validate_ws_url(url: &str) -> Result<Url> {
    let parsed = Url::parse(url).map_err(|e| HeliusError::InvalidInput(format!("Invalid WebSocket URL: {}", e)))?;

    match parsed.scheme() {
        "ws" | "wss" => {}
        scheme => {
            return Err(HeliusError::InvalidInput(format!(
                "Invalid WebSocket scheme '{}'. Only 'ws' and 'wss' are allowed.",
                scheme
            )))
        }
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(HeliusError::InvalidInput(
            "WebSocket URL contains embedded credentials.".to_string(),
        ));
    }

    Ok(parsed)
}
