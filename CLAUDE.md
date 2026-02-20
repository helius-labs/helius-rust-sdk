# Helius Rust SDK - Contributing Guide

Guide for AI agents contributing to the Helius Rust SDK repository.

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
- `src/client.rs` — `Helius` struct, constructors (`new`, `new_async`, `new_with_url`), accessor methods
- `src/builder.rs` — `HeliusBuilder` for flexible client configuration
- `src/factory.rs` — `HeliusFactory` for creating clients across multiple clusters
- `src/config.rs` — `Config` struct (`api_key`, `cluster`, `endpoints`, `custom_url`)
- `src/rpc_client.rs` — `RpcClient` with embedded Solana client, DAS API methods, RPC V2 methods
- `src/request_handler.rs` — HTTP request handling
- `src/error.rs` — `HeliusError` enum & `Result<T>` type alias

### Features
- `src/enhanced_transactions.rs` — `parse_transactions`, `parsed_transaction_history`
- `src/optimized_transaction.rs` — Smart transactions, Helius Sender, priority fees
- `src/webhook.rs` — Webhook CRUD and address management
- `src/wallet.rs` — Wallet API (identity, balances, transfers, history, funding source)
- `src/staking.rs` — Stake account creation, delegation, unstaking, withdrawal
- `src/websocket.rs` — Enhanced WebSocket (Geyser) for transaction/account streaming

### Types
- `src/types/inner.rs` — Core types (`Asset`, `Cluster`, request/response structs, filters)
- `src/types/enums.rs` — `TokenType`, `Interface`, `OwnershipModel`, etc.
- `src/types/options.rs` — Request option structs
- `src/types/enhanced_websocket.rs` — WebSocket subscription types
- `src/types/enhanced_transaction_types.rs` — Enhanced transaction types

### Examples (`examples/`)
- `das/` — DAS API examples (get_asset_batch, get_asset_proof_batch, get_all_*)
- `enhanced/` — Enhanced transaction parsing examples
- `helius/` — Helius-specific examples (config, RPC V2 methods, priority fees)
- `solana/` — Standard Solana RPC examples (get_latest_blockhash)
- `transactions/` — Smart transaction and Helius Sender examples
- `wallet/` — Wallet API examples (identity, balances, transfers, history)
- `webhooks/` — Webhook CRUD examples
- `websockets/` — Enhanced WebSocket streaming examples

### Tests
- `tests/rpc/` — RPC integration tests with mockito
- `tests/webhook/` — Webhook tests
- `tests/wallet/` — Wallet API tests
- `tests/utils/` — Utility tests

## Code Style

### Error Handling
Always use the `Result<T>` alias (not `std::result::Result<T, HeliusError>`):
```rust
// In SDK source code (src/)
use crate::error::Result;

// In examples and tests
use helius::error::Result;

pub async fn my_function() -> Result<Asset> { ... }
```

### Async
All async functions return `Result` or `Option`:
```rust
pub async fn fetch_data(&self) -> Result<Data> { ... }
```

### Serialization
Use `#[serde(rename_all = "camelCase")]` for JSON structs:
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_details: Option<TransactionDetails>,
}
```

### Naming
- `snake_case` for functions and variables
- `PascalCase` for types and structs
- `SCREAMING_SNAKE_CASE` for constants

### Documentation
Use `///` doc comments with examples for public items.

## Testing

Integration tests in `tests/rpc/`, `tests/webhook/`, `tests/wallet/` use `mockito` for API mocking:
```rust
#[tokio::test]
async fn test_get_asset_success() {
    let mut server = Server::new_with_opts_async(...).await;
    // Setup mock, create client, assert results
}
```

### Test Priorities
1. RPC methods
2. Type serialization (camelCase mapping)
3. Error paths
4. Filter combinations

### CI/CD
GitHub Actions runs `cargo fmt`, `cargo clippy`, and `cargo test` on all PRs.

## Git Workflow

### Branches
- `main` — stable releases
- `dev` — target for PRs

### PR Process
1. Branch from `dev`
2. Run `cargo fmt && cargo clippy && cargo test`
3. Open PR to `dev`
4. Title format: `feat(domain): [title]` or `fix(domain): [title]`
5. Include Co-Authored-By for AI: `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>`

### Changelog
`CHANGELOG.md` follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format. Before cutting a release:
1. Move all entries from `## [Unreleased]` into a new versioned section (e.g. `## [1.0.0] - YYYY-MM-DD`)
2. Update the comparison links at the bottom of `CHANGELOG.md`
3. Update the version in `Cargo.toml` and `llms.txt`

### Releases
Releases use a two-stage pipeline:

**Stage 1 — Tag the release:**
```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```
This triggers `.github/workflows/release.yml`, which extracts the matching section from `CHANGELOG.md` and opens a **draft** GitHub Release for review.

**Stage 2 — Publish:**
Once the draft release is reviewed and published on GitHub, `.github/workflows/publish_crate.yml` triggers automatically and publishes the crate to crates.io.

## Boundaries

### Never Commit
API keys, secrets, `.env` files, private keys.

### Compatibility
Match Helius API specs exactly. Sync types with API changes.

### Deprecation
Mark deprecated items with `#[deprecated]`.

### Breaking Changes
Bump version, document in CHANGELOG, provide migration guide.

---

See [README.md](README.md), [CONTRIBUTIONS.md](CONTRIBUTIONS.md) | Docs: https://www.helius.dev/docs
