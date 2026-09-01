# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Upgraded the Solana dependencies to Agave 4.3.0-beta.3 and wincode 0.6.1. The minimum supported Rust version is now 1.97.1, matching Agave.
- **`send_and_confirm_transaction` gained `Clone + Send + 'static` bounds** on its transaction parameter (`transaction: &impl SerializableTransaction` became a named generic `T`). Each send attempt now runs on the blocking pool, which needs an owned value outliving the call frame, so the transaction is cloned per attempt. `Transaction` and `VersionedTransaction` both satisfy the bounds, so callers passing either are unaffected; a caller passing a custom `SerializableTransaction` type must add the bounds.

### Added
- `Helius::poll_transaction_confirmation_with_timeout` and the `DEFAULT_CONFIRMATION_POLL_TIMEOUT` constant, so a caller holding an overall deadline can bound a confirmation poll by the time actually remaining rather than by the fixed default. `poll_transaction_confirmation` is unchanged and delegates to it with the default.

### Fixed
- **Blocking Solana RPC calls ran on tokio worker threads.** `Helius::connection()` returns the *blocking* `solana_client::rpc_client::RpcClient` — itself a `block_on` wrapper around a nonblocking client and its own runtime — and 21 call sites invoked it directly from inside `async fn`s (12 in `optimized_transaction.rs`, 9 in `staking.rs`). Each one parked a tokio worker for the whole round trip, and a parked worker cannot be yielded or stolen, so enough concurrent callers starve the runtime: unrelated tasks stop making progress and a server-side caller can be taken down by ordinary traffic. Every site now runs on tokio's blocking pool via the internal `run_blocking_rpc` helper. Error mapping and serialization are unchanged at every site. The previous release fixed only the one instance that compounded with a hot loop (`get_signature_statuses` in `poll_transaction_confirmation`); this covers the rest.
- **`get_stake_accounts` filtered on the wrong authority.** The `memcmp` filter used offset 44, which is `Authorized::withdrawer` in the bincode encoding of `StakeStateV2`, not `Authorized::staker` (offset 12) as the method name and docs claim. For accounts where the two authorities differ — custodial setups, liquid staking, delegated management — this returned the wrong set: it omitted accounts the wallet controls as staker and included ones where it is only the withdrawer. A caller building a `deactivate` (which requires staker authority) against those results would have it rejected on-chain. **Behavior change:** the method now returns a different, correct set of accounts; callers who compensated for the old behavior should drop the workaround. Adds a unit test pinning both authority offsets against a serialized `StakeStateV2`, and an integration test asserting the outgoing `getProgramAccounts` filter uses offset 12. The offset table in the doc comment was also wrong in its own right and has been rewritten.
- **`poll_transaction_confirmation` spun a hot loop and blocked a tokio worker.** A status of `Processed` — the normal state in the moments right after submission — matched neither the confirmed nor the error branch and fell through to the next iteration with no sleep. Because the underlying `get_signature_statuses` is the *blocking* Solana client and that path contained no `.await`, the loop never yielded, pinning a runtime worker thread for the full 15-second timeout while issuing back-to-back RPC calls. Every send/confirm path funnels through this function, so a handful of ordinary concurrent transactions could starve the runtime of a server-side caller and trip upstream rate limits. The poll now always sleeps between checks and runs the blocking call on `spawn_blocking`. The wait is an exponential backoff (400ms, doubling to a 5s ceiling) rather than the previous fixed 5s: a flat interval would have fixed the spin at the cost of adding up to 5 seconds of latency to every successful send, since the first check almost always lands while the transaction is still `Processed`. **Request-volume trade-off:** a successful confirmation now typically costs a single `getSignatureStatuses` call rather than one made up to 5 seconds late, while a transaction that never confirms costs about six calls over the timeout instead of three. The spin this replaces could issue hundreds.
- **`send_and_confirm_transaction` discarded the send error and retried without backoff.** A failed send hit `Err(_) => continue`, so a permanently-failing transaction (a malformed one, say) was retried in a tight loop until the outer timeout and surfaced only as a generic "failed to confirm", with the actual cause thrown away. The error is now retained and returned in place of the timeout, and a 500ms pause separates send attempts. A retained error is cleared as soon as a later attempt sends successfully, so a transient early failure cannot mask a genuine confirmation timeout.
- **The `timeout` argument to `send_and_confirm_transaction` was not a real deadline.** Each confirmation poll ran its own fixed 15-second budget to completion, so a caller passing `Some(Duration::from_secs(2))` could still block for 15+ seconds. Polls are now bounded by whatever remains of the caller's deadline. `send_and_confirm_via_sender` and `send_bundle_with_sender` had the same problem with `SenderSendOptions::poll_timeout_ms` and get the same fix.
- **The `serde_json` fallback in the request handler parsed a buffer `simd_json` had already corrupted.** `simd_json` unescapes strings in place, so on a parse or schema failure the retry — and the `Raw JSON` debug log — were reading a mutated buffer rather than the response body. Any payload containing an escaped string made the fallback fail with a misleading error (typically `control character (\u0000-\u001F) found while parsing a string`) which then *replaced* the accurate `simd_json` diagnostic in the returned error. `simd_json` now gets a scratch copy and the fallback reads the original body. This restores real recovery rather than just better diagnostics: payloads `simd_json`'s serde bridge rejects but `serde_json` accepts (`u128` values, for one) were failing the whole request whenever the body also contained an escaped string.

