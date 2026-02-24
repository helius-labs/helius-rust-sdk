//! Types for the ZK Compression API
//!
//! ZK Compression is a protocol on Solana that reduces on-chain storage costs by compressing
//! account state into concurrent Merkle trees. Instead of storing full account data in
//! individual Solana accounts, compressed state is verified via zero-knowledge proofs and
//! stored in shared tree structures, dramatically lowering rent costs while preserving the
//! same security guarantees.
//!
//! This module defines the request and response types used by the ZK Compression RPC methods.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::inner::DataSlice;

/// Slot context returned by ZK Compression RPC responses.
///
/// Indicates which Solana slot the response data corresponds to.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct ZkCompressedContext {
    /// The Solana slot at the time the request was processed.
    pub slot: u64,
}

/// On-chain data stored in a compressed account.
///
/// Contains the raw account data (base64-encoded), its hash for verification,
/// and a discriminator used to identify the account type within a program.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompressedAccountData {
    /// The raw account data, encoded as a base64 string.
    pub data: String,
    /// Cryptographic hash of the account data for verification.
    pub data_hash: String,
    /// Discriminator value used to identify the account type within a program.
    pub discriminator: u64,
}

/// A compressed account stored in a concurrent Merkle tree on Solana.
///
/// Compressed accounts hold the same information as regular Solana accounts but are stored
/// in shared Merkle trees, reducing rent costs by up to 1000x. The account data is verified
/// via zero-knowledge proofs rather than being stored in individual on-chain accounts.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompressedAccount {
    /// The Solana public key of this compressed account, if assigned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// The on-chain data stored in this compressed account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<CompressedAccountData>,
    /// Cryptographic hash of the compressed account for Merkle tree verification.
    pub hash: String,
    /// SOL balance held by this compressed account, in lamports.
    pub lamports: u64,
    /// Position index in the Merkle tree where this compressed account is stored.
    pub leaf_index: u64,
    /// The Solana program that owns this compressed account.
    pub owner: String,
    /// Sequence number for tracking updates to this compressed account.
    pub seq: u64,
    /// Solana slot when this compressed account was first created.
    pub slot_created: u64,
    /// Address of the Merkle tree where this compressed account is stored.
    pub tree: String,
}

/// A memory comparison filter for matching compressed account data.
///
/// Compares a slice of bytes at a given offset in the account data against
/// the provided base58-encoded byte string.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Memcmp {
    /// The byte offset in the account data to start comparing from.
    pub offset: u64,
    /// The base58-encoded bytes to compare against.
    pub bytes: String,
}

/// A filter for querying compressed accounts.
///
/// Currently supports memory comparison (`memcmp`) filters for matching
/// specific byte patterns in compressed account data.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FilterSelector {
    /// A memory comparison filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memcmp: Option<Memcmp>,
}

/// A paginated list of compressed accounts.
///
/// Contains the list of matching accounts and an optional cursor for fetching
/// the next page of results.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PaginatedAccountList {
    /// The compressed accounts in this page.
    pub items: Vec<CompressedAccount>,
    /// Cursor for fetching the next page. `None` when there are no more results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Request parameters for [`getCompressedAccount`](crate::Helius::get_compressed_account).
///
/// Provide either `address` or `hash` to identify the compressed account.
/// At least one must be specified.
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressedAccountRequest;
///
/// // Query by hash
/// let by_hash = GetCompressedAccountRequest {
///     hash: Some("11111111111111111111111111111111".to_string()),
///     ..Default::default()
/// };
///
/// // Query by address
/// let by_address = GetCompressedAccountRequest {
///     address: Some("11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedAccountRequest {
    /// The Solana public key (base58) of the compressed account to retrieve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// The data hash (base58) of the compressed account, used when the address is not available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// Response from [`getCompressedAccount`](crate::Helius::get_compressed_account).
///
/// Contains slot context and an optional compressed account value. The value is `None`
/// if no account matches the provided address or hash.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedAccountResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The compressed account data, or `None` if no matching account was found.
    pub value: Option<CompressedAccount>,
}

