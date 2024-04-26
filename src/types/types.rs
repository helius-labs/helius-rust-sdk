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
pub struct DisplayOptions {
    #[serde(rename = "showUnverifiedCollections")]
    pub show_unverified_collections: Option<bool>,
    #[serde(rename = "showCollectionMetadata")]
    pub show_collection_metadata: Option<bool>,
    #[serde(rename = "showGrandTotal")]
    pub show_grand_total: Option<bool>,
    #[serde(rename = "showFungible")]
    pub show_fungible: Option<bool>,
    #[serde(rename = "showInscription")]
    pub show_inscription: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetSortingRequest {
    #[serde(rename = "sortBy")]
    pub sort_by: AssetSortBy,
    #[serde(rename = "sortDirection")]
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
    GetAssetResponse(GetAssetResponse),
    Other(Value),
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GetAssetResponseList {
    pub total: Option<i32>,
    pub limit: Option<i32>,
    pub page: Option<i32>,
    #[serde(rename = "items")]
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

#[derive(Serialize, Deserialize, Debug)]
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
pub struct MintExtensions {
    #[serde(rename = "confidentialTransferMint")]
    pub confidential_transfer_mint: Option<ConfidentialTransferMint>,
    #[serde(rename = "confidentialTransferFeeConfig")]
    pub confidential_transfer_fee_config: Option<ConfidentialTransferFeeConfig>,
    #[serde(rename = "transferFeeConfig")]
    pub transfer_fee_config: Option<TransferFeeConfig>,
    #[serde(rename = "metadataPointer")]
    pub metadata_pointer: MetadataPointer,
    #[serde(rename = "mintCloseAuthority")]
    pub mint_close_authority: MintCloseAuthority,
    #[serde(rename = "permanentDelegate")]
    pub permanent_delegate: PermanentDelegate,
    #[serde(rename = "transferHook")]
    pub transfer_hook: TransferHook,
    #[serde(rename = "interestBearingConfig")]
    pub interest_bearing_config: InterestBearingConfig,
    #[serde(rename = "defaultAccountState")]
    pub default_account_state: DefaultAccountState,
    #[serde(rename = "confidentialTransferAccount")]
    pub confidential_transfer_account: ConfidentialTransferAccount,
    pub metadata: MintExtensionMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfidentialTransferMint {
    pub authority: String,
    #[serde(rename = "autoApproveNewAccounts")]
    pub auto_approve_new_accounts: bool,
    #[serde(rename = "auditorElgamalPubkey")]
    pub auditor_elgamal_pubkey: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfidentialTransferFeeConfig {
    pub authority: String,
    #[serde(rename = "withdrawWithheldAuthorityElgamalPubkey")]
    pub withdraw_withheld_authority_elgamal_pubkey: String,
    #[serde(rename = "harvestToMintEnabled")]
    pub harvest_to_mint_enabled: bool,
    #[serde(rename = "withheldAmount")]
    pub withheld_amount: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransferFeeConfig {
    #[serde(rename = "transferFeeConfigAuthority")]
    pub transfer_fee_config_authority: String,
    #[serde(rename = "withdrawWithheldAuthority")]
    pub withdraw_withheld_authority: String,
    #[serde(rename = "withheldAmount")]
    pub withheld_amount: i32,
    #[serde(rename = "olderTransferFee")]
    pub older_transfer_fee: OlderTransferFee,
    #[serde(rename = "newTransferFee")]
    pub new_trasfer_fee: NewTransferFee,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OlderTransferFee {
    pub epoch: String,
    #[serde(rename = "maximumFee")]
    pub maximum_fee: String,
    #[serde(rename = "transferFeeBasisPoints")]
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
pub struct MintCloseAuthority {
    #[serde(rename = "closeAuthority")]
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
pub struct InterestBearingConfig {
    #[serde(rename = "rateAuthority")]
    pub rate_authority: String,
    #[serde(rename = "initializationTimestamp")]
    pub initialization_timestamp: i32,
    #[serde(rename = "preUpdateAverageRate")]
    pub pre_update_average_rate: i32,
    #[serde(rename = "lastUpdateTimestamp")]
    pub last_update_timestamp: i32,
    #[serde(rename = "currentRate")]
    pub current_rate: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DefaultAccountState {
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfidentialTransferAccount {
    pub approved: bool,
    #[serde(rename = "elgamalPubkey")]
    pub elgamal_pubkey: String,
    #[serde(rename = "pendingBalanceLo")]
    pub pending_balance_lo: String,
    #[serde(rename = "pendingBalanceHi")]
    pub pending_balance_hi: String,
    #[serde(rename = "availableBalance")]
    pub available_balance: String,
    #[serde(rename = "decryptableAvailableBalance")]
    pub decryptable_available_balance: String,
    #[serde(rename = "allowConfidentialCredits")]
    pub allow_confidential_credits: bool,
    #[serde(rename = "allowNonConfidentialCredits")]
    pub allow_non_confidential_credits: bool,
    #[serde(rename = "pendingBalanceCreditCounter")]
    pub pending_balance_credit_counter: i32,
    #[serde(rename = "maximumPendingBalanceCreditCounter")]
    pub maximum_pending_balance_credit_counter: i32,
    #[serde(rename = "expectedPendingBalanceCreditCounter")]
    pub expected_pending_balance_credit_counter: i32,
    #[serde(rename = "actualPendingBalanceCreditCounter")]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Supply {
    #[serde(rename = "printMaxSupply")]
    pub print_max_supply: u32,
    #[serde(rename = "printCurrentSupply")]
    pub print_current_supply: u32,
    #[serde(rename = "editionNonce")]
    pub edition_nonce: Option<i32>,
    #[serde(rename = "editionNumber")]
    pub edition_number: Option<i32>,
    #[serde(rename = "masterEditionMint")]
    pub master_edition_mint: Option<String>,
}
