use super::{
    enums::{AssetSortBy, AssetSortDirection, Context, Interface, OwnershipModel, RoyaltyModel, Scope, UseMethod},
    AccountWebhookEncoding, CollectionIdentifier, PriorityLevel, SearchAssetsOptions, SearchConditionType, TokenType,
    TransactionStatus, TransactionType, UiTransactionEncoding, WebhookType,
};
use crate::types::{
    DisplayOptions, Encoding, GetAssetOptions, GpaFilter, TokenAccountsOwnerFilter, TransactionVersion,
};
use serde::ser::SerializeTuple;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_commitment_config::CommitmentLevel;
use solana_sdk::{instruction::Instruction, message::AddressLookupTableAccount, signature::Signer};
use solana_transaction_status_client_types::{EncodedTransaction, UiTransactionStatusMeta};

/// Defines the available clusters supported by Helius
#[derive(Debug, Clone, PartialEq)]
pub enum Cluster {
    Devnet,
    MainnetBeta,
    StakedMainnetBeta,
}

/// Stores the API and RPC endpoint URLs for a specific Helius cluster
#[derive(Debug, Clone)]
pub struct HeliusEndpoints {
    pub api: String,
    pub rpc: String,
}

impl HeliusEndpoints {
    pub fn for_cluster(cluster: &Cluster) -> Self {
        match cluster {
            Cluster::Devnet => HeliusEndpoints {
                api: "https://api-devnet.helius-rpc.com/".to_string(),
                rpc: "https://devnet.helius-rpc.com/".to_string(),
            },
            Cluster::MainnetBeta => HeliusEndpoints {
                api: "https://api-mainnet.helius-rpc.com/".to_string(),
                rpc: "https://mainnet.helius-rpc.com/".to_string(),
            },
            Cluster::StakedMainnetBeta => HeliusEndpoints {
                api: "https://api-mainnet.helius-rpc.com/".to_string(),
                rpc: "https://staked.helius-rpc.com/".to_string(),
            },
        }
    }
}

/// A JSON-RPC 2.0 request envelope used for DAS API and other Helius RPC calls.
///
/// Wraps the method name and typed parameters into the standard JSON-RPC format.
/// The SDK sets `jsonrpc` to `"2.0"` and `id` to `"helius-rust-sdk"` automatically
/// via [`RpcRequest::new`].
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct RpcRequest<T> {
    /// The JSON-RPC protocol version (always `"2.0"`)
    pub jsonrpc: String,
    /// An identifier for the request, used to match responses
    pub id: String,
    /// The RPC method name (e.g., `"getAsset"`, `"getAssetsByOwner"`)
    pub method: String,
    /// The method-specific parameters
    #[serde(rename = "params")]
    pub parameters: T,
}

impl<T> RpcRequest<T> {
    /// Creates a new JSON-RPC request with the given method and parameters.
    pub fn new(method: String, parameters: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: "helius-rust-sdk".to_string(),
            method,
            parameters,
        }
    }
}

/// A JSON-RPC 2.0 response envelope returned by DAS API and other Helius RPC calls.
///
/// On success the server populates `result`; on a method-level failure (e.g. invalid
/// params or an unknown method) it instead populates `error` and omits `result`, while
/// still returning an HTTP 200 status. Exactly one of `result` or `error` is present for
/// a well-formed response, so both fields are optional.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct RpcResponse<T> {
    /// The JSON-RPC protocol version (always `"2.0"`)
    pub jsonrpc: String,
    /// The request identifier, matching the corresponding [`RpcRequest::id`]
    ///
    /// Optional because the JSON-RPC spec requires servers to return `"id": null` for
    /// parse errors and invalid requests (codes `-32700` / `-32600`) — exactly the error
    /// class this envelope surfaces
    #[serde(default)]
    pub id: Option<String>,
    /// The method-specific result data, present when the call succeeds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    /// The JSON-RPC error object, present when the call fails server-side
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

/// A JSON-RPC 2.0 error object, returned in the `error` field of an [`RpcResponse`] when a
/// method call fails server-side (for example invalid params or an unknown method).
///
/// Per the JSON-RPC spec these arrive with an HTTP 200 status and no `result`, so the SDK
/// inspects `error` before returning `result`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RpcError {
    /// The JSON-RPC error code (e.g. `-32601` for "method not found", `-32602` for "invalid params")
    pub code: i64,
    /// A human-readable description of the error
    pub message: String,
    /// Optional structured data supplied by the server for additional context
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Request parameters for the `getAssetsByOwner` DAS API method.
///
/// Retrieves all digital assets (NFTs, compressed NFTs, fungible tokens) owned by a
/// specific wallet address. Supports pagination, sorting, and display options.
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GetAssetsByOwner {
    /// The owner wallet address to query (base-58 encoded public key)
    #[serde(rename = "ownerAddress")]
    pub owner_address: String,
    /// The 1-indexed page number for page-based pagination
    pub page: u32,
    /// Maximum number of assets to return per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Retrieve assets listed before this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Retrieve assets listed after this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Controls which optional fields are included in the response
    #[serde(rename = "displayOptions", skip_serializing_if = "Option::is_none")]
    pub display_options: Option<DisplayOptions>,
    /// Sort criteria and direction for the returned assets
    #[serde(rename = "sortBy", skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<AssetSorting>,
    /// Opaque cursor for cursor-based pagination (faster than page-based for large sets)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Request parameters for the `getAssetsByAuthority` DAS API method.
///
/// Retrieves all digital assets where the specified address is the update authority.
/// Useful for finding all assets managed by a particular authority (e.g., a collection creator).
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GetAssetsByAuthority {
    /// The authority address to query (base-58 encoded public key)
    #[serde(rename = "authorityAddress")]
    pub authority_address: String,
    /// The 1-indexed page number for page-based pagination
    pub page: u32,
    /// Maximum number of assets to return per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Retrieve assets listed before this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Retrieve assets listed after this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Controls which optional fields are included in the response
    #[serde(rename = "displayOptions", skip_serializing_if = "Option::is_none")]
    pub display_options: Option<DisplayOptions>,
    /// Sort criteria and direction for the returned assets
    #[serde(rename = "sortBy", skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<AssetSorting>,
    /// Opaque cursor for cursor-based pagination
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Request parameters for the `getAsset` DAS API method.
///
/// Retrieves a single digital asset by its ID (mint address or asset ID for compressed NFTs).
/// Returns detailed information including metadata, ownership, royalty configuration,
/// compression state, and optionally token/inscription data.
///
/// Price data returned in `token_info` is cached and may lag by up to 15 minutes.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetAsset {
    /// The asset ID to look up (mint address for standard NFTs, or derived asset ID for cNFTs)
    pub id: String,
    /// Display options controlling which optional fields to include in the response
    #[serde(rename = "displayOptions")]
    pub display_options: Option<GetAssetOptions>,
}

/// Request parameters for the `getAssetsByCreator` DAS API method.
///
/// Retrieves all digital assets where the specified address is listed as a creator
/// in the asset's metadata. Optionally filters to only verified creators.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetsByCreator {
    /// The creator address to query (base-58 encoded public key)
    pub creator_address: String,
    /// If `true`, only return assets where this creator is verified on-chain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_verified: Option<bool>,
    /// Sort criteria and direction for the returned assets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<AssetSorting>,
    /// Maximum number of assets to return per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// The 1-indexed page number for page-based pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Retrieve assets listed before this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Retrieve assets listed after this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Controls which optional fields are included in the response
    #[serde(default, alias = "displayOptions", skip_serializing_if = "Option::is_none")]
    pub options: Option<DisplayOptions>,
    /// Opaque cursor for cursor-based pagination
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Request parameters for the `getAssetBatch` DAS API method.
///
/// Retrieves multiple digital assets in a single request. More efficient than
/// making individual `getAsset` calls when you need data for multiple assets.
/// Accepts up to 1,000 asset IDs per request.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetAssetBatch {
    /// The asset IDs to look up (mint addresses or derived asset IDs for cNFTs)
    pub ids: Vec<String>,
    /// Display options controlling which optional fields to include in the response
    #[serde(rename = "displayOptions")]
    pub display_options: Option<GetAssetOptions>,
}

/// Request parameters for the `getAssetProof` DAS API method.
///
/// Retrieves the Merkle proof for a compressed NFT (cNFT). The proof is required
/// to perform on-chain operations (transfer, burn, delegate) on compressed assets.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetAssetProof {
    /// The asset ID of the compressed NFT
    pub id: String,
}

/// Request parameters for the `getAssetProofBatch` DAS API method.
///
/// Retrieves Merkle proofs for multiple compressed NFTs in a single request.
/// More efficient than individual `getAssetProof` calls for batch operations.
#[derive(Serialize, Deserialize, Debug)]
pub struct GetAssetProofBatch {
    /// The asset IDs of the compressed NFTs to get proofs for
    pub ids: Vec<String>,
}

/// The Merkle proof for a compressed NFT, returned by `getAssetProof`.
///
/// Contains the data needed to verify and modify a compressed NFT's leaf in its
/// concurrent Merkle tree. Required for on-chain cNFT operations such as transfers,
/// burns, and delegate changes.
#[derive(Serialize, Deserialize, Debug)]
pub struct AssetProof {
    /// The current root hash of the Merkle tree (base-58 encoded)
    pub root: String,
    /// The ordered list of sibling hashes forming the Merkle path from leaf to root
    pub proof: Vec<String>,
    /// The index of the leaf node within the Merkle tree
    pub node_index: i32,
    /// The hash of the asset's leaf data (base-58 encoded)
    pub leaf: String,
    /// The address of the concurrent Merkle tree account
    pub tree_id: String,
}

/// Request parameters for the `getAssetsByGroup` DAS API method.
///
/// Retrieves all digital assets belonging to a specific group, most commonly a
/// collection. For example, passing `group_key: "collection"` and `group_value`
/// set to a collection mint address returns all NFTs in that collection.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetsByGroup {
    /// The group key type (e.g., `"collection"`)
    pub group_key: String,
    /// The group value to match (e.g., the collection mint address)
    pub group_value: String,
    /// Sort criteria and direction for the returned assets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<AssetSorting>,
    /// Maximum number of assets to return per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// The 1-indexed page number for page-based pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Retrieve assets listed before this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Retrieve assets listed after this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Controls which optional fields are included in the response
    #[serde(default, alias = "displayOptions", skip_serializing_if = "Option::is_none")]
    pub options: Option<DisplayOptions>,
    /// Opaque cursor for cursor-based pagination
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Request parameters for the `searchAssets` DAS API method.
///
/// Provides advanced search across all Solana digital assets with fine-grained filtering.
/// Supports filtering by owner, creator, authority, collection, compression state,
/// royalty configuration, token type, and more. All filter fields are optional —
/// combine them to narrow results.
///
/// Filters are combined with AND logic by default. Set `condition_type` to `Any`
/// for OR logic, or use `negate` to invert the entire filter set.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchAssets {
    /// If `true`, inverts the search criteria (returns assets that do NOT match)
    pub negate: Option<bool>,
    /// How multiple conditions are combined: `All` (AND, default) or `Any` (OR)
    pub condition_type: Option<SearchConditionType>,
    /// Filter by asset interface/token standard (e.g., `V1_NFT`, `ProgrammableNFT`, `FungibleToken`)
    pub interface: Option<Interface>,
    /// Filter by current owner wallet address
    pub owner_address: Option<String>,
    /// Filter by ownership model (e.g., `Single`, `Token`)
    pub owner_type: Option<OwnershipModel>,
    /// Filter by creator address listed in the asset's metadata
    pub creator_address: Option<String>,
    /// If `true`, only match assets where the creator is verified on-chain
    pub creator_verified: Option<bool>,
    /// Filter by update authority address
    pub authority_address: Option<String>,
    /// Filter by group as a `(group_key, group_value)` tuple (e.g., `("collection", "<mint>")`)
    pub grouping: Option<(String, String)>,
    /// Filter by delegate address
    pub delegate: Option<String>,
    /// Filter by frozen state
    pub frozen: Option<bool>,
    /// Filter by token supply amount
    pub supply: Option<u64>,
    /// Filter by supply mint address
    pub supply_mint: Option<String>,
    /// If `true`, only return compressed NFTs; if `false`, only uncompressed
    pub compressed: Option<bool>,
    /// If `true`, only return assets eligible for compression
    pub compressible: Option<bool>,
    /// Filter by royalty model (e.g., `Creators`, `Fanout`, `Single`)
    pub royalty_target_type: Option<RoyaltyModel>,
    /// Filter by royalty target address
    pub royalty_target: Option<String>,
    /// Filter by royalty amount in basis points
    pub royalty_amount: Option<u32>,
    /// If `true`, only return burnt assets; if `false`, only non-burnt
    pub burnt: Option<bool>,
    /// Sort criteria and direction for the returned assets
    pub sort_by: Option<AssetSorting>,
    /// Maximum number of assets to return per page (default 1000)
    pub limit: Option<u32>,
    /// The 1-indexed page number for page-based pagination
    pub page: Option<u32>,
    /// Retrieve assets listed before this cursor value
    pub before: Option<String>,
    /// Retrieve assets listed after this cursor value
    pub after: Option<String>,
    /// Filter by the off-chain JSON metadata URI
    #[serde(default)]
    pub json_uri: Option<String>,
    /// Negation filter to exclude specific collections, owners, creators, or authorities
    #[serde(default)]
    pub not: Option<NotFilter>,
    /// Display and formatting options for the response
    #[serde(default, alias = "displayOptions")]
    pub options: Option<SearchAssetsOptions>,
    /// Opaque cursor for cursor-based pagination
    #[serde(default)]
    pub cursor: Option<String>,
    /// Filter by asset name (partial match supported)
    #[serde(default)]
    pub name: Option<String>,
    /// Filter to assets belonging to any of these collection addresses
    #[serde(default)]
    pub collections: Option<Vec<String>>,
    /// Filter by token type (e.g., `Fungible`, `NonFungible`, `RegularNft`, `CompressedNft`)
    #[serde(default)]
    pub token_type: Option<TokenType>,
    /// Filter by asset creation timestamp range
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<CreatedAtFilter>,
    /// Filter by the Merkle tree address (for compressed NFTs)
    #[serde(default)]
    pub tree: Option<String>,
    /// If `true`, only return collection-level NFTs
    #[serde(default)]
    pub collection_nft: Option<bool>,
}