/// A Merkle proof for verifying a compressed account's existence and data integrity.
///
/// Contains the full proof path from the leaf (the compressed account) to the root
/// of the Merkle tree, along with tree metadata needed for on-chain verification.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MerkleProofWithContext {
    /// The hash of the compressed account being verified.
    pub hash: String,
    /// The position index of this compressed account in the Merkle tree.
    pub leaf_index: u32,
    /// The address of the Merkle tree where this compressed account is stored.
    pub merkle_tree: String,
    /// The hashes forming the proof path from the leaf to the root.
    pub proof: Vec<String>,
    /// The root hash of the Merkle tree containing this compressed account.
    pub root: String,
    /// The sequence number of the root for version tracking.
    pub root_seq: u64,
}

/// Request parameters for [`getCompressedAccountProof`](crate::Helius::get_compressed_account_proof).
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressedAccountProofRequest;
///
/// let request = GetCompressedAccountProofRequest {
///     hash: "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetCompressedAccountProofRequest {
    /// The hash (base58) of the compressed account to retrieve the proof for.
    pub hash: String,
}

/// Response from [`getCompressedAccountProof`](crate::Helius::get_compressed_account_proof).
///
/// Contains slot context and the Merkle proof for the requested compressed account.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedAccountProofResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The Merkle proof with tree context for the requested compressed account.
    pub value: MerkleProofWithContext,
}

/// Request parameters for [`getCompressedAccountsByOwner`](crate::Helius::get_compressed_accounts_by_owner).
///
/// Retrieves all compressed accounts owned by a given Solana public key, with optional
/// pagination, filtering, and data slicing.
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressedAccountsByOwnerRequest;
///
/// let request = GetCompressedAccountsByOwnerRequest {
///     owner: "11111114d3RrygbPdAtMuFnDmzsN8T5fYKVQ7FVr7".to_string(),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetCompressedAccountsByOwnerRequest {
    /// The Solana public key (base58) of the account owner.
    pub owner: String,
    /// Pagination cursor from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Return only a sub-range of each account's data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_slice: Option<DataSlice>,
    /// Filters to narrow the set of returned accounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<FilterSelector>>,
    /// Maximum number of accounts to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

/// Response from [`getCompressedAccountsByOwner`](crate::Helius::get_compressed_accounts_by_owner).
///
/// Contains slot context and a paginated list of compressed accounts owned by the
/// requested address.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedAccountsByOwnerResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The paginated list of compressed accounts.
    pub value: PaginatedAccountList,
}

/// Request parameters for [`getCompressedBalanceByOwner`](crate::Helius::get_compressed_balance_by_owner).
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressedBalanceByOwnerRequest;
///
/// let request = GetCompressedBalanceByOwnerRequest {
///     owner: "11111113R2cuenjG5nFubqX9Wzuukdin2YfGQVzu5".to_string(),
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetCompressedBalanceByOwnerRequest {
    /// The Solana public key (base58) of the account owner.
    pub owner: String,
}

/// A token holder's balance for a compressed mint.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OwnerBalance {
    /// The owner's Solana public key (base58).
    pub owner: String,
    /// The token balance held by this owner.
    pub balance: u64,
}

/// A paginated list of token holders and their balances.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OwnerBalanceList {
    /// The token holder balances in this page.
    pub items: Vec<OwnerBalance>,
    /// Cursor for fetching the next page. `None` when there are no more results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Request parameters for [`getCompressedMintTokenHolders`](crate::Helius::get_compressed_mint_token_holders).
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressedMintTokenHoldersRequest;
///
/// let request = GetCompressedMintTokenHoldersRequest {
///     mint: "111111152P2r5yt6odmBLPsFCLBrFisJ3aS7LqLAT".to_string(),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedMintTokenHoldersRequest {
    /// The compressed token mint address (base58).
    pub mint: String,
    /// Pagination cursor from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Maximum number of holders to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

/// Response from [`getCompressedMintTokenHolders`](crate::Helius::get_compressed_mint_token_holders).
///
/// Contains slot context and a paginated list of token holders with their balances.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedMintTokenHoldersResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The paginated list of token holders and their balances.
    pub value: OwnerBalanceList,
}

/// The state of a compressed token account.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AccountState {
    /// The token account is active and can send/receive tokens.
    #[default]
    Initialized,
    /// The token account is frozen and cannot send tokens until thawed.
    Frozen,
}

