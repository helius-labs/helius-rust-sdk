# Helius Rust SDK

Machine-readable guide for AI agents using the Helius Rust SDK to build on Solana.

- Crate: `helius` (crates.io)
- Version: 1.x (uses solana-sdk 3.0, solana-client 3.0)
- Runtime: Async (tokio 1.x)
- Rust: 1.85+
- License: MIT

## Getting an API Key

Get an API key from https://dashboard.helius.dev, or sign up programmatically via the Helius CLI. See the [CLI agent guide](https://dashboard.helius.dev/agents.md) for automated signup instructions.

## Quick Start

```toml
# Cargo.toml
[dependencies]
helius = "1.0.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
solana-sdk = "3.0.0"
```

```rust
use helius::Helius;
use helius::types::Cluster;

#[tokio::main]
async fn main() -> helius::error::Result<()> {
    let helius = Helius::new("your-api-key", Cluster::MainnetBeta)?;

    // Get all NFTs owned by a wallet
    let assets = helius.rpc().get_assets_by_owner(GetAssetsByOwner {
        owner_address: "wallet_address".to_string(),
        page: 1,
        limit: Some(50),
        ..Default::default()
    }).await?;

    // Get transaction history with token account activity
    let txs = helius.rpc().get_transactions_for_address(
        "wallet_address".to_string(),
        GetTransactionsForAddressOptions {
            limit: Some(100),
            transaction_details: Some(TransactionDetails::Full),
            filters: Some(GetTransactionsFilters {
                token_accounts: Some(TokenAccountsFilter::BalanceChanged),
                ..Default::default()
            }),
            ..Default::default()
        },
    ).await?;

    // Send a transaction via Helius Sender (ultra-low latency)
    let sig = helius.send_smart_transaction_with_sender(
        SmartTransactionConfig {
            create_config: CreateSmartTransactionConfig {
                instructions: vec![transfer_instruction],
                signers: vec![wallet_signer],
                ..Default::default()
            },
            ..Default::default()
        },
        SenderSendOptions {
            region: "US_EAST".to_string(),  // Default, US_SLC, US_EAST, EU_WEST, EU_CENTRAL, EU_NORTH, AP_SINGAPORE, AP_TOKYO
            ..Default::default()
        },
    ).await?;

    Ok(())
}
```

## Client Constructors

### `Helius::new` — Basic Sync Client

```rust
let helius = Helius::new("your-api-key", Cluster::MainnetBeta)?;
```

Simplest constructor. Synchronous — no `.await` needed. Provides RPC methods, webhooks, Enhanced Transactions, smart transactions, and Wallet API. No async Solana client or WebSocket support.

### `Helius::new_async` — Full-Featured Async Client

```rust
let helius = Helius::new_async("your-api-key", Cluster::MainnetBeta).await?;
```

Recommended for production. Includes async Solana RPC client, and Enhanced WebSocket streaming, using a confirmed commitment level. Requires `.await` because it establishes a WebSocket connection.

### `Helius::new_with_url` — Custom RPC Endpoint

```rust
let helius = Helius::new_with_url("http://localhost:8899")?;
```

For dedicated RPC nodes, proxies, or local development. No API key required.

### `HeliusBuilder` — Advanced Configuration

```rust
use helius::HeliusBuilder;

let helius = HeliusBuilder::new()
    .with_api_key("your-api-key")?
    .with_cluster(Cluster::MainnetBeta)
    .with_async_solana()
    .with_websocket(None, None)
    .with_commitment(CommitmentConfig::confirmed())
    .build()
    .await?;
```

Builder methods: `with_api_key()`, `with_cluster()`, `with_custom_url()`, `with_custom_api_url()`, `with_custom_ws_url()`, `with_commitment()`, `with_async_solana()`, `with_websocket()`, `with_http_client()`.

### `HeliusFactory` — Multi-Cluster

```rust
let factory = HeliusFactory::new("your-api-key");
let devnet_client = factory.create(Cluster::Devnet)?;
let mainnet_client = factory.create(Cluster::MainnetBeta)?;
```

### Accessing Embedded Solana Clients

```rust
// Synchronous Solana RPC client
let balance = helius.connection().get_balance(&pubkey)?;

// Asynchronous Solana RPC client (requires new_async or HeliusBuilder with_async_solana)
let async_client = helius.async_connection()?;
let balance = async_client.get_balance(&pubkey).await?;

// Enhanced WebSocket client (requires new_async or HeliusBuilder with_websocket)
let ws = helius.ws().expect("WebSocket available");
```