## [2.0.0] - 2026-08-18

### Added
- **Transaction v1 (larger transactions) support** for Agave 4.2. Transaction v1 (SIMD-0385) unlocks larger transactions — up to the new `MAX_TRANSACTION_V1_SIZE` (4,096 bytes, SIMD-0296), vs. the ~1,232-byte legacy/v0 limit. v1 carries its compute-unit limit and **total** priority fee (in lamports) in the message header config rather than in `ComputeBudget` instructions, and does not support address lookup tables.
  - `TransactionVersion` enum (`Auto` | `V1`) and a `version` field on `CreateSmartTransactionConfig` / `CreateSmartTransactionSeedConfig` (defaults to `Auto`, preserving legacy/v0 behavior). Set `TransactionVersion::V1` to build a v1 smart transaction; it flows through `create_smart_transaction`, `create_smart_transaction_with_seeds`, `send_smart_transaction`, and the Helius Sender path. A new `priority_fee_lamports_cap` field caps the absolute v1 priority fee.
  - `build_v1_transaction` low-level helper and the `MAX_TRANSACTION_V1_SIZE` constant (aliased to `solana_message::v1::MAX_TRANSACTION_SIZE`). v1 transactions are serialized with the **wincode** wire format (version byte `0x81` first, signatures as a fixed-length array last) — the format the RPC client and validators use — on both the size check and the Helius Sender submission path. bincode (used for legacy/v0) misencodes v1, so the Sender path now serializes via wincode for all versions (byte-identical to bincode for legacy/v0).
  - v1 rejects `lookup_tables` up front, adds no `ComputeBudget` instructions (they are no-ops on v1), always sets the loaded-accounts data-size limit (defaulting to `MAX_LOADED_ACCOUNTS_DATA_SIZE_BYTES`, since v1 treats an unset limit as 0), and enforces the v1 instruction/address/signature limits via `Message::validate`.
  - Compute-unit simulation is compiled as a **v1** message when building a v1 transaction, so transactions in the 1,232–4,096-byte range (the point of v1) are not rejected as oversized against the v0 size limit. The priority-fee estimate is requested via `account_keys` (the writable accounts) rather than a serialized draft, avoiding the size cap and needing no server-side v1 parsing. The per-CU estimate is converted to v1's total-lamports fee.
  - **Not yet active**: v1 is not activated on any cluster as of this release — `simulateTransaction` returns `UnsupportedVersion`, so building/sending a v1 smart transaction errors until the SIMD-0296/0385 feature gate activates.
  - **Breaking**: `get_compute_units` and `get_compute_units_thread_safe` gained a required `version: TransactionVersion` parameter (they now compile a v1 message for simulation when it is `V1`). They also now return an error when the simulation itself fails instead of proceeding with a zero compute-unit count.
  - `create_smart_transaction_without_signers` (the unsigned/offline-signing path) does not yet support v1 and returns `HeliusError::InvalidInput` when `V1` is requested.
  - **Receive side**: `max_supported_transaction_version` now defaults to `1` (was `0` / unset) on `TransactionSubscribeOptions` (enhanced WebSocket) and `GetTransactionsForAddressOptions`, so v1 transactions are returned/streamed rather than triggering a version error. Requesting a transaction newer than this value still fails fast.
  - Requires the Agave 4.2 (`solana-*` 4.x) crates. Example: `examples/transactions/create_smart_transaction_v1.rs`.