/// Token-specific data for a compressed token account.
///
/// Contains the mint, owner, balance, delegation, and optional TLV (tag-length-value)
/// extension data for a compressed SPL token account.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenData {
    /// The mint address (base58) of the token.
    pub mint: String,
    /// The owner address (base58) of the token account.
    pub owner: String,
    /// The token balance.
    pub amount: u64,
    /// The current state of the token account.
    pub state: AccountState,
    /// The delegate address (base58) authorized to transfer tokens, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegate: Option<String>,
    /// Optional TLV (tag-length-value) extension data, encoded as a base64 string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tlv: Option<String>,
}

/// A compressed token account combining base account data with token-specific data.
///
/// Pairs the underlying [`CompressedAccount`] (which holds the Merkle tree position,
/// lamports, and owner) with [`TokenData`] containing the SPL token fields.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompressedTokenAccount {
    /// The underlying compressed account data.
    pub account: CompressedAccount,
    /// The token-specific data (mint, owner, amount, state, delegate, tlv).
    pub token_data: TokenData,
}

/// A paginated list of compressed token accounts.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TokenAccountList {
    /// The compressed token accounts in this page.
    pub items: Vec<CompressedTokenAccount>,
    /// Cursor for fetching the next page. `None` when there are no more results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Request parameters for [`getCompressedTokenAccountsByDelegate`](crate::Helius::get_compressed_token_accounts_by_delegate).
///
/// Retrieves all compressed token accounts delegated to a given address, with optional
/// pagination and mint filtering.
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressedTokenAccountsByDelegateRequest;
///
/// let request = GetCompressedTokenAccountsByDelegateRequest {
///     delegate: "11111116EPqoQskEM2Pddp8KTL9JdYEBZMGF3aq7V".to_string(),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetCompressedTokenAccountsByDelegateRequest {
    /// The delegate address (base58) to query token accounts for.
    pub delegate: String,
    /// Pagination cursor from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Maximum number of token accounts to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    /// Filter results to a specific token mint address (base58).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,
}

/// Response from [`getCompressedTokenAccountsByDelegate`](crate::Helius::get_compressed_token_accounts_by_delegate).
///
/// Contains slot context and a paginated list of compressed token accounts
/// delegated to the requested address.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedTokenAccountsByDelegateResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The paginated list of compressed token accounts.
    pub value: TokenAccountList,
}

/// Request parameters for [`getCompressedTokenAccountsByOwner`](crate::Helius::get_compressed_token_accounts_by_owner).
///
/// Retrieves all compressed token accounts owned by a given Solana wallet address,
/// with optional pagination and mint filtering.
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressedTokenAccountsByOwnerRequest;
///
/// let request = GetCompressedTokenAccountsByOwnerRequest {
///     owner: "11111116EPqoQskEM2Pddp8KTL9JdYEBZMGF3aq7V".to_string(),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetCompressedTokenAccountsByOwnerRequest {
    /// The Solana wallet address (base58) that owns the compressed token accounts.
    pub owner: String,
    /// Pagination cursor from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Maximum number of token accounts to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    /// Filter results to a specific token mint address (base58).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,
}

/// Response from [`getCompressedTokenAccountsByOwner`](crate::Helius::get_compressed_token_accounts_by_owner).
///
/// Contains slot context and a paginated list of compressed token accounts
/// owned by the requested address.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedTokenAccountsByOwnerResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The paginated list of compressed token accounts.
    pub value: TokenAccountList,
}

/// A token mint and its aggregate balance for a wallet owner.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CompressedTokenBalance {
    /// The token mint address (base58).
    pub mint: String,
    /// The aggregate token balance across all compressed token accounts for this mint.
    pub balance: u64,
}

/// A paginated list of token balances grouped by mint.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TokenBalanceList {
    /// The token balances in this page.
    pub token_balances: Vec<CompressedTokenBalance>,
    /// Cursor for fetching the next page. `None` when there are no more results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Request parameters for [`getCompressedTokenBalancesByOwner`](crate::Helius::get_compressed_token_balances_by_owner).
///
/// Retrieves aggregate compressed token balances for a wallet, grouped by mint,
/// with optional pagination and mint filtering.
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressedTokenBalancesByOwnerRequest;
///
/// let request = GetCompressedTokenBalancesByOwnerRequest {
///     owner: "11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR".to_string(),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetCompressedTokenBalancesByOwnerRequest {
    /// The Solana wallet address (base58) to retrieve token balances for.
    pub owner: String,
    /// Pagination cursor from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Maximum number of token balances to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    /// Filter results to a specific token mint address (base58).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,
}

