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
pub struct AssetsByAuthorityRequest{
    #[serde(rename = "authorityAddress")]
    pub authority_address: String,
    pub page: u32,
    pub limit: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    #[serde(rename = "displayOptions")]
    pub display_options: Option<DisplayOptions>,
    #[serde(rename = "sortBy")]
    pub sort_by: Option<AssetSortingRequest>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DisplayOptions {
    #[serde(rename = "showUnverifiedCollections")]
    pub show_unverified_collections: Option<bool>,
    #[serde(rename = "showCollectionMetadata")]
    pub show_collection_metadata: Option<bool>,
    #[serde(rename = "showGrandTotal")]
    pub show_grand_total: Option<bool>,
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
    pub result: ResponseType, // Serde will automatically deserialize the response into the appropriate type
    pub id: u8,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(untagged)]
pub enum ResponseType {
    #[default]
    DefaultResponse, // This is a placeholder for the default response type. TODO: Replace this an appropriate type
    GetAssetResponseList(GetAssetResponseList),
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
    pub print_max_supply: u32,
    pub print_current_supply: u32,
    pub edition_nonce: Option<u32>,
}
