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

/// Re-exported from [`crate::types`] for the `token_accounts` filter on transaction history
/// and scans, which takes the same values as `getTransactionsForAddress`
pub use super::TokenAccountsFilter;

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

/// Which execution results a transaction history page or scan includes
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParsedEventsStatusFilter {
    /// Successful transactions only (the default)
    Succeeded,
    /// Failed transactions only
    Failed,
    /// Both
    Any,
}

/// Encoding of the raw Solana transaction returned next to each parsed one, as `raw_transaction`
/// in the `getTransaction` shape
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RawTransactionEncoding {
    /// No raw transaction (the default)
    None,
    /// JSON
    Json,
    /// JSON with parsed instructions and accounts
    JsonParsed,
    /// Base64-encoded wire bytes
    Base64,
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
    /// Also return each original Solana transaction as `raw_transaction`, in this encoding.
    /// Defaults to `none`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_transaction: Option<RawTransactionEncoding>,
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
    /// Execution result to include. Defaults to `succeeded`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ParsedEventsStatusFilter>,
    /// Transactions that reference only a token account owned by `address`: `None` (the
    /// default) leaves them out, `BalanceChanged` includes those that change its balance, and
    /// `All` includes all of them
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_accounts: Option<TokenAccountsFilter>,
    /// Minimum slot at which the request can be evaluated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_context_slot: Option<u64>,
    /// Also return each raw Solana transaction as `raw_transaction`, in this encoding. Defaults
    /// to `none`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_transaction: Option<RawTransactionEncoding>,
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

/// Request for [`ParsedEvents::scan`](crate::parsed_events::ParsedEvents::scan),
/// [`scan_signatures`](crate::parsed_events::ParsedEvents::scan_signatures), and
/// [`scan_count`](crate::parsed_events::ParsedEvents::scan_count) (`POST /v1/parsed-events/scan`)
///
/// Walks one address's history and returns only the transactions that match `criteria`.
/// One request scans at most `budget.max_scanned_transactions` transactions, so a scan for rare
/// matches can return an empty page with a `pagination_token`; keep calling while one is returned
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    /// Address whose history is scanned. Scan the narrowest address you can: every transaction
    /// in its history counts toward the budget
    pub address: String,
    /// What a transaction must match (serialized as `match`). Set `programs`, `mints`, or both
    #[serde(rename = "match")]
    pub criteria: ScanMatch,
    /// Most matches to return, from 1 to 100. Defaults to 100. Applies to
    /// [`scan`](crate::parsed_events::ParsedEvents::scan) only: the signatures and count shapes
    /// return every match in the scanned range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Limits on how much history one request scans
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<ScanBudget>,
    /// Scan transactions before this signature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_signature: Option<String>,
    /// Scan transactions after this signature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_signature: Option<String>,
    /// Cursor returned by a previous scan. An empty string is treated as no cursor, so a
    /// response's `pagination_token` can be passed straight back
    #[serde(skip_serializing_if = "is_none_or_empty")]
    pub pagination_token: Option<String>,
    /// Order in which history is scanned and matches are returned. Defaults to `desc`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ParsedEventsSortOrder>,
    /// Commitment level for the history fetch. Defaults to `confirmed`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<ParsedEventsCommitment>,
    /// Execution result to include. Defaults to `succeeded`. Excluded transactions do not count
    /// toward the budget
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ParsedEventsStatusFilter>,
    /// Transactions that reference only a token account owned by `address`: `None` (the
    /// default) leaves them out, `BalanceChanged` includes those that change its balance, and
    /// `All` includes all of them
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_accounts: Option<TokenAccountsFilter>,
    /// Minimum slot at which the request can be evaluated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_context_slot: Option<u64>,
    /// Also return each raw Solana transaction as `raw_transaction`, in this encoding. Defaults
    /// to `none`. Applies to [`scan`](crate::parsed_events::ParsedEvents::scan) only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_transaction: Option<RawTransactionEncoding>,
    /// Slot bounds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<ComparisonBounds>,
    /// Block-time bounds in whole Unix seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<ComparisonBounds>,
}

/// What a scanned transaction must match. Set `programs`, `mints`, or both; with both, a
/// transaction must match both. Setting neither is rejected by the server
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanMatch {
    /// Match transactions with an instruction of any of these programs, top-level or inner
    /// (CPI). Max 20, each program at most once
    #[serde(skip_serializing_if = "Option::is_none")]
    pub programs: Option<Vec<ScanProgramMatch>>,
    /// Match transactions with one of these mints in their pre or post token balances. Max 20
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mints: Option<Vec<String>>,
}