/// Response from [`getCompressedTokenBalancesByOwner`](crate::Helius::get_compressed_token_balances_by_owner).
///
/// Contains slot context and a paginated list of token balances grouped by mint.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedTokenBalancesByOwnerResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The paginated list of token balances.
    pub value: TokenBalanceList,
}

/// A paginated list of token balances (V2 format).
///
/// Unlike [`TokenBalanceList`] which uses `token_balances`, this V2 variant uses `items`
/// as the field name, consistent with other paginated list types.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TokenBalanceListV2 {
    /// The token balances in this page.
    pub items: Vec<CompressedTokenBalance>,
    /// Cursor for fetching the next page. `None` when there are no more results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Response from [`getCompressedTokenBalancesByOwnerV2`](crate::Helius::get_compressed_token_balances_by_owner_v2).
///
/// Contains slot context and a paginated list of token balances grouped by mint.
/// This V2 response uses the `items` field name instead of `token_balances`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedTokenBalancesByOwnerV2Response {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The paginated list of token balances.
    pub value: TokenBalanceListV2,
}

/// Information about a transaction signature related to a compressed account.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SignatureInfo {
    /// The Solana transaction signature (base58).
    pub signature: String,
    /// The Solana slot in which this transaction was processed.
    pub slot: u64,
    /// Unix timestamp (seconds) of the block containing this transaction.
    pub block_time: i64,
}

/// A list of transaction signatures related to a compressed account.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignatureInfoList {
    /// The transaction signatures.
    pub items: Vec<SignatureInfo>,
}

/// Request parameters for [`getCompressionSignaturesForAccount`](crate::Helius::get_compression_signatures_for_account).
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressionSignaturesForAccountRequest;
///
/// let request = GetCompressionSignaturesForAccountRequest {
///     hash: "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetCompressionSignaturesForAccountRequest {
    /// The hash (base58) of the compressed account to retrieve signatures for.
    pub hash: String,
}

/// Response from [`getCompressionSignaturesForAccount`](crate::Helius::get_compression_signatures_for_account).
///
/// Contains slot context and a list of transaction signatures that affected the
/// specified compressed account.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressionSignaturesForAccountResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The list of transaction signatures.
    pub value: SignatureInfoList,
}

/// A paginated list of transaction signatures related to a compressed address.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct PaginatedSignatureInfoList {
    /// The transaction signatures in this page.
    pub items: Vec<SignatureInfo>,
    /// Cursor for fetching the next page. `None` when there are no more results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Request parameters for [`getCompressionSignaturesForAddress`](crate::Helius::get_compression_signatures_for_address).
///
/// Retrieves transaction signatures for a compressed address, with optional pagination.
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressionSignaturesForAddressRequest;
///
/// let request = GetCompressionSignaturesForAddressRequest {
///     address: "11111119T6fgHG3unjQB6vpWozhBdiXDbQovvFVeF".to_string(),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressionSignaturesForAddressRequest {
    /// The Solana public key (base58) of the compressed address.
    pub address: String,
    /// Pagination cursor from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Maximum number of signatures to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

/// Response from [`getCompressionSignaturesForAddress`](crate::Helius::get_compression_signatures_for_address).
///
/// Contains slot context and a paginated list of transaction signatures that affected
/// the specified compressed address.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressionSignaturesForAddressResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The paginated list of transaction signatures.
    pub value: PaginatedSignatureInfoList,
}

/// Request parameters for [`getCompressionSignaturesForOwner`](crate::Helius::get_compression_signatures_for_owner).
///
/// Retrieves transaction signatures for all compressed accounts owned by a given address,
/// with optional pagination.
///
/// # Examples
///
/// ```
/// use helius::types::GetCompressionSignaturesForOwnerRequest;
///
/// let request = GetCompressionSignaturesForOwnerRequest {
///     owner: "11111119T6fgHG3unjQB6vpWozhBdiXDbQovvFVeF".to_string(),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressionSignaturesForOwnerRequest {
    /// The Solana public key (base58) of the account owner.
    pub owner: String,
    /// Pagination cursor from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Maximum number of signatures to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

