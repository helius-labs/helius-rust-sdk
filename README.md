# Helius SDK
An asynchronous Helius Rust SDK for building the future of Solana

## Documentation
The latest documentation can be found [here on docs.rs](https://docs.rs/helius/latest/helius/)

## Contributions
Interested in contributing to the Helius Rust SDK? Read the following [contributions guide](https://github.com/helius-labs/helius-rust-sdk/blob/dev/CONTRIBUTIONS.md) before opening up a pull request!

## Installation
To start using the Helius Rust SDK in your project, add it as a dependency via `cargo`. Open your project's `Cargo.toml` and add the following line under `[dependencies]`:
```toml
helius = "x.y.z"
```
where `x.y.z` is your desired version. Alternatively, use `cargo add helius` to add the dependency directly via the command line. This will automatically find the latest version compatible with your project and add it to your `Cargo.toml`.

Remember to run `cargo update` regularly to fetch the latest version of the SDK.

### TLS Options
The Helius Rust SDK uses the native TLS implementation by default via:
```toml
[dependencies]
helius = "x.y.z"
```

However, the SDK also supports `rustls`. Add the following to your `Cargo.toml` to use `rustls` instead of the native TLS implementation:
```toml
[dependencies]
helius = { version = "x.y.z", default-features = false, features = ["rustls"] }
```

Using `rustls` may be preferred in environments where OpenSSL is not available or when a pure Rust TLS implementation is desired. However, it may not support all the same features as the native TLS implementation

## Usage
### Quick Start
The SDK provides a [`Helius`](https://github.com/helius-labs/helius-rust-sdk/blob/dev/src/client.rs) instance that can be configured with an API key and a given Solana cluster. Developers can generate a new API key on the [Helius Developer Dashboard](https://dev.helius.xyz/dashboard/app). This instance acts as the main entry point for interacting with the SDK by providing methods to access different Solana and RPC client functionalities.

There are three simple constructors and a builder for advanced configuration:

| Constructor | Use Case |
|---|---|
| `Helius::new(api_key, cluster)` | Basic RPC operations with Helius endpoints |
| `Helius::new_async(api_key, cluster)` | Full-featured: async Solana client + WebSocket + confirmed commitment |
| `Helius::new_with_url(url)` | Custom RPC endpoint (no API key required) |
| `HeliusBuilder::new()` | Advanced configuration (custom timeouts, TLS, etc.) |

### `Helius::new()` — Basic Client
The simplest way to get started. Creates a client for standard Helius RPC operations. This constructor is **synchronous**:
```rust
use helius::Helius;
use helius::error::Result;
use helius::types::{Cluster, GetAssetRequest, GetAssetResponseForAsset};

#[tokio::main]
async fn main() -> Result<()> {
    let helius = Helius::new("YOUR_API_KEY", Cluster::MainnetBeta)?;

    let request = GetAssetRequest {
        id: "F9Lw3ki3hJ7PF9HQXsBzoY8GyE6sPoEZZdXJBsTTD2rk".to_string(),
        display_options: None,
    };

    let response: Result<Option<GetAssetResponseForAsset>> = helius.rpc().get_asset(request).await;

    match response {
        Ok(Some(asset)) => println!("Asset: {:?}", asset),
        Ok(None) => println!("No asset found."),
        Err(e) => println!("Error retrieving asset: {:?}", e),
    }

    Ok(())
}
```

### `Helius::new_async()` — Full-Featured Client
The recommended constructor for production applications. Includes async Solana RPC, WebSocket streaming, and confirmed commitment level:
```rust
use helius::Helius;
use helius::types::Cluster;

#[tokio::main]
async fn main() {
    let helius = Helius::new_async("YOUR_API_KEY", Cluster::MainnetBeta)
        .await
        .expect("Failed to create client");

    // Async Solana RPC
    let async_client = helius.async_connection().expect("Async client available");

    // Enhanced WebSocket streaming
    let ws = helius.ws().expect("WebSocket available");
}
```

### `Helius::new_with_url()` — Custom RPC Endpoint
Use your own RPC node, a third-party provider, or localhost for development. No API key required. This constructor is **synchronous**:
```rust
use helius::Helius;

fn main() {
    // Custom RPC provider
    let helius = Helius::new_with_url("https://my-rpc-provider.com/")
        .expect("Failed to create client");

    // Local development
    let local = Helius::new_with_url("http://localhost:8899")
        .expect("Failed to create client");
}
```

### `HeliusBuilder` — Advanced Configuration
For fine-grained control over the client configuration, use the builder pattern:
```rust
use helius::HeliusBuilder;
use helius::types::Cluster;
use solana_commitment_config::CommitmentConfig;
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Custom RPC with API key and specific commitment
    let helius = HeliusBuilder::new()
        .with_api_key("YOUR_API_KEY").unwrap()
        .with_cluster(Cluster::MainnetBeta)
        .with_async_solana()
        .with_commitment(CommitmentConfig::confirmed())
        .build()
        .await
        .expect("Failed to build client");

    // Custom URL with separate API and RPC endpoints
    let helius = HeliusBuilder::new()
        .with_custom_url("https://rpc.example.com/").unwrap()
        .with_custom_api_url("https://api.example.com/").unwrap()
        .with_api_key("optional-key").unwrap()
        .build()
        .await
        .expect("Failed to build client");

    // Custom HTTP client with timeouts
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let helius = HeliusBuilder::new()
        .with_api_key("YOUR_API_KEY").unwrap()
        .with_cluster(Cluster::MainnetBeta)
        .with_http_client(http_client)
        .with_websocket(Some(5), Some(15)) // custom ping/pong timeouts
        .build()
        .await
        .expect("Failed to build client");
}
```

### `HeliusFactory`
The SDK also comes equipped with `HeliusFactory`, a factory for creating instances of `Helius`. This factory allows for a centralized configuration and creation of `Helius` clients so work can be done across multiple clusters at the same time. Using a factory simplifies client code and enhances maintainability by ensuring that all `Helius` clients are configured consistently. It has the following functionality:
- A [`new` method](https://github.com/helius-labs/helius-rust-sdk/blob/a79a751e1a064125010bdb359068a366d635d005/src/factory.rs#L21-L36) used to create a new `HeliusFactory` capable of producing `Helius` clients. Note this method does not create a `reqwest` client
- A [`with_client` method](https://github.com/helius-labs/helius-rust-sdk/blob/a79a751e1a064125010bdb359068a366d635d005/src/factory.rs#L38-L53) used to provide a given `HeliusFactory` created with the `new` method its own `reqwest` client
- A [`create` method](https://github.com/helius-labs/helius-rust-sdk/blob/a79a751e1a064125010bdb359068a366d635d005/src/factory.rs#L55-L94) used to create multiple `Helius` clients in a thread-safe manner

### Embedded Solana Client
The `Helius` client has an embedded [Solana client](https://docs.rs/solana-client/latest/solana_client/rpc_client/struct.RpcClient.html) that can be accessed via `helius.connection().request_name()` where `request_name()` is a given [RPC method](https://docs.rs/solana-client/latest/solana_client/rpc_client/struct.RpcClient.html#implementations). A full list of all Solana RPC HTTP methods can be found [here](https://solana.com/docs/rpc/http).

For asynchronous operations, use `Helius::new_async()` or `HeliusBuilder` with `.with_async_solana()`. The async client can be accessed via `helius.async_connection()?.some_async_method().await?` where `some_async_method()` is a given async RPC method.

### Enhanced WebSockets
Use `Helius::new_async()` or `HeliusBuilder` with `.with_websocket(None, None)` to create a client with WebSocket support. This enables the [Geyser Enhanced WebSocket methods](https://docs.helius.dev/webhooks-and-websockets/websockets#helius-geyser-enhanced-websockets-beta) [`transactionSubscribe`](https://docs.helius.dev/webhooks-and-websockets/websockets#transaction-subscribe) and [`accountSubscribe`](https://docs.helius.dev/webhooks-and-websockets/websockets#account-subscribe)

### Examples
More examples of how to use the SDK can be found in the [`examples`](https://github.com/helius-labs/helius-rust-sdk/tree/dev/examples) directory.

## Error Handling
### Common Error Codes
You may encounter several error codes when working with the Helius Rust SDK. Below is a table detailing some of the common error codes along with additional information to aid with troubleshooting:

| Error Code | Error Message             | More Information                                                                           |
|------------|---------------------------|---------------------------------------------------------------------------------------------|
| 401        | Unauthorized              | This occurs when an invalid API key is provided or access is restricted |
| 429        | Too Many Requests         | This indicates that the user has exceeded the request limit in a given timeframe or is out of credits |
| 5XX        | Internal Server Error     | This is a generic error message for server-side issues. Please contact Helius support for assistance |

If you encounter any of these errors:
- Refer to [`errors.rs`](https://github.com/helius-labs/helius-rust-sdk/blob/dev/src/error.rs) for a list of all possible errors returned by the `Helius` client
- Refer to the [Helius documentation](https://docs.helius.dev/) for further guidance
- Reach out to the Helius support team for more detailed assistance

### Result Type Alias
The SDK also has [a handy type alias for `Result`](https://github.com/helius-labs/helius-rust-sdk/blob/c24bdf3179998895e73fe455d38bd7faa2c50df5/src/error.rs#L147-L148) where `Result<(some type), HeliusError>` and be simplified to `Result<(some type)>`

## Methods
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

### Enhanced Transactions API
- [`parse_transactions`](https://docs.helius.dev/solana-apis/enhanced-transactions-api/parse-transaction-s) - Parses transactions given an array of transaction IDs
- [`parsed_transaction_history`](https://docs.helius.dev/solana-apis/enhanced-transactions-api/parsed-transaction-history) - Retrieves a parsed transaction history for a specific address

### Webhooks
- [`append_addresses_to_webhook`](https://github.com/helius-labs/helius-rust-sdk/blob/2d161e1ebf6d06df686d9e248ea80de215457b40/src/webhook.rs#L50-L73) - Appends a set of addresses to a given webhook
- [`create_webhook`](https://docs.helius.dev/webhooks-and-websockets/api-reference/create-webhook) - Creates a webhook given account addresses
- [`delete_webhook`](https://docs.helius.dev/webhooks-and-websockets/api-reference/delete-webhook) - Deletes a given Helius webhook programmatically
- [`edit_webhook`](https://docs.helius.dev/webhooks-and-websockets/api-reference/edit-webhook) - Edits a Helius webhook programmatically
- [`get_all_webhooks`](https://docs.helius.dev/webhooks-and-websockets/api-reference/get-all-webhooks) - Retrieves all Helius webhooks programmatically
- [`get_webhook_by_id`](https://docs.helius.dev/webhooks-and-websockets/api-reference/get-webhook) - Gets a webhook config given a webhook ID
- [`remove_addresses_from_webhook`](https://github.com/helius-labs/helius-rust-sdk/blob/bf24259e3333ae93126bb65b342c2c63e80e07a6/src/webhook.rs#L75-L105) - Removes a list of addresses from an existing webhook by its ID

### Helius Sender
- [`create_smart_transaction_with_tip_for_sender`](https://github.com/helius-labs/helius-rust-sdk/blob/47d68afcf644938bc474f609368b214170423bba/src/optimized_transaction.rs#L978-L1007) - Creates an optimized smart transaction with an appended tip transfer instruction for Sender
- [`determine_tip_lamports`](https://github.com/helius-labs/helius-rust-sdk/blob/47d68afcf644938bc474f609368b214170423bba/src/optimized_transaction.rs#L966-L976) - Determines the tip amount in lamports using the 75th percentile floor or falling back to the minimum required by Sender
- [`fetch_tip_floor_75th`](https://github.com/helius-labs/helius-rust-sdk/blob/47d68afcf644938bc474f609368b214170423bba/src/optimized_transaction.rs#L940-L964) - Fetches the 75th percentile landed tip floor from Jito's endpoint (in SOL)
- [`send_and_confirm_via_sender`](https://github.com/helius-labs/helius-rust-sdk/blob/47d68afcf644938bc474f609368b214170423bba/src/optimized_transaction.rs#L1023-L1071) - Send a signed tx via Sender `/fast` and poll until confirmed (or until timeout/last valid blockhash expiry)
- [`send_smart_transaction_with_sender`](https://github.com/helius-labs/helius-rust-sdk/blob/47d68afcf644938bc474f609368b214170423bba/src/optimized_transaction.rs#L1073-L1113) - Builds an optimized tx and sent via Sender
- [`warm_sender_connection`](https://github.com/helius-labs/helius-rust-sdk/blob/47d68afcf644938bc474f609368b214170423bba/src/optimized_transaction.rs#L1009-L1021) - Warms Sender connection by hitting `/ping`

### Smart Transactions
- [`create_smart_transaction`](https://github.com/helius-labs/helius-rust-sdk/blob/bd9e0b10c81ab9ea56dfcd286336b086f6737b64/src/optimized_transaction.rs#L131-L331) - Creates an optimized transaction based on the provided configuration 
- [`create_smart_transaction_with_seeds`](https://github.com/helius-labs/helius-rust-sdk/blob/8102d87c6551c7645389a813e60a832a2eaf98c7/src/optimized_transaction.rs#L478-L633) - Creates a thread-safe, optimized transaction using seed bytes
- [`create_smart_transaction_without_signers`](https://github.com/helius-labs/helius-rust-sdk/blob/47d68afcf644938bc474f609368b214170423bba/src/optimized_transaction.rs#L809-L938) - Creates an optimized transaction without requiring any signers
- [`get_compute_units`](https://github.com/helius-labs/helius-rust-sdk/blob/bd9e0b10c81ab9ea56dfcd286336b086f6737b64/src/optimized_transaction.rs#L34-L87) - Simulates a transaction to get the total compute units consumed
- [`get_compute_units_thread_safe`](https://github.com/helius-labs/helius-rust-sdk/blob/8102d87c6551c7645389a813e60a832a2eaf98c7/src/optimized_transaction.rs#L421-L476) - A thread-safe version of `get_compute_units`
- [`poll_transaction_confirmation`](https://github.com/helius-labs/helius-rust-sdk/blob/bd9e0b10c81ab9ea56dfcd286336b086f6737b64/src/optimized_transaction.rs#L89-L129) - Polls a transaction to check whether it has been confirmed in 5 second intervals with a 15 second timeout
- [`send_smart_transaction`](https://github.com/helius-labs/helius-rust-sdk/blob/bd9e0b10c81ab9ea56dfcd286336b086f6737b64/src/optimized_transaction.rs#L333-L364) - Builds and sends an optimized transaction, and handles its confirmation status
- [`send_and_confirm_transaction`](https://github.com/helius-labs/helius-rust-sdk/blob/bd9e0b10c81ab9ea56dfcd286336b086f6737b64/src/optimized_transaction.rs#L366-L412) - Sends a transaction and handles its confirmation status with retry logic
- [`send_smart_transaction_with_seeds`](https://github.com/helius-labs/helius-rust-sdk/blob/bd9e0b10c81ab9ea56dfcd286336b086f6737b64/src/optimized_transaction.rs#L414-L487) - Sends a smart transaction using seed bytes. This function is thread-safe

### Jito Smart Transactions and Helper Methods (Deprecated)

Note that these methods have been deprecated, and will be removed in a future release. Instead, please use [Helius Sender](https://www.helius.dev/docs/sending-transactions/sender) for ultra-low latency transaction submission

- [`add_tip_instruction`](https://github.com/helius-labs/helius-rust-sdk/blob/02b351a5ee3fe16a36078b40f92dc72d0ad077ed/src/jito.rs#L66-L83) - Adds a tip instruction to the instructions provided
- [`create_smart_transaction_with_tip`](https://github.com/helius-labs/helius-rust-sdk/blob/bd9e0b10c81ab9ea56dfcd286336b086f6737b64/src/jito.rs#L85-L125) - Creates a smart transaction with a Jito tip
- [`get_bundle_statuses`](https://github.com/helius-labs/helius-rust-sdk/blob/bd9e0b10c81ab9ea56dfcd286336b086f6737b64/src/jito.rs#L170-L203) - Get the status of Jito bundles
- [`send_jito_bundle`](https://github.com/helius-labs/helius-rust-sdk/blob/bd9e0b10c81ab9ea56dfcd286336b086f6737b64/src/jito.rs#L127-L168) - Sends a bundle of transactions to the Jito Block Engine
- [`send_smart_transaction_with_seeds_and_tip`](https://github.com/helius-labs/helius-rust-sdk/blob/8102d87c6551c7645389a813e60a832a2eaf98c7/src/jito.rs#L272-L353) - Sends a smart transaction as a Jito bundle with a tip using seed bytes to create the transaction
- [`send_smart_transaction_with_tip`](https://github.com/helius-labs/helius-rust-sdk/blob/bd9e0b10c81ab9ea56dfcd286336b086f6737b64/src/jito.rs#L205-L270) - Sends a smart transaction as a Jito bundle with a tip

### RPC Methods
- [`get_priority_fee_estimate`](https://www.helius.dev/docs/api-reference/priority-fee/getpriorityfeeestimate#getpriorityfeeestimate) - Gets an estimate of the priority fees required for a transaction to be processed more quickly
- [`get_transactions_for_address`](https://www.helius.dev/docs/api-reference/rpc/http/gettransactionsforaddress) - Gets transaction history for a specific address with advanced filtering, sorting, and pagination. Optionally include transactions from associated token accounts

### Helper Methods
- [`deserialize_str_to_number`](https://github.com/helius-labs/helius-rust-sdk/blob/dev/src/utils/deserialize_str_to_number.rs) - Deserializes a `String` to a `Number`
- [`is_valid_solana_address`](https://github.com/helius-labs/helius-rust-sdk/blob/dev/src/utils/is_valid_solana_address.rs) - Returns whether a given string slice is a valid Solana address
- [`make_keypairs`](https://github.com/helius-labs/helius-rust-sdk/blob/dev/src/utils/make_keypairs.rs) - Generates a specified number of keypairs

## Migrating from 0.x
If you're upgrading from 0.x, see the [Migration Guide](MIGRATION.md) for details on breaking changes and how to update your code.