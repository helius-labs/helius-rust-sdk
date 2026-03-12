use super::*;
use crate::utils::deserialize_str_to_number;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use solana_commitment_config::CommitmentLevel;

/// A human-readable, enhanced representation of a Solana transaction returned by the Helius
/// Enhanced Transactions API (`POST /v0/transactions`).
///
/// Unlike raw Solana transactions, enhanced transactions include decoded instruction data,
/// labeled account information, and contextual event details (NFT sales, swaps, etc.).
///
/// See also: [`ParseTransactionsRequest`], [`ParsedTransactionHistoryRequest`]
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnhancedTransaction {
    /// Per-account balance snapshots showing native SOL and token balance changes
    pub account_data: Vec<AccountData>,
    /// A human-readable summary of the transaction (e.g., "X transferred 1 SOL to Y")
    pub description: String,
    /// The classified transaction type (e.g., `NftSale`, `Transfer`, `Swap`)
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    /// The protocol or marketplace that originated the transaction
    pub source: Source,
    /// The transaction fee paid in lamports
    pub fee: i32,
    /// The public key of the account that paid the transaction fee
    pub fee_payer: String,
    /// The transaction signature (base-58 encoded)
    pub signature: String,
    /// The slot in which the transaction was confirmed
    pub slot: i32,
    /// Native SOL transfers that occurred within the transaction, if any
    pub native_transfers: Option<Vec<NativeTransfer>>,
    /// SPL token transfers that occurred within the transaction, if any
    pub token_transfers: Option<Vec<TokenTransfer>>,
    /// The error returned by the Solana runtime, if the transaction failed
    pub transaction_error: Option<TransactionError>,
    /// The top-level instructions executed in the transaction, including inner instructions
    pub instructions: Vec<Instruction>,
    /// Structured event data extracted from the transaction (NFT events, swaps, etc.)
    pub events: TransactionEvent,
    /// The estimated block production time as a Unix timestamp (seconds since epoch)
    pub timestamp: u64,
}

/// Balance information for a single account involved in an [`EnhancedTransaction`].
///
/// Captures the account's native SOL balance and any SPL token balance changes.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountData {
    /// The public key of the account (base-58 encoded)
    pub account: String,
    /// The account's native SOL balance in lamports after the transaction
    pub native_token_balance: Option<Number>,
    /// SPL token balance changes for this account, if any
    pub token_balance_changes: Option<Vec<TokenBalanceChange>>,
}

/// A change in SPL token balance for a specific token account within a transaction.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalanceChange {
    /// The owner (wallet) of the token account
    pub user_account: String,
    /// The associated token account address
    pub token_account: String,
    /// The raw (non-UI) token amount delta with decimal information
    pub raw_token_amount: RawTokenAmount,
    /// The mint address of the token
    pub mint: String,
}

/// A raw token amount with its decimal precision, before any UI formatting.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RawTokenAmount {
    /// The token amount as a string to avoid floating-point precision issues
    pub token_amount: String,
    /// The number of decimals for the token's mint
    pub decimals: Number,
}

/// A native SOL transfer between two accounts within a transaction.
#[derive(Serialize, Deserialize, Debug)]
pub struct NativeTransfer {
    /// The sender and receiver wallet addresses
    #[serde(rename = "userAccounts", flatten)]
    pub user_accounts: TransferUserAccounts,
    /// The transfer amount in lamports
    pub amount: Number,
}

/// The sender and receiver wallet addresses for a transfer.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferUserAccounts {
    /// The public key of the sending wallet, if identifiable
    pub from_user_account: Option<String>,
    /// The public key of the receiving wallet, if identifiable
    pub to_user_account: Option<String>,
}

/// An SPL token transfer between accounts within a transaction.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenTransfer {
    /// The sender and receiver wallet addresses
    #[serde(flatten)]
    pub user_accounts: TransferUserAccounts,
    /// The source associated token account address, if identifiable
    pub from_token_account: Option<String>,
    /// The destination associated token account address, if identifiable
    pub to_token_account: Option<String>,
    /// The amount of tokens transferred (in the token's smallest unit)
    pub token_amount: Number,
    /// The token standard (e.g., `Fungible`, `NonFungible`)
    pub token_standard: TokenStandard,
    /// The mint address of the transferred token
    pub mint: String,
}

/// An error returned by the Solana runtime when a transaction fails.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransactionError {
    /// The instruction-level error details, typically a `[index, { "Custom": code }]` tuple
    #[serde(rename = "InstructionError")]
    pub instruction_error: Option<serde_json::Value>,
}