/// Request parameters for the `getSignaturesForAsset` DAS API method.
///
/// Retrieves all transaction signatures associated with a specific compressed NFT.
/// Can look up by asset ID or by the combination of Merkle tree address and leaf index.
///
/// This method is specifically designed for compressed NFTs. For standard NFTs,
/// use `getSignaturesForAddress` instead.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetSignatures {
    /// The asset ID of the compressed NFT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Maximum number of signatures to return per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// The 1-indexed page number for page-based pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Retrieve signatures listed before this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Retrieve signatures listed after this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// The Merkle tree address (alternative to `id` for looking up by tree + leaf)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree: Option<String>,
    /// The leaf index within the Merkle tree (used with `tree`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leaf_index: Option<i64>,
    /// Opaque cursor for cursor-based pagination
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Sort direction for the returned signatures (`Asc` or `Desc`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<AssetSortDirection>,
}

/// Request parameters for the `getTokenAccounts` DAS API method.
///
/// Retrieves token accounts filtered by owner address, mint address, or both.
/// Useful for finding all holders of a specific token or all tokens held by a wallet.
/// At least one of `owner` or `mint` must be provided.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTokenAccounts {
    /// Filter by the wallet that owns the token accounts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Filter by the token mint address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,
    /// Maximum number of token accounts to return per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// The 1-indexed page number for page-based pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Retrieve accounts listed before this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Retrieve accounts listed after this cursor value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Controls which optional fields are included in the response
    #[serde(default, alias = "displayOptions", skip_serializing_if = "Option::is_none")]
    pub options: Option<DisplayOptions>,
    /// Opaque cursor for cursor-based pagination
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Request parameters for the `getNftEditions` DAS API method.
///
/// Retrieves all printed editions of a Metaplex master edition NFT.
/// Provide the mint address of the master edition to get its prints.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetNftEditions {
    /// The mint address of the master edition NFT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,
    /// Maximum number of editions to return per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// The 1-indexed page number for page-based pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
}

/// Sorting configuration for DAS API queries that return asset lists.
///
/// Used by `getAssetsByOwner`, `getAssetsByCreator`, `getAssetsByGroup`,
/// `searchAssets`, and other paginated DAS methods.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetSorting {
    /// The field to sort by (e.g., `Created`, `Updated`, `RecentAction`, `None`)
    pub sort_by: AssetSortBy,
    /// The sort direction (`Asc` or `Desc`). Defaults to `Desc` if not specified.
    pub sort_direction: Option<AssetSortDirection>,
}

/// A JSON-RPC response wrapper used for DAS API calls.
///
/// Similar to [`RpcResponse`] but used in contexts where the response
/// is parsed directly from the API without the SDK's request layer.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ApiResponse<T> {
    /// The JSON-RPC protocol version (always `"2.0"`)
    pub jsonrpc: String,
    /// The method-specific result data
    pub result: T,
    /// The request identifier
    pub id: String,
}

/// Native SOL balance information for a Solana wallet.
///
/// Included in API responses when `show_native_balance: true` is set in display options
/// for methods like `getAssetsByOwner` or `searchAssets`.
///
/// # Fields
///
/// - `lamports`: The wallet's SOL balance in lamports (1 SOL = 1,000,000,000 lamports)
/// - `price_per_sol`: Current market price of 1 SOL in USD
/// - `total_price`: Total USD value of the wallet's SOL balance (lamports × price_per_sol / 1e9)
///
/// # Example
///
/// ```json
/// {
///   "lamports": 5000000000,
///   "price_per_sol": 100.50,
///   "total_price": 502.50
/// }
/// ```
/// This represents 5 SOL worth $502.50 USD at $100.50 per SOL.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NativeBalance {
    pub lamports: u64,
    pub price_per_sol: f64,
    pub total_price: f64,
}

/// Paginated list of Solana digital assets.
///
/// Returned by DAS API methods including:
/// - `getAssetsByOwner` - All assets owned by a wallet
/// - `getAssetsByCreator` - All assets created by an address
/// - `getAssetsByGroup` - All assets in a collection/group
/// - `getAssetsByAuthority` - All assets with a specific authority
///
/// # Pagination
///
/// Supports two pagination strategies:
///
/// **Page-based** (traditional numbered pages):
/// ```ignore
/// let options = GetAssetsByOwner {
///     page: 2,
///     limit: Some(50),
///     ..Default::default()
/// };
/// ```
///
/// **Cursor-based** (faster for infinite scrolling):
/// ```ignore
/// let options = GetAssetsByOwner {
///     after: Some(previous_response.cursor.unwrap()),
///     ..Default::default()
/// };
/// ```
///
/// # Performance
///
/// - `grand_total` is only populated when `show_grand_total: true` is set in display options
/// - Computing `grand_total` significantly increases response time for large collections
/// - For large result sets, prefer cursor-based pagination over page-based
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AssetList {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grand_total: Option<u64>,
    pub total: u32,
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    pub items: Vec<Asset>,
    #[serde(rename = "nativeBalance", skip_serializing_if = "Option::is_none")]
    pub native_balance: Option<NativeBalance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<AssetError>>,
}

/// Paginated list of transaction signatures for a Solana digital asset.
///
/// Returned by `getSignaturesForAsset`, which provides a complete chronological
/// history of all transactions involving a specific compressed NFT (cNFT).
///
/// **Note:** This method is specifically designed for compressed NFTs. For regular NFTs,
/// use `getSignaturesForAddress` instead, as the standard method doesn't work with
/// compressed assets.
///
/// # Item Format
///
/// Each item in the `items` array is a tuple of `(signature, operation_type)`:
/// - **signature**: Base58-encoded transaction signature
/// - **operation_type**: The type of operation performed (e.g., "Transfer", "MintToCollectionV1", "Burn")
///
/// # Example Response
///
/// ```json
/// {
///   "total": 3,
///   "limit": 1000,
///   "items": [
///     ["5nLi8m72bU6PBcz4Xrk23P6KTGy9ufF92kZiQXjTv9EL...", "MintToCollectionV1"],
///     ["323Ag4J69gagBt3neUvajNauMydiXZTmXYSfdK5swWcK...", "Transfer"],
///     ["3TbybyYRtNjVMhhahTNbd4bbpiEacZn2qkwtH7ByL7tC...", "Transfer"]
///   ]
/// }
/// ```
///
/// # Common Operation Types
///
/// - `Transfer` - Asset ownership transferred
/// - `MintToCollectionV1` - Asset minted into a collection
/// - `Burn` - Asset permanently destroyed
/// - `Redeem` - Compressed asset redeemed
/// - `Decompress` - Compressed asset decompressed to regular NFT
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct TransactionSignatureList {
    pub total: u32,
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    pub items: Vec<(String, String)>,
}

/// Paginated list of SPL token accounts.
///
/// Returned by `getTokenAccounts`, which retrieves all token accounts for a given
/// mint address and/or owner address. Useful for finding all holders of a specific
/// token or all tokens held by a specific wallet.
///
/// # Pagination
///
/// Supports both page-based and cursor-based pagination via `page`, `cursor`,
/// `before`, and `after` fields.
///
/// # Example Response
///
/// ```json
/// {
///   "total": 150,
///   "limit": 100,
///   "page": 1,
///   "token_accounts": [
///     {
///       "address": "H8sMJSCQxfKiFTCfDR3DUMLPwcRbM61LGFJ8N4dK3WjS",
///       "mint": "So11111111111111111111111111111111111111112",
///       "owner": "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY",
///       "amount": 1000000000
///     }
///   ]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct TokenAccountsList {
    pub total: u32,
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    pub token_accounts: Vec<TokenAccount>,
}

/// Paginated list of NFT editions for a master edition.
///
/// Returned by `getNftEditions`, which retrieves all printed editions of a
/// Metaplex master edition NFT. Master editions can have multiple prints,
/// each with a unique edition number.
///
/// # Fields
///
/// - `total`: Total number of editions returned in this response
/// - `limit`: Maximum number of editions requested per page
/// - `page`: Current page number (optional, for page-based pagination)
/// - `master_edition_address`: The mint address of the master edition NFT
/// - `supply`: Current number of editions that have been printed
/// - `max_supply`: Maximum number of editions that can be printed (None = unlimited)
/// - `editions`: Array of printed edition details
///
/// # Example Response
///
/// ```json
/// {
///   "total": 50,
///   "limit": 100,
///   "page": 1,
///   "master_edition_address": "5wF2w...",
///   "supply": 50,
///   "max_supply": 100,
///   "editions": [
///     { "mint": "...", "edition_address": "...", "edition": 1 },
///     { "mint": "...", "edition_address": "...", "edition": 2 }
///   ]
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EditionsList {
    pub total: u32,
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    pub master_edition_address: String,
    pub supply: u64,
    pub max_supply: Option<u64>,
    pub editions: Vec<Edition>,
}

/// A Solana digital asset returned by the DAS API.
///
/// This is the primary response type for `getAsset`, `getAssetBatch`, and the items
/// in paginated list responses from `getAssetsByOwner`, `getAssetsByCreator`,
/// `getAssetsByGroup`, `getAssetsByAuthority`, and `searchAssets`.
///
/// Supports all Solana token standards including regular NFTs, compressed NFTs (cNFTs),
/// programmable NFTs (pNFTs), fungible tokens, and MPL Core assets.
#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
    /// The token standard/interface of this asset (e.g., `V1_NFT`, `ProgrammableNFT`, `FungibleToken`)
    pub interface: Interface,
    /// The unique identifier (mint address for standard tokens, derived ID for cNFTs)
    pub id: String,
    /// On-chain and off-chain content including metadata, files, and links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    /// Authorities with control over this asset and their permission scopes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorities: Option<Vec<Authorities>>,
    /// Compression state for compressed NFTs (Merkle tree data, hashes, leaf info)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<Compression>,
    /// Collection and other group memberships for this asset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouping: Option<Vec<Group>>,
    /// Royalty configuration for marketplace sales
    #[serde(skip_serializing_if = "Option::is_none")]
    pub royalty: Option<Royalty>,
    /// The list of creators and their share percentages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creators: Option<Vec<Creator>>,
    /// Current ownership details including owner, delegate, and frozen state
    pub ownership: Ownership,
    /// Use/utility tracking for the asset (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<Uses>,
    /// Edition supply information (for master editions and prints)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supply: Option<Supply>,
    /// Whether the asset's metadata can be updated
    pub mutable: bool,
    /// Whether the asset has been permanently destroyed
    pub burnt: bool,
    /// Token-2022 mint extension data, if the asset uses the Token Extensions program
    pub mint_extensions: Option<Value>,
    /// Token-specific information (supply, decimals, price, authorities)
    pub token_info: Option<TokenInfo>,
    /// Group definition data if this asset defines a collection or group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_definition: Option<GroupDefinition>,
    /// System-level information (e.g., creation timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemInfo>,
    /// MPL Core plugin data as raw JSON
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Value>,
    /// Unrecognized MPL Core plugin data as raw JSON
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_plugins: Option<Value>,
    /// MPL Core collection information (minted count, current size)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mpl_core_info: Option<MplCoreInfo>,
}

/// An error for an individual asset in a batch DAS API response.
///
/// When a batch request (e.g., `getAssetBatch`) encounters an error for a specific
/// asset, the error is returned inline alongside successful results rather than
/// failing the entire request.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct AssetError {
    /// The asset ID that caused the error
    pub id: String,
    /// A description of the error encountered for this asset
    pub error: String,
}

/// Token-2022 (Token Extensions) mint extension data for a digital asset.
///
/// Contains the various extension configurations that can be enabled on a Token-2022
/// mint account. Fields with `Option` types are only present when that extension is
/// active; non-optional fields use default/empty values when inactive.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MintExtensions {
    /// Confidential transfer configuration for privacy-preserving transfers
    pub confidential_transfer_mint: Option<ConfidentialTransferMint>,
    /// Confidential transfer fee configuration
    pub confidential_transfer_fee_config: Option<ConfidentialTransferFeeConfig>,
    /// Transfer fee configuration (automatic fee on every transfer)
    pub transfer_fee_config: Option<TransferFeeConfig>,
    /// Pointer to the account that holds the token's metadata
    pub metadata_pointer: MetadataPointer,
    /// Authority that can close the mint account
    pub mint_close_authority: MintCloseAuthority,
    /// A delegate that can transfer or burn tokens from any account
    pub permanent_delegate: PermanentDelegate,
    /// A program invoked on every token transfer for custom validation
    pub transfer_hook: TransferHook,
    /// Configuration for interest-bearing token balances
    pub interest_bearing_config: InterestBearingConfig,
    /// The default state for new token accounts (e.g., `Initialized`, `Frozen`)
    pub default_account_state: DefaultAccountState,
    /// Confidential transfer account-level configuration
    pub confidential_transfer_account: ConfidentialTransferAccount,
    /// On-chain metadata stored directly in the mint account
    pub metadata: MintExtensionMetadata,
}

/// Token-2022 confidential transfer mint configuration.
///
/// Enables privacy-preserving token transfers where amounts are encrypted.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialTransferMint {
    /// The authority that manages confidential transfer settings
    pub authority: String,
    /// Whether new token accounts are automatically approved for confidential transfers
    pub auto_approve_new_accounts: bool,
    /// The auditor's ElGamal public key for optional transfer auditing
    pub auditor_elgamal_pubkey: String,
}

