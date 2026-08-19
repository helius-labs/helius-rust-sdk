# Migration Guide

This guide covers breaking changes between major releases of the Helius Rust SDK and how to
update your code. The most recent upgrade is listed first.

- [1.x → 2.0](#1x--20)
- [0.x → 1.0](#0x--10)

---

## 1.x → 2.0

2.0 is a correctness-focused release. Most breaking changes are narrow type refinements — many
codebases need no changes, or only trivial ones. Each change below includes what to do.

### Summary of Changes

- **`RpcResponse<T>` fields are now optional**: `result` is `Option<T>`, `id` is `Option<String>`, and there is a new `error` field. Only affects code that inspects `RpcResponse` directly.
- **`SenderSendOptions` is `#[non_exhaustive]`**: construct it via `default()`/`new()` + builder methods instead of a struct literal.
- **`ProgramName::Unkown` renamed to `ProgramName::Unknown`**: fixes a typo that never matched the API's `"UNKNOWN"`.
- **`GetAssetsByOwner.limit` is now `Option<u32>`** (was `Option<i32>`).
- **`EnhancedTransaction.fee` and `.slot` are now `u64`** (were `i32`).
- **Sender tip defaults changed**: the non-SWQOS tier ("Sender Max") now floors tips at 0.001 SOL.
- **Solana crates upgraded to the Agave 4.2 line (`solana-*` 4.x)**: align your own `solana-*` dependencies to 4.x if you share Solana types with the SDK.
- **Smart-transaction configs gained fields**: `CreateSmartTransactionConfig` and `CreateSmartTransactionSeedConfig` have new `version` and `priority_fee_lamports_cap` fields (for Transaction v1). Struct-literal construction now needs them — add `..Default::default()`.

### `RpcResponse<T>`

Server-side JSON-RPC errors (invalid params, unknown method, etc.) are now surfaced as
`HeliusError::RpcError { code, message }` instead of a misleading deserialization error. To model
this, the envelope changed:

```rust
// Before
pub struct RpcResponse<T> { pub jsonrpc: String, pub id: String, pub result: T }

// After
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub id: Option<String>,
    pub result: Option<T>,
    pub error: Option<RpcError>,
}
```

**If you call the high-level methods** (`helius.rpc().get_asset(...)`, etc.) you are unaffected —
they still return `T`, and now return `Err(HeliusError::RpcError { .. })` on a server error.

**If you read `RpcResponse` directly**, handle the `Option`:

```rust
// Before
let asset = response.result;
// After
let asset = response.result.ok_or_else(|| /* your error */)?;
```

### `SenderSendOptions` is `#[non_exhaustive]`

Struct-literal construction no longer compiles. Use the constructors and builder methods:

```rust
// Before
let opts = SenderSendOptions { region: Some(r), swqos_only: true, ..Default::default() };

// After
let opts = SenderSendOptions::new()
    .with_region(r)
    .with_swqos_only(true);
```

Reading and writing fields on an existing value is unchanged.

### `ProgramName::Unkown` → `ProgramName::Unknown`

Rename any references. Previously the typo'd variant serialized to `"UNKOWN"` and the API's real
`"UNKNOWN"` fell through to `ProgramName::Other("UNKNOWN")`, so if you matched that string via
`Other`, switch to the typed variant:

```rust
// Before
ProgramName::Unkown => { /* ... */ }
ProgramName::Other(s) if s == "UNKNOWN" => { /* ... */ }
// After
ProgramName::Unknown => { /* ... */ }
```

### Numeric type refinements

`limit`, `fee`, and `slot` are non-negative, so they moved from signed to unsigned types:

```rust
// GetAssetsByOwner.limit:      Option<i32> -> Option<u32>
// EnhancedTransaction.fee:     i32         -> u64
// EnhancedTransaction.slot:    i32         -> u64
```

Integer literals (`Some(1000)`, `assert_eq!(tx.fee, 5000)`) need no change. Update only code that
stored these in an explicit `i32`/`i64` binding or did signed arithmetic on them.

### Sender tip defaults

The non-SWQOS Sender tier is now branded **Sender Max** and floors tips at **0.001 SOL**
(`MIN_TIP_LAMPORTS_MAX = 1_000_000`), up from the removed 0.0002 SOL tier. SWQOS-only
(`swqos_only = true`) is unchanged at 0.000005 SOL. `MIN_TIP_LAMPORTS_DUAL` is deprecated; use
`MIN_TIP_LAMPORTS_MAX`. If you relied on the old lower non-SWQOS floor, budget for the higher tip.

### Solana 4.x and smart-transaction configs

The SDK now targets the Agave 4.2 (`solana-*` 4.x) crates. If your code shares Solana types with
the SDK (e.g. `Instruction`, `Pubkey`, `VersionedTransaction`), align your own `solana-*`
dependencies to the 4.x line.

`CreateSmartTransactionConfig` and `CreateSmartTransactionSeedConfig` gained `version` and
`priority_fee_lamports_cap` fields. If you build them with a struct literal, add
`..Default::default()`:

```rust
// Before
let config = CreateSmartTransactionConfig {
    instructions,
    signers,
    lookup_tables: None,
    fee_payer: None,
    priority_fee_cap: None,
    cu_buffer_multiplier: None,
};

// After
let config = CreateSmartTransactionConfig {
    instructions,
    signers,
    ..Default::default()
};
```

To opt into a larger **Transaction v1** (Agave 4.2), set `version: TransactionVersion::V1` (v1 does
not support `lookup_tables`). This is additive — the default (`TransactionVersion::Auto`) preserves
the previous legacy/v0 behavior.

On the receive side, the default `max_supported_transaction_version` on `TransactionSubscribeOptions`
and `GetTransactionsForAddressOptions` changed from `0` to `1`, so v1 transactions are returned/
streamed instead of triggering a version error. There is no compatibility risk — the RPC accepts any
`u8` — but if you explicitly want v0-only behavior, set the field to `Some(0)`.

The public `get_compute_units` and `get_compute_units_thread_safe` methods gained a required
`version: TransactionVersion` argument (pass `TransactionVersion::Auto` to preserve the previous
legacy/v0 simulation behavior). They now also return an error when the simulation fails rather than
reporting zero compute units.

Note: Transaction v1 is not active on any cluster yet — `simulateTransaction` returns
`UnsupportedVersion`, so building or sending a v1 smart transaction errors until the feature gate
activates.

---

## 0.x → 1.0

This section covers the breaking changes in the Helius Rust SDK 1.0 release and how to update your code.

### Summary of Changes

- **Simplified constructors**: 6 constructors replaced with 3 (`new`, `new_async`, `new_with_url`)
- **New `HeliusBuilder`**: Builder pattern for advanced configuration
- **Custom URL support**: Use any RPC endpoint without an API key
- **`Config.api_key` is now `Option<ApiKey>`**: Type-safe API key with validation
- **`new()` is still synchronous**: No `.await` needed for basic clients
- **`new_async()` is async**: Only the WebSocket constructor requires `.await`
- **Jito methods removed**: Use Helius Sender instead
- **`UiTransactionEncoding` serialization fixed**: Variants now serialize as lowercase (`"json"`, `"jsonParsed"`)

### Constructor Changes

#### `Helius::new()` — unchanged signature, still sync

```rust
// Before (0.x)
let helius = Helius::new("api-key", Cluster::MainnetBeta)?;

// After (1.0) — same!
let helius = Helius::new("api-key", Cluster::MainnetBeta)?;
```

#### Removed constructors — use `HeliusBuilder` instead

| Removed Constructor | Replacement |
|---|---|
| `new_with_commitment(key, cluster, commitment)` | `HeliusBuilder::new().with_api_key(key)?.with_cluster(cluster).with_commitment(commitment).build().await?` |
| `new_with_async_solana(key, cluster)` | `HeliusBuilder::new().with_api_key(key)?.with_cluster(cluster).with_async_solana().build().await?` |
| `new_with_async_solana_and_commitment(key, cluster, commitment)` | `HeliusBuilder::new().with_api_key(key)?.with_cluster(cluster).with_async_solana().with_commitment(commitment).build().await?` |
| `new_with_ws(key, cluster)` | `Helius::new_async(key, cluster).await?` |
| `new_with_ws_with_timeouts(key, cluster, ping, pong)` | `HeliusBuilder::new().with_api_key(key)?.with_cluster(cluster).with_websocket(ping, pong).build().await?` |

#### New constructors

```rust
// Full-featured: async Solana + WebSocket + confirmed commitment (async)
let helius = Helius::new_async("api-key", Cluster::MainnetBeta).await?;

// Custom URL, no API key required (sync)
let helius = Helius::new_with_url("http://localhost:8899")?;
```

### Config Struct Changes

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

#### Accessing the API key

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

### HeliusBuilder (New)

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

#### Custom URL with builder

```rust
let helius = HeliusBuilder::new()
    .with_custom_url("https://my-rpc-provider.com/")?
    .with_custom_api_url("https://api.example.com/")?  // optional separate API endpoint
    .with_api_key("optional-key")?
    .build()
    .await?;
```

### Jito Methods Removed

All Jito methods (deprecated in 0.3.0) have been removed. Use Helius Sender instead:

| Removed Method | Replacement |
|---|---|
| `add_tip_instruction(...)` | Not needed — Helius Sender handles fees automatically |
| `send_jito_bundle(...)` | `send_smart_transaction_with_sender(config, options).await?` |
| `get_bundle_statuses(...)` | Check transaction status via `connection().get_signature_statuses(...)` |
| `create_smart_transaction_with_tip(...)` | `create_smart_transaction(config).await?` |
| `send_smart_transaction_with_tip(...)` | `send_smart_transaction_with_sender(config, options).await?` |
| `send_smart_transaction_with_seeds_and_tip(...)` | `send_smart_transaction_with_seeds_and_sender(config, options).await?` |

```rust
// Before (0.x) — Jito
let sig = helius.send_smart_transaction_with_tip(config, None, Some(50_000)).await?;

// After (1.0) — Helius Sender
use helius::types::SendOptions;
let sig = helius.send_smart_transaction_with_sender(config, Some(SendOptions {
    skip_preflight: true,
    ..Default::default()
})).await?;
```

### UiTransactionEncoding Serialization Fix

`UiTransactionEncoding` variants now serialize as lowercase camelCase to match the Solana RPC spec. If you were passing raw encoding strings to work around this bug, you can now use the enum directly:

```rust
// Before (0.x) — enum serialized incorrectly ("Json", "JsonParsed")
// You may have used raw strings as a workaround

// After (1.0) — enum serializes correctly ("json", "jsonParsed")
use helius::types::UiTransactionEncoding;
let encoding = UiTransactionEncoding::Json;       // serializes as "json"
let encoding = UiTransactionEncoding::JsonParsed;  // serializes as "jsonParsed"
```

### Quick Find-and-Replace

For most codebases, these replacements will cover the migration:

| Find | Replace |
|---|---|
| `Helius::new_with_ws(key, cluster).await?` | `Helius::new_async(key, cluster).await?` |
| `Helius::new_with_async_solana(key, cluster)?` | `HeliusBuilder::new().with_api_key(key)?.with_cluster(cluster).with_async_solana().build().await?` |
| `api_key: "key".to_string()` (in Config) | `api_key: Some(ApiKey::new("key")?)` |
| `config.api_key` (as &str) | `config.api_key.as_ref().unwrap().as_str()` |