/// A top-level instruction executed in the transaction.
#[derive(Serialize, Deserialize, Debug)]
pub struct Instruction {
    /// The account public keys referenced by this instruction
    pub accounts: Vec<String>,
    /// The base-58 encoded instruction data
    pub data: String,
    /// The program that executed this instruction
    #[serde(rename = "programId")]
    pub program_id: String,
    /// Cross-program invocations triggered by this instruction
    #[serde(rename = "innerInstructions")]
    pub inner_instructions: Vec<InnerInstruction>,
}

/// A cross-program invocation (CPI) triggered by a top-level [`Instruction`].
#[derive(Serialize, Deserialize, Debug)]
pub struct InnerInstruction {
    /// The account public keys referenced by this inner instruction
    pub accounts: Vec<String>,
    /// The base-58 encoded instruction data
    pub data: String,
    /// The program that executed this inner instruction
    #[serde(rename = "programId")]
    pub program_id: String,
}

/// Structured event data extracted from an [`EnhancedTransaction`].
///
/// Events are high-level interpretations of what happened in a transaction, such as
/// NFT sales, token swaps, compressed NFT operations, or authority changes.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TransactionEvent {
    /// An NFT marketplace event (sale, listing, bid, etc.), if detected
    pub nft: Option<NftEvent>,
    /// A token swap event (DEX trade), if detected
    pub swap: Option<SwapEvent>,
    /// Compressed NFT events (mint, transfer, burn, etc.), if detected
    pub compressed: Option<Vec<CompressedNftEvent>>,
    /// Authority change events, if detected
    #[serde(rename = "setAuthority")]
    pub set_authority: Option<Vec<Authority>>,
}

/// An NFT marketplace event (sale, listing, bid, etc.) extracted from a transaction.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NftEvent {
    /// The seller's wallet address
    pub seller: String,
    /// The buyer's wallet address
    pub buyer: String,
    /// Unix timestamp of the event
    pub timestamp: Number,
    /// The sale price in lamports
    pub amount: Number,
    /// The marketplace fee in lamports
    pub fee: Number,
    /// The transaction signature (base-58 encoded)
    pub signature: String,
    /// The marketplace or protocol that facilitated the event
    pub source: Source,
    /// The type of NFT event (e.g., `NftSale`, `NftListing`, `NftBid`)
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    /// The context of the sale (e.g., `Auction`, `InstantSale`, `Offer`)
    pub sale_type: TransactionContext,
    /// The NFTs involved in the event
    pub nfts: Vec<Token>,
}

/// A token swap (DEX trade) event extracted from a transaction.
///
/// Represents the aggregate inputs, outputs, and fees of a swap, along with
/// the individual route hops in [`inner_swaps`](SwapEvent::inner_swaps).
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SwapEvent {
    /// Native SOL sent as input to the swap, if any
    pub native_input: Option<NativeBalanceChange>,
    /// Native SOL received as output from the swap, if any
    pub native_output: Option<NativeBalanceChange>,
    /// SPL tokens sent as input to the swap
    pub token_inputs: Vec<TokenBalanceChange>,
    /// SPL tokens received as output from the swap
    pub token_outputs: Vec<TokenBalanceChange>,
    /// SPL token fees charged during the swap
    pub token_fees: Vec<TokenBalanceChange>,
    /// Native SOL fees charged during the swap
    pub native_fees: Vec<NativeBalanceChange>,
    /// Individual swap hops when the trade is routed through multiple pools
    pub inner_swaps: Vec<TokenSwap>,
}

/// An NFT or token identifier with its standard classification.
#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    /// The mint address of the token
    pub mint: String,
    /// The token standard (e.g., `NonFungible`, `Fungible`, `ProgrammableNonFungible`)
    #[serde(rename = "tokenStandard")]
    pub token_standard: TokenStandard,
}

/// A single swap hop in a multi-hop trade route (e.g., one leg of a Jupiter aggregator swap).
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenSwap {
    /// Native SOL input to this swap hop, if any
    pub native_input: Option<NativeTransfer>,
    /// Native SOL output from this swap hop, if any
    pub native_output: Option<NativeTransfer>,
    /// SPL tokens input to this swap hop
    pub token_inputs: Vec<TokenTransfer>,
    /// SPL tokens output from this swap hop
    pub token_outputs: Vec<TokenTransfer>,
    /// SPL token fees for this swap hop
    pub token_fees: Vec<TokenTransfer>,
    /// Native SOL fees for this swap hop
    pub native_fees: Vec<NativeTransfer>,
    /// The DEX program that executed this swap hop
    pub program_info: ProgramInfo,
}