/// Token-2022 confidential transfer fee configuration.
///
/// Manages fees on confidential transfers while keeping transfer amounts private.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialTransferFeeConfig {
    /// The authority that manages the fee configuration
    pub authority: String,
    /// The ElGamal public key of the authority that can withdraw withheld fees
    pub withdraw_withheld_authority_elgamal_pubkey: String,
    /// Whether harvesting withheld fees to the mint is enabled
    pub harvest_to_mint_enabled: bool,
    /// The encrypted amount of fees withheld across all accounts
    pub withheld_amount: String,
}

/// Token-2022 transfer fee configuration.
///
/// Enables automatic fee collection on every token transfer. The fee is calculated
/// as a percentage (in basis points) of the transfer amount, capped at a maximum.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferFeeConfig {
    /// The authority that can modify the transfer fee configuration
    pub transfer_fee_config_authority: String,
    /// The authority that can withdraw collected fees
    pub withdraw_withheld_authority: String,
    /// The total amount of fees withheld across all token accounts
    pub withheld_amount: i32,
    /// The transfer fee schedule from the previous epoch
    pub older_transfer_fee: OlderTransferFee,
    /// The transfer fee schedule for the current/upcoming epoch
    pub new_transfer_fee: NewTransferFee,
}

/// A transfer fee schedule from a previous epoch.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OlderTransferFee {
    /// The epoch this fee schedule was active
    pub epoch: String,
    /// The maximum fee that can be charged per transfer
    pub maximum_fee: String,
    /// The fee rate in basis points (1 bp = 0.01%)
    pub transfer_fee_basis_points: String,
}

/// The epoch at which a new transfer fee configuration takes effect.
#[derive(Serialize, Deserialize, Debug)]
pub struct NewTransferFee {
    /// The epoch number when the new transfer fee becomes active
    pub epoch: String,
}

/// Token-2022 metadata pointer extension.
///
/// Points to the account that holds the token's metadata. When the metadata
/// is stored in the mint account itself, `metadata_address` equals the mint address.
#[derive(Serialize, Deserialize, Debug)]
pub struct MetadataPointer {
    /// The authority that can update the metadata pointer
    pub authority: String,
    /// The account address where the metadata is stored
    #[serde(rename = "metadataAddress")]
    pub metadata_address: String,
}

/// Token-2022 mint close authority extension.
///
/// Allows the designated authority to close the mint account and reclaim its rent.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MintCloseAuthority {
    /// The authority that can close the mint account
    pub close_authority: String,
}

/// Token-2022 permanent delegate extension.
///
/// Designates a delegate that can transfer or burn tokens from any token account
/// for this mint, regardless of the account owner's approval.
#[derive(Serialize, Deserialize, Debug)]
pub struct PermanentDelegate {
    /// The permanent delegate's public key
    pub delegate: String,
}

/// Token-2022 transfer hook extension.
///
/// Specifies a program that is invoked on every token transfer for custom
/// validation or side effects (e.g., enforcing transfer restrictions).
#[derive(Serialize, Deserialize, Debug)]
pub struct TransferHook {
    /// The authority that can update the transfer hook program
    pub authority: String,
    /// The program ID that is invoked on every transfer
    #[serde(rename = "programId")]
    pub program_id: String,
}

/// Token-2022 interest-bearing configuration.
///
/// Enables token balances to accrue interest over time based on a configurable rate.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InterestBearingConfig {
    /// The authority that can update the interest rate
    pub rate_authority: String,
    /// Unix timestamp when interest accrual was initialized
    pub initialization_timestamp: i32,
    /// The average interest rate before the last update
    pub pre_update_average_rate: i32,
    /// Unix timestamp of the last rate update
    pub last_update_timestamp: i32,
    /// The current interest rate in basis points
    pub current_rate: i32,
}

/// Token-2022 default account state extension.
///
/// Sets the initial state for all new token accounts created for this mint.
#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultAccountState {
    /// The default state (e.g., `"initialized"`, `"frozen"`)
    pub state: String,
}

/// Token-2022 confidential transfer account-level configuration.
///
/// Contains the encryption keys and balance ciphertexts for an account
/// participating in confidential transfers.
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialTransferAccount {
    /// Whether this account is approved for confidential transfers
    pub approved: bool,
    /// The account's ElGamal public key for encrypting balances
    pub elgamal_pubkey: String,
    /// The low bits of the encrypted pending balance
    pub pending_balance_lo: String,
    /// The high bits of the encrypted pending balance
    pub pending_balance_hi: String,
    /// The encrypted available (spendable) balance
    pub available_balance: String,
    /// The decryptable available balance (can be decrypted by the owner)
    pub decryptable_available_balance: String,
    /// Whether this account accepts incoming confidential credits
    pub allow_confidential_credits: bool,
    /// Whether this account accepts incoming non-confidential credits
    pub allow_non_confidential_credits: bool,
    /// Counter of pending balance credits
    pub pending_balance_credit_counter: i32,
    /// Maximum allowed pending balance credit counter
    pub maximum_pending_balance_credit_counter: i32,
    /// Expected pending balance credit counter
    pub expected_pending_balance_credit_counter: i32,
    /// Actual pending balance credit counter
    pub actual_pending_balance_credit_counter: i32,
}

/// On-chain metadata stored directly in a Token-2022 mint account.
///
/// Part of the Token-2022 metadata extension, which allows token metadata
/// to be stored in the mint account itself rather than in a separate Metaplex account.
#[derive(Serialize, Deserialize, Debug)]
pub struct MintExtensionMetadata {
    /// The authority that can update this metadata
    #[serde(rename = "updateAuthority")]
    pub update_authority: String,
    /// The mint address this metadata belongs to
    pub mint: String,
    /// The token name
    pub name: String,
    /// The token symbol
    pub symbol: String,
    /// URI pointing to additional off-chain metadata (JSON)
    pub uri: String,
    /// Additional custom key-value metadata pairs
    #[serde(rename = "additionalMetadata")]
    pub additional_metadata: AdditionalMetadata,
}

/// A custom key-value metadata pair in a Token-2022 mint extension.
#[derive(Serialize, Deserialize, Debug)]
pub struct AdditionalMetadata {
    /// The metadata key
    pub key: String,
    /// The metadata value
    pub value: String,
}

/// Token-specific information for a digital asset.
///
/// Contains supply, decimal precision, program addresses, and optionally price data.
/// Present on fungible tokens and NFTs when `showFungible: true` is set.
/// Price data is cached and may lag by up to 15 minutes.
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenInfo {
    /// The token ticker symbol (e.g., `"SOL"`, `"USDC"`)
    pub symbol: Option<String>,
    /// The token balance held by the owner (in the token's smallest unit)
    pub balance: Option<u64>,
    /// The total supply of the token
    pub supply: Option<u64>,
    /// The number of decimal places for the token
    pub decimals: Option<i32>,
    /// The token program ID (e.g., `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` for SPL Token)
    pub token_program: Option<String>,
    /// The associated token account address for this owner and mint
    pub associated_token_address: Option<String>,
    /// Current price information for the token (cached, may lag up to 15 minutes)
    pub price_info: Option<PriceInfo>,
    /// The mint authority address (can mint new tokens)
    pub mint_authority: Option<String>,
    /// The freeze authority address (can freeze token accounts)
    pub freeze_authority: Option<String>,
}

/// Price information for a token, provided by Helius.
///
/// Price data is cached and may lag by up to 15 minutes.
#[derive(Serialize, Deserialize, Debug)]
pub struct PriceInfo {
    /// The current price per individual token
    pub price_per_token: f32,
    /// The currency the price is denominated in (e.g., `"USDC"`)
    pub currency: String,
}

/// On-chain inscription data for a digital asset.
///
/// Inscriptions store data directly on the Solana blockchain (similar to Bitcoin Ordinals).
/// Only present when `showInscription: true` is set in the request's display options.
#[derive(Serialize, Deserialize, Debug)]
pub struct Inscription {
    /// The ordinal position of this inscription
    pub order: i32,
    /// The size of the inscription data in bytes
    pub size: i32,
    /// The MIME type of the inscribed content (e.g., `"image/png"`, `"text/plain"`)
    #[serde(rename = "contentType")]
    pub content_type: String,
    /// The encoding of the inscription data (e.g., `"base64"`)
    pub encoding: String,
    /// A hash used to validate the integrity of the inscription data
    #[serde(rename = "validationHash")]
    pub validation_hash: String,
    /// The account that stores the actual inscription data
    #[serde(rename = "inscriptionDataAccount")]
    pub inscription_data_account: String,
    /// The authority that controls the inscription
    pub authority: String,
}

/// Content information for a digital asset, including metadata, files, and links.
///
/// Contains both the URI pointing to off-chain JSON metadata and the resolved
/// metadata fields (name, symbol, description, attributes). Files include the
/// original URIs and Helius CDN-cached versions for faster loading.
#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    /// The metadata schema URL (e.g., `"https://schema.metaplex.com/nft1.0.json"`)
    #[serde(rename = "schema", default)]
    #[serde(alias = "$schema")]
    pub schema: String,
    /// URI pointing to the off-chain JSON metadata, typically on Arweave or IPFS
    pub json_uri: String,
    /// Files associated with the asset (images, animations, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<File>>,
    /// Resolved metadata including name, symbol, description, and trait attributes
    pub metadata: Metadata,
    /// External links associated with the asset (website, image, animation URLs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

/// A file associated with a digital asset (image, animation, document, etc.).
///
/// Each file includes the original URI from the asset's metadata and optionally
/// a Helius CDN-cached URI for faster and more reliable access.
#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    /// The original file URI from the asset's metadata
    pub uri: Option<String>,
    /// The MIME type of the file (e.g., `"image/png"`, `"video/mp4"`)
    pub mime: Option<String>,
    /// A Helius CDN-cached URI for faster loading
    pub cdn_uri: Option<String>,
    /// Quality metadata for the file
    pub quality: Option<FileQuality>,
    /// The contexts in which this file is used (e.g., `WebDesktop`, `AppMobile`)
    pub contexts: Option<Vec<Context>>,
}

/// Quality metadata for an asset file.
#[derive(Serialize, Deserialize, Debug)]
pub struct FileQuality {
    /// The quality schema identifier
    #[serde(rename = "$$schema")]
    pub schema: String,
}

/// Asset trait attributes, which can appear as either an array or a key-value map.
///
/// Most NFTs use the list format (`[{"trait_type": "Color", "value": "Blue"}, ...]`),
/// but some use a flat JSON object map. This enum handles both formats transparently.
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Attributes {
    /// Standard array of trait-value pairs (most common format)
    List(Vec<Attribute>),
    /// Flat key-value map of attributes
    Map(serde_json::Map<String, Value>),
}

/// Resolved metadata for a digital asset.
///
/// Contains the human-readable information about the asset extracted from its
/// on-chain and off-chain metadata (name, symbol, description, and trait attributes).
#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    /// The trait attributes of the asset (e.g., `[{"trait_type": "Color", "value": "Blue"}]`)
    pub attributes: Option<Attributes>,
    /// A human-readable description of the asset
    pub description: Option<String>,
    /// The display name of the asset (e.g., `"Mad Lads #8420"`)
    pub name: Option<String>,
    /// The ticker symbol of the asset (e.g., `"MAD"`)
    pub symbol: Option<String>,
}

/// A single trait attribute on a digital asset (e.g., `{"trait_type": "Color", "value": "Blue"}`).
#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    /// The trait value, which can be a string, number, or boolean
    pub value: Value,
    /// The trait category name (e.g., `"Color"`, `"Background"`, `"Rarity"`)
    pub trait_type: String,
}

/// External links associated with a digital asset.
#[derive(Serialize, Deserialize, Debug)]
pub struct Links {
    /// The project or creator website URL
    pub external_url: Option<String>,
    /// The primary image URL for the asset
    pub image: Option<String>,
    /// An animation or video URL for the asset
    pub animation_url: Option<String>,
}

/// An authority with control over a digital asset and their permission scopes.
#[derive(Serialize, Deserialize, Debug)]
pub struct Authorities {
    /// The authority's public key (base-58 encoded)
    pub address: String,
    /// The permission scopes granted to this authority (e.g., `Full`, `Royalty`, `Metadata`, `Extension`)
    pub scopes: Vec<Scope>,
}

/// A group membership for a digital asset (e.g., belonging to a collection).
///
/// The most common grouping is `group_key: "collection"` with `group_value` set
/// to the collection mint address. When `showCollectionMetadata` is enabled in
/// display options, the `collection_metadata` field is populated.
#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    /// The group type key (e.g., `"collection"`)
    pub group_key: String,
    /// The group identifier value (e.g., the collection mint address)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_value: Option<String>,
    /// Whether this group membership has been verified on-chain
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    /// Metadata for the collection (only present when `showCollectionMetadata` is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_metadata: Option<CollectionMetadata>,
}

/// Metadata for a collection that an asset belongs to.
///
/// Only populated when `showCollectionMetadata: true` is set in the request's
/// display options.
#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionMetadata {
    /// The collection display name
    pub name: Option<String>,
    /// The collection ticker symbol
    pub symbol: Option<String>,
    /// The collection image URL
    pub image: Option<String>,
    /// A description of the collection
    pub description: Option<String>,
    /// The collection's website URL
    pub external_url: Option<String>,
}

