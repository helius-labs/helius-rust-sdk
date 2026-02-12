# Migration Guide: 0.x to 1.0

This guide covers the breaking changes in the Helius Rust SDK 1.0 release and how to update your code.

## Summary of Changes

- **Simplified constructors**: 6 constructors replaced with 3 (`new`, `new_async`, `new_with_url`)
- **New `HeliusBuilder`**: Builder pattern for advanced configuration
- **Custom URL support**: Use any RPC endpoint without an API key
- **`Config.api_key` is now `Option<ApiKey>`**: Type-safe API key with validation
- **`new()` is still synchronous**: No `.await` needed for basic clients
- **`new_async()` is async**: Only the WebSocket constructor requires `.await`

## Constructor Changes

### `Helius::new()` — unchanged signature, still sync

```rust
// Before (0.x)
let helius = Helius::new("api-key", Cluster::MainnetBeta)?;

// After (1.0) — same!
let helius = Helius::new("api-key", Cluster::MainnetBeta)?;
```

### Removed constructors — use `HeliusBuilder` instead

| Removed Constructor | Replacement |
|---|---|
| `new_with_commitment(key, cluster, commitment)` | `HeliusBuilder::new().with_api_key(key)?.with_cluster(cluster).with_commitment(commitment).build().await?` |
| `new_with_async_solana(key, cluster)` | `HeliusBuilder::new().with_api_key(key)?.with_cluster(cluster).with_async_solana().build().await?` |
| `new_with_async_solana_and_commitment(key, cluster, commitment)` | `HeliusBuilder::new().with_api_key(key)?.with_cluster(cluster).with_async_solana().with_commitment(commitment).build().await?` |
| `new_with_ws(key, cluster)` | `Helius::new_async(key, cluster).await?` |
| `new_with_ws_with_timeouts(key, cluster, ping, pong)` | `HeliusBuilder::new().with_api_key(key)?.with_cluster(cluster).with_websocket(ping, pong).build().await?` |

### New constructors

```rust
// Full-featured: async Solana + WebSocket + confirmed commitment (async)
let helius = Helius::new_async("api-key", Cluster::MainnetBeta).await?;

// Custom URL, no API key required (sync)
let helius = Helius::new_with_url("http://localhost:8899")?;
```

## Config Struct Changes

The `Config` struct has two new fields:

```rust
// Before (0.x)
let config = Config {
    api_key: "my-key".to_string(),
    cluster: Cluster::Devnet,
    endpoints: HeliusEndpoints::for_cluster(&Cluster::Devnet),
};

// After (1.0)
use helius::types::ApiKey;

let config = Config {
    api_key: Some(ApiKey::new("my-key")?),
    cluster: Cluster::Devnet,
    endpoints: HeliusEndpoints::for_cluster(&Cluster::Devnet),
    custom_url: None,  // new field
};
```

### Accessing the API key

```rust
// Before (0.x)
let key: &str = &config.api_key;

// After (1.0)
let key: &str = config.api_key.as_ref().unwrap().as_str();

// Or use the helper method
if config.has_api_key() {
    let key = config.require_api_key("my feature")?;
}
```

## HeliusBuilder (New)

The builder provides fine-grained control over client configuration:

```rust
use helius::HeliusBuilder;
use helius::types::Cluster;
use solana_commitment_config::CommitmentConfig;

let helius = HeliusBuilder::new()
    .with_api_key("your-key")?
    .with_cluster(Cluster::MainnetBeta)
    .with_async_solana()
    .with_commitment(CommitmentConfig::confirmed())
    .with_websocket(Some(5), Some(15))  // custom ping/pong
    .build()
    .await?;
```

### Custom URL with builder

```rust
let helius = HeliusBuilder::new()
    .with_custom_url("https://my-rpc-provider.com/")?
    .with_custom_api_url("https://api.example.com/")?  // optional separate API endpoint
    .with_api_key("optional-key")?
    .build()
    .await?;
```

## Quick Find-and-Replace

For most codebases, these replacements will cover the migration:

| Find | Replace |
|---|---|
| `Helius::new_with_ws(key, cluster).await?` | `Helius::new_async(key, cluster).await?` |
| `Helius::new_with_async_solana(key, cluster)?` | `HeliusBuilder::new().with_api_key(key)?.with_cluster(cluster).with_async_solana().build().await?` |
| `api_key: "key".to_string()` (in Config) | `api_key: Some(ApiKey::new("key")?)` |
| `config.api_key` (as &str) | `config.api_key.as_ref().unwrap().as_str()` |