/// Response from [`getCompressionSignaturesForOwner`](crate::Helius::get_compression_signatures_for_owner).
///
/// Contains slot context and a paginated list of transaction signatures that affected
/// compressed accounts owned by the specified address.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressionSignaturesForOwnerResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The paginated list of transaction signatures.
    pub value: PaginatedSignatureInfoList,
}

/// Response from [`getCompressionSignaturesForTokenOwner`](crate::Helius::get_compression_signatures_for_token_owner).
///
/// Contains slot context and a paginated list of transaction signatures that affected
/// compressed token accounts owned by the specified address.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressionSignaturesForTokenOwnerResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The paginated list of transaction signatures.
    pub value: PaginatedSignatureInfoList,
}

/// Request parameters for [`getLatestCompressionSignatures`](crate::Helius::get_latest_compression_signatures).
///
/// Retrieves the most recent compression transaction signatures, with optional pagination.
///
/// # Examples
///
/// ```
/// use helius::types::GetLatestCompressionSignaturesRequest;
///
/// let request = GetLatestCompressionSignaturesRequest {
///     limit: Some(10),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetLatestCompressionSignaturesRequest {
    /// Pagination cursor from a previous response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Maximum number of signatures to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}

/// Response from [`getLatestCompressionSignatures`](crate::Helius::get_latest_compression_signatures).
///
/// Contains slot context and a paginated list of the most recent compression
/// transaction signatures.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetLatestCompressionSignaturesResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The paginated list of transaction signatures.
    pub value: PaginatedSignatureInfoList,
}

/// Information about a transaction signature that may include an error.
///
/// Similar to [`SignatureInfo`] but includes an optional error string for
/// transactions that failed during processing.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SignatureInfoWithError {
    /// The Solana transaction signature (base58).
    pub signature: String,
    /// The Solana slot in which this transaction was processed.
    pub slot: u64,
    /// Unix timestamp (seconds) of the block containing this transaction.
    pub block_time: i64,
    /// Error message if the transaction failed, or `None` if it succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// A list of transaction signatures that may include errors.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignatureInfoListWithError {
    /// The transaction signatures, each potentially including an error.
    pub items: Vec<SignatureInfoWithError>,
}

/// Response from [`getLatestNonVotingSignatures`](crate::Helius::get_latest_non_voting_signatures).
///
/// Contains slot context and a list of the most recent non-voting transaction
/// signatures, each of which may include an error if the transaction failed.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetLatestNonVotingSignaturesResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The list of non-voting transaction signatures.
    pub value: SignatureInfoListWithError,
}

/// Response from [`getMultipleCompressedAccountProofs`](crate::Helius::get_multiple_compressed_account_proofs).
///
/// Contains slot context and a vector of Merkle proofs, one for each requested
/// compressed account hash.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetMultipleCompressedAccountProofsResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The Merkle proofs, in the same order as the requested hashes.
    pub value: Vec<MerkleProofWithContext>,
}

/// Request parameters for [`getMultipleCompressedAccounts`](crate::Helius::get_multiple_compressed_accounts).
///
/// Provide either `addresses` or `hashes` (or both) to identify the compressed accounts.
/// At least one must be specified.
///
/// # Examples
///
/// ```
/// use helius::types::GetMultipleCompressedAccountsRequest;
///
/// // Query by addresses
/// let by_addresses = GetMultipleCompressedAccountsRequest {
///     addresses: Some(vec![
///         "11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3".to_string(),
///         "11111114d3RrygbPdAtMuFnDmzsN8T5fYKVQ7FVr7".to_string(),
///     ]),
///     ..Default::default()
/// };
///
/// // Query by hashes
/// let by_hashes = GetMultipleCompressedAccountsRequest {
///     hashes: Some(vec!["11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string()]),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetMultipleCompressedAccountsRequest {
    /// Base58-encoded public keys of the compressed accounts to retrieve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<String>>,
    /// Base58-encoded data hashes of the compressed accounts to retrieve.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashes: Option<Vec<String>>,
}

