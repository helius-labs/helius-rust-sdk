# GitHub Copilot Instructions — Helius Rust SDK

<!-- Keep this file in sync with CLAUDE.md. Both describe the same codebase conventions; changes to one should be reflected in the other. -->

Rust SDK for Helius APIs and Solana development. Built on `solana-client` and `tokio`, with async-first design and modular architecture. For full contributor details see [CLAUDE.md](../CLAUDE.md) and [CONTRIBUTIONS.md](../CONTRIBUTIONS.md).

## Critical: Use the Custom `Result<T>` Type

All fallible functions must use the SDK's `Result<T>` alias — never write `std::result::Result<T, HeliusError>` directly:

```rust
// In SDK source code (src/)
use crate::error::Result;

// In examples and tests
use helius::error::Result;

pub async fn my_function(&self) -> Result<Asset> { ... }
```

## Critical: Serde Conventions

JSON structs must use `camelCase` renaming to match the structure of Helius APIs. Always skip serializing `None` fields:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MyRequest {
    pub owner_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_options: Option<DisplayOptions>,
}
```

If a struct only needs field-level renames (not `rename_all`), use `#[serde(rename = "fieldName")]` on individual fields.

## Critical: Error Handling with `HeliusError`

All errors flow through the `HeliusError` enum defined in `src/error.rs`. Use `thiserror` derives and `#[from]` for conversions. Never panic — always propagate errors with `?`:

```rust
// Good
let config = Config::new(api_key, cluster)?;

// Bad — never panic in library code
let config = Config::new(api_key, cluster).unwrap();
```

## Async Patterns

The SDK uses `tokio` as its async runtime. All async functions return `Result`:

```rust
pub async fn fetch_data(&self) -> Result<Data> {
    let response = self.client.get(url).send().await?;
    // ...
}
```

Use `Arc` for shared state across async boundaries (e.g., `Arc<Config>`, `Arc<RpcClient>`).

## Code Style

- **Naming:** `snake_case` functions/variables, `PascalCase` types/structs, `SCREAMING_SNAKE_CASE` constants
- **Line width:** 120 characters max (`rustfmt.toml` enforces this)
- **Imports:** Group related imports together, separated by blank lines. No strict ordering is enforced across the codebase
- **Cloning:** Avoid unnecessary `.clone()`. Use references where possible
- **Mutability:** Immutable by default. Only use `mut` when reassignment is required
- **Documentation:** `///` doc comments on all public items, with usage examples for complex functionality
- **Filenames:** Use underscores, not dashes. Do not name files `main.rs` — use descriptive names

## Project Structure

