//! Types for [Parsed Streams](https://www.helius.dev/docs/parsed-streams)
//!
//! Parsed Streams pushes decoded transactions over WebSocket. Notifications reuse the transfer,
//! summary, and decoded-instruction shapes of [Parsed Events](super::parsed_events), so stream
//! and API consumers can share processing code. Like Parsed Events, these types are not
//! glob-exported from [`crate::types`]. Import them by path:
//!
//! ```rust
//! use helius::types::parsed_streams::{ParsedTransactionFilter, ParsedTransactionNotification};
//! ```

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::parsed_events::{DecodedInstruction, ParsedNativeTransfer, ParsedSummary, ParsedTokenTransfer};

/// Which instructions a subscription matches
///
/// At least one of `programs` or `accounts.include` is required. The fields you set combine
/// with AND: an instruction must satisfy all of them to match. Take instruction and role names
/// from [`ParsedStreamsClient::describe_program`](crate::parsed_streams::ParsedStreamsClient::describe_program),
/// since a guessed name is valid but matches nothing. The server rejects unknown fields and
/// filters over its limits with a JSON-RPC error
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTransactionFilter {
    /// Program IDs (base58 addresses, not names). OR within the list. Max 10
    #[serde(skip_serializing_if = "Option::is_none")]
    pub programs: Option<Vec<String>>,
    /// Decoded instruction names, such as `"route"`. OR within the list. Max 50
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_names: Option<Vec<String>>,
    /// Account conditions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<ParsedTransactionAccountsFilter>,
    /// Include instructions from failed transactions. Defaults to `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_failed: Option<bool>,
    /// Let inner (CPI) instructions match. Defaults to `true`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_cpi: Option<bool>,
}

/// Account conditions in a [`ParsedTransactionFilter`]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTransactionAccountsFilter {
    /// Addresses an instruction must touch (any of). The program ID itself does not count. Max 100
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    /// Decoded role name to address. Every entry must hold, and role names match exactly. Max 20
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<BTreeMap<String, String>>,
}

/// Commitment level for Parsed Streams. Only `confirmed` is supported
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ParsedStreamsCommitment {
    Confirmed,
}

/// How much of a matching transaction each notification carries
///
/// For base58 instruction data only, use
/// [`ParsedStreamsClient::raw_transaction_subscribe`](crate::parsed_streams::ParsedStreamsClient::raw_transaction_subscribe),
/// which has its own notification type
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ParsedStreamDetails {
    /// The whole transaction and every instruction, plus `matched_indexes` (the default)
    Full,
    /// Only the instructions that matched, with no `matched_indexes`. Full payloads average
    /// roughly three times the size
    Matched,
}

/// Options for [`ParsedStreamsClient::parsed_transaction_subscribe`](crate::parsed_streams::ParsedStreamsClient::parsed_transaction_subscribe)
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTransactionSubscribeOptions {
    /// Commitment level. Defaults to `confirmed`, the only supported value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<ParsedStreamsCommitment>,
    /// Payload size. Defaults to `full`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ParsedStreamDetails>,
}

/// One matching transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTransactionNotification<V = ParsedTransactionValue> {
    /// Where the notification sits in the chain
    pub context: ParsedStreamContext,
    /// The transaction
    pub value: V,
}

/// Notification context
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedStreamContext {
    /// Slot of the transaction. Across a reconnect, it bounds the window that was not delivered
    pub slot: u64,
}

/// Notification payload for `full` and `matched` details
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTransactionValue {
    /// Transaction context
    pub transaction: ParsedStreamTransaction,
    /// `full`: every instruction in execution order. `matched`: only the hits. Use each
    /// instruction's indexes, not its position in this list, to locate it in the transaction
    #[serde(default)]
    pub instructions: Vec<ParsedStreamInstruction>,
    /// Indexes into `instructions` that matched the filter. `None` with `matched` details
    #[serde(default)]
    pub matched_indexes: Option<Vec<usize>>,
}

/// Whether a streamed transaction executed successfully
///
/// Unrecognized values deserialize as `Unknown`, so a status added to the API later does not
/// fail a notification
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParsedStreamTransactionStatus {
    /// The transaction executed successfully
    #[serde(rename = "ok")]
    Ok,
    /// The transaction executed and failed. Only delivered with `include_failed`
    #[serde(rename = "error")]
    Error,
    /// A value this SDK does not recognize. It serializes as `"unknown"`, so the original value
    /// is not preserved through a round trip
    #[serde(other, rename = "unknown")]
    Unknown,
}

