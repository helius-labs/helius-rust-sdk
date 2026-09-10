# Migration Guide

This guide covers breaking changes between major releases of the Helius Rust SDK and how to
update your code. The most recent upgrade is listed first.

- [2.x → 3.0](#2x--30)
- [1.x → 2.0](#1x--20)
- [0.x → 1.0](#0x--10)

---

## 2.x → 3.0

3.0 is a correctness and resilience release. Only two changes can fail to compile, and both are
one-line fixes. The rest are behavior changes: they need attention only if you depended on the
old behavior, and several of them fix results that were previously wrong. Each change below
includes what to do.

### Summary of Changes

- **Smart-transaction configs gained `loaded_accounts_data_size_limit`**: `CreateSmartTransactionConfig` and `CreateSmartTransactionSeedConfig` have a new field. Struct-literal construction now needs it — add `..Default::default()`.
- **`send_and_confirm_transaction` gained `Clone + Send + 'static` bounds**: only affects callers passing a custom `SerializableTransaction` type.
- **Sender and tip-floor transport failures are no longer `HeliusError::InvalidInput`**: they are now `Network`, and Sender HTTP failure statuses are classified like every other endpoint's.
- **`HeliusError` now exposes its cause chain**: `source()` returns the underlying error instead of `None`.
- **`get_stake_accounts` returns a corrected set of accounts**: it filtered on the withdrawer authority instead of the staker.
- **`loaded_accounts_data_size_limit` is rejected on legacy and v0** instead of being silently ignored, and out-of-range v1 budgets now error.
- **The auto-derived Sender tip is now capped** at `DEFAULT_MAX_TIP_LAMPORTS` (0.01 SOL).
- **Confirmation polling honors your deadline and backs off**: `timeout` is now a real bound, and a failed send returns the send error rather than a generic confirmation timeout.

### Smart-transaction configs

`CreateSmartTransactionConfig` and `CreateSmartTransactionSeedConfig` gained
`loaded_accounts_data_size_limit`, which sets the account-data budget a Transaction v1 carries in
its message header. If you build either with a struct literal that enumerates every field, add
`..Default::default()` — the same fix as the 2.0 upgrade, and it makes future field additions
non-breaking for your code:

```rust
// Before
let config = CreateSmartTransactionConfig {
    instructions,
    signers,
    version: TransactionVersion::V1,
    lookup_tables: None,
    fee_payer: None,
    priority_fee_cap: None,
    cu_buffer_multiplier: None,
    priority_fee_lamports_cap: None,
};

// After
let config = CreateSmartTransactionConfig {
    instructions,
    signers,
    version: TransactionVersion::V1,
    ..Default::default()
};
```

The field defaults to `MAX_LOADED_ACCOUNTS_DATA_SIZE_BYTES` (64 MiB) — the implicit budget legacy
and v0 transactions already get — so the default preserves prior behavior.

### `send_and_confirm_transaction` bounds

Each send attempt now runs on tokio's blocking pool, which needs an owned value outliving the call
frame, so the transaction is cloned per attempt:

```rust
// Before
pub async fn send_and_confirm_transaction(
    transaction: &impl SerializableTransaction, ...

// After
pub async fn send_and_confirm_transaction<T>(transaction: &T, ...) -> Result<Signature>
where
    T: SerializableTransaction + Clone + Send + 'static,
```

`Transaction` and `VersionedTransaction` both already satisfy the bounds, so if you pass either,
no change is needed. If you pass your own `SerializableTransaction` type, add `Clone` (and make
sure it is `Send + 'static`).

### Sender and tip-floor error variants

Transport failures against Sender and the Jito tip-floor feed used to be flattened into
`HeliusError::InvalidInput` with a formatted string, which put network faults in the variant that
otherwise means "you passed bad arguments". They are now `HeliusError::Network`, and Sender HTTP
failure statuses go through the same classification as every other endpoint:

```rust
// Before — matched on a message substring, and could not tell a rate limit from a bad transaction
match helius.send_smart_transaction_with_sender(config, options).await {
    Err(HeliusError::InvalidInput(msg)) if msg.contains("Sender request error") => retry(),
    Err(HeliusError::InvalidInput(msg)) if msg.contains("Sender HTTP 429") => back_off(),
    ...
}

// After — classified variants, plus accessors for the transport cases
match helius.send_smart_transaction_with_sender(config, options).await {
    Err(e) if e.is_connect() || e.is_timeout() => retry(),
    Err(HeliusError::RateLimitExceeded { path }) => back_off(&path),
    Err(HeliusError::Unauthorized { .. }) => refresh_credentials(),
    ...
}
```

If you match `InvalidInput` to detect *your own* bad input, that now works as intended — network
faults no longer land there. If you matched it to catch Sender failures, switch to the above.

### `HeliusError` cause chain

`HeliusError::Network`, `ReqwestError`, and `SerdeJson` now mark the wrapped error as their
`source`, so the cause chain is reachable:

```rust
use std::error::Error;

let mut cause = err.source();
while let Some(c) = cause {
    eprintln!("caused by: {c}");   // client error (Connect) -> tcp connect error -> Connection refused
    cause = c.source();
}
```

`Display` is unchanged, so anything printing `{}` keeps the message it had. Two notes: code that
asserted `source().is_none()` will now see `Some`, and a chain-printing consumer (`anyhow`'s
`{:#}`) repeats the wrapped error's message once, because the variant's own `Display` embeds it.
For classification, prefer `is_connect()` / `is_timeout()` / `as_reqwest()` over walking the chain.

### `get_stake_accounts` authority filter

The `memcmp` filter used offset 44 — `Authorized::withdrawer` — rather than offset 12,
`Authorized::staker`, which is what the method name and docs describe. Where the two authorities
differ (custodial setups, liquid staking, delegated management), this returned the wrong set:
accounts you control as staker were omitted, and accounts where you are only the withdrawer were
included. **The method now returns a different, correct set.** If you compensated for the old
behavior — filtering results client-side, or querying the withdrawer separately — drop the
workaround.

### Budget-field validation

Three cases that used to pass silently now return `HeliusError::InvalidInput`:

- `loaded_accounts_data_size_limit` set on a legacy or v0 transaction. Those formats have nowhere
  to carry it, so it was being dropped. Either move to `version: TransactionVersion::V1` or stop
  setting the field.
- A v1 `compute_unit_limit` outside `1..=MAX_COMPUTE_UNIT_LIMIT`.
- A `loaded_accounts_data_size_limit` outside `1..=MAX_LOADED_ACCOUNTS_DATA_SIZE_BYTES` (`0` read
  as a zero-byte budget under which every account load fails).

### Sender tip ceiling

`determine_tip_lamports` keeps its signature but now clamps the tip derived from the third-party
feed to `DEFAULT_MAX_TIP_LAMPORTS` (0.01 SOL, 10x the Sender Max minimum) instead of applying only
a floor. Existing callers are bounded without a code change. If your workload legitimately tips
above that during congestion, set your own ceiling with
`SenderSendOptions::with_max_tip_lamports` or `determine_tip_lamports_with_cap`. A ceiling below
the tier's minimum tip is unsatisfiable and errors rather than silently paying above it. This
bounds only the *derived* tip — a tip you build into the instructions yourself is untouched.

### Confirmation polling and deadlines

Three related changes to `send_and_confirm_transaction` and the Sender send paths:

- **`timeout` is now a real deadline.** Each confirmation poll previously ran its own fixed
  15-second budget, so `Some(Duration::from_secs(2))` could still block for 15+ seconds. If you
  passed a short timeout and relied on the longer actual wait to get a confirmation, raise it to
  the value you actually want.
- **A failed send returns the send error.** Previously a permanently-failing transaction was
  retried in a tight loop and surfaced as a generic "failed to confirm" with the cause discarded.
  Code matching on that timeout message should expect the underlying error instead.
- **Polling backs off** (400ms, doubling to a 5s ceiling) instead of spinning. A successful
  confirmation now typically costs one `getSignatureStatuses` call; a transaction that never
  confirms costs about six over the timeout.

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