```
src/
  client.rs                         # Helius struct — main entry point, constructors, accessors
  builder.rs                        # HeliusBuilder for flexible client configuration
  config.rs                         # Config struct (api_key, cluster, endpoints, custom_url)
  rpc_client.rs                     # RpcClient with embedded Solana client, DAS API, RPC V2 methods
  request_handler.rs                # HTTP request handling, SDK user agent
  error.rs                          # HeliusError enum, Result<T> alias
  factory.rs                        # HeliusFactory for multi-cluster client creation
  lib.rs                            # Module declarations and public re-exports
  enhanced_transactions.rs          # parse_transactions, parsed_transaction_history
  optimized_transaction.rs          # Helius Sender, smart transactions, priority fees
  wallet.rs                         # Wallet API (identity, balances, transfers, history, funding info)
  webhook.rs                        # Webhook CRUD and address management
  staking.rs                        # Stake account creation, delegation, withdrawal
  websocket.rs                      # Enhanced WebSocket (Geyser) streaming
  types/
    inner.rs                        # Core types (Asset, Cluster, request/response structs)
    enums.rs                        # TokenType, Interface, OwnershipModel, etc.
    options.rs                      # Request option structs
    enhanced_websocket.rs           # WebSocket subscription types
    enhanced_transaction_types.rs   # Enhanced transaction types
  utils/
    is_valid_solana_address.rs      # Address validation
    make_keypairs.rs                # Keypair generation helpers
    deserialize_str_to_number.rs    # Custom serde deserializer
examples/
  das/                              # DAS API examples (get_asset_batch, get_asset_proof_batch, etc.)
  enhanced/                         # Enhanced Transactions API examples (parse, history, filters)
  helius/                           # Helius-specific examples (config, RPC V2 methods)
  solana/                           # Standard Solana RPC examples (get_latest_blockhash)
  transactions/                     # Helius Sender and smart transaction examples (send_smart_transaction)
  wallet/                           # Wallet API examples (balances, identity, transfers, history, funding info)
  webhooks/                         # Webhook CRUD examples (create, edit, delete, append/remove)
  websockets/                       # Enhanced WebSocket examples (transaction/account streaming)
tests/
  tests.rs                          # Test module declarations
  test_builder.rs                   # Builder tests
  test_client.rs                    # Client constructor tests
  test_client_staked.rs             # Staked cluster client tests
  test_config.rs                    # Config tests
  test_enhanced_transactions.rs     # Enhanced transaction tests
  test_factory.rs                   # Factory tests
  test_request_handler.rs           # Request handler tests
  rpc/                              # RPC integration tests with mockito
  webhook/                          # Webhook CRUD tests
  wallet/                           # Wallet API tests
  utils/                            # Utility tests
```

## Adding a New RPC Method

1. **Types** — Add request/response structs in `src/types/inner.rs` (or the appropriate types file) with serde derives and `camelCase` renaming
2. **Re-export** — Add the new types to `src/types/mod.rs` so they're publicly accessible
3. **Implementation** — Add the method to `RpcClient` in `src/rpc_client.rs`, using the `post_rpc_request` helper
4. **Test** — Add an integration test in `tests/rpc/` using `mockito` to mock the API response
5. **Example** — Add a usage example in `examples/` in the relevant subdirectory

### Method Pattern

```rust
// In src/rpc_client.rs — use the post_rpc_request helper
pub async fn get_something(&self, request: GetSomething) -> Result<SomethingResponse> {
    self.post_rpc_request("getSomething", request).await
}
```

## Adding New Types

Match existing patterns. All API-facing types need:

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct NewType {
    pub required_field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_field: Option<u64>,
}
```

For enums that map to string values, use `serde-enum-str`:

```rust
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

#[derive(Serialize_enum_str, Deserialize_enum_str, Debug, Clone, PartialEq)]
pub enum MyEnum {
    #[serde(rename = "value1")]
    Variant1,
    #[serde(rename = "value2")]
    Variant2,
}
```

## Testing

Integration tests use `mockito` for API mocking with `tokio::test`:

```rust
use std::sync::Arc;
use helius::client::Helius;
use helius::config::Config;
use helius::rpc_client::RpcClient;
use helius::types::*;
use mockito::{self, Server};
use reqwest::Client;

#[tokio::test]
async fn test_something_success() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url = server.url();

    // Mock the expected API response
    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    // Create client pointing at mock server
    let config: Arc<Config> = Arc::new(Config {
        api_key: Some(ApiKey::new("fake_api_key").unwrap()),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
        custom_url: None,
    });

    let client = Client::new();
    let rpc_client = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };

    // Make request and assert
}
```

### Test Priorities

1. RPC method correctness
2. Type serialization (`camelCase` mapping round-trips)
3. Error paths and `HeliusError` variants
4. Filter/option combinations

## TLS Features

The SDK supports two TLS backends via Cargo features:

- `native-tls` (default) — uses the OS-native TLS stack
- `rustls` — pure Rust TLS, useful for cross-compilation

Never hardcode a TLS dependency. Use the feature flags so users can choose.

## Do Not Commit

- `Cargo.lock` — gitignored (standard for libraries); managed by Cargo
- `target/` — build output
- `src/main.rs` — gitignored scratch file
