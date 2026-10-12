//! Types for the [Parsed Events API](https://www.helius.dev/docs/parsed-events)
//!
//! Parsed Events is the successor to the Enhanced Transactions API. Its types live in their own
//! module rather than being glob-exported from [`crate::types`], because several names
//! (`ParseTransactionsRequest`, for one) already belong to the enhanced API. Import them by path:
//!
//! ```rust
//! use helius::types::parsed_events::{GetTransactionHistoryRequest, ParseTransactionsRequest};
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Commitment level for Parsed Events queries. `processed` is not supported.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParsedEventsCommitment {
    Confirmed,
    Finalized,
}

/// Sort order for Parsed Events transaction history.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParsedEventsSortOrder {
    Asc,
    Desc,
}

/// Whether a transaction executed successfully.
///
/// Unrecognized values deserialize as `Unknown`, so a status added to the API later does not
/// fail a whole page of results.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParsedTransactionStatus {
    /// The transaction executed successfully
    #[serde(rename = "OK")]
    Ok,
    /// The transaction executed and failed. It is still parsed
    #[serde(rename = "ERROR")]
    Error,
    /// A value this SDK does not recognize. It serializes as `"UNKNOWN"`, so the original
    /// value is not preserved through a round trip
    #[serde(other, rename = "UNKNOWN")]
    Unknown,
}

/// Whether a signature was fetched and parsed.
///
/// Unrecognized values deserialize as `Unknown`, so a status added to the API later does not
/// fail a whole page of results.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParserStatus {
    /// `parsed` is set
    #[serde(rename = "OK")]
    Ok,
    /// The transaction could not be fetched or parsed; `parser_error` says why
    #[serde(rename = "ERROR")]
    Error,
    /// A value this SDK does not recognize. It serializes as `"UNKNOWN"`, so the original
    /// value is not preserved through a round trip
    #[serde(other, rename = "UNKNOWN")]
    Unknown,
}

/// Integer comparison bounds. Each bound is optional.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonBounds {
    /// Match values greater than this value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<u64>,
    /// Match values greater than or equal to this value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<u64>,
    /// Match values less than this value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<u64>,
    /// Match values less than or equal to this value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<u64>,
}

/// Request for [`ParsedEvents::parse_transactions`](crate::parsed_events::ParsedEvents::parse_transactions)
/// (`POST /v1/parsed-events/transactions`)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParseTransactionsRequest {
    /// Transaction signatures to parse (max 100). Results come back in the same order, including duplicates
    pub transactions: Vec<String>,
    /// Commitment level. Defaults to `confirmed`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<ParsedEventsCommitment>,
    /// Include the original Solana transaction payload as `raw_transaction`. Defaults to `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_raw_transaction: Option<bool>,
}

/// Request for [`ParsedEvents::get_transaction_history`](crate::parsed_events::ParsedEvents::get_transaction_history)
/// (`POST /v1/parsed-events/transaction-history`)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionHistoryRequest {
    /// Address whose transaction history should be fetched. Program-wide scans are not supported
    pub address: String,
    /// Number of transactions to return, from 1 to 100. Defaults to 100
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Return transactions before this signature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_signature: Option<String>,
    /// Return transactions after this signature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_signature: Option<String>,
    /// Cursor returned by a previous page. An empty string is treated as no cursor (the first
    /// page), so a response's `pagination_token` can be passed straight back
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub pagination_token: Option<String>,
    /// Sort order. Defaults to `desc` (newest first)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ParsedEventsSortOrder>,
    /// Commitment level. Defaults to `confirmed`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<ParsedEventsCommitment>,
    /// Include raw Solana transaction payloads as `raw_transaction`. Defaults to `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_raw_transaction: Option<bool>,
    /// Slot bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<ComparisonBounds>,
    /// Block-time bounds in whole Unix seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<ComparisonBounds>,
}

fn is_none_or_empty(value: &Option<String>) -> bool {
    value.as_deref().is_none_or(str::is_empty)
}

/// A page of parsed transaction history
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionHistoryResponse {
    /// Parsed transaction results for this page
    #[serde(default)]
    pub data: Vec<ParsedTransactionResult>,
    /// Cursor for the next page. `None` once the end of the available range is reached
    #[serde(default)]
    pub pagination_token: Option<String>,
}

/// The result for one signature.
///
/// A missing or unparseable transaction does not fail the request: its item has
/// `parser_status: ParserStatus::Error` and a `parser_error`. Check `parser_status` (or match on
/// `parsed`) before reading the transaction.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTransactionResult {
    /// Transaction signature for this item
    pub signature: String,
    /// Whether the transaction was fetched and parsed
    pub parser_status: ParserStatus,
    /// The parsed transaction, when `parser_status` is `Ok`
    #[serde(default)]
    pub parsed: Option<ParsedTransaction>,
    /// The item-level error, when `parser_status` is `Error`
    #[serde(default)]
    pub parser_error: Option<ParserError>,
    /// The original Solana transaction payload. Present only when `include_raw_transaction` is set
    #[serde(default)]
    pub raw_transaction: Option<Value>,
}