/// A list of compressed accounts returned by a batch lookup.
///
/// Items are positional: each entry corresponds to the address or hash at the same
/// index in the request. Entries are `None` when no account matches the given
/// identifier.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AccountList {
    /// The compressed accounts, in the same order as the requested identifiers.
    /// `None` entries indicate no matching account was found.
    pub items: Vec<Option<CompressedAccount>>,
}

/// Response from [`getMultipleCompressedAccounts`](crate::Helius::get_multiple_compressed_accounts).
///
/// Contains slot context and a list of compressed accounts corresponding to the
/// requested addresses or hashes. Entries may be `None` for identifiers that did
/// not match any account.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetMultipleCompressedAccountsResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The list of compressed accounts.
    pub value: AccountList,
}

/// A Merkle proof for verifying that a new compressed address can be created.
///
/// Contains the proof data needed to demonstrate that an address does not yet exist
/// in the address Merkle tree, enabling safe creation of a new compressed account at
/// that address. Includes the lower and higher range addresses that bound the new
/// address in the tree's sorted order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MerkleContextWithNewAddressProof {
    /// The address being proven as new (base58).
    pub address: String,
    /// The nearest higher address in the tree's sorted order (base58).
    pub higher_range_address: String,
    /// The leaf index of the lower-range element in the Merkle tree.
    pub low_element_leaf_index: u32,
    /// The nearest lower address in the tree's sorted order (base58).
    pub lower_range_address: String,
    /// The address of the Merkle tree containing the address set (base58).
    pub merkle_tree: String,
    /// The next available index in the Merkle tree.
    pub next_index: u32,
    /// The hashes forming the proof path from the leaf to the root.
    pub proof: Vec<String>,
    /// The root hash of the address Merkle tree.
    pub root: String,
    /// The sequence number of the root for version tracking.
    pub root_seq: u64,
}

/// Response from [`getMultipleNewAddressProofs`](crate::Helius::get_multiple_new_address_proofs).
///
/// Contains slot context and a vector of new-address proofs, one for each requested
/// address.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetMultipleNewAddressProofsResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The new-address proofs, in the same order as the requested addresses.
    pub value: Vec<MerkleContextWithNewAddressProof>,
}

/// An address paired with its target Merkle tree.
///
/// Used by [`getMultipleNewAddressProofsV2`](crate::Helius::get_multiple_new_address_proofs_v2)
/// to specify both the address to prove and the specific address Merkle tree to check against.
///
/// # Examples
///
/// ```
/// use helius::types::AddressWithTree;
///
/// let entry = AddressWithTree {
///     address: "11111117qkFjr4u54stuNNUR8fRF8dNhaP35yvANs".to_string(),
///     tree: "11111118F5rixNBnFLmioWZSYzjjFuAL5dyoDVzhD".to_string(),
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddressWithTree {
    /// The Solana public key (base58) of the address to prove.
    pub address: String,
    /// The Solana public key (base58) of the address Merkle tree to check against.
    pub tree: String,
}

/// Response from [`getMultipleNewAddressProofsV2`](crate::Helius::get_multiple_new_address_proofs_v2).
///
/// Contains slot context and a vector of new-address proofs, one for each requested
/// address-tree pair. This V2 response is identical in shape to
/// [`GetMultipleNewAddressProofsResponse`] but corresponds to the V2 endpoint which
/// accepts [`AddressWithTree`] pairs instead of plain addresses.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetMultipleNewAddressProofsV2Response {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The new-address proofs, in the same order as the requested address-tree pairs.
    pub value: Vec<MerkleContextWithNewAddressProof>,
}

/// Request parameters for [`getTransactionWithCompressionInfo`](crate::Helius::get_transaction_with_compression_info).
///
/// # Examples
///
/// ```
/// use helius::types::GetTransactionWithCompressionInfoRequest;
///
/// let request = GetTransactionWithCompressionInfoRequest {
///     signature: "5J8H5sTvEhnGcB4R8K1n7mfoiWUD9RzPVGES7e3WxC7c".to_string(),
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTransactionWithCompressionInfoRequest {
    /// The Solana transaction signature (base58) to retrieve compression info for.
    pub signature: String,
}

/// A compressed account paired with optional token data.
///
/// Represents an account that was opened or closed in a compression transaction.
/// If the account is a compressed token account, `optional_token_data` contains
/// the SPL token fields.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountWithOptionalTokenData {
    /// The compressed account data.
    pub account: CompressedAccount,
    /// Token-specific data, present only if this is a compressed token account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_token_data: Option<TokenData>,
}

