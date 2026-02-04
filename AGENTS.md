# Helius Rust SDK - Agent Guide

## Overview

Asynchronous Rust SDK for Helius APIs and Solana development.

**Stack:** Rust 1.85+, tokio, reqwest, solana-client 3.0.x

**Features:** DAS API, Enhanced Transactions, Webhooks, Smart Transactions, RPC Methods, Helius Sender

## Commands

```bash
cargo build --release          # Build
cargo test                     # Run all tests
cargo fmt && cargo clippy      # Format and lint
cargo run --example <name>     # Run example
cargo doc --open               # Generate docs
cargo publish                  # Publish to crates.io
```

## Structure

### Core
- `src/client.rs` - Helius client & HeliusFactory
- `src/rpc_client.rs` - RPC methods (DAS API, getTransactionsForAddress)
- `src/error.rs` - HeliusError & Result<T> alias
- `src/webhook.rs` - Webhook management
- `src/optimized_transaction.rs` - Smart transactions & Helius Sender
- `src/jito.rs` - ⚠️ DEPRECATED (use Helius Sender)

### Types
- `src/types/inner.rs` - Core types (Asset, filters, request/response)
- `src/types/enums.rs` - Cluster, TokenType, etc.
- `src/types/options.rs` - Request options

### Other
- `examples/` - Usage examples
- `tests/rpc/` - Integration tests with mockito

## Code Style

### Error Handling
Always use `Result<T>` alias (not `std::result::Result<T, HeliusError>`)
```rust
use helius::error::Result;
pub async fn my_function() -> Result<Asset> { ... }
```

### Async
All async functions return Result or Option
```rust
pub async fn fetch_data(&self) -> Result<Data> { ... }
```

### Serialization
Use `#[serde(rename_all = "camelCase")]` for JSON
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_details: Option<TransactionDetails>,
}
```

### Client Usage
```rust
let helius = Helius::new("api_key", Cluster::MainnetBeta)?;
let asset = helius.rpc().get_asset(request).await?;
let balance = helius.connection().get_balance(&pubkey)?;  // sync
```

### TokenAccountsFilter
For `getTransactionsForAddress`:
```rust
use helius::types::inner::TokenAccountsFilter;
// Options: None, BalanceChanged (recommended), All
let options = GetTransactionsForAddressOptions {
    filters: Some(GetTransactionsFilters {
        token_accounts: Some(TokenAccountsFilter::BalanceChanged),
        ..Default::default()
    }),
    ..Default::default()
};
```

### Naming
snake_case (functions), PascalCase (types), SCREAMING_SNAKE_CASE (constants)

### Documentation
Use `///` with examples for public items

## Testing

Integration tests in `tests/rpc/` use mockito for API mocking:
```rust
#[tokio::test]
async fn test_get_asset_success() {
    let mut server = Server::new_with_opts_async(...).await;
    // Setup mock, create client, assert results
}
```

### Test Priorities
RPC methods, type serialization (camelCase), error paths, filter combinations

### CI/CD
GitHub Actions runs fmt, clippy, tests on all PRs

## Git Workflow

### Branches
`main` (stable), `dev` (target for PRs)

### PR Process
1. Branch from `dev`
2. Run `cargo fmt && cargo clippy && cargo test`
3. Open PR to `dev`
4. Title format: `feat(domain): [title]` or `fix(domain): [title]`
5. Include Co-Authored-By for AI: `Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>`

### Releases
```bash
git tag -a v0.5.1 -m "Release v0.5.1"
git push origin v0.5.1
cargo publish
```

## Boundaries

### Never Commit
API keys, secrets, .env files, private keys

### Compatibility
Match Helius API specs exactly. Sync types with API changes.

### Deprecation
Jito methods deprecated - use Helius Sender. Mark with `#[deprecated]`.

### Breaking Changes
Bump version, document in CHANGELOG, provide migration guide.

---

See [README.md](README.md), [CONTRIBUTIONS.md](CONTRIBUTIONS.md) | Docs: https://docs.helius.dev