## Recommendations

### Use `get_transactions_for_address` Instead of `get_signatures_for_address` + `get_transaction`

`get_transactions_for_address` combines signature lookup and transaction fetching into a single call with server-side filtering. It supports time/slot ranges, token account filtering, and pagination. This is always preferable to the two-step approach.

```rust
// GOOD: Single call, server-side filtering
let txs = helius.rpc().get_transactions_for_address(
    "address".to_string(),
    GetTransactionsForAddressOptions {
        transaction_details: Some(TransactionDetails::Full),
        limit: Some(100),
        filters: Some(GetTransactionsFilters {
            token_accounts: Some(TokenAccountsFilter::BalanceChanged),
            ..Default::default()
        }),
        ..Default::default()
    },
).await?;

// BAD: Two calls, client-side filtering
let sigs = helius.connection().get_signatures_for_address(&address)?;
let txs: Vec<_> = futures::future::join_all(
    sigs.iter().map(|s| helius.connection().get_transaction(&s.signature, ...))
).await;
```

### Use `send_smart_transaction` for Standard Sends

It automatically simulates, estimates compute units, fetches priority fees, and confirms. Don't manually build `ComputeBudget` instructions.

```rust
let sig = helius.send_smart_transaction(SmartTransactionConfig {
    create_config: CreateSmartTransactionConfig {
        instructions: vec![your_instruction],
        signers: vec![wallet_signer],
        priority_fee_cap: Some(100_000),    // Optional: cap fees in microlamports/CU
        cu_buffer_multiplier: Some(1.1),    // 10% compute unit headroom (default: 1.25)
        ..Default::default()
    },
    ..Default::default()
}).await?;
```

### Use Helius Sender for Ultra-Low Latency

For time-sensitive transactions (arbitrage, sniping, liquidations), reliability, and the fastest landing speeds on Solana, use `send_smart_transaction_with_sender`. It routes through Helius's multi-region infrastructure and Jito.

```rust
let sig = helius.send_smart_transaction_with_sender(
    SmartTransactionConfig {
        create_config: CreateSmartTransactionConfig {
            instructions: vec![your_instruction],
            signers: vec![wallet_signer],
            ..Default::default()
        },
        ..Default::default()
    },
    SenderSendOptions {
        region: "US_EAST".to_string(),
        swqos_only: false,          // true = SWQOS only, false = Dual (SWQOS + Jito)
        poll_timeout_ms: 60_000,
        poll_interval_ms: 2_000,
    },
).await?;
```

### Use `get_asset_batch` for Multiple Assets

When fetching more than one asset, batch them. Don't call `get_asset` in a loop.

```rust
// GOOD: Single request
let assets = helius.rpc().get_asset_batch(GetAssetBatch {
    ids: vec!["mint1".to_string(), "mint2".to_string(), "mint3".to_string()],
    ..Default::default()
}).await?;

// BAD: N requests
let assets: Vec<_> = futures::future::join_all(
    mints.iter().map(|id| helius.rpc().get_asset(GetAsset { id: id.clone(), ..Default::default() }))
).await;
```

### Use Webhooks Instead of Polling

Don't poll `get_transactions_for_address` in a loop. Use webhooks for server-to-server notifications.

```rust
let webhook = helius.create_webhook(CreateWebhookRequest {
    webhook_url: "https://your-server.com/webhook".to_string(),
    webhook_type: WebhookType::Enhanced,
    transaction_types: vec![TransactionType::Transfer, TransactionType::NftSale, TransactionType::Swap],
    account_addresses: vec!["address_to_monitor".to_string()],
    auth_header: Some("Bearer your-secret".to_string()),
    ..Default::default()
}).await?;
```

## Pagination

The SDK uses different pagination strategies depending on the method:

### Token/Cursor-Based (RPC V2 Methods)