- **Pre Confirmations** (`preconfSubscribe`): new standalone `preconf::PreconfClient` for Helius's lowest-latency transaction stream, delivering scheduled transactions over WebSocket before they are shredded. Yields a stream of `PreconfNotification { version, slot, transaction_index, status, transaction, transaction_bytes }`, deserializing the bincode `VersionedTransaction` and exposing the raw bytes. Served from the Gatekeeper endpoint (`wss://beta.helius-rpc.com`). Notifications are **binary** frames (the subscribe ack is a JSON text frame); the little-endian layout is `version:u8 | slot:u64_le | transaction_index:u64_le | status:u8 | bincode(VersionedTransaction)`. The `version` byte is checked first (currently `1`; unknown versions are dropped) and `status` is exposed as the `PreconfStatus` enum (`Failed = 0`, `Success = 1`, `Unknown = 2`). Credit-based pricing (10 credits per notification). Includes `PreconfClient::connect`/`connect_with_api_key`/`shutdown`, `PreconfStream`, `CURRENT_VERSION`, and `examples/websockets/preconf_subscribe.rs`. A pre-confirmation is an early signal, not a guarantee; coverage is not continuous (scales with stake forwarding to Helius — expect gaps).
- **Sender Max bundles**: new `send_bundle_with_sender` submits up to 5 transactions to Sender Max via `sendBundle` (`params: [[base64Tx, ...], { encoding: "base64" }]`). The caller includes only the 0.001 SOL Sender tip in ≥1 transaction; Helius adds any pathway tips — callers must not add a separate pathway-specific tip. Landing is tracked per-transaction by signature (not bundle IDs / `getBundleStatuses`).
- `get_wallet_balance_at` Wallet API method for querying a wallet's balance of a specific token or native SOL at a past timestamp, datetime, or slot
- `BalanceAtQuery` enum (`Time`, `Datetime`, `Slot`) for selecting the historical point to query, plus `BalanceAtResponse`, `BalanceAtRequested`, and `BalanceAtAsOf` response types
- `RpcError` type modeling the JSON-RPC 2.0 `error` object (`code`, `message`, optional `data`), and a `HeliusError::RpcError { code, message }` variant so server-side JSON-RPC failures are surfaced with their code and description (any `data` is appended to the message)

### Changed
- **Breaking**: `RpcResponse<T>.result` changed from `T` to `Option<T>`, `RpcResponse<T>.id` changed from `String` to `Option<String>` (servers return `"id": null` on parse/invalid-request errors), and `RpcResponse<T>` gained an `error: Option<RpcError>` field, to represent JSON-RPC responses that carry an `error` instead of a `result`. Code that reads `response.result`/`response.id` directly must now handle the `Option`. This is the JSON-RPC envelope type; most callers use the high-level RPC methods (which return `T`) and are unaffected.
- **BREAKING (defaults): Sender tip tiers.** The non-SWQOS tier is now branded **Sender Max** (`swqos_only = false`) with a minimum tip of **0.001 SOL** (`MIN_TIP_LAMPORTS_MAX = 1_000_000`), up from the removed 0.0002 SOL tier. SWQOS-only (`swqos_only = true`) is unchanged at 0.000005 SOL (`MIN_TIP_LAMPORTS_SWQOS = 5_000`). `determine_tip_lamports` and `send_smart_transaction_with_sender` now floor non-SWQOS tips at 0.001 SOL.
- **Sender requirements relaxed.** `skip_preflight` is no longer mandated — it is now a caller-controlled passthrough on `SenderSendOptions` (defaults to `true` for backward compatibility, set `false` to run preflight). A priority fee is recommended but no longer documented as a hard requirement; only the tip is mandatory. Doc comments describe routing generically (multiple high-speed pathways + priority auction).
- **BREAKING (API): `SenderSendOptions` is now `#[non_exhaustive]`** and gains a public `skip_preflight: bool` field. Because the struct is `#[non_exhaustive]`, downstream code can no longer construct it with a struct literal; use `SenderSendOptions::default()` / `SenderSendOptions::new()` plus the new `with_region`/`with_swqos_only`/`with_skip_preflight`/`with_poll_timeout_ms`/`with_poll_interval_ms` builder methods. Field reads/writes on an existing value are unaffected.
- **Breaking**: Renamed `ProgramName::Unkown` to `ProgramName::Unknown`, fixing a typo that caused the variant to (de)serialize as `"UNKOWN"`. The API's real `"UNKNOWN"` never matched it and fell through to `Other("UNKNOWN")`; it now maps to/from `"UNKNOWN"` correctly.
- **Breaking**: `GetAssetsByOwner.limit` changed from `Option<i32>` to `Option<u32>`, matching the sibling `GetAssetsBy*` request structs (a page size is never negative).
- **Breaking**: `EnhancedTransaction.fee` and `EnhancedTransaction.slot` changed from `i32` to `u64`. Fees and slots are non-negative and slots already exceed `i32::MAX`'s effective headroom; `u64` also matches `TransactionSignatureEntry.slot`.
- **Breaking**: Upgraded the Solana crate dependencies from the `3.0.x` line to the Agave 4.2 set (`solana-sdk` 4.x; `solana-client`, `solana-transaction-status`, `solana-account-decoder`, `solana-rpc-client-api` 4.2.x). This is the groundwork for Transaction v1 / larger-transaction support and is a breaking change for downstreams that share Solana types — align your own `solana-*` dependencies to the 4.x line. The resolved `solana-message` exposes `VersionedMessage::V1`.
- Replaced the direct `solana-transaction-status` dependency with `solana-transaction-status-client-types`: in 4.x the parent crate's root is gated behind the `agave-unstable-api` feature, while the wire/status types the SDK uses (`EncodedTransaction`, `EncodedTransactionWithStatusMeta`, `UiTransactionStatusMeta`, `TransactionConfirmationStatus`) live in the ungated companion crate.
- Dropped the redundant direct `solana-program` dependency; `Hash` and `Pubkey` are now imported from `solana-sdk`.