/// Compression-specific information extracted from a transaction.
///
/// Contains the lists of compressed accounts that were opened (created) and
/// closed (destroyed) by the transaction.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompressionInfo {
    /// Compressed accounts that were closed (destroyed) by this transaction.
    pub closed_accounts: Vec<AccountWithOptionalTokenData>,
    /// Compressed accounts that were opened (created) by this transaction.
    pub opened_accounts: Vec<AccountWithOptionalTokenData>,
}

/// Response from [`getTransactionWithCompressionInfo`](crate::Helius::get_transaction_with_compression_info).
///
/// Contains the full Solana transaction along with compression-specific metadata
/// about which compressed accounts were opened and closed.
///
/// **Note:** Unlike all other ZK Compression responses, this type does NOT use the
/// standard `{context, value}` envelope. The `compression_info` and `transaction`
/// fields sit directly at the JSON-RPC result level. This matches the Photon indexer
/// implementation and the Light Protocol reference client.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionWithCompressionInfoResponse {
    /// Compression-specific information (opened and closed accounts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_info: Option<CompressionInfo>,
    /// The full encoded Solana transaction with status metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<Value>,
}

/// Request parameters for [`getValidityProof`](crate::Helius::get_validity_proof).
///
/// Provide `hashes` for compressed accounts to verify, `new_addresses_with_trees` for
/// new addresses to prove, or both.
///
/// # Examples
///
/// ```
/// use helius::types::GetValidityProofRequest;
///
/// let request = GetValidityProofRequest {
///     hashes: Some(vec!["11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string()]),
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetValidityProofRequest {
    /// Base58-encoded hashes of compressed accounts to verify.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hashes: Option<Vec<String>>,
    /// Address-tree pairs for new addresses to prove non-existence of.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_addresses_with_trees: Option<Vec<AddressWithTree>>,
}

/// The three components of a zero-knowledge compressed proof.
///
/// These are the cryptographic proof elements (`a`, `b`, `c`) that together form
/// a validity proof for compressed account state on Solana.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct CompressedProof {
    /// First component of the zero-knowledge validity proof.
    pub a: String,
    /// Second component of the zero-knowledge validity proof.
    pub b: String,
    /// Third component of the zero-knowledge validity proof.
    pub c: String,
}

/// A zero-knowledge validity proof with full Merkle tree context.
///
/// Contains the cryptographic proof along with the roots, leaf indices, leaves,
/// and tree addresses needed to verify compressed account state on-chain.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompressedProofWithContext {
    /// The zero-knowledge cryptographic proof.
    pub compressed_proof: CompressedProof,
    /// Indices of the leaves in the Merkle trees.
    pub leaf_indices: Vec<u32>,
    /// Leaf node data for the compressed accounts being verified.
    pub leaves: Vec<String>,
    /// Addresses of the Merkle trees containing the compressed accounts.
    pub merkle_trees: Vec<String>,
    /// Indices of the roots in the Merkle trees.
    pub root_indices: Vec<u64>,
    /// Root hashes of the Merkle trees.
    pub roots: Vec<String>,
}

/// Response from [`getValidityProof`](crate::Helius::get_validity_proof).
///
/// Contains slot context and the zero-knowledge validity proof with full Merkle
/// tree context for the requested compressed accounts and/or new addresses.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetValidityProofResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The validity proof with Merkle tree context.
    pub value: CompressedProofWithContext,
}

/// The token balance of a compressed token account.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct TokenAccountBalance {
    /// The token amount.
    pub amount: u64,
}

/// Response from [`getCompressedTokenAccountBalance`](crate::Helius::get_compressed_token_account_balance).
///
/// Contains slot context and the token account's balance.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedTokenAccountBalanceResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The token account balance.
    pub value: TokenAccountBalance,
}

/// Response from [`getCompressedBalance`](crate::Helius::get_compressed_balance).
///
/// Contains slot context and the compressed account's balance in lamports.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GetCompressedBalanceResponse {
    /// The slot context for this response.
    pub context: ZkCompressedContext,
    /// The balance in lamports.
    pub value: u64,
}