/// Compression state of a digital asset, indicating whether it is a compressed NFT (cNFT).
///
/// Compressed NFTs use Solana's state compression technology to store asset data in
/// concurrent Merkle trees, dramatically reducing on-chain storage costs. The hash fields
/// and tree reference are needed for Merkle proof verification during on-chain operations.
///
/// For uncompressed assets, `compressed` is `false` and the hash/tree fields are empty strings.
#[derive(Serialize, Deserialize, Debug)]
pub struct Compression {
    /// Whether this asset is eligible for compression
    pub eligible: bool,
    /// Whether this asset is currently compressed (stored in a Merkle tree)
    pub compressed: bool,
    /// Hash of the asset's data (base-58 encoded, empty for uncompressed assets)
    pub data_hash: String,
    /// Hash of the creator data (base-58 encoded, empty for uncompressed assets)
    pub creator_hash: String,
    /// Hash of the entire asset (base-58 encoded, empty for uncompressed assets)
    pub asset_hash: String,
    /// The concurrent Merkle tree address (empty for uncompressed assets)
    pub tree: String,
    /// The sequence number for ordering concurrent updates to this leaf
    pub seq: i64,
    /// The leaf index within the Merkle tree
    pub leaf_id: i64,
}

/// A creator listed in a digital asset's metadata.
///
/// Creators receive royalty payments proportional to their `share` percentage.
/// The `verified` flag indicates whether the creator has signed the asset on-chain,
/// confirming their participation.
#[derive(Serialize, Deserialize, Debug)]
pub struct Creator {
    /// The creator's public key (base-58 encoded)
    pub address: String,
    /// The creator's share of royalty payments (0-100, as a percentage)
    pub share: i32,
    /// Whether this creator has been verified on-chain
    pub verified: bool,
}

/// Royalty configuration for a digital asset, used for marketplace fee calculations.
///
/// Royalties are enforced by marketplaces and define the percentage of secondary sales
/// that goes to the asset's creators. The `basis_points` field is the canonical value
/// (1 basis point = 0.01%), while `percent` is a convenience representation.
#[derive(Serialize, Deserialize, Debug)]
pub struct Royalty {
    /// The royalty distribution model (e.g., `Creators`, `Fanout`, `Single`)
    pub royalty_model: RoyaltyModel,
    /// The target address for royalty payments (if using `Single` model)
    pub target: Option<String>,
    /// The royalty percentage as a decimal (e.g., `0.042` = 4.2%)
    pub percent: f64,
    /// The royalty in basis points (e.g., `420` = 4.2%)
    pub basis_points: u32,
    /// Whether the primary sale has occurred (affects royalty enforcement)
    pub primary_sale_happened: bool,
    /// Whether the royalty configuration is locked and cannot be changed
    pub locked: bool,
}

/// Ownership details for a digital asset.
///
/// Includes the current owner, delegation status, and whether the asset is frozen
/// (preventing transfers).
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Ownership {
    /// Whether the asset is frozen (transfers are blocked)
    pub frozen: bool,
    /// Whether the asset has been delegated to another account
    pub delegated: bool,
    /// The delegate's public key, if the asset is delegated
    pub delegate: Option<String>,
    /// The ownership model (e.g., `Single` for NFTs, `Token` for fungible tokens)
    pub ownership_model: OwnershipModel,
    /// The current owner's public key (base-58 encoded)
    pub owner: String,
}

/// Use/utility tracking for a digital asset.
///
/// Some NFTs have limited uses (e.g., redeemable tickets, consumable items).
/// This tracks how many uses remain out of the total allocation.
#[derive(Serialize, Deserialize, Debug)]
pub struct Uses {
    /// How uses are consumed (`Burn`, `Multiple`, `Single`)
    pub use_method: UseMethod,
    /// The number of uses remaining
    pub remaining: u64,
    /// The total number of uses originally allocated
    pub total: u64,
}

/// Edition supply information for a digital asset.
///
/// Tracks the print supply for Metaplex master edition NFTs. A master edition
/// can produce a limited or unlimited number of prints (editions).
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Supply {
    /// Maximum number of editions that can be printed (`None` or `0` = unlimited)
    pub print_max_supply: Option<u64>,
    /// The number of editions that have been printed so far
    pub print_current_supply: Option<u64>,
    /// The edition nonce used for PDA derivation
    pub edition_nonce: Option<u64>,
    /// The edition number (for printed editions, not master editions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition_number: Option<u64>,
    /// The mint address of the master edition (for printed editions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_edition_mint: Option<String>,
}

/// Group definition data for an asset that defines a collection or group.
///
/// Present on assets that act as collection-level entities, providing metadata
/// about the group they define (e.g., collection size).
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GroupDefinition {
    /// The group type key (e.g., `"collection"`)
    pub group_key: String,
    /// The group identifier value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_value: Option<String>,
    /// The number of assets in this group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// The asset ID bytes for this group definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<Vec<u8>>,
}

/// System information for an asset
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

/// Filter for created_at timestamps
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CreatedAtFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<i64>,
}

/// MPL Core collection information for an asset.
///
/// Present on MPL Core collection assets, providing stats about the
/// collection's minting activity and current membership.
#[derive(Serialize, Deserialize, Debug)]
pub struct MplCoreInfo {
    /// Total number of assets minted into this collection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_minted: Option<i32>,
    /// Current number of assets in this collection (minted minus burned)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_size: Option<i32>,
    /// The version of the plugins JSON schema
    pub plugins_json_version: Option<i32>,
}

/// Negation filter for `searchAssets` to exclude specific entities.
///
/// Used with the `not` field on [`SearchAssets`] to exclude assets matching
/// certain collections, owners, creators, or authorities from search results.
#[derive(Serialize, Deserialize, Debug)]
pub struct NotFilter {
    /// Collection addresses to exclude
    pub collections: Option<Vec<String>>,
    /// Owner public keys to exclude (as raw 32-byte public key bytes)
    pub owners: Option<Vec<Vec<u8>>>,
    /// Creator public keys to exclude (as raw 32-byte public key bytes)
    pub creators: Option<Vec<Vec<u8>>>,
    /// Authority public keys to exclude (as raw 32-byte public key bytes)
    pub authorities: Option<Vec<Vec<u8>>>,
}
/// An SPL token account returned by the `getTokenAccounts` DAS API method.
///
/// Represents a single token account holding a specific token (identified by `mint`)
/// on behalf of an `owner`. Includes balance, delegation, and freeze status.
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenAccount {
    /// The token account address (base-58 encoded)
    pub address: String,
    /// The mint address of the token held in this account
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,
    /// The wallet that owns this token account
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// The token balance in the smallest unit (before decimal adjustment)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
    /// The delegate authorized to transfer tokens from this account
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegate: Option<String>,
    /// The amount of tokens the delegate is authorized to transfer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegated_amount: Option<u64>,
    /// Token-2022 extension data, if this account uses the Token Extensions program
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_extensions: Option<Value>,
    /// Whether this token account is frozen (transfers are blocked)
    pub frozen: bool,
}

/// A printed edition of a Metaplex master edition NFT.
///
/// Returned as items in the [`EditionsList`] from `getNftEditions`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Edition {
    /// The mint address of this edition
    pub mint: String,
    /// The edition PDA address
    pub edition_address: String,
    /// The edition number (sequential from 1)
    pub edition: Option<u64>,
}

/// Request body for the Helius Mint API to create a compressed NFT (cNFT).
///
/// Provides all the metadata and configuration needed to mint a new compressed NFT
/// via the Helius mint API endpoint. The API handles Merkle tree allocation and
/// the minting transaction.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MintCompressedNftRequest {
    /// The display name of the NFT
    pub name: String,
    /// The ticker symbol for the NFT
    pub symbol: String,
    /// A human-readable description of the NFT
    pub description: String,
    /// The wallet address that will own the minted NFT
    pub owner: String,
    /// An optional delegate authority for the NFT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegate: Option<String>,
    /// The collection address to mint the NFT into
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    /// URI pointing to off-chain JSON metadata (if not using inline metadata)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Trait attributes for the NFT (e.g., `[{"trait_type": "Color", "value": "Blue"}]`)
    pub attributes: Vec<Attribute>,
    /// URL of the NFT's image
    #[serde(rename = "imageUrl", skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// URL of the project's website
    #[serde(rename = "externalUrl", skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
    /// Secondary sale royalty in basis points (e.g., `500` = 5%)
    #[serde(rename = "sellerFeeBasisPoints", skip_serializing_if = "Option::is_none")]
    pub seller_fee_basis_points: Option<u32>,
    /// The list of creators and their share percentages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creators: Option<Vec<Creator>>,
    /// If `true`, waits for the minting transaction to be confirmed before returning
    #[serde(rename = "confirmTransaction", skip_serializing_if = "Option::is_none")]
    pub confirm_transaction: Option<bool>,
}

/// Response from the Helius Mint API after creating a compressed NFT.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MintResponse {
    /// The transaction signature of the minting transaction (base-58 encoded)
    pub signature: String,
    /// Whether the NFT was successfully minted
    pub minted: bool,
    /// The derived asset ID of the newly minted compressed NFT
    #[serde(rename = "assetId")]
    pub asset_id: Option<String>,
}

/// Options for the `getPriorityFeeEstimate` Helius RPC method.
///
/// Controls which fee data to return and how the estimate is calculated.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetPriorityFeeEstimateOptions {
    /// The desired priority level (`Min`, `Low`, `Medium`, `High`, `VeryHigh`, `UnsafeMax`, `Default`)
    pub priority_level: Option<PriorityLevel>,
    /// If `true`, returns fee estimates for all priority levels
    pub include_all_priority_fee_levels: Option<bool>,
    /// The encoding of the `transaction` field in the request
    pub transaction_encoding: Option<UiTransactionEncoding>,
    /// Number of recent slots to analyze for fee estimation (default: 150)
    pub lookback_slots: Option<u8>,
    /// If `true`, returns Helius's recommended priority fee
    pub recommended: Option<bool>,
    /// If `true`, includes vote transactions in the fee calculation
    pub include_vote: Option<bool>,
}

/// Request parameters for the `getPriorityFeeEstimate` Helius RPC method.
///
/// Provide either a serialized `transaction` or a list of `account_keys` to get
/// priority fee estimates based on recent network activity for those accounts.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetPriorityFeeEstimateRequest {
    /// A serialized transaction to estimate priority fees for
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<String>,
    /// Account public keys to estimate priority fees for (alternative to `transaction`)
    #[serde(rename = "accountKeys", skip_serializing_if = "Option::is_none")]
    pub account_keys: Option<Vec<String>>,
    /// Options controlling the fee estimation behavior
    pub options: Option<GetPriorityFeeEstimateOptions>,
}

/// Priority fee levels in micro-lamports per compute unit.
///
/// Each level represents a percentile of recent priority fees observed on the network.
/// Higher levels provide faster transaction inclusion but cost more.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MicroLamportPriorityFeeLevels {
    /// Minimum observed priority fee
    pub min: f64,
    /// 25th percentile priority fee
    pub low: f64,
    /// 50th percentile (median) priority fee
    pub medium: f64,
    /// 75th percentile priority fee
    pub high: f64,
    /// 95th percentile priority fee
    #[serde(rename = "veryHigh")]
    pub very_high: f64,
    /// Maximum observed priority fee (use with caution — may overpay significantly)
    #[serde(rename = "unsafeMax")]
    pub unsafe_max: f64,
}

/// Response from the `getPriorityFeeEstimate` Helius RPC method.
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetPriorityFeeEstimateResponse {
    /// The recommended priority fee in micro-lamports per compute unit
    pub priority_fee_estimate: Option<f64>,
    /// Fee estimates at all priority levels (only present when `include_all_priority_fee_levels` is `true`)
    pub priority_fee_levels: Option<MicroLamportPriorityFeeLevels>,
}

/// A Helius webhook configuration for receiving real-time Solana on-chain event notifications.
///
/// Webhooks deliver transaction data to your HTTP endpoint whenever matching on-chain
/// events occur. Supports enhanced (parsed), raw, and Discord webhook types.
///
/// Webhook events are charged at 1 credit per event. Editing, adding, or deleting
/// a webhook via the API costs 100 credits per request.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    /// The unique identifier for this webhook
    #[serde(rename = "webhookID")]
    pub webhook_id: String,
    /// The wallet address associated with this webhook
    pub wallet: String,
    /// The project ID that owns this webhook
    pub project: String,
    /// The URL where webhook events will be delivered
    #[serde(rename = "webhookURL")]
    pub webhook_url: String,
    /// Transaction types to filter for (only for enhanced webhooks; raw webhooks ignore this)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_types: Vec<TransactionType>,
    /// The Solana addresses to monitor for transactions
    pub account_addresses: Vec<String>,
    /// The webhook type (`Enhanced`, `EnhancedDevnet`, `Raw`, `RawDevnet`, `Discord`, `DiscordDevnet`)
    pub webhook_type: WebhookType,
    /// An optional authorization header value sent with each webhook request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    /// Filter by transaction status (`All`, `Success`, `Failed`)
    #[serde(default)]
    pub txn_status: TransactionStatus,
    /// The encoding for raw webhook payloads (default: `JsonParsed`)
    #[serde(default)]
    pub encoding: AccountWebhookEncoding,
    /// Whether the webhook is actively receiving deliveries. Defaults to `true`.
    /// Webhooks may be automatically disabled if the endpoint has a high failure rate.
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_active() -> bool {
    true
}

/// Request body for creating a new Helius webhook.
///
/// Creates a webhook that delivers real-time on-chain event notifications to
/// your specified URL. You can monitor up to 100,000 addresses per webhook.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhookRequest {
    /// The URL where webhook events will be delivered
    #[serde(rename = "webhookURL")]
    pub webhook_url: String,
    /// Transaction types to filter for (only for enhanced webhooks)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_types: Vec<TransactionType>,
    /// The Solana addresses to monitor for transactions
    pub account_addresses: Vec<String>,
    /// The webhook type (`Enhanced`, `EnhancedDevnet`, `Raw`, `RawDevnet`, `Discord`, `DiscordDevnet`)
    pub webhook_type: WebhookType,
    /// An optional authorization header value sent with each webhook request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    /// Filter by transaction status (`All`, `Success`, `Failed`)
    #[serde(default)]
    pub txn_status: TransactionStatus,
    /// The encoding for raw webhook payloads (default: `JsonParsed`)
    #[serde(default)]
    pub encoding: AccountWebhookEncoding,
}

