use super::enums::{
    AssetSortBy, AssetSortDirection, Context, Interface, OwnershipModel, RoyaltyModel, Scope, UseMethods,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Defines the available clusters supported by Helius
#[derive(Debug, Clone, PartialEq)]
pub enum Cluster {
    Devnet,
    MainnetBeta,
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
            id: "1".to_string(),
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

#[derive(Serialize, Deserialize, Default)]
pub struct AssetsByOwnerRequest {
    #[serde(rename = "ownerAddress")]
    pub owner_address: String,
    #[serde(rename = "page")]
    pub page: u32,
    #[serde(rename = "limit")]
    pub limit: Option<i32>,
    #[serde(rename = "before")]
    pub before: Option<String>,
    #[serde(rename = "after")]
    pub after: Option<String>,
    #[serde(rename = "displayOptions")]
    pub display_options: Option<DisplayOptions>,
    #[serde(rename = "sortBy")]
    pub sort_by: Option<AssetSortingRequest>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct AssetsByAuthorityRequest {
    #[serde(rename = "authorityAddress")]
    pub authority_address: String,
    pub page: u32,
    pub limit: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    #[serde(rename = "displayOptions")]
    pub display_options: Option<DisplayOptions>,
    #[serde(rename = "sortBy")]
    pub sort_by: Option<AssetSortingRequest>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAssetRequest {
    pub id: String,
    #[serde(rename = "displayOptions")]
    pub display_options: Option<DisplayOptions>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DisplayOptions {
    pub show_unverified_collections: bool,
    pub show_collection_metadata: bool,
    pub show_fungible: bool,
    pub show_inscription: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetSortingRequest {
    pub sort_by: AssetSortBy,
    pub sort_direction: AssetSortDirection,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ApiResponse {
    pub jsonrpc: String,
    pub result: ResponseType,
    pub id: u8,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(untagged)]
pub enum ResponseType {
    #[default]
    DefaultResponse, // This is a placeholder for the default response type. TODO: Replace this an appropriate type
    GetAssetResponseList(GetAssetResponseList),
    GetAssetResponseForAsset(GetAssetResponseForAsset),
    Other(Value),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetAssetResponseList {
    pub total: Option<i32>,
    pub limit: Option<i32>,
    pub page: Option<i32>,
    pub items: Option<Vec<GetAssetResponse>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAssetResponse {
    pub interface: Interface,
    pub id: String,
    pub content: Option<Content>,
    pub authorities: Option<Vec<Authorities>>,
    pub compression: Option<Compression>,
    pub grouping: Option<Vec<Grouping>>,
    pub royalty: Option<Royalty>,
    pub ownership: Ownership,
    pub creators: Option<Vec<Creators>>,
    pub uses: Option<Uses>,
    pub supply: Option<Supply>,
    pub mutable: bool,
    pub burnt: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetAssetResponseForAsset {
    pub interface: Interface,
    pub id: String,
    pub content: Option<Content>,
    pub authorities: Option<Vec<Authorities>>,
    pub compression: Option<Compression>,
    pub grouping: Option<Vec<Grouping>>,
    pub royalty: Option<Royalty>,
    pub creators: Option<Vec<Creators>>,
    pub ownership: Ownership,
    #[serde(rename = "mintExtensions")]
    pub mint_extensions: Option<MintExtensions>,
    pub supply: Option<Supply>,
    #[serde(rename = "tokenSupply")]
    pub token_info: Option<TokenInfo>,
    pub inscription: Option<Inscription>,
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
    pub new_trasfer_fee: NewTransferFee,
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
    pub symbol: String,
    pub supply: i32,
    pub decimals: i32,
    #[serde(rename = "tokenProgram")]
    pub token_program: String,
    #[serde(rename = "priceInfo")]
    pub price_info: PriceInfo,
    #[serde(rename = "mintAuthority")]
    pub mint_authority: String,
    #[serde(rename = "freezeAuthority")]
    pub freeze_authority: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PriceInfo {
    #[serde(rename = "pricePerToken")]
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
    pub files: Option<Vec<File>>,
    pub metadata: Metadata,
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
    pub schema: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub attributes: Option<Vec<Attribute>>,
    pub description: Option<String>,
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    pub value: String,
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
pub struct Grouping {
    pub group_key: String,
    pub group_value: String,
    pub verified: Option<bool>,
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
    pub seq: u32,
    pub leaf_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Creators {
    pub address: String,
    pub share: u8,
    pub verified: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Royalty {
    pub royalty_model: RoyaltyModel,
    pub target: Option<String>,
    pub percent: f32,
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
    pub use_method: UseMethods,
    pub remaining: u32,
    pub total: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Supply {
    #[serde(default)]
    pub print_max_supply: Option<u32>,
    pub print_current_supply: Option<u32>,
    pub edition_nonce: Option<i32>,
    pub edition_number: Option<i32>,
    pub master_edition_mint: Option<String>,
}
