# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0-alpha.1] - 2026-07-02

### Added
- **Pre Confirmations** (`preconfSubscribe`): new standalone `preconf::PreconfClient` for Helius's lowest-latency transaction stream, delivering scheduled transactions over WebSocket before they are shredded. Yields a stream of `PreconfNotification { version, slot, transaction_index, status, transaction, transaction_bytes }`, deserializing the bincode `VersionedTransaction` and exposing the raw bytes. Served from the Gatekeeper endpoint (`wss://beta.helius-rpc.com`). Notifications are **binary** frames (the subscribe ack is a JSON text frame); the little-endian layout is `version:u8 | slot:u64_le | transaction_index:u64_le | status:u8 | bincode(VersionedTransaction)`. The `version` byte is checked first (currently `1`; unknown versions are dropped) and `status` is exposed as the `PreconfStatus` enum (`Failed = 0`, `Success = 1`, `Unknown = 2`). Credit-based pricing (10 credits per notification). Includes `PreconfClient::connect`/`connect_with_api_key`/`shutdown`, `PreconfStream`, `CURRENT_VERSION`, and `examples/websockets/preconf_subscribe.rs`. A pre-confirmation is an early signal, not a guarantee; coverage is not continuous (scales with stake forwarding to Helius — expect gaps).
- **Sender Max bundles**: new `send_bundle_with_sender` submits up to 5 transactions to Sender Max via `sendBundle` (`params: [[base64Tx, ...], { encoding: "base64" }]`). The caller includes only the 0.001 SOL Sender tip in ≥1 transaction; Helius adds any pathway tips — callers must not add a separate pathway-specific tip. Landing is tracked per-transaction by signature (not bundle IDs / `getBundleStatuses`).
- `get_wallet_balance_at` Wallet API method for querying a wallet's balance of a specific token or native SOL at a past timestamp, datetime, or slot
- `BalanceAtQuery` enum (`Time`, `Datetime`, `Slot`) for selecting the historical point to query, plus `BalanceAtResponse`, `BalanceAtRequested`, and `BalanceAtAsOf` response types
- Versioned Enhanced Transactions namespace: `helius.enhanced().v1()` for existing v1 parse/history methods and `helius.enhanced().v2()` for parser-v2 methods
- Enhanced Transactions v2 support via `transactions(TransactionsV2Request)` and `transaction_history(TransactionHistoryV2Request)` using the unprefixed `POST /transactions` and `POST /transaction-history` API paths
- Enhanced Transactions v2 program filters now support `instructionNames` as an alternative to raw instruction discriminators

### Changed
- Minimum advertised Rust version is now 1.89.0 to match the resolved Solana 3.x dependency requirements

### Fixed
- Rust 1.96 compatibility for Enhanced Transactions v2 status enums by avoiding ambiguous `Error` variant macro expansion

### Changed
- **BREAKING (defaults): Sender tip tiers.** The non-SWQOS tier is now branded **Sender Max** (`swqos_only = false`) with a minimum tip of **0.001 SOL** (`MIN_TIP_LAMPORTS_MAX = 1_000_000`), up from the removed 0.0002 SOL tier. SWQOS-only (`swqos_only = true`) is unchanged at 0.000005 SOL (`MIN_TIP_LAMPORTS_SWQOS = 5_000`). `determine_tip_lamports` and `send_smart_transaction_with_sender` now floor non-SWQOS tips at 0.001 SOL.
- **Sender requirements relaxed.** `skip_preflight` is no longer mandated — it is now a caller-controlled passthrough on `SenderSendOptions` (defaults to `true` for backward compatibility, set `false` to run preflight). A priority fee is recommended but no longer documented as a hard requirement; only the tip is mandatory. Doc comments describe routing generically (multiple high-speed pathways + priority auction).
- **BREAKING (API): `SenderSendOptions` is now `#[non_exhaustive]`** and gains a public `skip_preflight: bool` field. Because the struct is `#[non_exhaustive]`, downstream code can no longer construct it with a struct literal; use `SenderSendOptions::default()` / `SenderSendOptions::new()` plus the new `with_region`/`with_swqos_only`/`with_skip_preflight`/`with_poll_timeout_ms`/`with_poll_interval_ms` builder methods. Field reads/writes on an existing value are unaffected.

### Deprecated
- `MIN_TIP_LAMPORTS_DUAL` is deprecated in favor of `MIN_TIP_LAMPORTS_MAX`. It is now an alias resolving to the Sender Max minimum (0.001 SOL), not the removed 0.0002 SOL value.

## [1.1.0] - 2026-04-29

### Added
- `TransactionSignatureEntry` struct with typed fields (`signature`, `slot`, `transaction_index`, `err`, `memo`, `block_time`, `confirmation_status`) for the `signatures` response mode of `getTransactionsForAddress`
- `FullTransactionEntry` struct with typed fields (`slot`, `transaction_index`, `transaction`, `meta`, `block_time`) for the `full` response mode, matching the Helius OpenAPI spec
- `TransactionEntry` enum with `Signature`, `Full`, and `Unknown` variants for type-safe access with a fallback for forward compatibility
- `FullTransactionNotification` struct for full-mode `transactionSubscribe` payloads, including `transaction`, `signature`, `slot`, and `transaction_index`
- Admin API support via `get_project_usage`, including typed project usage models, tests, examples, and docs
- `simd-json` parser on the HTTP response hot path for ~20–25% faster deserialization on DAS-shaped payloads. Falls back to `serde_json` automatically when `simd-json` rejects a payload, so behavior is unchanged for all callers
- `HeliusError::SimdJson` variant for `simd-json` parser failures

### Changed
- **Breaking**: `GetTransactionsForAddressResponse.data` changed from `Vec<serde_json::Value>` to `Vec<TransactionEntry>`, providing typed access to transaction data instead of raw JSON
- **Breaking**: `TransactionNotification` changed from a struct to an enum with `Full`, `Signature`, and `Unknown` variants to support all `transactionSubscribe` detail modes. Code that accessed `event.signature` or `event.transaction` directly must now match on the variant first.

### Fixed
- `transactionSubscribe` with `transactionDetails: "signatures"` no longer causes a decode error due to the missing `transaction` field

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

[Unreleased]: https://github.com/helius-labs/helius-rust-sdk/compare/v1.2.0-alpha.1...HEAD
[1.2.0-alpha.1]: https://github.com/helius-labs/helius-rust-sdk/compare/v1.1.0...v1.2.0-alpha.1
[1.1.0]: https://github.com/helius-labs/helius-rust-sdk/compare/v1.0.1...v1.1.0
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