/// Item-level parser error
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParserError {
    /// Machine-readable error code (e.g. `"transaction_not_found"`)
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

/// A transaction parsed by Parsed Events
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTransaction {
    /// Slot containing the transaction
    pub slot: u64,
    /// Unix timestamp (seconds) of the block, when available
    #[serde(default)]
    pub block_time: Option<i64>,
    /// Transaction fee in lamports
    pub fee: u64,
    /// Fee payer account, when available
    #[serde(default)]
    pub fee_payer: Option<String>,
    /// Whether the transaction executed successfully. Failed transactions are still parsed
    pub transaction_status: ParsedTransactionStatus,
    /// Raw execution error when `transaction_status` is `Error`
    #[serde(default)]
    pub error: Option<Value>,
    /// Custom program error decoded from `error`, when available
    #[serde(default)]
    pub decoded_error: Option<ParsedDecodedError>,
    /// Transaction-level SOL transfers, in lamports
    #[serde(default)]
    pub native_transfers: Vec<ParsedNativeTransfer>,
    /// Transaction-level SPL Token and Token-2022 transfers
    #[serde(default)]
    pub token_transfers: Vec<ParsedTokenTransfer>,
    /// Best transaction-level summary, or `None` for unrecognized transactions
    #[serde(default)]
    pub summary: Option<ParsedSummary>,
    /// Parsed top-level and inner instructions in execution order
    #[serde(default)]
    pub instructions: Vec<ParsedInstruction>,
}

/// A SOL movement detected at transaction level
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedNativeTransfer {
    /// Account sending lamports
    #[serde(default)]
    pub from_user_account: Option<String>,
    /// Account receiving lamports
    #[serde(default)]
    pub to_user_account: Option<String>,
    /// Amount in lamports
    pub amount: u64,
}

/// An SPL Token or Token-2022 movement detected at transaction level
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTokenTransfer {
    /// Owner or authority for the source side, when known
    #[serde(default)]
    pub from_user_account: Option<String>,
    /// Owner for the destination side, when known
    #[serde(default)]
    pub to_user_account: Option<String>,
    /// Source token account
    #[serde(default)]
    pub from_token_account: Option<String>,
    /// Destination token account
    #[serde(default)]
    pub to_token_account: Option<String>,
    /// Raw integer token amount. Divide by `10^decimals` for the UI amount
    pub raw_token_amount: u64,
    /// Mint decimals used to interpret `raw_token_amount`
    pub decimals: u8,
    /// Token standard when known (e.g. `"Fungible"`, `"UnknownStandard"`)
    pub token_standard: String,
    /// Token mint address
    pub mint: String,
}

/// Human-readable summary of a transaction or instruction
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedSummary {
    /// Summary type (e.g. `"swap"`, `"transfer"`, `"create_account"`)
    #[serde(rename = "type")]
    pub summary_type: String,
    /// Human-readable summary text
    pub description: String,
    /// Structured payload behind the summary, when available (e.g. swap metadata with
    /// `protocol`, `kind`, amounts, and mints). The shape varies by summary type and protocol
    #[serde(default)]
    pub parsed_data: Option<Value>,
}

/// A parsed top-level or inner (CPI) instruction
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedInstruction {
    /// Top-level instruction index
    pub instruction_index: u32,
    /// Inner instruction index within the top-level instruction, or `None` for top-level instructions
    #[serde(default)]
    pub inner_instruction_index: Option<u32>,
    /// Stack height, when available
    #[serde(default)]
    pub stack_height: Option<u32>,
    /// Program ID that executed the instruction
    pub program_id: String,
    /// Account pubkeys resolved from the instruction's account indexes
    #[serde(default)]
    pub raw_accounts: Vec<String>,
    /// Raw instruction data
    pub raw_data: String,
    /// Instruction-level summary, when available
    #[serde(default)]
    pub summary: Option<ParsedSummary>,
    /// Parser-known program name (e.g. `"jupiter"`, `"token_2022"`), when available
    #[serde(default)]
    pub program_name: Option<String>,
    /// Decoded instruction name (e.g. `"transfer_checked"`), when available
    #[serde(default)]
    pub instruction_name: Option<String>,
    /// Decoded args and accounts, when decoding succeeds
    #[serde(default)]
    pub decoded: Option<DecodedInstruction>,
}

/// Decoded instruction args and named accounts
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecodedInstruction {
    /// Decoded instruction arguments. Names and shapes depend on the instruction; u64 values arrive as strings
    #[serde(default)]
    pub args: Value,
    /// Decoded instruction accounts with parser-known names
    #[serde(default)]
    pub accounts: Vec<DecodedAccount>,
}

/// A named account used by a decoded instruction
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecodedAccount {
    /// Parser-known account name (snake_case)
    pub name: String,
    /// Account public key
    pub pubkey: String,
    /// Whether the account signed the transaction
    pub is_signer: bool,
    /// Whether the account was writable for the instruction
    pub is_writable: bool,
}

/// A custom program error decoded from a failed transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedDecodedError {
    /// Index of the failing instruction
    pub instruction_index: u32,
    /// Failing program ID
    pub program_id: String,
    /// Failing program name
    pub program_name: String,
    /// Program error code
    pub code: u32,
    /// Program error name
    pub name: String,
    /// Program error message, when available
    #[serde(default)]
    pub msg: Option<String>,
}