/// Request body for creating a collection-based Helius webhook.
///
/// Automatically monitors all addresses in a given NFT collection, rather than
/// specifying individual addresses. The collection is identified by mint address
/// or first verified creator.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionWebhookRequest {
    /// The collection identifier (by mint address or first verified creator)
    pub collection_query: CollectionIdentifier,
    /// The URL where webhook events will be delivered
    #[serde(rename = "webhookURL")]
    pub webhook_url: String,
    /// Transaction types to filter for (only for enhanced webhooks)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_types: Vec<TransactionType>,
    /// Additional Solana addresses to monitor alongside the collection
    pub account_addresses: Vec<String>,
    /// The webhook type (`Enhanced`, `EnhancedDevnet`, `Raw`, `RawDevnet`, `Discord`, `DiscordDevnet`)
    pub webhook_type: WebhookType,
    /// An optional authorization header value sent with each webhook request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    /// Filter by transaction status (`All`, `Success`, `Failed`)
    #[serde(default)]
    pub txn_status: TransactionStatus,
    /// The encoding for raw webhook payloads (default: `JsonParsed`)
    #[serde(default)]
    pub encoding: AccountWebhookEncoding,
}

/// Request body for editing an existing Helius webhook.
///
/// Updates an existing webhook's configuration. The `webhook_id` is used for
/// routing but is not serialized into the request body.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EditWebhookRequest {
    /// The ID of the webhook to edit (not serialized — used in the URL path)
    #[serde(skip_serializing)]
    pub webhook_id: String,
    /// The URL where webhook events will be delivered
    #[serde(rename = "webhookURL")]
    pub webhook_url: String,
    /// Transaction types to filter for (only for enhanced webhooks)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_types: Vec<TransactionType>,
    /// The Solana addresses to monitor for transactions
    pub account_addresses: Vec<String>,
    /// The webhook type (`Enhanced`, `EnhancedDevnet`, `Raw`, `RawDevnet`, `Discord`, `DiscordDevnet`)
    pub webhook_type: WebhookType,
    /// An optional authorization header value sent with each webhook request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    /// Filter by transaction status (`All`, `Success`, `Failed`)
    #[serde(default)]
    pub txn_status: TransactionStatus,
    /// The encoding for raw webhook payloads
    #[serde(default)]
    pub encoding: AccountWebhookEncoding,
}

/// Request body for toggling a webhook on or off.
///
/// Enables or disables a webhook without deleting it. Use this to re-enable a webhook
/// that was automatically disabled due to a high endpoint failure rate, or to temporarily
/// pause deliveries. After re-enabling, the webhook enters a 24-hour grace period during
/// which it will not be automatically disabled again.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToggleWebhookRequest {
    /// The ID of the webhook to toggle (not serialized — used in the URL path)
    #[serde(skip_serializing)]
    pub webhook_id: String,
    /// Set to `true` to enable the webhook or `false` to disable it
    pub active: bool,
}

/// Configuration for creating and sending a smart transaction via Helius.
///
/// Smart transactions automatically handle priority fee estimation, compute unit
/// optimization, and transaction retry logic. The SDK simulates the transaction
/// to determine optimal compute units, applies a priority fee, and sends with
/// retry handling.
pub struct CreateSmartTransactionConfig {
    /// The instructions to include in the transaction
    pub instructions: Vec<Instruction>,
    /// The signers for the transaction (first signer is the fee payer by default)
    pub signers: Vec<Arc<dyn Signer>>,
    /// Address lookup tables for v0 transactions (enables more accounts per transaction)
    pub lookup_tables: Option<Vec<AddressLookupTableAccount>>,
    /// An optional separate fee payer (defaults to the first signer)
    pub fee_payer: Option<Arc<dyn Signer>>,
    /// Maximum priority fee **rate** in micro-lamports per compute unit (prevents overpaying during
    /// fee spikes). Applies to every version. On [`TransactionVersion::V1`] this caps the per-CU
    /// rate that is then converted to the total-lamport fee, and `priority_fee_lamports_cap` caps
    /// that resulting total — both apply.
    pub priority_fee_cap: Option<u64>,
    /// Multiplier applied to simulated compute units as a safety buffer (default: `1.25`)
    pub cu_buffer_multiplier: Option<f32>,
    /// The transaction format to build. Defaults to [`TransactionVersion::Auto`] (legacy/v0).
    /// Set to [`TransactionVersion::V1`] to build a larger Transaction v1 (SIMD-0385); `V1` is
    /// incompatible with `lookup_tables`.
    pub version: TransactionVersion,
    /// Maximum **total** priority fee in lamports, applied to [`TransactionVersion::V1`]
    /// transactions (whose fee is a flat lamport amount, not a per-CU rate) to cap absolute spend.
    /// Applied after `priority_fee_cap`: the per-CU rate is capped by `priority_fee_cap`, converted
    /// to a total-lamport fee, then capped by this — both apply on V1.
    pub priority_fee_lamports_cap: Option<u64>,
    /// Maximum account data, in bytes, the transaction may load. Applies to
    /// [`TransactionVersion::V1`] only, which carries it in the message header config.
    ///
    /// Defaults to [`MAX_LOADED_ACCOUNTS_DATA_SIZE_BYTES`] (64 MiB), matching the implicit budget
    /// legacy and v0 transactions get. Lower it to reduce the transaction's cost. Must be between
    /// 1 and that maximum; an out-of-range value fails the build.
    ///
    /// [`MAX_LOADED_ACCOUNTS_DATA_SIZE_BYTES`]: crate::optimized_transaction::MAX_LOADED_ACCOUNTS_DATA_SIZE_BYTES
    pub loaded_accounts_data_size_limit: Option<u32>,
}

impl Default for CreateSmartTransactionConfig {
    fn default() -> Self {
        Self {
            instructions: Vec::new(),
            signers: Vec::new(),
            lookup_tables: None,
            fee_payer: None,
            priority_fee_cap: None,
            cu_buffer_multiplier: Some(1.25),
            version: TransactionVersion::Auto,
            priority_fee_lamports_cap: None,
            loaded_accounts_data_size_limit: None,
        }
    }
}

impl CreateSmartTransactionConfig {
    /// Creates a new smart transaction config with the given instructions and signers.
    pub fn new(instructions: Vec<Instruction>, signers: Vec<Arc<dyn Signer>>) -> Self {
        Self {
            instructions,
            signers,
            lookup_tables: None,
            fee_payer: None,
            priority_fee_cap: None,
            cu_buffer_multiplier: None,
            version: TransactionVersion::Auto,
            priority_fee_lamports_cap: None,
            loaded_accounts_data_size_limit: None,
        }
    }

    /// Sets the address lookup tables (legacy/v0 only).
    pub fn with_lookup_tables(mut self, lookup_tables: Vec<AddressLookupTableAccount>) -> Self {
        self.lookup_tables = Some(lookup_tables);
        self
    }

    /// Sets a separate fee payer.
    pub fn with_fee_payer(mut self, fee_payer: Arc<dyn Signer>) -> Self {
        self.fee_payer = Some(fee_payer);
        self
    }

    /// Sets the maximum priority fee rate, in micro-lamports per compute unit.
    pub fn with_priority_fee_cap(mut self, priority_fee_cap: u64) -> Self {
        self.priority_fee_cap = Some(priority_fee_cap);
        self
    }

    /// Sets the compute-unit safety buffer applied to the simulated estimate.
    pub fn with_cu_buffer_multiplier(mut self, cu_buffer_multiplier: f32) -> Self {
        self.cu_buffer_multiplier = Some(cu_buffer_multiplier);
        self
    }

    /// Sets the maximum total priority fee in lamports (Transaction v1 only).
    pub fn with_priority_fee_lamports_cap(mut self, priority_fee_lamports_cap: u64) -> Self {
        self.priority_fee_lamports_cap = Some(priority_fee_lamports_cap);
        self
    }

    /// Sets the loaded-accounts data-size limit in bytes (Transaction v1 only).
    ///
    /// See [`CreateSmartTransactionConfig::loaded_accounts_data_size_limit`].
    pub fn with_loaded_accounts_data_size_limit(mut self, loaded_accounts_data_size_limit: u32) -> Self {
        self.loaded_accounts_data_size_limit = Some(loaded_accounts_data_size_limit);
        self
    }
}

/// A timeout configuration for smart transaction polling.
///
/// Wraps a [`Duration`] and defaults to 60 seconds. Used to control how long
/// the SDK waits for a smart transaction to be confirmed before giving up.
pub struct Timeout {
    /// The maximum time to wait for transaction confirmation
    pub duration: Duration,
}

impl Default for Timeout {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(60),
        }
    }
}

impl From<Timeout> for Duration {
    fn from(timeout: Timeout) -> Self {
        timeout.duration
    }
}

/// Full configuration for a smart transaction, including creation, send options, and timeout.
///
/// Combines [`CreateSmartTransactionConfig`] with Solana send options and a polling timeout.
/// Used internally by `send_smart_transaction`.
pub struct SmartTransactionConfig {
    /// Configuration for building the transaction (instructions, signers, lookup tables, etc.)
    pub create_config: CreateSmartTransactionConfig,
    /// Solana RPC send options (preflight checks, commitment, retries)
    pub send_options: RpcSendTransactionConfig,
    /// How long to poll for transaction confirmation before timing out
    pub timeout: Timeout,
}

impl SmartTransactionConfig {
    /// Creates a new smart transaction config with the given instructions, signers, and timeout.
    pub fn new(instructions: Vec<Instruction>, signers: Vec<Arc<dyn Signer>>, timeout: Timeout) -> Self {
        Self {
            create_config: CreateSmartTransactionConfig::new(instructions, signers),
            send_options: RpcSendTransactionConfig::default(),
            timeout,
        }
    }
}

/// Configuration for creating a smart transaction using raw 32-byte seeds as signers.
///
/// Similar to [`CreateSmartTransactionConfig`] but accepts raw seed bytes instead of
/// `Signer` trait objects. Useful when deriving keypairs from seeds (e.g., in deterministic
/// wallet implementations).
#[derive(Clone)]
pub struct CreateSmartTransactionSeedConfig {
    /// The instructions to include in the transaction
    pub instructions: Vec<Instruction>,
    /// Raw 32-byte seeds for deriving signer keypairs
    pub signer_seeds: Vec<[u8; 32]>,
    /// An optional raw seed for deriving a separate fee payer keypair
    pub fee_payer_seed: Option<[u8; 32]>,
    /// Address lookup tables for v0 transactions
    pub lookup_tables: Option<Vec<AddressLookupTableAccount>>,
    /// Maximum priority fee in micro-lamports
    pub priority_fee_cap: Option<u64>,
    /// Multiplier applied to simulated compute units as a safety buffer (default: `1.25`)
    pub cu_buffer_multiplier: Option<f32>,
    /// The transaction format to build. Defaults to [`TransactionVersion::Auto`] (legacy/v0).
    /// Set to [`TransactionVersion::V1`] to build a larger Transaction v1; incompatible with
    /// `lookup_tables`.
    pub version: TransactionVersion,
    /// Maximum **total** priority fee in lamports, applied to [`TransactionVersion::V1`]
    /// transactions.
    pub priority_fee_lamports_cap: Option<u64>,
    /// Maximum account data, in bytes, the transaction may load ([`TransactionVersion::V1`] only).
    ///
    /// See [`CreateSmartTransactionConfig::loaded_accounts_data_size_limit`].
    pub loaded_accounts_data_size_limit: Option<u32>,
}

impl Default for CreateSmartTransactionSeedConfig {
    fn default() -> Self {
        Self {
            instructions: Vec::new(),
            signer_seeds: Vec::new(),
            fee_payer_seed: None,
            lookup_tables: None,
            priority_fee_cap: None,
            cu_buffer_multiplier: Some(1.25),
            version: TransactionVersion::Auto,
            priority_fee_lamports_cap: None,
            loaded_accounts_data_size_limit: None,
        }
    }
}

impl CreateSmartTransactionSeedConfig {
    pub fn new(instructions: Vec<Instruction>, signer_seeds: Vec<[u8; 32]>) -> Self {
        Self {
            instructions,
            signer_seeds,
            fee_payer_seed: None,
            lookup_tables: None,
            priority_fee_cap: None,
            cu_buffer_multiplier: None,
            version: TransactionVersion::Auto,
            priority_fee_lamports_cap: None,
            loaded_accounts_data_size_limit: None,
        }
    }

    pub fn with_fee_payer_seed(mut self, seed: [u8; 32]) -> Self {
        self.fee_payer_seed = Some(seed);
        self
    }

    pub fn with_lookup_tables(mut self, lookup_tables: Vec<AddressLookupTableAccount>) -> Self {
        self.lookup_tables = Some(lookup_tables);
        self
    }

    /// Selects [`TransactionVersion::V1`] for this transaction (larger transactions, header-config
    /// fees, no lookup tables).
    pub fn with_v1(mut self) -> Self {
        self.version = TransactionVersion::V1;
        self
    }

    /// Sets the loaded-accounts data-size limit in bytes (Transaction v1 only).
    ///
    /// See [`CreateSmartTransactionConfig::loaded_accounts_data_size_limit`].
    pub fn with_loaded_accounts_data_size_limit(mut self, loaded_accounts_data_size_limit: u32) -> Self {
        self.loaded_accounts_data_size_limit = Some(loaded_accounts_data_size_limit);
        self
    }
}

