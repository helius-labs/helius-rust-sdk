# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `HeliusBuilder` for flexible client configuration (custom timeouts, TLS, connection settings)
- ZK compression RPC methods
- `llms.txt` for improved AI discoverability
- `AGENTS.md` and `CLAUDE.md` contributing guides for AI agents
- GitHub issue templates for bug reports and feature requests
- Extensive doc comments and code documentation across the codebase
- Wallet API support: identity, balances, transfers, transaction history, and funding source endpoints

### Changed
- **Breaking**: Client creation is now handled via `HeliusBuilder::new()` for advanced configuration; `Helius::new()`, `Helius::new_async()`, and `Helius::new_with_url()` remain available as convenience constructors
- Improved SOL to lamports conversion with overflow and precision validation

### Removed
- Deprecated Jito methods that were previously marked with `#[deprecated]`

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
- Initial release of the Helius Rust SDK
- DAS API methods: `get_asset`, `get_asset_batch`, `get_asset_proof`, `get_asset_proof_batch`, `get_assets_by_authority`, `get_assets_by_owner`, `get_assets_by_creator`, `get_assets_by_group`
- Enhanced transaction parsing: `parse_transactions`, `parsed_transaction_history`
- Webhook CRUD and address management
- `HeliusFactory` for creating clients across multiple clusters
- `HeliusError` enum with typed error variants and `Result<T>` alias
- Smart transaction creation with priority fee estimation via `send_smart_transaction`
- Integration test suite using `mockito`

[Unreleased]: https://github.com/helius-labs/helius-rust-sdk/compare/v0.5.1...HEAD
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
[0.1.3]: https://github.com/helius-labs/helius-rust-sdk/releases/tag/v0.1.3
