use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DisplayOptions {
    pub show_collection_metadata: bool,
    pub show_grand_total: bool,
    pub show_unverified_collections: bool,
    pub show_raw_data: bool,
    pub show_fungible: bool,
    pub require_full_index: bool,
    pub show_system_metadata: bool,
    pub show_zero_balance: bool,
    pub show_closed_accounts: bool,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            show_unverified_collections: false,
            show_collection_metadata: false,
            show_fungible: false,
            show_closed_accounts: false,
            show_grand_total: false,
            show_raw_data: false,
            require_full_index: false,
            show_system_metadata: false,
            show_zero_balance: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetOptions {
    pub show_collection_metadata: bool,
    pub show_unverified_collections: bool,
    pub show_raw_data: bool,
    pub show_fungible: bool,
    pub require_full_index: bool,
    pub show_system_metadata: bool,
    pub show_native_balance: bool,
    pub show_inscription: bool,
}

impl Default for GetAssetOptions {
    fn default() -> Self {
        Self {
            show_collection_metadata: false,
            show_unverified_collections: false,
            show_raw_data: false,
            show_fungible: false,
            require_full_index: false,
            show_system_metadata: false,
            show_native_balance: false,
            show_inscription: false,
        }
    }
}
