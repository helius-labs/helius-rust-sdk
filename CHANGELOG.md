# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.1] - 2026-03-20

### Changed
- Replaced `println!`/`eprintln!` in library code with `log` crate macros (`info!`, `warn!`, `error!`, `debug!`) so consumers can capture SDK diagnostics as structured logs
- Added Wallet API to README

### Fixed
- Smart transaction confirmation now surfaces on-chain `TransactionError`s immediately instead of retrying until timeout

## [1.0.0] - 2026-03-12

### Added
- `toggle_webhook` method to enable or disable a webhook without deleting it (PATCH `/v0/webhooks/{webhookID}`)
- `ToggleWebhookRequest` type for the toggle webhook endpoint
- `active` field on the `Webhook` struct indicating whether the webhook is actively receiving deliveries
- `HeliusBuilder` for flexible client configuration (custom timeouts, TLS, connection settings)
- ZK Compression support: 20+ new RPC methods for compressed accounts, token accounts, balances, proofs, and signatures
- Wallet API support: identity, balances, transfers, transaction history, and funding source endpoints
- `llms.txt` for improved AI discoverability
- `AGENTS.md` and `CLAUDE.md` contributing guides for AI agents
- GitHub issue templates for bug reports and feature requests
- Extensive doc comments and code documentation across the codebase
- Migration guide (`MIGRATION.md`) for upgrading from 0.x to 1.0

### Changed
- **Breaking**: Client creation is now handled via `HeliusBuilder::new()` for advanced configuration; `Helius::new()`, `Helius::new_async()`, and `Helius::new_with_url()` remain available as convenience constructors
- **Breaking**: `Config.api_key` changed from `String` to `Option<ApiKey>` with type-safe validation
- Improved SOL to lamports conversion with overflow and precision validation

### Removed
- **Breaking**: Deprecated Jito methods removed (`add_tip_instruction`, `create_smart_transaction_with_tip`, `send_jito_bundle`, `get_bundle_statuses`, `send_smart_transaction_with_tip`, `send_smart_transaction_with_seeds_and_tip`); use Helius Sender instead
- **Breaking**: Removed 5 legacy constructors (`new_with_commitment`, `new_with_async_solana`, `new_with_async_solana_and_commitment`, `new_with_ws`, `new_with_ws_with_timeouts`); use `HeliusBuilder` instead

### Fixed
- `UiTransactionEncoding` variants now serialize correctly as lowercase (`"json"`, `"jsonParsed"`) instead of PascalCase (`"Json"`, `"JsonParsed"`)
- Versioned (v0) smart transactions now produce correct signature ordering when the fee payer differs from the provided signers

## [0.5.1] - 2026-01-19

### Added
- SDK version tracking via `User-Agent` header on all outgoing HTTP requests

### Changed
- Renamed `include_token_accounts` field to `token_accounts` for consistency

## [0.5.0] - 2026-01-08

### Added
- `include_token_accounts` support for `get_token_accounts_for_address`
- RPC methods reference section in `README.md`

## [0.4.1] - 2025-12-20

### Added
- `get_transaction_for_address` RPC method with filter support

### Fixed
- Type mismatch in enhanced WebSocket transaction example
- Clippy warnings following Solana dependency update

### Changed
- Updated Solana dependencies

## [0.3.2] - 2025-09-22

### Added
- `cu_buffer_multiplier` option for compute unit management in smart transactions
- Commitment level options for client initialization
- Explicit request ID in JSON-RPC requests for better tracking and debugging

### Fixed
- Error thrown on compute unit simulation failure instead of silently proceeding

## [0.3.1] - 2025-09-03

### Added
- RPC V2 methods: `get_all_program_accounts` and `get_all_token_accounts_by_owner`

### Fixed
- Module inception errors in the codebase structure

## [0.3.0] - 2025-08-25

### Added
- Helius Sender support for ultra-low-latency transaction submission
- Staking functionality: stake account creation, delegation, unstaking, and withdrawal
- WebSocket devnet cluster support

### Changed
- Default sender endpoint updated for improved reliability
- Priority fee estimation now uses an unsigned pre-flight transaction for better accuracy

### Deprecated
- Jito methods marked with `#[deprecated]`; use Helius Sender instead

### Removed
- Mint API support

### Fixed
- Made `asset_id` field in `GroupDefinition` optional

## [0.2.6] - 2025-04-14

### Changed
- Improved `Metadata` type with better field handling

### Fixed
- `native-tls` and `rustls` are now properly optional features; the `reqwest` and `tokio-tungstenite` TLS dependencies are no longer forced

## [0.2.5] - 2025-02-24

### Changed
- Updated Solana dependencies from 1.18.x to 2.0.x

## [0.2.3] - 2024-12-05

### Fixed
- Minor bug fixes and stability improvements

## [0.2.0] - 2024-06-04

### Added
- Enhanced WebSocket (Geyser) support for real-time transaction and account streaming
- Async Solana client via `Helius::new_async()` with confirmed commitment level
- `create_smart_transaction` for building smart transactions separately from sending
- Multiple signer support for smart transactions via `SmartTransactionConfig`
- Webhook usage examples

### Changed
- Improved `send_smart_transaction` with better priority fee handling

## [0.1.5] - 2024-05-29

### Added
- Webhook address management examples: `append_address_to_webhook`, `remove_address_from_webhook`

### Changed
- Improved `send_smart_transaction` reliability and error handling

## [0.1.4] - 2024-05-26

### Added
- Versioned transaction support in smart transaction building and sending

## [0.1.3] - 2024-05-20

### Added
- Transaction history pagination support
- Option to supply a custom `reqwest::Client` to `HeliusFactory`

### Fixed
- Priority fee estimate levels (`Min`, `Recommended`)
- Various bug fixes and stability improvements

## [0.1.2] - 2024-05-10

### Added
- `remove_addresses_from_webhook` for bulk address removal
- Documentation updates

## [0.1.1] - 2024-05-08

### Added
- Webhook CRUD: create, edit, get, delete, and list webhooks
- `CONTRIBUTIONS.md` contributing guide
- Formatting check in CI workflow
- `get_latest_blockhash` example

### Changed
- Crate renamed to `helius` on crates.io

## [0.1.0] - 2024-04-17

### Added
- Initial release of the Helius Rust SDK
- `Helius` client with `Config`, `HeliusFactory`, and embedded Solana RPC client
- `HeliusError` enum with typed error variants and `Result<T>` type alias
- DAS API methods: `get_asset`, `get_asset_batch`, `get_asset_proof`, `get_asset_proof_batch`, `get_assets_by_authority`, `get_assets_by_owner`, `get_assets_by_creator`, `get_assets_by_group`, `search_assets`, `get_token_accounts`, `get_signatures_for_asset`, `get_nft_editions`
- `get_rwa_asset` RPC method
- Enhanced transaction parsing: `parse_transactions`, `parsed_transaction_history`
- Priority fee estimation via `get_priority_fee_estimate`
- Mint API: minting compressed NFTs
- Utility functions (`make_keypairs`, `deserialize_str_to_number`)
- Integration test suite using `mockito`
- GitHub Actions CI workflow

[Unreleased]: https://github.com/helius-labs/helius-rust-sdk/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/helius-labs/helius-rust-sdk/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.5.1...v1.0.0
[0.5.1]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.3.2...v0.4.1
[0.3.2]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.2.6...v0.3.0
[0.2.6]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.2.3...v0.2.5
[0.2.3]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.2.0...v0.2.3
[0.2.0]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.1.5...v0.2.0
[0.1.5]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/helius-labs/helius-rust-sdk/releases/tag/v0.1.0
