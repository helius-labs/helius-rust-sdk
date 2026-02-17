use super::{
    enums::{AssetSortBy, AssetSortDirection, Context, Interface, OwnershipModel, RoyaltyModel, Scope, UseMethod},
    AccountWebhookEncoding, CollectionIdentifier, PriorityLevel, SearchAssetsOptions, SearchConditionType, TokenType,
    TransactionStatus, TransactionType, UiTransactionEncoding, WebhookType,
};
use crate::types::{DisplayOptions, Encoding, GetAssetOptions, GpaFilter, TokenAccountsOwnerFilter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_commitment_config::CommitmentLevel;
use solana_sdk::{instruction::Instruction, message::AddressLookupTableAccount, signature::Signer};

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

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct RpcRequest<T> {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    #[serde(rename = "params")]
    pub parameters: T,
}

impl<T> RpcRequest<T> {
    pub fn new(method: String, parameters: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: "helius-rust-sdk".to_string(),
            method,
            parameters,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: String,
    pub result: T,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GetAssetsByOwner {
    #[serde(rename = "ownerAddress")]
    pub owner_address: String,
    pub page: u32,
    pub limit: Option<i32>,
    pub before: Option<String>,
    pub after: Option<String>,
    #[serde(rename = "displayOptions")]
    pub display_options: Option<DisplayOptions>,
    #[serde(rename = "sortBy")]
    pub sort_by: Option<AssetSorting>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct GetAssetsByAuthority {
    #[serde(rename = "authorityAddress")]
    pub authority_address: String,
    pub page: u32,
    pub limit: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    #[serde(rename = "displayOptions")]
    pub display_options: Option<DisplayOptions>,
    #[serde(rename = "sortBy")]
    pub sort_by: Option<AssetSorting>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetAsset {
    pub id: String,
    #[serde(rename = "displayOptions")]
    pub display_options: Option<GetAssetOptions>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetsByCreator {
    pub creator_address: String,
    pub only_verified: Option<bool>,
    pub sort_by: Option<AssetSorting>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    #[serde(default, alias = "displayOptions")]
    pub options: Option<DisplayOptions>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAssetBatch {
    pub ids: Vec<String>,
    #[serde(rename = "displayOptions")]
    pub display_options: Option<GetAssetOptions>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAssetProof {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAssetProofBatch {
    pub ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetProof {
    pub root: String,
    pub proof: Vec<String>,
    pub node_index: i32,
    pub leaf: String,
    pub tree_id: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetsByGroup {
    pub group_key: String,
    pub group_value: String,
    pub sort_by: Option<AssetSorting>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    #[serde(default, alias = "displayOptions")]
    pub options: Option<DisplayOptions>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchAssets {
    pub negate: Option<bool>,
    pub condition_type: Option<SearchConditionType>,
    pub interface: Option<Interface>,
    pub owner_address: Option<String>,
    pub owner_type: Option<OwnershipModel>,
    pub creator_address: Option<String>,
    pub creator_verified: Option<bool>,
    pub authority_address: Option<String>,
    pub grouping: Option<(String, String)>,
    pub delegate: Option<String>,
    pub frozen: Option<bool>,
    pub supply: Option<u64>,
    pub supply_mint: Option<String>,
    pub compressed: Option<bool>,
    pub compressible: Option<bool>,
    pub royalty_target_type: Option<RoyaltyModel>,
    pub royalty_target: Option<String>,
    pub royalty_amount: Option<u32>,
    pub burnt: Option<bool>,
    pub sort_by: Option<AssetSorting>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    #[serde(default)]
    pub json_uri: Option<String>,
    #[serde(default)]
    pub not: Option<NotFilter>,
    #[serde(default, alias = "displayOptions")]
    pub options: Option<SearchAssetsOptions>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub collections: Option<Vec<String>>,
    #[serde(default)]
    pub token_type: Option<TokenType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<CreatedAtFilter>,
    #[serde(default)]
    pub tree: Option<String>,
    #[serde(default)]
    pub collection_nft: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetSignatures {
    pub id: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub tree: Option<String>,
    pub leaf_index: Option<i64>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub sort_direction: Option<AssetSortDirection>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTokenAccounts {
    pub owner: Option<String>,
    pub mint: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    #[serde(default, alias = "displayOptions")]
    pub options: Option<DisplayOptions>,
    #[serde(default)]
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetNftEditions {
    pub mint: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetSorting {
    pub sort_by: AssetSortBy,
    pub sort_direction: Option<AssetSortDirection>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ApiResponse<T> {
    pub jsonrpc: String,
    pub result: T,
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
///     page: Some(2),
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
///     { "edition": 1, "address": "...", "owner": "..." },
///     { "edition": 2, "address": "...", "owner": "..." }
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
    pub interface: Interface,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorities: Option<Vec<Authorities>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<Compression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grouping: Option<Vec<Group>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub royalty: Option<Royalty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creators: Option<Vec<Creator>>,
    pub ownership: Ownership,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses: Option<Uses>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supply: Option<Supply>,
    pub mutable: bool,
    pub burnt: bool,
    pub mint_extensions: Option<Value>,
    pub token_info: Option<TokenInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_definition: Option<GroupDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_plugins: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mpl_core_info: Option<MplCoreInfo>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct AssetError {
    pub id: String,
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MintExtensions {
    pub confidential_transfer_mint: Option<ConfidentialTransferMint>,
    pub confidential_transfer_fee_config: Option<ConfidentialTransferFeeConfig>,
    pub transfer_fee_config: Option<TransferFeeConfig>,
    pub metadata_pointer: MetadataPointer,
    pub mint_close_authority: MintCloseAuthority,
    pub permanent_delegate: PermanentDelegate,
    pub transfer_hook: TransferHook,
    pub interest_bearing_config: InterestBearingConfig,
    pub default_account_state: DefaultAccountState,
    pub confidential_transfer_account: ConfidentialTransferAccount,
    pub metadata: MintExtensionMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialTransferMint {
    pub authority: String,
    pub auto_approve_new_accounts: bool,
    pub auditor_elgamal_pubkey: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialTransferFeeConfig {
    pub authority: String,
    pub withdraw_withheld_authority_elgamal_pubkey: String,
    pub harvest_to_mint_enabled: bool,
    pub withheld_amount: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TransferFeeConfig {
    pub transfer_fee_config_authority: String,
    pub withdraw_withheld_authority: String,
    pub withheld_amount: i32,
    pub older_transfer_fee: OlderTransferFee,
    pub new_transfer_fee: NewTransferFee,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OlderTransferFee {
    pub epoch: String,
    pub maximum_fee: String,
    pub transfer_fee_basis_points: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewTransferFee {
    pub epoch: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetadataPointer {
    pub authority: String,
    #[serde(rename = "metadataAddress")]
    pub metadata_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MintCloseAuthority {
    pub close_authority: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PermanentDelegate {
    pub delegate: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferHook {
    pub authority: String,
    #[serde(rename = "programId")]
    pub program_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InterestBearingConfig {
    pub rate_authority: String,
    pub initialization_timestamp: i32,
    pub pre_update_average_rate: i32,
    pub last_update_timestamp: i32,
    pub current_rate: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultAccountState {
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfidentialTransferAccount {
    pub approved: bool,
    pub elgamal_pubkey: String,
    pub pending_balance_lo: String,
    pub pending_balance_hi: String,
    pub available_balance: String,
    pub decryptable_available_balance: String,
    pub allow_confidential_credits: bool,
    pub allow_non_confidential_credits: bool,
    pub pending_balance_credit_counter: i32,
    pub maximum_pending_balance_credit_counter: i32,
    pub expected_pending_balance_credit_counter: i32,
    pub actual_pending_balance_credit_counter: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MintExtensionMetadata {
    #[serde(rename = "updateAuthority")]
    pub update_authority: String,
    pub mint: String,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    #[serde(rename = "additionalMetadata")]
    pub additional_metadata: AdditionalMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdditionalMetadata {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenInfo {
    pub symbol: Option<String>,
    pub balance: Option<u64>,
    pub supply: Option<u64>,
    pub decimals: Option<i32>,
    pub token_program: Option<String>,
    pub associated_token_address: Option<String>,
    pub price_info: Option<PriceInfo>,
    pub mint_authority: Option<String>,
    pub freeze_authority: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PriceInfo {
    pub price_per_token: f32,
    pub currency: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Inscription {
    pub order: i32,
    pub size: i32,
    #[serde(rename = "contentType")]
    pub content_type: String,
    pub encoding: String,
    #[serde(rename = "validationHash")]
    pub validation_hash: String,
    #[serde(rename = "inscriptionDataAccount")]
    pub inscription_data_account: String,
    pub authority: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    #[serde(rename = "schema", default)]
    #[serde(alias = "$schema")]
    pub schema: String,
    pub json_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<File>>,
    pub metadata: Metadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub uri: Option<String>,
    pub mime: Option<String>,
    pub cdn_uri: Option<String>,
    pub quality: Option<FileQuality>,
    pub contexts: Option<Vec<Context>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileQuality {
    #[serde(rename = "$$schema")]
    pub schema: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Attributes {
    List(Vec<Attribute>),
    Map(serde_json::Map<String, Value>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub attributes: Option<Attributes>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    pub value: Value,
    pub trait_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Links {
    pub external_url: Option<String>,
    pub image: Option<String>,
    pub animation_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Authorities {
    pub address: String,
    pub scopes: Vec<Scope>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    pub group_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_metadata: Option<CollectionMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CollectionMetadata {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub image: Option<String>,
    pub description: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Compression {
    pub eligible: bool,
    pub compressed: bool,
    pub data_hash: String,
    pub creator_hash: String,
    pub asset_hash: String,
    pub tree: String,
    pub seq: i64,
    pub leaf_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Creator {
    pub address: String,
    pub share: i32,
    pub verified: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Royalty {
    pub royalty_model: RoyaltyModel,
    pub target: Option<String>,
    pub percent: f64,
    pub basis_points: u32,
    pub primary_sale_happened: bool,
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Ownership {
    pub frozen: bool,
    pub delegated: bool,
    pub delegate: Option<String>,
    pub ownership_model: OwnershipModel,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Uses {
    pub use_method: UseMethod,
    pub remaining: u64,
    pub total: u64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Supply {
    pub print_max_supply: Option<u64>,
    pub print_current_supply: Option<u64>,
    pub edition_nonce: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_edition_mint: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GroupDefinition {
    pub group_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct MplCoreInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_minted: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_size: Option<i32>,
    pub plugins_json_version: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotFilter {
    pub collections: Option<Vec<String>>,
    pub owners: Option<Vec<Vec<u8>>>,
    pub creators: Option<Vec<Vec<u8>>>,
    pub authorities: Option<Vec<Vec<u8>>>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenAccount {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegated_amount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_extensions: Option<Value>,
    pub frozen: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Edition {
    pub mint: String,
    pub edition_address: String,
    pub edition: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MintCompressedNftRequest {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub owner: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    pub attributes: Vec<Attribute>,
    #[serde(rename = "imageUrl", skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(rename = "externalUrl", skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
    #[serde(rename = "sellerFeeBasisPoints", skip_serializing_if = "Option::is_none")]
    pub seller_fee_basis_points: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creators: Option<Vec<Creator>>,
    #[serde(rename = "confirmTransaction", skip_serializing_if = "Option::is_none")]
    pub confirm_transaction: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MintResponse {
    pub signature: String,
    pub minted: bool,
    #[serde(rename = "assetId")]
    pub asset_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetPriorityFeeEstimateOptions {
    pub priority_level: Option<PriorityLevel>,
    pub include_all_priority_fee_levels: Option<bool>,
    pub transaction_encoding: Option<UiTransactionEncoding>,
    pub lookback_slots: Option<u8>,
    pub recommended: Option<bool>,
    pub include_vote: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetPriorityFeeEstimateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<String>,
    #[serde(rename = "accountKeys", skip_serializing_if = "Option::is_none")]
    pub account_keys: Option<Vec<String>>,
    pub options: Option<GetPriorityFeeEstimateOptions>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MicroLamportPriorityFeeLevels {
    pub min: f64,
    pub low: f64,
    pub medium: f64,
    pub high: f64,
    #[serde(rename = "veryHigh")]
    pub very_high: f64,
    #[serde(rename = "unsafeMax")]
    pub unsafe_max: f64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetPriorityFeeEstimateResponse {
    pub priority_fee_estimate: Option<f64>,
    pub priority_fee_levels: Option<MicroLamportPriorityFeeLevels>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    #[serde(rename = "webhookID")]
    pub webhook_id: String,
    pub wallet: String,
    pub project: String,
    #[serde(rename = "webhookURL")]
    pub webhook_url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_types: Vec<TransactionType>,
    pub account_addresses: Vec<String>,
    pub webhook_type: WebhookType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    #[serde(default)]
    pub txn_status: TransactionStatus,
    #[serde(default)]
    pub encoding: AccountWebhookEncoding,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhookRequest {
    #[serde(rename = "webhookURL")]
    pub webhook_url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_types: Vec<TransactionType>,
    pub account_addresses: Vec<String>,
    pub webhook_type: WebhookType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    #[serde(default)]
    pub txn_status: TransactionStatus,
    #[serde(default)]
    pub encoding: AccountWebhookEncoding,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionWebhookRequest {
    pub collection_query: CollectionIdentifier,
    #[serde(rename = "webhookURL")]
    pub webhook_url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_types: Vec<TransactionType>,
    pub account_addresses: Vec<String>,
    pub webhook_type: WebhookType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    #[serde(default)]
    pub txn_status: TransactionStatus,
    #[serde(default)]
    pub encoding: AccountWebhookEncoding,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EditWebhookRequest {
    #[serde(skip_serializing)]
    pub webhook_id: String,
    #[serde(rename = "webhookURL")]
    pub webhook_url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_types: Vec<TransactionType>,
    pub account_addresses: Vec<String>,
    pub webhook_type: WebhookType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_header: Option<String>,
    #[serde(default)]
    pub txn_status: TransactionStatus,
    #[serde(default)]
    pub encoding: AccountWebhookEncoding,
}

pub struct CreateSmartTransactionConfig {
    pub instructions: Vec<Instruction>,
    pub signers: Vec<Arc<dyn Signer>>,
    pub lookup_tables: Option<Vec<AddressLookupTableAccount>>,
    pub fee_payer: Option<Arc<dyn Signer>>,
    pub priority_fee_cap: Option<u64>,
    pub cu_buffer_multiplier: Option<f32>,
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
        }
    }
}

impl CreateSmartTransactionConfig {
    pub fn new(instructions: Vec<Instruction>, signers: Vec<Arc<dyn Signer>>) -> Self {
        Self {
            instructions,
            signers,
            lookup_tables: None,
            fee_payer: None,
            priority_fee_cap: None,
            cu_buffer_multiplier: None,
        }
    }
}

pub struct Timeout {
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

pub struct SmartTransactionConfig {
    pub create_config: CreateSmartTransactionConfig,
    pub send_options: RpcSendTransactionConfig,
    pub timeout: Timeout,
}

impl SmartTransactionConfig {
    pub fn new(instructions: Vec<Instruction>, signers: Vec<Arc<dyn Signer>>, timeout: Timeout) -> Self {
        Self {
            create_config: CreateSmartTransactionConfig::new(instructions, signers),
            send_options: RpcSendTransactionConfig::default(),
            timeout,
        }
    }
}

#[derive(Clone)]
pub struct CreateSmartTransactionSeedConfig {
    pub instructions: Vec<Instruction>,
    pub signer_seeds: Vec<[u8; 32]>,
    pub fee_payer_seed: Option<[u8; 32]>,
    pub lookup_tables: Option<Vec<AddressLookupTableAccount>>,
    pub priority_fee_cap: Option<u64>,
    pub cu_buffer_multiplier: Option<f32>,
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
}

/// Options for sending via Sender
#[derive(Clone, Debug)]
pub struct SenderSendOptions {
    /// Must match a key in SENDER_ENDPOINTS (e.g., "Default", "US_EAST")
    pub region: String,
    /// If true, appends `?swqos_only=true` to `/fast`
    pub swqos_only: bool,
    /// Poll settings
    pub poll_timeout_ms: u64,
    pub poll_interval_ms: u64,
}

impl Default for SenderSendOptions {
    fn default() -> Self {
        Self {
            region: "Default".to_string(),
            swqos_only: false,
            poll_timeout_ms: 60_000,
            poll_interval_ms: 2_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSlice {
    pub length: u64,
    pub offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpaMemcmp {
    pub offset: u64,
    pub bytes: String, // base58 string
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetProgramAccountsV2Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<CommitmentLevel>,

    // camelCase in JSON:
    #[serde(rename = "minContextSlot", skip_serializing_if = "Option::is_none")]
    pub min_context_slot: Option<u64>,
    #[serde(rename = "withContext", skip_serializing_if = "Option::is_none")]
    pub with_context: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,
    #[serde(rename = "dataSlice", skip_serializing_if = "Option::is_none")]
    pub data_slice: Option<DataSlice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    #[serde(rename = "paginationKey", skip_serializing_if = "Option::is_none")]
    pub pagination_key: Option<String>,
    #[serde(rename = "changedSinceSlot", skip_serializing_if = "Option::is_none")]
    pub changed_since_slot: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<Vec<GpaFilter>>,
}

pub type GetProgramAccountsV2Request = (String, GetProgramAccountsV2Config);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpaAccount {
    pub pubkey: String,
    pub account: AccountInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub lamports: u64,
    pub owner: String,
    pub data: Value, // Varies by encoding
    pub executable: bool,
    #[serde(rename = "rentEpoch")]
    pub rent_epoch: u64,
    #[serde(default)]
    pub space: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetProgramAccountsV2Response {
    pub accounts: Vec<GpaAccount>,
    #[serde(rename = "paginationKey")]
    pub pagination_key: Option<String>,
    #[serde(rename = "totalResults")]
    pub total_results: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetTokenAccountsByOwnerV2Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<CommitmentLevel>,
    #[serde(rename = "minContextSlot", skip_serializing_if = "Option::is_none")]
    pub min_context_slot: Option<u64>,

    #[serde(rename = "dataSlice", skip_serializing_if = "Option::is_none")]
    pub data_slice: Option<DataSlice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<Encoding>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "paginationKey", skip_serializing_if = "Option::is_none")]
    pub pagination_key: Option<String>,
    #[serde(rename = "changedSinceSlot", skip_serializing_if = "Option::is_none")]
    pub changed_since_slot: Option<u64>,
}

pub type GetTokenAccountsByOwnerV2Request = (String, TokenAccountsOwnerFilter, GetTokenAccountsByOwnerV2Config);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcContext {
    pub slot: u64,
    #[serde(rename = "apiVersion")]
    pub api_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountRecord {
    pub pubkey: String,
    pub account: AccountInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetTokenAccountsByOwnerV2Response {
    pub context: Option<RpcContext>,
    pub value: GetTokenAccountsByOwnerV2Value,
    #[serde(rename = "paginationKey")]
    pub pagination_key: Option<String>,
    #[serde(rename = "totalResults")]
    pub total_results: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GetTokenAccountsByOwnerV2Value {
    pub accounts: Vec<TokenAccountRecord>,
    #[serde(rename = "paginationKey")]
    pub pagination_key: Option<String>,
    pub count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum TransactionDetails {
    #[default]
    Signatures,
    Full,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum TransactionStatusFilter {
    #[default]
    Any,
    Succeeded,
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionsForAddressOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_details: Option<TransactionDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commitment: Option<CommitmentLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<GetTransactionsFilters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<UiTransactionEncoding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_supported_transaction_version: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_context_slot: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetTransactionsForAddressResponse {
    pub data: Vec<serde_json::Value>,
    pub pagination_token: Option<String>,
}

pub type GetTransactionsForAddressRequest = (String, GetTransactionsForAddressOptions);