/// A change in native SOL balance for a single account.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NativeBalanceChange {
    /// The public key of the account whose balance changed
    pub account: String,
    /// The balance change amount in lamports (positive = received, negative = sent)
    #[serde(deserialize_with = "deserialize_str_to_number")]
    pub amount: Number,
}

/// Metadata about the program that executed a swap instruction.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProgramInfo {
    /// The protocol source identifier
    pub source: Source,
    /// The program account address
    pub account: String,
    /// The human-readable program name (e.g., `RaydiumLiquidityPoolV4`, `OrcaWhirlpools`)
    #[serde(rename = "programName")]
    pub program_name: ProgramName,
    /// The name of the instruction that was invoked
    #[serde(rename = "instructionName")]
    pub instruction_name: String,
}

/// A set-authority event where account ownership or delegation was changed.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Authority {
    /// The account whose authority was changed
    pub account: String,
    /// The previous authority public key
    pub from: String,
    /// The new authority public key
    pub to: String,
    /// The index of the top-level instruction that triggered this change
    #[serde(rename = "instructionIndex")]
    pub instruction_index: Option<i32>,
    /// The index of the inner instruction, if the change was triggered by a CPI
    #[serde(rename = "innerInstructionIndex")]
    pub inner_instruction_index: Option<i32>,
}

/// A compressed NFT (cNFT) event extracted from a Bubblegum transaction.
///
/// Compressed NFTs are stored in concurrent Merkle trees on-chain. This event captures
/// operations like mints, transfers, burns, and metadata updates for cNFTs.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CompressedNftEvent {
    /// The type of compressed NFT operation (e.g., `CompressedNftMint`, `CompressedNftTransfer`)
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    /// The address of the Merkle tree account that stores the cNFT
    pub tree_id: String,
    /// The leaf index within the Merkle tree
    pub leaf_index: Option<i32>,
    /// The sequence number for ordering concurrent updates
    pub seq: Option<i32>,
    /// The derived asset ID of the compressed NFT
    pub asset_id: Option<String>,
    /// The index of the top-level instruction
    pub instruction_index: Option<i32>,
    /// The index of the inner instruction, if applicable
    pub inner_instruction_index: Option<i32>,
    /// The new owner after the operation
    pub new_leaf_owner: Option<String>,
    /// The previous owner before the operation
    pub old_leaf_owner: Option<String>,
    /// The new delegate after the operation
    pub new_leaf_delegate: Option<String>,
    /// The previous delegate before the operation
    pub old_leaf_delegate: Option<Value>,
    /// The delegate authority of the Merkle tree
    pub tree_delegate: Option<String>,
    /// The NFT metadata (name, symbol, URI, etc.), present for mint events
    pub metadata: Option<Metadata>,
    /// Arguments for metadata update operations
    pub update_args: Option<Value>,
}

/// Request body for the `POST /v0/transactions` endpoint.
///
/// Converts raw Solana transaction signatures into enhanced, human-readable formats.
/// A maximum of 100 transactions can be parsed per request.
#[derive(Serialize, Deserialize, Debug)]
pub struct ParseTransactionsRequest {
    /// A list of transaction signatures (base-58 encoded) to parse
    pub transactions: Vec<String>,
}

/// Request parameters for the `GET /v0/addresses/{address}/transactions` endpoint.
///
/// Retrieves an enhanced transaction history for a given address with optional filtering
/// by transaction type, source, and pagination controls.
#[derive(Serialize, Deserialize, Debug)]
pub struct ParsedTransactionHistoryRequest {
    /// The Solana address to fetch transaction history for
    pub address: String,
    /// Start searching backwards from this transaction signature
    pub before: Option<String>,
    /// Search until this transaction signature
    pub until: Option<String>,
    /// The commitment level — only `finalized` and `confirmed` are supported
    pub commitment: Option<CommitmentLevel>,
    /// Filter by transaction source (e.g., marketplace or protocol)
    pub source: Option<Source>,
    /// Filter by transaction type (e.g., `NftSale`, `Transfer`)
    #[serde(rename = "type")]
    pub transaction_type: Option<TransactionType>,
    /// Number of transactions to retrieve (between 1 and 100)
    pub limit: Option<u64>,
}

/// We have a limit of 100 transactions per call, so this helps split the signatures into different chunks
impl ParseTransactionsRequest {
    pub fn from_slice(signatures: &[String]) -> Vec<Self> {
        signatures
            .chunks(100)
            .map(|chunk| Self {
                transactions: chunk.to_vec(),
            })
            .collect()
    }
}
