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
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "YOUR_API_KEY";
    let cluster: Cluster = Cluster::MainnetBeta;

    let config: Config = Config::new(api_key, cluster)?;
    let client: reqwest::Client = reqwest::Client::new();
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

DAS API

Mint API

Webhooks

Helper Methods