/// Options for sending via Sender
///
/// This struct is `#[non_exhaustive]`: construct it via [`SenderSendOptions::default`]
/// or [`SenderSendOptions::new`] and the `with_*` builder methods rather than a struct
/// literal, so that adding future fields remains non-breaking.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct SenderSendOptions {
    /// Must match a key in SENDER_ENDPOINTS (e.g., "Default", "US_EAST")
    pub region: String,
    /// If `false` (default), uses the **Sender Max** tier (multi-path routing +
    /// priority auction, 0.001 SOL minimum tip). If `true`, uses SWQOS-only
    /// (lower 0.000005 SOL minimum tip) and appends `?swqos_only=true` to `/fast`.
    pub swqos_only: bool,
    /// Whether to skip Solana's preflight checks on the Sender side.
    ///
    /// Sender no longer *requires* `skip_preflight = true`; this is now a
    /// caller-controlled passthrough. Defaults to `true` to preserve prior
    /// behavior, but you may set it to `false` to have preflight run.
    pub skip_preflight: bool,
    /// Poll settings
    pub poll_timeout_ms: u64,
    pub poll_interval_ms: u64,
    /// Hard ceiling, in lamports, on the tip the SDK derives automatically. Defaults to
    /// [`DEFAULT_MAX_TIP_LAMPORTS`] (0.01 SOL).
    ///
    /// Bounds only the *derived* tip; a tip you build into the instructions yourself is untouched.
    /// Must be at least the tier's minimum ([`MIN_TIP_LAMPORTS_MAX`], or [`MIN_TIP_LAMPORTS_SWQOS`]
    /// when `swqos_only`) — a lower ceiling is unsatisfiable and fails the send rather than paying
    /// above it.
    ///
    /// [`DEFAULT_MAX_TIP_LAMPORTS`]: crate::optimized_transaction::DEFAULT_MAX_TIP_LAMPORTS
    /// [`MIN_TIP_LAMPORTS_MAX`]: crate::optimized_transaction::MIN_TIP_LAMPORTS_MAX
    /// [`MIN_TIP_LAMPORTS_SWQOS`]: crate::optimized_transaction::MIN_TIP_LAMPORTS_SWQOS
    pub max_tip_lamports: u64,
}

impl Default for SenderSendOptions {
    fn default() -> Self {
        Self {
            region: "Default".to_string(),
            swqos_only: false,
            skip_preflight: true,
            poll_timeout_ms: 60_000,
            poll_interval_ms: 2_000,
            max_tip_lamports: crate::optimized_transaction::DEFAULT_MAX_TIP_LAMPORTS,
        }
    }
}

impl SenderSendOptions {
    /// Creates a new `SenderSendOptions` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Sender region (must match a key in `SENDER_ENDPOINTS`).
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    /// Sets the SWQOS-only flag.
    pub fn with_swqos_only(mut self, swqos_only: bool) -> Self {
        self.swqos_only = swqos_only;
        self
    }

    /// Sets whether Sender skips Solana's preflight checks.
    pub fn with_skip_preflight(mut self, skip_preflight: bool) -> Self {
        self.skip_preflight = skip_preflight;
        self
    }

    /// Sets the poll timeout in milliseconds.
    pub fn with_poll_timeout_ms(mut self, poll_timeout_ms: u64) -> Self {
        self.poll_timeout_ms = poll_timeout_ms;
        self
    }

    /// Sets the poll interval in milliseconds.
    pub fn with_poll_interval_ms(mut self, poll_interval_ms: u64) -> Self {
        self.poll_interval_ms = poll_interval_ms;
        self
    }

    /// Sets the hard ceiling on the auto-derived tip, in lamports.
    ///
    /// See [`SenderSendOptions::max_tip_lamports`]. Must be at least the tier's minimum tip.
    pub fn with_max_tip_lamports(mut self, max_tip_lamports: u64) -> Self {
        self.max_tip_lamports = max_tip_lamports;
        self
    }
}

/// Specifies a sub-range of account data to return.
///
/// Used with `getProgramAccountsV2` and `getTokenAccountsByOwnerV2` to reduce
/// response size by returning only a slice of each account's data buffer instead
/// of the full contents.
///
/// # Fields
///
/// - `length`: Number of bytes to return
/// - `offset`: Byte offset from the start of the account data to begin the slice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSlice {
    /// Number of bytes to return
    pub length: u64,
    /// Byte offset from the start of the account data
    pub offset: u64,
}

/// Memory comparison filter for `getProgramAccountsV2`.
///
/// Matches accounts whose data at the specified byte offset contains the given
/// base58-encoded bytes. Commonly used to filter program accounts by discriminator,
/// owner pubkey, or other fixed-position fields.
///
/// # Fields
///
/// - `offset`: Byte offset into account data where comparison begins
/// - `bytes`: Base58-encoded bytes to match against
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpaMemcmp {
    /// Byte offset into account data where comparison begins
    pub offset: u64,
    /// Base58-encoded bytes to match against
    pub bytes: String,
}

/// Configuration for [`getProgramAccountsV2`](https://www.helius.dev/docs/solana-rpc-nodes/helius-exclusive-methods/get-program-accounts-v2).
///
/// An enhanced version of Solana's `getProgramAccounts` with cursor-based pagination
/// and delta queries. This avoids the performance issues of the standard RPC method
/// when programs have large numbers of accounts.
///
/// # Key Enhancements
///
/// - **Cursor-based pagination**: Use `limit` and `pagination_key` to page through
///   large result sets without missed or duplicate accounts
/// - **Delta queries**: Use `changed_since_slot` to retrieve only accounts that have
///   been modified since a given slot, enabling efficient incremental syncing
///
/// # Fields
///
/// - `commitment`: Commitment level for the query
/// - `min_context_slot`: Minimum slot at which the request can be evaluated
/// - `with_context`: Whether to wrap the result in an RPC context object
/// - `encoding`: Account data encoding (`base58`, `base64`, `base64+zstd`, `jsonParsed`)
/// - `data_slice`: Sub-range of account data to return (reduces response size)
/// - `limit`: Maximum number of accounts per page (default: 1000)
/// - `pagination_key`: Cursor from a previous response to fetch the next page
/// - `changed_since_slot`: Only return accounts modified after this slot (for delta queries)
/// - `filters`: Standard `getProgramAccounts` filters (`memcmp`, `dataSize`)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetProgramAccountsV2Config {
    /// Commitment level for the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<CommitmentLevel>,

    /// Minimum slot at which the request can be evaluated
    #[serde(rename = "minContextSlot", skip_serializing_if = "Option::is_none")]
    pub min_context_slot: Option<u64>,
    /// Whether to wrap the result in an RPC context object
    #[serde(rename = "withContext", skip_serializing_if = "Option::is_none")]
    pub with_context: Option<bool>,

    /// Account data encoding (`base58`, `base64`, `base64+zstd`, `jsonParsed`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    /// Sub-range of account data to return (reduces response size)
    #[serde(rename = "dataSlice", skip_serializing_if = "Option::is_none")]
    pub data_slice: Option<DataSlice>,

    /// Maximum number of accounts per page (default: 1000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    /// Cursor from a previous response to fetch the next page
    #[serde(rename = "paginationKey", skip_serializing_if = "Option::is_none")]
    pub pagination_key: Option<String>,
    /// Only return accounts modified after this slot (for delta queries)
    #[serde(rename = "changedSinceSlot", skip_serializing_if = "Option::is_none")]
    pub changed_since_slot: Option<u64>,

    /// Standard `getProgramAccounts` filters (`memcmp`, `dataSize`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<GpaFilter>>,

    /// Client-side only (never sent to the server): the maximum number of pages
    /// [`get_all_program_accounts`](crate::rpc_client::RpcClient::get_all_program_accounts) will
    /// fetch before stopping. `None` uses [`DEFAULT_MAX_AUTO_PAGINATION_PAGES`](crate::rpc_client::DEFAULT_MAX_AUTO_PAGINATION_PAGES).
    /// Raise it to fetch a larger result set, or lower it to bound work. Ignored by the
    /// single-page `get_program_accounts_v2`.
    #[serde(skip)]
    pub max_pages: Option<usize>,
}

/// Request type for `getProgramAccountsV2`: a tuple of `(program_id, config)`.
pub type GetProgramAccountsV2Request = (String, GetProgramAccountsV2Config);

/// A single account returned by `getProgramAccountsV2`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpaAccount {
    /// The account's public key (base58-encoded)
    pub pubkey: String,
    /// The account data and metadata
    pub account: AccountInfo,
}

/// On-chain account information returned by RPC V2 methods.
///
/// The `data` field format varies based on the requested encoding:
/// - `base58` / `base64` / `base64+zstd`: a JSON array `[encoded_string, encoding]`
/// - `jsonParsed`: a JSON object with parsed account data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Account balance in lamports
    pub lamports: u64,
    /// Program that owns this account (base58-encoded public key)
    pub owner: String,
    /// Account data (format depends on the requested encoding)
    pub data: Value,
    /// Whether the account contains an executable program
    pub executable: bool,
    /// Epoch at which this account will next owe rent
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: u64,
    /// Size of the account data in bytes
    #[serde(default)]
    pub space: Option<u64>,
}

/// Response from `getProgramAccountsV2`.
///
/// Contains a page of accounts and an optional pagination cursor for fetching the
/// next page. When `pagination_key` is `None`, all results have been returned.
///
/// When [`GetProgramAccountsV2Config::with_context`] is set to `true`, the API wraps the
/// response in a `{ context, value }` envelope. This struct transparently handles both
/// shapes via a custom deserializer — the `context` field is `Some` when `with_context`
/// is `true` and `None` otherwise.
#[derive(Debug, Clone, Serialize, Default)]
pub struct GetProgramAccountsV2Response {
    /// RPC context metadata (slot, API version). Present when `with_context` is `true`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<RpcContext>,
    /// The accounts matching the query for this page
    pub accounts: Vec<GpaAccount>,
    /// Cursor for the next page; `None` when no more results remain
    #[serde(rename = "paginationKey")]
    pub pagination_key: Option<String>,
    /// Total number of matching accounts across all pages
    #[serde(rename = "totalResults")]
    pub total_results: Option<u64>,
}

impl<'de> serde::Deserialize<'de> for GetProgramAccountsV2Response {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = serde_json::Value::deserialize(deserializer)?;

        // If `context` and `value` are present, the response is wrapped (withContext: true)
        if raw.get("context").is_some() && raw.get("value").is_some() {
            let context: RpcContext =
                serde_json::from_value(raw["context"].clone()).map_err(serde::de::Error::custom)?;
            let value = &raw["value"];
            Ok(Self {
                context: Some(context),
                accounts: serde_json::from_value(value.get("accounts").cloned().unwrap_or_default())
                    .map_err(serde::de::Error::custom)?,
                pagination_key: value.get("paginationKey").and_then(|v| v.as_str()).map(String::from),
                total_results: value.get("totalResults").and_then(|v| v.as_u64()),
            })
        } else {
            // Direct shape (withContext: false or omitted)
            Ok(Self {
                context: None,
                accounts: serde_json::from_value(raw.get("accounts").cloned().unwrap_or_default())
                    .map_err(serde::de::Error::custom)?,
                pagination_key: raw.get("paginationKey").and_then(|v| v.as_str()).map(String::from),
                total_results: raw.get("totalResults").and_then(|v| v.as_u64()),
            })
        }
    }
}

/// Configuration for [`getTokenAccountsByOwnerV2`](https://www.helius.dev/docs/solana-rpc-nodes/helius-exclusive-methods/get-token-accounts-by-owner-v2).
///
/// An enhanced version of Solana's `getTokenAccountsByOwner` with cursor-based
/// pagination and delta queries. Solves performance issues when wallets hold many
/// token accounts.
///
/// # Key Enhancements
///
/// - **Cursor-based pagination**: Use `limit` and `pagination_key` to page through
///   wallets with many token accounts
/// - **Delta queries**: Use `changed_since_slot` to retrieve only token accounts
///   that have been modified since a given slot
///
/// # Fields
///
/// - `commitment`: Commitment level for the query
/// - `min_context_slot`: Minimum slot at which the request can be evaluated
/// - `data_slice`: Sub-range of account data to return (reduces response size)
/// - `encoding`: Account data encoding (`base58`, `base64`, `base64+zstd`, `jsonParsed`)
/// - `limit`: Maximum number of token accounts per page
/// - `pagination_key`: Cursor from a previous response to fetch the next page
/// - `changed_since_slot`: Only return token accounts modified after this slot
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetTokenAccountsByOwnerV2Config {
    /// Commitment level for the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<CommitmentLevel>,
    /// Minimum slot at which the request can be evaluated
    #[serde(rename = "minContextSlot", skip_serializing_if = "Option::is_none")]
    pub min_context_slot: Option<u64>,

    /// Sub-range of account data to return (reduces response size)
    #[serde(rename = "dataSlice", skip_serializing_if = "Option::is_none")]
    pub data_slice: Option<DataSlice>,
    /// Account data encoding (`base58`, `base64`, `base64+zstd`, `jsonParsed`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,

    /// Maximum number of token accounts per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Cursor from a previous response to fetch the next page
    #[serde(rename = "paginationKey", skip_serializing_if = "Option::is_none")]
    pub pagination_key: Option<String>,
    /// Only return token accounts modified after this slot
    #[serde(rename = "changedSinceSlot", skip_serializing_if = "Option::is_none")]
    pub changed_since_slot: Option<u64>,

    /// Client-side only (never sent to the server): the maximum number of pages
    /// [`get_all_token_accounts_by_owner`](crate::rpc_client::RpcClient::get_all_token_accounts_by_owner)
    /// will fetch before stopping. `None` uses [`DEFAULT_MAX_AUTO_PAGINATION_PAGES`](crate::rpc_client::DEFAULT_MAX_AUTO_PAGINATION_PAGES).
    /// Raise it to fetch a larger result set, or lower it to bound work. Ignored by the
    /// single-page `get_token_accounts_by_owner_v2`.
    #[serde(skip)]
    pub max_pages: Option<usize>,
}