### Deprecated
- `MIN_TIP_LAMPORTS_DUAL` is deprecated in favor of `MIN_TIP_LAMPORTS_MAX`. It is now an alias resolving to the Sender Max minimum (0.001 SOL), not the removed 0.0002 SOL value.

### Fixed
- Server-side JSON-RPC errors are now surfaced instead of swallowed. Solana/Helius RPC methods report method-level failures (invalid params, unknown method, transaction preflight failures, etc.) as a JSON-RPC `error` object with an HTTP 200 status. Previously `RpcResponse<T>` had no `error` field and a required `result`, so these responses failed to deserialize and returned a misleading `missing field 'result'` error, discarding the real error code and message. `post_rpc_request` now inspects `error` and returns a `HeliusError::RpcError { code, message }` carrying the server's details. Affects all DAS and RPC V2 methods.
- The paginated DAS request structs (`GetAssetsByOwner`, `GetAssetsByAuthority`, `GetAssetsByCreator`, `GetAssetsByGroup`, `GetAssetSignatures`, `GetTokenAccounts`, `GetNftEditions`) no longer serialize unset optional fields as explicit `null`s in the JSON-RPC `params`; they are now omitted via `skip_serializing_if` uniformly across the family.
- `send_and_confirm_transaction` now honors its timeout. The retry loop used `||` where it needed `&&`, so it kept retrying until *both* the timeout elapsed *and* the blockhash expired instead of stopping as soon as either did. The timeout error message now reflects the actual configured duration rather than a hardcoded "60s".
- `get_withdrawable_amount` no longer reports a stake account as withdrawable one epoch early. A stake deactivated in epoch N is still cooling down during epoch N; the check now uses `>=` so funds are only reported withdrawable once `current_epoch > deactivation_epoch`.
- `create_smart_transaction_with_seeds` now returns `HeliusError::InvalidInput` on a keypair-from-seed failure instead of panicking via `expect`, matching its documented `# Errors` contract.
- `poll_transaction_confirmation` no longer panics if `getSignatureStatuses` returns an empty `value` array (e.g. from a misbehaving provider); a missing status entry is now treated as "not yet confirmed" and retried.
- Hardened per-request URL construction across the Wallet, Webhook, enhanced-transaction, and admin APIs (and `post_rpc_request`): `Url::parse` failures now propagate as `HeliusError::UrlParseError` instead of panicking via `expect`. In practice the `url` parser tolerates the interpolated values, so this is defensive; behavior is unchanged for all valid inputs.
- The auto-paginating `get_all_program_accounts` and `get_all_token_accounts_by_owner` helpers can no longer loop forever against a misbehaving server. They now stop when the pagination cursor stops advancing (the same key is returned) and are bounded by a page cap, logging a warning rather than truncating silently. The cap defaults to `DEFAULT_MAX_AUTO_PAGINATION_PAGES` (10,000) and is overridable per call via the new `max_pages` field on `GetProgramAccountsV2Config` / `GetTokenAccountsByOwnerV2Config` (client-side only; not sent to the server).
- `RequestHandler::handle_response` no longer silently swallows a response body-read failure. A mid-body network error previously became an empty string, which deserialized to `T::default()` on a 2xx (a bogus "success"); it now propagates as `HeliusError::Network`.
- `get_stake_accounts` now uses `get_program_ui_accounts_with_config` and decodes the returned UI accounts back into `Account`. The previous `get_program_accounts_with_config` method was removed in solana-client 4.x.

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

[Unreleased]: https://github.com/helius-labs/helius-rust-sdk/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/helius-labs/helius-rust-sdk/compare/v1.1.0...v2.0.0
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