/// Matches transactions with an instruction of `program_id`
///
/// Narrow the match with `instructions`, `discriminators`, or both: an instruction matches when
/// it is one of the named instructions or its data starts with one of the discriminators. With
/// neither, any instruction of the program matches
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgramMatch {
    /// Program address
    pub program_id: String,
    /// Instruction names from any IDL version of the program, as the IDL spells them or in
    /// snake_case. A name not in the program's IDLs is rejected. Max 50
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Vec<String>>,
    /// Instruction data prefixes, each 1 to 8 hex-encoded bytes with an optional `0x` prefix,
    /// for programs without an IDL. Max 50
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminators: Option<Vec<String>>,
}

/// Limits on how much history one scan request covers
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanBudget {
    /// Most transactions to scan in this request, from 1 to 10,000. Defaults to 10,000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_scanned_transactions: Option<u32>,
}

/// A page of scan matches, parsed, from [`ParsedEvents::scan`](crate::parsed_events::ParsedEvents::scan)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanResponse {
    /// Parsed matches, each with the same shape as a
    /// [`parse_transactions`](crate::parsed_events::ParsedEvents::parse_transactions) result.
    /// Can be empty while `pagination_token` is set
    #[serde(default)]
    pub data: Vec<ParsedTransactionResult>,
    /// Cursor where the next scan continues. `None` once the scan reached the end of the
    /// history or window
    #[serde(default)]
    pub pagination_token: Option<String>,
    /// How far this request scanned and why it stopped
    #[serde(default)]
    pub scan: ScanSummary,
}

/// A page of scan matches as signatures, from
/// [`ParsedEvents::scan_signatures`](crate::parsed_events::ParsedEvents::scan_signatures)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanSignaturesResponse {
    /// Every match in the scanned range. Can be empty while `pagination_token` is set
    #[serde(default)]
    pub data: Vec<ScanSignature>,
    /// Cursor where the next scan continues. `None` once the scan reached the end of the
    /// history or window
    #[serde(default)]
    pub pagination_token: Option<String>,
    /// How far this request scanned and why it stopped
    #[serde(default)]
    pub scan: ScanSummary,
}

/// One scan match, without the parsed transaction
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScanSignature {
    /// Transaction signature
    pub signature: String,
    /// Slot containing the transaction
    pub slot: u64,
    /// Position of the transaction within its slot
    pub transaction_index: u32,
    /// Unix timestamp (seconds) of the block, when available
    #[serde(default)]
    pub block_time: Option<i64>,
}

/// The number of scan matches in a scanned range, from
/// [`ParsedEvents::scan_count`](crate::parsed_events::ParsedEvents::scan_count). The count is
/// `scan.matched_transactions`
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanCountResponse {
    /// Cursor where the next scan continues. `None` once the scan reached the end of the
    /// history or window. Add up `scan.matched_transactions` across pages for a total
    #[serde(default)]
    pub pagination_token: Option<String>,
    /// How far this request scanned, why it stopped, and how many transactions matched
    #[serde(default)]
    pub scan: ScanSummary,
}

/// How far a scan request got and why it stopped
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    /// Why the scan stopped
    pub stopped_on: ScanStopReason,
    /// Transactions scanned in this request
    #[serde(default)]
    pub scanned_transactions: u64,
    /// Matches in this request. For [`scan_count`](crate::parsed_events::ParsedEvents::scan_count),
    /// this is the count
    #[serde(default)]
    pub matched_transactions: u64,
    /// The last transaction scanned, or `None` when nothing was scanned
    #[serde(default)]
    pub scanned_through: Option<ScannedThrough>,
    /// Credits charged: 10 per request plus 1 per history page scanned (up to 1,000
    /// transactions each)
    #[serde(default)]
    pub credits_charged: u64,
}

/// Why a scan request stopped
///
/// Unrecognized values deserialize as `Unknown`, so a reason added to the API later does not
/// fail the response
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum ScanStopReason {
    /// Reached the end of the history or the requested window. No `pagination_token`
    WindowEnd,
    /// Returned `limit` matches. The token resumes right after the last match. Only for
    /// [`scan`](crate::parsed_events::ParsedEvents::scan)
    Limit,
    /// Scanned `budget.max_scanned_transactions` transactions. The token resumes at the next
    /// history page
    Budget,
    /// Reached the request time limit. The token resumes at the next history page
    Deadline,
    /// A value this SDK does not recognize, or a response without a summary. It serializes as
    /// `"unknown"`, so the original value is not preserved through a round trip
    #[default]
    #[serde(other)]
    Unknown,
}

/// Position of the last transaction a scan request covered
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScannedThrough {
    /// Slot of the last transaction scanned
    pub slot: u64,
    /// Block time of the last transaction scanned, in Unix seconds, when available
    #[serde(default)]
    pub block_time: Option<i64>,
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
    /// The original Solana transaction in the `getTransaction` shape, encoded as the request's
    /// `raw_transaction` asked. Present only when that is set to something other than `None`
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