/// Request type for `getTokenAccountsByOwnerV2`: a tuple of `(owner_pubkey, filter, config)`.
pub type GetTokenAccountsByOwnerV2Request = (String, TokenAccountsOwnerFilter, GetTokenAccountsByOwnerV2Config);

/// RPC response context metadata.
///
/// Provides information about the slot at which the response was computed and the
/// API version of the node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcContext {
    /// The slot at which the response was computed
    pub slot: u64,
    /// The API version of the RPC node
    #[serde(rename = "apiVersion")]
    pub api_version: Option<String>,
}

/// A single token account returned by `getTokenAccountsByOwnerV2`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountRecord {
    /// The token account's public key (base58-encoded)
    pub pubkey: String,
    /// The token account data and metadata
    pub account: AccountInfo,
}

/// Response from `getTokenAccountsByOwnerV2`.
///
/// Contains a page of token accounts, an optional RPC context, and pagination
/// metadata. When `pagination_key` is `None`, all results have been returned.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetTokenAccountsByOwnerV2Response {
    /// RPC context metadata (slot, API version)
    pub context: Option<RpcContext>,
    /// The token accounts and inner pagination data
    pub value: GetTokenAccountsByOwnerV2Value,
    /// Cursor for the next page; `None` when no more results remain
    #[serde(rename = "paginationKey")]
    pub pagination_key: Option<String>,
    /// Total number of matching token accounts across all pages
    #[serde(rename = "totalResults")]
    pub total_results: Option<u64>,
}

/// Inner value of the `getTokenAccountsByOwnerV2` response.
///
/// Contains the token account records for the current page along with pagination
/// metadata and a count of accounts in this page.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetTokenAccountsByOwnerV2Value {
    /// Token accounts in this page
    pub accounts: Vec<TokenAccountRecord>,
    /// Cursor for the next page; `None` when no more results remain
    #[serde(rename = "paginationKey")]
    pub pagination_key: Option<String>,
    /// Number of accounts in this page
    pub count: u64,
}

/// Level of detail to include for each transaction in `getTransactionsForAddress` responses.
///
/// Controls whether the response includes only transaction signatures or full
/// transaction data (similar to `getTransaction`).
///
/// Defaults to `Signatures`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum TransactionDetails {
    /// Return only transaction signatures (default, smaller response)
    #[default]
    Signatures,
    /// Return full transaction data including instructions and metadata
    Full,
}

/// Sort order for transaction results from `getTransactionsForAddress`.
///
/// Defaults to `Desc` (newest first).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    /// Newest transactions first (default)
    #[default]
    Desc,
    /// Oldest transactions first
    Asc,
}

/// Filter transactions by execution status in `getTransactionsForAddress`.
///
/// Defaults to `Any` (no status filtering).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum TransactionStatusFilter {
    /// Include both successful and failed transactions (default)
    #[default]
    Any,
    /// Only include transactions that executed successfully
    Succeeded,
    /// Only include transactions that failed
    Failed,
}

/// Filter transactions by slot number range.
///
/// Used with `getTransactionsForAddress` to retrieve transactions from specific
/// slots. Slots are the basic unit of time in Solana (~400ms per slot).
///
/// All fields are optional and can be combined to create range queries.
/// Queries are inclusive for `gte`/`lte` and exclusive for `gt`/`lt`.
///
/// # Examples
///
/// ```ignore
/// // Get transactions from slot 150000000 onwards
/// let filter = SlotFilter {
///     gte: Some(150000000),
///     ..Default::default()
/// };
///
/// // Get transactions in a specific slot range
/// let filter = SlotFilter {
///     gte: Some(150000000),
///     lt: Some(150010000),
///     ..Default::default()
/// };
/// ```
///
/// # Fields
///
/// - `gte`: Greater than or equal to slot number (inclusive)
/// - `gt`: Greater than slot number (exclusive)
/// - `lte`: Less than or equal to slot number (inclusive)
/// - `lt`: Less than slot number (exclusive)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SlotFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<u64>,
}

/// Filter transactions by block timestamp (Unix time).
///
/// Used with `getTransactionsForAddress` to retrieve transactions from specific
/// time periods. Block timestamps are Unix timestamps (seconds since epoch).
///
/// All fields are optional and can be combined to create range queries.
/// Queries are inclusive for `gte`/`lte` and exclusive for `gt`/`lt`.
///
/// # Examples
///
/// ```ignore
/// // Get transactions from January 1, 2024 onwards
/// let filter = BlockTimeFilter {
///     gte: Some(1704067200),  // Unix timestamp for Jan 1, 2024
///     ..Default::default()
/// };
///
/// // Get transactions from a specific time range
/// let filter = BlockTimeFilter {
///     gte: Some(1704067200),  // Jan 1, 2024
///     lt: Some(1706745600),   // Feb 1, 2024
///     ..Default::default()
/// };
///
/// // Get transactions at an exact timestamp
/// let filter = BlockTimeFilter {
///     eq: Some(1704067200),
///     ..Default::default()
/// };
/// ```
///
/// # Fields
///
/// - `gte`: Greater than or equal to timestamp (inclusive)
/// - `gt`: Greater than timestamp (exclusive)
/// - `lte`: Less than or equal to timestamp (inclusive)
/// - `lt`: Less than timestamp (exclusive)
/// - `eq`: Equal to timestamp (exact match)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BlockTimeFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eq: Option<i64>,
}

/// Filter transactions by signature range.
///
/// Used with `getTransactionsForAddress` to retrieve transactions before or after
/// a specific transaction signature. Signatures are compared lexicographically
/// (alphabetically as base58 strings).
///
/// Useful for implementing cursor-based pagination when combined with `paginationToken`,
/// or for fetching transactions relative to a known transaction.
///
/// # Examples
///
/// ```ignore
/// // Get transactions after a specific signature
/// let filter = SignatureFilter {
///     gt: Some("5h6xBEauJ3PK6SWCZ1PGjBvj8vDdWG3KpwATGy1ARAXF...".to_string()),
///     ..Default::default()
/// };
///
/// // Get transactions in a signature range
/// let filter = SignatureFilter {
///     gte: Some("3jweEauJ3PK6SWCZ1PGjBvj8vDdWG3KpwATGy1ARAXF...".to_string()),
///     lt: Some("6k7xBEauJ3PK6SWCZ1PGjBvj8vDdWG3KpwATGy1ARAXF...".to_string()),
///     ..Default::default()
/// };
/// ```
///
/// # Fields
///
/// - `gte`: Greater than or equal to signature (inclusive, lexicographic order)
/// - `gt`: Greater than signature (exclusive, lexicographic order)
/// - `lte`: Less than or equal to signature (inclusive, lexicographic order)
/// - `lt`: Less than signature (exclusive, lexicographic order)
///
/// # Note
///
/// Signatures are base58-encoded strings and are compared lexicographically.
/// This means "3..." comes before "5..." which comes before "6...".
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignatureFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<String>,
}

/// Controls whether `getTransactionsForAddress` includes transactions from associated
/// token accounts owned by the queried address.
///
/// By default, only transactions that directly reference the address are returned.
/// Enabling token account inclusion broadens results to capture token transfers and
/// other SPL Token activity.
///
/// Defaults to `None`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum TokenAccountsFilter {
    /// Only return transactions that reference the provided address (default)
    #[default]
    None,
    /// Return transactions that reference either the provided address or modify the balance of a token account owned by the provided address (recommended)
    BalanceChanged,
    /// Return transactions that reference either the provided address or any token account owned by the provided address
    All,
}

/// Combined filters for `getTransactionsForAddress`.
///
/// Allows filtering transaction history by multiple criteria simultaneously.
/// All filters are optional and work together with AND logic (a transaction
/// must match all specified filters to be included).
///
/// # Filter Combination
///
/// Filters are combined with AND logic:
/// - A transaction must match ALL specified filters to be returned
/// - Omitted filters are ignored (no filtering on that criteria)
/// - Multiple filters can narrow results significantly
///
/// # Examples
///
/// ```ignore
/// // Get only successful transactions from a specific time range
/// let filters = GetTransactionsFilters {
///     block_time: Some(BlockTimeFilter {
///         gte: Some(1704067200),  // Jan 1, 2024
///         lt: Some(1706745600),   // Feb 1, 2024
///         ..Default::default()
///     }),
///     status: Some(TransactionStatusFilter::Succeeded),
///     ..Default::default()
/// };
///
/// // Get transactions in a slot range that modified token balances
/// let filters = GetTransactionsFilters {
///     slot: Some(SlotFilter {
///         gte: Some(150000000),
///         lt: Some(150010000),
///         ..Default::default()
///     }),
///     token_accounts: Some(TokenAccountsFilter::BalanceChanged),
///     ..Default::default()
/// };
///
/// // Get failed transactions after a specific signature
/// let filters = GetTransactionsFilters {
///     signature: Some(SignatureFilter {
///         gt: Some("5h6xBEau...".to_string()),
///         ..Default::default()
///     }),
///     status: Some(TransactionStatusFilter::Failed),
///     ..Default::default()
/// };
/// ```
///
/// # Performance Tips
///
/// - Slot-based filtering is fastest (slots are indexed efficiently)
/// - Time-based filtering is slower but more intuitive for users
/// - Combining multiple filters can significantly reduce result sets
/// - Use `token_accounts: BalanceChanged` to focus on economically significant transactions
///
/// # Fields
///
/// - `slot`: Filter by slot number range
/// - `block_time`: Filter by block timestamp (Unix time)
/// - `signature`: Filter by transaction signature range
/// - `status`: Filter by transaction success/failure status
/// - `token_accounts`: Include transactions from associated token accounts
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionsFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<SlotFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_time: Option<BlockTimeFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<SignatureFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TransactionStatusFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_accounts: Option<TokenAccountsFilter>,
}

/// Options for the `getTransactionsForAddress` RPC method.
///
/// Retrieves transaction history for a given Solana address with powerful filtering,
/// sorting, and pagination capabilities. Supports filtering by slot range, block time,
/// transaction status, and associated token accounts.
///
/// # Fields
///
/// - `transaction_details`: Level of detail (`Signatures` or `Full`)
/// - `sort_order`: Sort direction (`Desc` for newest first, `Asc` for oldest first)
/// - `limit`: Maximum number of transactions per page
/// - `pagination_token`: Cursor from a previous response for fetching the next page
/// - `commitment`: Commitment level for the query
/// - `filters`: Combined filters (slot, block time, signature, status, token accounts)
/// - `encoding`: Transaction encoding when `transaction_details` is `Full`
/// - `max_supported_transaction_version`: Maximum transaction version to return
/// - `min_context_slot`: Minimum slot at which the request can be evaluated
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionsForAddressOptions {
    /// Level of detail for returned transactions (`Signatures` or `Full`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_details: Option<TransactionDetails>,
    /// Sort direction (`Desc` for newest first, `Asc` for oldest first)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,
    /// Maximum number of transactions per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Cursor from a previous response for fetching the next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_token: Option<String>,
    /// Commitment level for the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<CommitmentLevel>,
    /// Combined filters (slot, block time, signature, status, token accounts)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<GetTransactionsFilters>,
    /// Transaction encoding when `transaction_details` is `Full`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<UiTransactionEncoding>,
    /// Maximum transaction version to return (`1` for Transaction v1 / Agave 4.2, `0` for v0).
    /// Defaults to `1` so v1 transactions are returned rather than triggering a version error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_supported_transaction_version: Option<u8>,
    /// Minimum slot at which the request can be evaluated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_context_slot: Option<u64>,
}

impl Default for GetTransactionsForAddressOptions {
    fn default() -> Self {
        Self {
            transaction_details: None,
            sort_order: None,
            limit: None,
            pagination_token: None,
            commitment: None,
            filters: None,
            encoding: None,
            // Accept up to Transaction v1 (Agave 4.2) by default; the SDK can parse v1.
            max_supported_transaction_version: Some(1),
            min_context_slot: None,
        }
    }
}

/// A transaction entry returned in "signatures" mode from `getTransactionsForAddress`.
///
/// Contains the transaction signature along with slot, timing, and status metadata.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSignatureEntry {
    /// The transaction signature (base-58 encoded)
    pub signature: String,
    /// The slot in which the transaction was processed
    pub slot: u64,
    /// Zero-based position of the transaction within its block
    pub transaction_index: u64,
    /// Transaction error, if any (Solana runtime error format)
    pub err: Option<serde_json::Value>,
    /// Memo associated with the transaction, if any
    pub memo: Option<String>,
    /// Estimated block production time as a Unix timestamp (seconds since epoch)
    pub block_time: Option<i64>,
    /// The transaction's confirmation status
    pub confirmation_status: Option<String>,
}

/// A transaction entry returned in "full" mode from `getTransactionsForAddress`.
///
/// Contains the full transaction data along with block-level metadata like slot,
/// transaction index, and block time.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FullTransactionEntry {
    /// The slot in which the transaction was processed
    pub slot: u64,
    /// Zero-based position of the transaction within its block
    pub transaction_index: u64,
    /// The encoded transaction object
    pub transaction: EncodedTransaction,
    /// Transaction status metadata (fees, balances, logs, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<UiTransactionStatusMeta>,
    /// Estimated block production time as a Unix timestamp (seconds since epoch)
    pub block_time: Option<i64>,
}