/// Transaction context in a `full` or `matched` notification
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedStreamTransaction {
    /// Transaction signature
    pub signature: String,
    /// Slot containing the transaction
    pub slot: u64,
    /// Currently always `None`; do not build on it
    #[serde(default)]
    pub block_time: Option<i64>,
    /// Fee payer, always `account_keys[0]`
    pub fee_payer: String,
    /// Fee in lamports
    pub fee: u64,
    /// Every account key, including those loaded from lookup tables, in chain order
    #[serde(default)]
    pub account_keys: Vec<String>,
    /// Whether the transaction executed successfully
    pub status: ParsedStreamTransactionStatus,
    /// Structured transaction error when `status` is `Error`, e.g.
    /// `{"InstructionError": [2, {"Custom": 6001}]}`
    #[serde(default)]
    pub error: Option<Value>,
    /// The transaction's headline action, when recognized
    #[serde(default)]
    pub summary: Option<ParsedSummary>,
    /// SOL movements, same shape as Parsed Events
    #[serde(default)]
    pub native_transfers: Vec<ParsedNativeTransfer>,
    /// Token movements, same shape as Parsed Events
    #[serde(default)]
    pub token_transfers: Vec<ParsedTokenTransfer>,
}

/// An instruction in a `full` or `matched` notification
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParsedStreamInstruction {
    /// Top-level instruction this belongs to, from 0
    pub instruction_index: u32,
    /// Position among the top-level instruction's inner calls, or `None` for the top-level
    /// instruction itself
    #[serde(default)]
    pub inner_instruction_index: Option<u32>,
    /// Call depth, 1 for top level
    #[serde(default)]
    pub stack_height: Option<u32>,
    /// Program that executed the instruction
    pub program_id: String,
    /// Parser-known program name (e.g. `"jupiter"`), when available
    #[serde(default)]
    pub program_name: Option<String>,
    /// Decoded instruction name (e.g. `"route"`), when available
    #[serde(default)]
    pub instruction_name: Option<String>,
    /// Instruction-level summary, when recognized
    #[serde(default)]
    pub summary: Option<ParsedSummary>,
    /// Named args and accounts (snake_case, u64 values as strings), or `None` when undecoded
    #[serde(default)]
    pub decoded: Option<DecodedInstruction>,
    /// Base58 instruction data, present when `decoded` is `None`
    #[serde(default)]
    pub raw_data: Option<String>,
    /// Plain account list, present when `decoded` is `None`
    #[serde(default)]
    pub raw_accounts: Option<Vec<String>>,
}

/// Notification payload for
/// [`ParsedStreamsClient::raw_transaction_subscribe`](crate::parsed_streams::ParsedStreamsClient::raw_transaction_subscribe)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RawTransactionValue {
    /// Transaction context, without account keys or transfers
    pub transaction: RawStreamTransaction,
    /// The matching instructions
    #[serde(default)]
    pub instructions: Vec<RawStreamInstruction>,
}

/// Transaction context in a raw notification
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RawStreamTransaction {
    /// Transaction signature
    pub signature: String,
    /// Slot containing the transaction
    pub slot: u64,
    /// Currently always `None`; do not build on it
    #[serde(default)]
    pub block_time: Option<i64>,
    /// Fee payer
    pub fee_payer: String,
    /// Fee in lamports
    pub fee: u64,
    /// Whether the transaction executed successfully
    pub status: ParsedStreamTransactionStatus,
    /// Structured transaction error when `status` is `Error`
    #[serde(default)]
    pub error: Option<Value>,
    /// The transaction's headline action, when recognized
    #[serde(default)]
    pub summary: Option<ParsedSummary>,
}

/// A matching instruction in a raw notification
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RawStreamInstruction {
    /// Top-level instruction this belongs to, from 0
    pub instruction_index: u32,
    /// Position among the top-level instruction's inner calls, or `None` for the top-level
    /// instruction itself
    #[serde(default)]
    pub inner_instruction_index: Option<u32>,
    /// Call depth, 1 for top level
    #[serde(default)]
    pub stack_height: Option<u32>,
    /// Program that executed the instruction
    pub program_id: String,
    /// Instruction data as it appears on chain, base58
    pub data: String,
}

/// A program's names as the Parsed Streams matcher compares them, from
/// [`ParsedStreamsClient::describe_program`](crate::parsed_streams::ParsedStreamsClient::describe_program)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProgramDescription {
    /// Program address (base58). Check it when you looked the program up by name
    pub id: String,
    /// Catalog name
    pub name: String,
    /// Decoded instruction names, for `instruction_names`
    #[serde(default)]
    pub instructions: Vec<String>,
    /// Event names the parser recognizes
    #[serde(default)]
    pub events: Vec<String>,
    /// Account role names, for the keys of `accounts.roles`
    #[serde(default)]
    pub roles: Vec<String>,
}

/// Passed to the `on_reconnect` callback once subscriptions are restored on a new connection
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedStreamsReconnectInfo {
    /// WebSocket close code that started the outage (e.g. 1000 idle, 1001 deploy, 1006 network)
    pub close_code: u16,
    /// Highest `context.slot` delivered before the disconnect, or `None` if none was.
    /// Transactions after it that confirmed while disconnected were not delivered; backfill them
    /// from RPC if you need them
    pub last_slot: Option<u64>,
}
