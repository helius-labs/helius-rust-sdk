use serde::{Deserialize, Serialize};

/// Display options for customizing the Solana asset response in methods like
/// `getAssetsByOwner`, `getAssetsByCreator`, `getAssetsByGroup`, and `getAssetsByAuthority`.
///
/// These flags control which additional data is included in the response. Each option
/// adds extra data to the response and may increase response time.
///
/// # Example
/// ```
/// use helius::types::DisplayOptions;
///
/// let options = DisplayOptions {
///     show_collection_metadata: true,
///     show_grand_total: true,
///     show_native_balance: true,
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DisplayOptions {
    /// Show metadata for the collection.
    ///
    /// When enabled, includes complete collection-level metadata in the response,
    /// such as collection name, symbol, and attributes.
    pub show_collection_metadata: bool,

    /// Show total number of matching assets (slower request).
    ///
    /// When enabled, the response includes the total count of all assets matching
    /// the query criteria. **Warning:** This increases response time
    /// for large collections as it requires counting all matching assets.
    pub show_grand_total: bool,

    /// Show unverified collections instead of skipping them.
    ///
    /// By default, assets from unverified collections are excluded from results.
    /// Enable this to include assets from collections that haven't been verified
    /// by their creators.
    pub show_unverified_collections: bool,

    /// Show raw on-chain data for the asset.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    pub show_raw_data: bool,

    /// Show fungible tokens held by the owner.
    ///
    /// When enabled, includes fungible SPL tokens (like USDC, SOL) in addition
    /// to NFTs. Useful for displaying complete wallet contents.
    pub show_fungible: bool,

    /// Require the asset to be fully indexed before returning.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    pub require_full_index: bool,

    /// Show system-level metadata for the asset.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    pub show_system_metadata: bool,

    /// Display assets with zero balance.
    ///
    /// By default, assets with zero balance (e.g., burned tokens, transferred NFTs)
    /// are excluded. Enable this to include them in results.
    pub show_zero_balance: bool,

    /// Display closed token accounts.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    pub show_closed_accounts: bool,
}

/// Display options for the `getAsset` and `getAssetBatch` methods.
///
/// These flags control which additional data is included when retrieving individual
/// assets. Each option adds extra data and may increase response time.
///
/// # Example
/// ```
/// use helius::types::GetAssetOptions;
///
/// let options = GetAssetOptions {
///     show_collection_metadata: true,
///     show_inscription: true,
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetOptions {
    /// Show metadata for the collection.
    ///
    /// When enabled, includes complete collection-level metadata such as
    /// collection name, symbol, and description.
    pub show_collection_metadata: bool,

    /// Show grouping information for unverified collections instead of skipping them.
    ///
    /// By default, unverified collections are excluded. Enable this to display
    /// grouping data for collections that haven't been verified by their creators.
    pub show_unverified_collections: bool,

    /// Show raw on-chain data for the asset.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    pub show_raw_data: bool,

    /// Show fungible tokens held by the owner.
    ///
    /// When enabled, includes fungible SPL token information in the response.
    pub show_fungible: bool,

    /// Require the asset to be fully indexed before returning.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    pub require_full_index: bool,

    /// Show system-level metadata for the asset.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    pub show_system_metadata: bool,

    /// Show the native (SOL) balance of the owner.
    ///
    /// When enabled, includes the SOL balance in lamports, current SOL price,
    /// and total USD value of the owner's wallet.
    pub show_native_balance: bool,

    /// Display inscription details of assets inscribed on-chain.
    ///
    /// When enabled, includes inscription data for assets that have been inscribed
    /// using Solana's inscription standards (experimental feature).
    pub show_inscription: bool,
}

/// Display options for the `searchAssets` method.
///
/// These flags control which additional data is included in search results.
/// Each option adds extra data and may increase response time.
///
/// # Example
/// ```
/// use helius::types::SearchAssetsOptions;
///
/// let options = SearchAssetsOptions {
///     show_collection_metadata: true,
///     show_grand_total: false,  // Skip for faster response
///     show_native_balance: true,
///     ..Default::default()
/// };
/// ```
#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SearchAssetsOptions {
    /// Show metadata for the collection.
    ///
    /// When enabled, includes complete collection-level metadata for each asset
    /// in the search results.
    #[serde(default)]
    pub show_collection_metadata: bool,

    /// Show total number of matching assets (slower request).
    ///
    /// When enabled, the response includes the total count of all assets matching
    /// the search criteria. **Warning:** This increases response time
    /// as it requires counting all matching assets before returning results.
    #[serde(default)]
    pub show_grand_total: bool,

    /// Show unverified collections instead of skipping them.
    ///
    /// By default, assets from unverified collections are excluded. Enable this
    /// to include assets from collections that haven't been verified by their creators.
    #[serde(default)]
    pub show_unverified_collections: bool,

    /// Show raw on-chain data for the asset.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    #[serde(default)]
    pub show_raw_data: bool,

    /// Require the asset to be fully indexed before returning.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    #[serde(default)]
    pub require_full_index: bool,

    /// Show system-level metadata for the asset.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    #[serde(default)]
    pub show_system_metadata: bool,

    /// Display assets with zero balance.
    ///
    /// By default, assets with zero balance (e.g., burned tokens, transferred NFTs)
    /// are excluded from search results. Enable this to include them.
    #[serde(default)]
    pub show_zero_balance: bool,

    /// Display closed token accounts.
    ///
    /// **Note:** This is a Helius-specific option not documented in the public API.
    /// Use with caution as behavior may change.
    #[serde(default)]
    pub show_closed_accounts: bool,

    /// Show the native (SOL) balance of the owner.
    ///
    /// When enabled, includes the owner's SOL balance in lamports, current SOL price,
    /// and total USD value in the response.
    #[serde(default)]
    pub show_native_balance: bool,
}