```rust
// get_transactions_for_address — uses pagination_token
let mut pagination_token: Option<String> = None;
let mut all_txs = Vec::new();
loop {
    let result = helius.rpc().get_transactions_for_address(
        "address".to_string(),
        GetTransactionsForAddressOptions {
            limit: Some(100),
            pagination_token: pagination_token.clone(),
            ..Default::default()
        },
    ).await?;
    all_txs.extend(result.data);
    pagination_token = result.pagination_token;
    if pagination_token.is_none() { break; }
}

// get_program_accounts_v2 — uses pagination_key
let mut pagination_key: Option<String> = None;
loop {
    let result = helius.rpc().get_program_accounts_v2(
        program_id.to_string(),
        GetProgramAccountsV2Config {
            limit: Some(1000),
            pagination_key: pagination_key.clone(),
            ..Default::default()
        },
    ).await?;
    // process result.accounts
    pagination_key = result.pagination_key;
    if pagination_key.is_none() { break; }
}

// Or use the auto-paginating variant:
let all_accounts = helius.rpc().get_all_program_accounts(
    program_id.to_string(),
    GetProgramAccountsV2Config::default(),
).await?;
```

### Page-Based (DAS API)

```rust
let mut page = 1;
let mut all_assets = Vec::new();
loop {
    let result = helius.rpc().get_assets_by_owner(GetAssetsByOwner {
        owner_address: "...".to_string(),
        page,
        limit: Some(1000),
        ..Default::default()
    }).await?;
    let count = result.items.len();
    all_assets.extend(result.items);
    if count < 1000 { break; }
    page += 1;
}
```

## tokenAccounts Filter

When querying `get_transactions_for_address`, the `token_accounts` filter controls whether token account activity is included:

| Value | Behavior | Use When |
|-------|----------|----------|
| `None` | Only transactions directly involving the address | You only care about SOL transfers and program calls |
| `BalanceChanged` | Also includes token transactions that changed a balance | **Recommended for most agents** — shows token sends/receives without noise |
| `All` | Includes all token account transactions | You need complete token activity (can return many results) |

## `changed_since_slot` — Incremental Account Fetching

`get_program_accounts_v2` and `get_token_accounts_by_owner_v2` (and their auto-paginating `get_all_*` variants) support `changed_since_slot`, a Helius-specific parameter that returns only accounts modified after a given slot.

```rust
// First fetch: get all accounts
let baseline = helius.rpc().get_program_accounts_v2(
    program_id.to_string(),
    GetProgramAccountsV2Config { limit: Some(10_000), ..Default::default() },
).await?;
let last_slot = current_slot; // save the slot you fetched at

// Later: only get accounts that changed since your last fetch
let updates = helius.rpc().get_program_accounts_v2(
    program_id.to_string(),
    GetProgramAccountsV2Config {
        limit: Some(10_000),
        changed_since_slot: Some(last_slot),
        ..Default::default()
    },
).await?;
```

| Use `changed_since_slot` when | Don't use it when |
|------|------|
| Polling for account updates on a schedule | Doing a one-time full fetch |
| Building an indexer that tracks state changes | You need the complete set regardless of recency |
| Reducing response size on large programs | The program has very few accounts |

## Rate Limits & Plans

| Feature | Free | Developer $49/mo | Business $499/mo | Professional $999/mo |
|---------|------|------------------|------------------|----------------------|
| Monthly Credits | 1M | 10M | 100M | 200M |
| RPC Rate Limit | 10 req/s | 50 req/s | 200 req/s | 500 req/s |
| DAS & Enhanced API | 2 req/s | 10 req/s | 50 req/s | 100 req/s |
| Helius Sender | 15/s | 15/s | 15/s | 15/s |
| Enhanced WebSockets | No | No | Yes | Yes |
| LaserStream gRPC | No | Devnet | Devnet | Devnet + Mainnet |

