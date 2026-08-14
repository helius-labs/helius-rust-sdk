use super::inner::TransactionSignatureEntry;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status_client_types::EncodedTransactionWithStatusMeta;

/// Filters for the `transactionSubscribe` enhanced WebSocket method.
///
/// Controls which transactions are delivered to the subscription. At minimum, provide
/// [`account_include`](TransactionSubscribeFilter::account_include) to receive transactions
/// involving specific accounts. All fields are optional — omitted fields impose no constraint.
///
/// Up to 50,000 addresses may be specified in each of `account_include`, `account_exclude`,
/// and `account_required`.
#[derive(Debug, Clone, PartialEq, Default, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSubscribeFilter {
    /// Include or exclude vote-related transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vote: Option<bool>,
    /// Include or exclude transactions that failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<bool>,
    /// Filter updates to a single transaction by its signature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Receive transactions that include **any** of these accounts (OR logic).
    /// Only one of the listed accounts needs to appear in the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_include: Option<Vec<String>>,
    /// Exclude transactions that involve any of these accounts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_exclude: Option<Vec<String>>,
    /// Transactions must include **all** of these accounts to match (AND logic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_required: Option<Vec<String>>,
}

impl TransactionSubscribeFilter {
    /// Creates a filter that subscribes to all transactions involving the given public key.
    pub fn standard(key: &Pubkey) -> Self {
        Self {
            account_include: Some(vec![key.to_string()]),
            ..Default::default()
        }
    }
}

/// The encoding format for transaction data returned by the enhanced WebSocket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UiEnhancedTransactionEncoding {
    Base58,
    Base64,
    #[serde(rename = "base64+zstd")]
    Base64Zstd,
    JsonParsed,
}

/// The commitment level for transaction subscription notifications.
///
/// Determines how finalized a transaction must be before it is delivered to the subscriber.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionCommitment {
    /// The transaction has been received by the leader and is being processed
    Processed,
    /// The transaction has been included in a block that has reached supermajority confirmation
    Confirmed,
    /// The transaction is in a block that has been finalized (maximum confirmation)
    Finalized,
}

/// The level of detail included in transaction subscription notifications.
///
/// Note: `Accounts` and `Full` require
/// [`max_supported_transaction_version`](TransactionSubscribeOptions::max_supported_transaction_version)
/// to be set (typically to `0`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionDetails {
    /// Include full transaction data with all instructions and metadata
    Full,
    /// Include only the transaction signatures
    Signatures,
    /// Include only the list of accounts involved
    Accounts,
    /// Include no transaction data (notification only)
    None,
}

/// Optional configuration for a `transactionSubscribe` subscription.
///
/// Controls the commitment level, encoding format, detail level, and other parameters for
/// transaction notifications. All fields are optional — the [`Default`] implementation provides
/// sensible defaults (`confirmed`, `jsonParsed`, `full`, rewards shown, version `0`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSubscribeOptions {
    /// The commitment level for notifications (`processed`, `confirmed`, or `finalized`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<TransactionCommitment>,
    /// The encoding format for returned transaction data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<UiEnhancedTransactionEncoding>,
    /// The level of detail for returned transaction data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_details: Option<TransactionDetails>,
    /// Whether reward data should be included in the notifications
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_rewards: Option<bool>,
    /// The highest transaction version to receive. Set to `0` for both legacy and v0 transactions.
    /// Required when `transaction_details` is `Full` or `Accounts`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_supported_transaction_version: Option<u8>,
}

impl Default for TransactionSubscribeOptions {
    fn default() -> Self {
        Self {
            commitment: Some(TransactionCommitment::Confirmed),
            encoding: Some(UiEnhancedTransactionEncoding::JsonParsed),
            transaction_details: Some(TransactionDetails::Full),
            show_rewards: Some(true),
            max_supported_transaction_version: Some(0),
        }
    }
}

/// Combined filter and options for a `transactionSubscribe` call.
///
/// Passed to [`EnhancedWebsocket::transaction_subscribe`](crate::websocket::EnhancedWebsocket::transaction_subscribe).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTransactionsConfig {
    /// The filter criteria that determine which transactions are delivered
    pub filter: TransactionSubscribeFilter,
    /// Optional configuration for commitment, encoding, and detail level
    pub options: TransactionSubscribeOptions,
}

/// A transaction notification returned in `"full"` mode from `transactionSubscribe`.
///
/// Contains the websocket-specific full transaction payload, including the top-level signature
/// and the transaction index within the slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullTransactionNotification {
    /// The full encoded transaction with status metadata.
    pub transaction: EncodedTransactionWithStatusMeta,
    /// The transaction signature (base-58 encoded).
    pub signature: String,
    /// The slot in which the transaction was processed.
    pub slot: u64,
    /// Zero-based position of the transaction within its block.
    pub transaction_index: u64,
}

/// A single transaction notification from `transactionSubscribe`.
///
/// The variant depends on the `transaction_details` option:
/// - [`TransactionDetails::Signatures`]: deserializes as [`TransactionNotification::Signature`]
/// - [`TransactionDetails::Full`] and [`TransactionDetails::Accounts`]: deserializes as
///   [`TransactionNotification::Full`]
///
/// If the API returns a shape that doesn't match a known variant,
/// [`TransactionNotification::Unknown`] captures the raw JSON so deserialization
/// never fails silently.
///
/// **Variant ordering matters:** serde tries `untagged` variants top-down.
/// `Full` must precede `Signature` because `Signature` would also match a full
/// payload (its extra fields are simply ignored). Reordering the variants will
/// cause full notifications to silently deserialize as `Signature`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransactionNotification {
    /// Full websocket transaction notification with the encoded transaction payload.
    Full(Box<FullTransactionNotification>),
    /// Lightweight signature notification with slot metadata only.
    Signature(TransactionSignatureEntry),
    /// Fallback for unrecognized response shapes (e.g. future API modes).
    Unknown(Value),
}

impl Default for TransactionNotification {
    fn default() -> Self {
        TransactionNotification::Signature(TransactionSignatureEntry {
            signature: String::new(),
            slot: 0,
            transaction_index: 0,
            err: None,
            memo: None,
            block_time: None,
            confirmation_status: None,
        })
    }
}
