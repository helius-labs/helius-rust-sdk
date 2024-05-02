use serde::{Deserialize, Serialize};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum Interface {
    #[serde(rename = "V1_NFT")]
    V1NFT,
    #[default]
    #[serde(rename = "Custom")]
    Custom,
    #[serde(rename = "V1_PRINT")]
    V1Print,
    #[serde(rename = "Legacy_NFT")]
    LegacyNFT,
    #[serde(rename = "V2_NFT")]
    V2NFT,
    #[serde(rename = "FungibleAsset")]
    FungibleAsset,
    #[serde(rename = "Identity")]
    Identity,
    #[serde(rename = "Executable")]
    Executable,
    #[serde(rename = "ProgrammableNFT")]
    ProgrammableNFT,
    #[serde(rename = "FungibleToken")]
    FungibleToken,
    #[serde(rename = "V1_PRINT")]
    V1PRINT,
    #[allow(non_camel_case_types)]
    #[serde(rename = "LEGACY_NFT")]
    LEGACY_NFT,
    #[serde(rename = "V2_NFT")]
    Nft,
    #[serde(rename = "MplCoreAsset")]
    MplCoreAsset,
    #[serde(rename = "MplCoreCollection")]
    MplCoreCollection,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum OwnershipModel {
    #[default]
    #[serde(rename = "single")]
    Single,
    #[serde(rename = "token")]
    Token,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RoyaltyModel {
    #[serde(rename = "creators")]
    Creators,
    #[serde(rename = "fanout")]
    Fanout,
    #[serde(rename = "single")]
    Single,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UseMethod {
    Burn,
    Single,
    Multiple,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Scope {
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "royalty")]
    Royalty,
    #[serde(rename = "metadata")]
    Metadata,
    #[serde(rename = "extension")]
    Extension,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Context {
    #[serde(rename = "wallet-default")]
    WalletDefault,
    #[serde(rename = "web-desktop")]
    WebDesktop,
    #[serde(rename = "web-mobile")]
    WebMobile,
    #[serde(rename = "app-mobile")]
    AppMobile,
    #[serde(rename = "app-desktop")]
    AppDesktop,
    #[serde(rename = "app")]
    App,
    #[serde(rename = "vr")]
    Vr,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AssetSortBy {
    #[serde(rename = "id")]
    Id,
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "updated")]
    Updated,
    #[serde(rename = "recent_action")]
    RecentAction,
    #[serde(rename = "none")]
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AssetSortDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum SearchConditionType {
    #[serde(rename = "all")]
    All,
    #[serde(rename = "any")]
    Any,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub enum TokenType {
    Fungible,
    NonFungible,
    CompressedNft,
    RegularNft,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MintApiAuthority {
    Mainnet(&'static str),
    Devnet(&'static str),
}

impl MintApiAuthority {
    pub fn from_cluster(cluster: Cluster) -> Result<Self, &'static str> {
        match cluster {
            Cluster::Devnet => Ok(MintApiAuthority::Devnet("2LbAtCJSaHqTnP9M5QSjvAMXk79RNLusFspFN5Ew67TC")),
            Cluster::MainnetBeta => Ok(MintApiAuthority::Mainnet(
                "HnT5KVAywGgQDhmh6Usk4bxRg4RwKxCK4jmECyaDth5R",
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PriorityLevel {
    None,
    Low,
    Medium,
    High,
    VeryHigh,
    UnsafeMax,
    Default,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UiTransactionEncoding {
    Binary,
    Base64,
    Base58,
    Json,
    JsonParsed,
}