Monitor usage via the [Helius CLI](https://www.helius.dev/docs/api-reference/helius-cli) using `helius usage --json`.

## Error Handling & Retries

```rust
use helius::error::{HeliusError, Result};

match helius.rpc().get_asset(request).await {
    Ok(asset) => { /* success */ }
    Err(HeliusError::Unauthorized { .. }) => { /* 401: invalid or missing API key */ }
    Err(HeliusError::RateLimitExceeded { .. }) => { /* 429: too many requests or out of credits */ }
    Err(HeliusError::InternalError { .. }) => { /* 5xx: server error, retry with backoff */ }
    Err(HeliusError::NotFound { .. }) => { /* 404: resource not found */ }
    Err(HeliusError::BadRequest { .. }) => { /* 400: malformed request */ }
    Err(HeliusError::Timeout { .. }) => { /* transaction confirmation timed out */ }
    Err(e) => { /* other errors: Network, SerdeJson, etc. */ }
}
```

### Retry Strategy

Retry on `RateLimitExceeded` and `InternalError` with exponential backoff. The SDK provides typed error variants, so you can match on them directly:

```rust
async fn with_retry<T, F, Fut>(f: F, max_retries: u32) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    for attempt in 0..=max_retries {
        match f().await {
            Ok(val) => return Ok(val),
            Err(HeliusError::RateLimitExceeded { .. }) | Err(HeliusError::InternalError { .. }) if attempt < max_retries => {
                tokio::time::sleep(std::time::Duration::from_millis(1000 * 2u64.pow(attempt))).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

## Common Gotchas

1. **Don't forget `transaction_details: Some(TransactionDetails::Full)`** — By default, `get_transactions_for_address` returns signatures only, not full transaction data.

2. **Don't manually add ComputeBudget instructions with `send_smart_transaction`** — The SDK adds them automatically. Adding your own will result in a `HeliusError::InvalidInput` error.

3. **Priority fees are in microlamports per compute unit** — Not lamports. When using `get_priority_fee_estimate`, the returned values are already in the right unit for `SetComputeUnitPrice`.

4. **DAS pagination is 1-indexed** — `page: 1` is the first page, not `page: 0`.

5. **`async_connection()` requires `new_async` or `HeliusBuilder`** — Calling `helius.async_connection()` on a client created with `Helius::new()` returns `Err(HeliusError::ClientNotInitialized)`.

6. **`get_asset` returns `Option<Asset>`** — A successful response may still be `None` if the asset doesn't exist. Handle the `Option` explicitly.

7. **Sender tips are mandatory** — `send_smart_transaction_with_sender` automatically determines and appends tips. Minimum 0.0002 SOL (Dual mode) or 0.000005 SOL (SWQOS-only).

8. **TLS feature flags** — The crate defaults to `native-tls`. Use `features = ["rustls"]` (and `default-features = false`) for pure-Rust TLS when OpenSSL is unavailable.

## API Quick Reference

### DAS API (Digital Asset Standard)

```rust
helius.rpc().get_asset(GetAsset { id, .. })                              // Single asset by mint
helius.rpc().get_asset_batch(GetAssetBatch { ids, .. })                  // Multiple assets
helius.rpc().get_assets_by_owner(GetAssetsByOwner { owner_address, page, .. })  // Assets by wallet
helius.rpc().get_assets_by_authority(GetAssetsByAuthority { .. })        // Assets by update authority
helius.rpc().get_assets_by_creator(GetAssetsByCreator { .. })            // Assets by creator
helius.rpc().get_assets_by_group(GetAssetsByGroup { .. })                // Assets by collection
helius.rpc().search_assets(SearchAssets { .. })                          // Flexible search
helius.rpc().get_asset_proof(GetAssetProof { id })                       // Merkle proof (cNFTs)
helius.rpc().get_asset_proof_batch(GetAssetProofBatch { ids })           // Batch Merkle proofs
helius.rpc().get_token_accounts(GetTokenAccounts { .. })                 // Token accounts
helius.rpc().get_nft_editions(GetNftEditions { .. })                     // Print editions
helius.rpc().get_signatures_for_asset(GetAssetSignatures { id, .. })     // Transaction history for asset
```

### RPC V2 Methods

```rust
helius.rpc().get_transactions_for_address(address, options)              // Transaction history (pagination_token)
helius.rpc().get_program_accounts_v2(program_id, config)                 // Program accounts (pagination_key)
helius.rpc().get_all_program_accounts(program_id, config)                // Auto-paginating variant
helius.rpc().get_token_accounts_by_owner_v2(owner, filter, config)       // Token accounts v2 (pagination_key)
helius.rpc().get_all_token_accounts_by_owner(owner, filter, config)      // Auto-paginating variant
helius.rpc().get_priority_fee_estimate(request)                          // Fee estimates
```

### Smart Transactions

```rust
helius.send_smart_transaction(config)                                    // Auto-optimized send
helius.create_smart_transaction(config)                                  // Build without sending
helius.create_smart_transaction_with_seeds(config)                       // Thread-safe (seed-based)
helius.send_smart_transaction_with_seeds(config, send_opts, timeout)     // Thread-safe send
helius.create_smart_transaction_without_signers(config)                  // Unsigned transaction
helius.get_compute_units(instructions, payer, lookup_tables, signers)    // Simulate CU usage
helius.poll_transaction_confirmation(signature)                          // Poll confirmation status
helius.send_and_confirm_transaction(tx, config, last_block, timeout)     // Send + confirm loop
```

### Helius Sender

```rust
helius.send_smart_transaction_with_sender(config, sender_opts)           // Build + send via Sender
helius.create_smart_transaction_with_tip_for_sender(config, tip)         // Build with tip
helius.send_and_confirm_via_sender(tx, last_block, sender_opts)          // Send pre-built tx via Sender
helius.determine_tip_lamports(swqos_only)                                // Calculate tip amount
helius.fetch_tip_floor_75th()                                            // Get Jito tip floor
helius.warm_sender_connection(region)                                    // Warm connection via /ping
```

### Enhanced Transactions

```rust
helius.parse_transactions(ParseTransactionsRequest { transactions })     // Parse by signatures
helius.parsed_transaction_history(ParsedTransactionHistoryRequest { address, .. })  // Parse by address
```

### Webhooks

```rust
helius.create_webhook(CreateWebhookRequest { .. })                       // Create webhook
helius.get_webhook_by_id(webhook_id)                                     // Get webhook config
helius.get_all_webhooks()                                                // List all webhooks
helius.edit_webhook(EditWebhookRequest { .. })                           // Update webhook
helius.delete_webhook(webhook_id)                                        // Delete webhook
helius.append_addresses_to_webhook(webhook_id, &addresses)               // Add monitored addresses
helius.remove_addresses_from_webhook(webhook_id, &addresses)             // Remove monitored addresses
```

### Wallet API

```rust
helius.get_wallet_identity(wallet)                                       // Identity info
helius.get_batch_wallet_identity(&addresses)                             // Batch identity (max 100)
helius.get_wallet_balances(wallet, page, limit, zero_bal, native, nfts)  // Token balances
helius.get_wallet_history(wallet, limit, before, after, tx_type, token_accts)  // Transaction history
helius.get_wallet_transfers(wallet, limit, cursor)                       // Transfer history
helius.get_wallet_funding_source(wallet)                                 // Funding source
```

### Staking

```rust
helius.create_stake_transaction(owner, amount_sol)                       // Create + delegate stake (unsigned tx)
helius.create_unstake_transaction(owner, stake_account)                  // Deactivate stake (unsigned tx)
helius.create_withdraw_transaction(owner, stake_acct, dest, lamports)    // Withdraw (unsigned tx)
helius.get_stake_instructions(owner, amount_sol)                         // Get raw instructions + keypair
helius.get_unstake_instruction(owner, stake_account)                     // Deactivate instruction
helius.get_withdraw_instruction(owner, stake_acct, dest, lamports)       // Withdraw instruction
helius.get_withdrawable_amount(stake_account, include_rent_exempt)       // Check withdrawable balance
helius.get_stake_accounts(wallet)                                        // List stake accounts
```

### Embedded Solana Client

```rust
helius.connection()                                                      // Sync SolanaRpcClient (Arc)
helius.async_connection()?                                               // Async SolanaRpcClient
helius.ws()                                                              // Enhanced WebSocket (Option)
helius.rpc()                                                             // Helius RpcClient (Arc)
helius.config()                                                          // Config (Arc)
```

## Documentation

- Full API Reference: https://docs.rs/helius/latest/helius/
- Helius API Docs: https://www.helius.dev/docs
- LLM-Optimized Docs: https://www.helius.dev/docs/llms.txt
- Examples: https://github.com/helius-labs/helius-rust-sdk/tree/main/examples
- Migration Guide (0.x to 1.0): https://github.com/helius-labs/helius-rust-sdk/blob/main/MIGRATION.md

---

## Contributing

For AI agents contributing to this repository, see [CLAUDE.md](CLAUDE.md) for full details. Quick reference:

```bash
cargo build --release          # Build
cargo test                     # Run all tests
cargo fmt && cargo clippy      # Format and lint
```

Before submitting a PR: `cargo fmt && cargo clippy && cargo test`

Key architecture decisions, code style conventions, and PR process are documented in [CLAUDE.md](CLAUDE.md) and [CONTRIBUTIONS.md](CONTRIBUTIONS.md).
