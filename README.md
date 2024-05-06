# Helius SDK
An asynchronous Helius Rust SDK for building the future of Solana

## Documentation

## Installation

## Usage
The SDK needs to be configured with your account's API key, which can be found on the [Helius Developer Dashboard](https://dev.helius.xyz/dashboard/app). The following code is an example of how to use the SDK to fetch info on [Mad Lad #8420](https://xray.helius.xyz/token/F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk?network=mainnet):
```rust
use helius_sdk::config::Config;
use helius_sdk::error::HeliusError;
use helius_sdk::rpc_client::RpcClient;
use helius_sdk::types::types::{GetAssetResponseForAsset, DisplayOptions};
use helius_sdk::types::{Cluster, GetAssetRequest};

use reqwest::Client;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "YOUR_API_KEY";
    let cluster: Cluster = Cluster::MainnetBeta;

    let config: Config = Config::new(api_key, cluster)?;
    let client: Client = Client::new();
    let rpc_client: RpcClient = RpcClient::new(Arc::new(client), Arc::new(config))?;

    let request: GetAssetRequest = GetAssetRequest {
        id: "F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk".to_string(),
        display_options: Some(DisplayOptions {
            show_unverified_collections: false,
            show_collection_metadata: false,
            show_fungible: false,
            show_inscription: false,
        }),
    };

    let response: Result<Option<GetAssetResponseForAsset>, HeliusError> = rpc_client.get_asset(request).await;

    match response {
        Ok(Some(asset)) => {
            println!("Asset: {:?}", asset);
        },
        Ok(None) => println!("No asset found."),
        Err(e) => println!("Error retrieving asset: {:?}", e),
    }

    Ok(())
}
```
More examples on how to use the SDK can be found in the [`examples`](https://github.com/helius-labs/helius-rust-sdk/tree/dev/examples) directory.

## Error Handling

### Common Error Codes
You may encounter several error codes when working with the Helius Rust SDK. Below is a table detailing some of the common error codes along with additional information to aid with troubleshooting:

| Error Code | Error Message             | More Information                                                                           |
|------------|---------------------------|---------------------------------------------------------------------------------------------|
| 401        | Unauthorized              | This occurs when an invalid API key is provided or access is restricted |
| 429        | Too Many Requests         | This indicates that the user has exceeded the request limit in a given timeframe or is out of credits |
| 5XX        | Internal Server Error     | This is a generic error message for server-side issues. Please contact Helius support for assistance |

If you encounter any of these errors, refer to the Helius documentation for further guidance, or reach out to the Helius support team for more detailed assistance

## Using the Helius SDK
Our SDK is designed to provide a seamless developer experience when building on Solana. We've separated the core functionality into various segments:

### DAS API
- [`get_asset`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-asset) - Gets an asset by its ID
- [`get_asset_batch`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-asset/get-asset-batch) - Gets multiple assets by their ID
- [`get_asset_proof`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-asset-proof) - Gets a merkle proof for a compressed asset by its ID
- [`get_asset_proof_batch`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-asset-proof/get-asset-proof-batch) - Gets multiple asset proofs by their IDs
- [`get_assets_by_owner`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-assets-by-owner) - Gets a list of assets owned by a given address
- [`get_assets_by_authority`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-assets-by-authority) - Gets a list of assets of a given authority
- [`get_assets_by_creator`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-assets-by-creator) - Gets a list of assets of a given creator
- [`get_assets_by_group`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-assets-by-group) - Gets a list of assets by a group key and value
- [`search_assets`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/search-assets) - Gets assets based on the custom search criteria passed in
- [`get_signatures_for_asset`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-signatures-for-asset) - Gets transaction signatures for a given asset
- [`get_token_accounts`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-token-accounts) - Gets information about all token accounts for a specific mint or owner
- [`get_nft_edition`](https://docs.helius.dev/compression-and-das-api/digital-asset-standard-das-api/get-nft-editions) - Gets all the NFT editions  associated with a specific master NFT
- [`get_rwa_asset`](https://github.com/helius-labs/helius-sdk/pull/71) - Gets a Real-World Asset (RWA) based on its mint address

### Mint API
- [`mint_compressed_nft`](https://docs.helius.dev/compression-and-das-api/mint-api/mint-compressed-nft) - The easiest way to mint a compressed NFT (cNFT)

Enhanced Transactions API
- [`parse_transactions`](https://docs.helius.dev/solana-apis/enhanced-transactions-api/parse-transaction-s) - Parses transactions given an array of transaction IDs
- [`parsed_transaction_history`](https://docs.helius.dev/solana-apis/enhanced-transactions-api/parsed-transaction-history) - Retrieves a parsed transaction history for a specific address

### Webhooks
- [`create_webhook`](https://docs.helius.dev/webhooks-and-websockets/api-reference/create-webhook)
- [`edit_webhook`](https://docs.helius.dev/webhooks-and-websockets/api-reference/edit-webhook)
- [`append_addresses_to_webhook`](https://github.com/helius-labs/helius-rust-sdk/blob/2d161e1ebf6d06df686d9e248ea80de215457b40/src/webhook.rs#L50-L73)
- [`get_webhook_by_id`](https://docs.helius.dev/webhooks-and-websockets/api-reference/get-webhook)
- [`get_all_webhooks`](https://docs.helius.dev/webhooks-and-websockets/api-reference/get-all-webhooks)
- [`delete_webhook`](https://docs.helius.dev/webhooks-and-websockets/api-reference/delete-webhook)

### Helper Methods
- [`get_priority_fee_estimate`](https://docs.helius.dev/solana-rpc-nodes/alpha-priority-fee-api) - Gets an estimate of the priority fees required for a transaction to be processed more quickly
- [`deserialize_str_to_number`](https://github.com/helius-labs/helius-rust-sdk/blob/dev/src/utils/deserialize_str_to_number.rs) - Deserializes a `String` to a `Number`
- [`is_valid_solana_address`](https://github.com/helius-labs/helius-rust-sdk/blob/dev/src/utils/is_valid_solana_address.rs) - Returns whether a given string slice is a valid Solana address
- [`make_keypairs`](https://github.com/helius-labs/helius-rust-sdk/blob/dev/src/utils/make_keypairs.rs) - Generates a specified number of keypairs