/// A single transaction entry from `getTransactionsForAddress`.
///
/// The variant depends on the `transaction_details` option:
/// - [`TransactionDetails::Signatures`]: deserializes as [`TransactionEntry::Signature`]
/// - [`TransactionDetails::Full`]: deserializes as [`TransactionEntry::Full`]
///
/// If the API returns a shape that doesn't match either known variant,
/// [`TransactionEntry::Unknown`] captures the raw JSON so deserialization
/// never fails silently.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum TransactionEntry {
    /// Full transaction data with block-level metadata
    Full(Box<FullTransactionEntry>),
    /// Lightweight signature entry with slot, timing, and status metadata
    Signature(TransactionSignatureEntry),
    /// Fallback for unrecognized response shapes (e.g., new API modes)
    Unknown(serde_json::Value),
}

impl Default for TransactionEntry {
    fn default() -> Self {
        TransactionEntry::Signature(TransactionSignatureEntry {
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

/// Response from `getTransactionsForAddress`.
///
/// Contains a page of transaction data and an optional pagination cursor. The format
/// of items in `data` depends on the `transaction_details` setting:
/// - `Signatures`: each item is a [`TransactionEntry::Signature`]
/// - `Full`: each item is a [`TransactionEntry::Full`]
///
/// When `pagination_token` is `None`, all results have been returned.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionsForAddressResponse {
    /// Transaction data for this page (signatures or full transactions depending on options)
    pub data: Vec<TransactionEntry>,
    /// Cursor for the next page; `None` when no more results remain
    pub pagination_token: Option<String>,
}

/// Request type for `getTransactionsForAddress`: a tuple of `(address, options)`.
pub type GetTransactionsForAddressRequest = (String, GetTransactionsForAddressOptions);

/// Direction of transfers relative to the queried address.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GetTransfersByAddressDirection {
    /// Incoming transfers only
    In,
    /// Outgoing transfers only
    Out,
    /// Incoming and outgoing transfers
    #[default]
    Any,
}

/// Native SOL grouping mode for `getTransfersByAddress`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GetTransfersByAddressSolMode {
    /// Merge native SOL and WSOL transfer activity
    #[default]
    Merged,
    /// Return native SOL and WSOL transfer activity separately
    Separate,
}

/// Filter transfers by raw amount.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TransferAmountFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<u64>,
}

/// Filter transfers by block timestamp (Unix seconds).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TransferBlockTimeFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<i64>,
}

/// Filter transfers by slot range.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TransferSlotFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gt: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gte: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lt: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lte: Option<u64>,
}

/// Combined filters for `getTransfersByAddress`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTransfersByAddressFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<TransferAmountFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_time: Option<TransferBlockTimeFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<TransferSlotFilter>,
}

/// Request config for the `getTransfersByAddress` RPC method.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTransfersByAddressConfig {
    /// Counterparty address to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with: Option<String>,
    /// Transfer direction relative to the queried address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<GetTransfersByAddressDirection>,
    /// Token mint address to filter by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,
    /// Native SOL grouping mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sol_mode: Option<GetTransfersByAddressSolMode>,
    /// Combined filters for amount, block time, and slot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<GetTransfersByAddressFilters>,
    /// Maximum number of transfers per page. The server accepts 1-100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Cursor from a previous response for fetching the next page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_token: Option<String>,
    /// Commitment level for the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<CommitmentLevel>,
    /// Sort direction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,
}

/// Request parameters for `getTransfersByAddress`.
///
/// Serializes to `[address]` when `config` is `None`, and `[address, config]` when
/// a config is provided.
#[derive(Debug, Clone, Default)]
pub struct GetTransfersByAddressRequest {
    pub address: String,
    pub config: Option<GetTransfersByAddressConfig>,
}

impl GetTransfersByAddressRequest {
    pub fn new(address: String, config: Option<GetTransfersByAddressConfig>) -> Self {
        Self { address, config }
    }
}

impl Serialize for GetTransfersByAddressRequest {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let len = if self.config.is_some() { 2 } else { 1 };
        let mut tuple = serializer.serialize_tuple(len)?;
        tuple.serialize_element(&self.address)?;
        if let Some(config) = &self.config {
            tuple.serialize_element(config)?;
        }
        tuple.end()
    }
}

/// Transfer event type returned by `getTransfersByAddress`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum GetTransfersByAddressTransferType {
    Transfer,
    Mint,
    Burn,
    Wrap,
    Unwrap,
    ChangeOwner,
    WithdrawWithheldFee,
}

/// Confirmation status returned by `getTransfersByAddress`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransferConfirmationStatus {
    Finalized,
    Confirmed,
}

/// A transfer returned by `getTransfersByAddress`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetTransfersByAddressTransfer {
    pub signature: String,
    pub slot: u64,
    pub block_time: i64,
    #[serde(rename = "type")]
    pub transfer_type: GetTransfersByAddressTransferType,
    pub from_user_account: Option<String>,
    pub to_user_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_token_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_token_account: Option<String>,
    pub mint: String,
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<String>,
    pub decimals: u8,
    pub ui_amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_ui_amount: Option<String>,
    pub confirmation_status: TransferConfirmationStatus,
    pub transaction_idx: u64,
    pub instruction_idx: u64,
    pub inner_instruction_idx: u64,
}

/// Response from `getTransfersByAddress`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTransfersByAddressResponse {
    pub data: Vec<GetTransfersByAddressTransfer>,
    pub pagination_token: Option<String>,
}

/// Identity information for a known wallet address (exchanges, protocols, etc.)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Identity {
    /// Solana wallet address
    pub address: String,
    /// Type of entity (e.g., "exchange", "protocol")
    #[serde(rename = "type")]
    pub entity_type: String,
    /// Display name (e.g., "Binance 1")
    pub name: String,
    /// Category classification (e.g., "Centralized Exchange")
    pub category: String,
    /// Additional classification tags
    pub tags: Vec<String>,
}

/// Request body for batch identity lookup
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchIdentityRequest {
    /// Array of Solana wallet addresses to lookup (1-100 addresses)
    pub addresses: Vec<String>,
}

/// Token balance information
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    /// Token mint address
    pub mint: String,
    /// Token symbol (e.g., "SOL")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Token name (e.g., "Solana")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Token balance (adjusted for decimals)
    pub balance: f64,
    /// Number of decimal places
    pub decimals: u8,
    /// Price per token in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_per_token: Option<f64>,
    /// Total USD value of holdings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usd_value: Option<f64>,
    /// URL to token logo image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,
    /// Token program type (spl-token or token-2022)
    pub token_program: String,
}

/// NFT information
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Nft {
    /// NFT mint address
    pub mint: String,
    /// NFT name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// NFT image URI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_uri: Option<String>,
    /// Collection name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_name: Option<String>,
    /// Collection address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_address: Option<String>,
    /// Whether this is a compressed NFT
    pub compressed: bool,
}

/// Pagination metadata for balances
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BalancesPagination {
    /// Current page number
    pub page: u32,
    /// Number of items per page
    pub limit: u32,
    /// True if more results are available
    pub has_more: bool,
}

/// Response from get balances endpoint
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BalancesResponse {
    /// Array of token balances for the current page
    pub balances: Vec<TokenBalance>,
    /// Array of NFT holdings (only if `show_nfts` is `true`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nfts: Option<Vec<Nft>>,
    /// Total USD value of balances on this page
    pub total_usd_value: f64,
    /// Pagination metadata
    pub pagination: BalancesPagination,
}

/// Balance change in a transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BalanceChange {
    /// Token mint address (or 'SOL' for native)
    pub mint: String,
    /// Change amount (positive for increase, negative for decrease)
    pub amount: f64,
    /// Token decimals
    pub decimals: u8,
}

/// Transaction with balance changes from history endpoint
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryTransaction {
    /// Transaction signature
    pub signature: String,
    /// Unix timestamp in seconds
    pub timestamp: Option<i64>,
    /// Slot number
    pub slot: u64,
    /// Transaction fee in SOL
    pub fee: f64,
    /// Address that paid the transaction fee
    pub fee_payer: String,
    /// Error message if transaction failed
    pub error: Option<String>,
    /// All balance changes in this transaction
    pub balance_changes: Vec<BalanceChange>,
}

/// Pagination metadata for history and transfers
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    /// Whether more results are available
    pub has_more: bool,
    /// Cursor to fetch the next page of results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Response from get history endpoint
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HistoryResponse {
    /// Array of transactions with balance changes
    pub data: Vec<HistoryTransaction>,
    /// Pagination information
    pub pagination: Pagination,
}

/// Transfer direction relative to the wallet
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransferDirection {
    In,
    Out,
}

/// Token transfer information
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
    /// Transaction signature
    pub signature: String,
    /// Unix timestamp in seconds
    pub timestamp: i64,
    /// Transfer direction relative to the wallet
    pub direction: TransferDirection,
    /// The other party in the transfer (sender if 'in', recipient if 'out')
    pub counterparty: String,
    /// Token mint address
    pub mint: String,
    /// Token symbol if known
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Transfer amount (human-readable, divided by decimals)
    pub amount: f64,
    /// Raw transfer amount in smallest unit
    pub amount_raw: String,
    /// Token decimals
    pub decimals: u8,
}

/// Response from get transfers endpoint
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TransfersResponse {
    /// Array of transfers
    pub data: Vec<Transfer>,
    /// Pagination information
    pub pagination: Pagination,
}

/// Wallet funding source information
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct FundingSource {
    /// Address that originally funded this wallet
    pub funder: String,
    /// Name of the funder if it's a known entity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funder_name: Option<String>,
    /// Type of the funder entity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funder_type: Option<String>,
    /// Token mint address
    pub mint: String,
    /// Token symbol
    pub symbol: String,
    /// Initial funding amount (human-readable, adjusted for decimals)
    pub amount: f64,
    /// Raw funding amount in smallest unit (e.g., lamports for SOL)
    pub amount_raw: String,
    /// Token decimals
    pub decimals: u8,
    /// Transaction signature of the funding transfer
    pub signature: String,
    /// Unix timestamp in seconds
    pub timestamp: i64,
    /// Human-readable UTC date in ISO 8601 format
    pub date: String,
    /// Slot number
    pub slot: u64,
    /// Explorer URL for the transaction
    pub explorer_url: String,
}

/// Point in time at which to read a wallet's historical balance
///
/// Exactly one of these must be provided to the balance-at endpoint. Use
/// [`BalanceAtQuery::Slot`] for exact, deterministic results, since block times
/// reported by validators can drift by a few seconds.
#[derive(Debug, Clone, PartialEq)]
pub enum BalanceAtQuery {
    /// Unix timestamp in seconds. Returns the balance as of this time.
    Time(i64),
    /// Datetime string (e.g. `2025-01-10`, `2025-01-10 19:20:00`, `2025-01-10T19:20:00Z`).
    /// Interpreted as UTC unless an explicit timezone is included.
    Datetime(String),
    /// Slot number. Returns the balance as of this slot. Exact and deterministic.
    Slot(u64),
}

/// Echo of the query parameters from the balance-at endpoint
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BalanceAtRequested {
    /// Requested time as epoch seconds (also set when `datetime` was used)
    pub time: Option<i64>,
    /// Requested slot, when `slot` was used
    pub slot: Option<u64>,
    /// The original datetime string, when `datetime` was used
    pub datetime: Option<String>,
}

/// The transaction a historical balance was read from
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceAtAsOf {
    /// Slot of the transaction
    pub slot: u64,
    /// Block time of the transaction in Unix seconds (may be null)
    pub block_time: Option<i64>,
    /// Transaction signature
    pub signature: String,
}

/// Response from the historical balance (balance-at) endpoint
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct BalanceAtResponse {
    /// Echo of the queried wallet address
    pub wallet: String,
    /// Echo of the queried mint (the SOL pseudo-mint when native)
    pub mint: String,
    /// Whether the result is native SOL
    pub is_native: bool,
    /// Human-readable amount as a decimal string. Trailing zeros are trimmed.
    pub balance: String,
    /// Exact amount in the smallest unit (lamports for SOL), as a string
    pub balance_raw: String,
    /// Token decimals (9 for SOL)
    pub decimals: u8,
    /// Echo of the query parameters
    pub requested: BalanceAtRequested,
    /// The transaction the balance was read from. `None` when the wallet had no
    /// matching transaction at or before the requested point — the balance is
    /// genuinely `0`, not an error.
    pub as_of: Option<BalanceAtAsOf>,
}

/// Billing cycle dates for an Admin API project usage response.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AdminBillingCycle {
    /// Inclusive start date of the current billing cycle in `YYYY-MM-DD` format.
    pub start: String,
    /// Exclusive end date of the current billing cycle in `YYYY-MM-DD` format.
    pub end: String,
}

/// Subscription metadata returned by the Admin API project usage endpoint.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdminSubscriptionDetails {
    /// Current billing cycle window for the project.
    pub billing_cycle: AdminBillingCycle,
    /// Included credit limit for the active plan.
    pub credits_limit: u64,
    /// Human-readable plan name.
    pub plan: String,
}

/// Per-product credit usage returned by the Admin API project usage endpoint.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdminUsageBreakdown {
    pub api: u64,
    pub archival: u64,
    pub das: u64,
    pub grpc: u64,
    pub grpc_geyser: u64,
    pub photon: u64,
    pub rpc: u64,
    pub stream: u64,
    pub webhook: u64,
    pub websocket: u64,
}

/// Project-level usage summary returned by the Admin API.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUsage {
    /// Remaining included credits for the current billing period.
    pub credits_remaining: u64,
    /// Total credits consumed in the current billing period.
    pub credits_used: u64,
    /// Remaining prepaid credits.
    pub prepaid_credits_remaining: u64,
    /// Prepaid credits consumed in the current billing period.
    pub prepaid_credits_used: u64,
    /// Plan and billing cycle details for the project.
    pub subscription_details: AdminSubscriptionDetails,
    /// Per-product usage counters for the current billing period.
    pub usage: AdminUsageBreakdown,
}

/// Options for the token accounts filter in the `get_wallet_history` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TokenAccountsOption {
    None,
    BalanceChanged,
    All,
}
