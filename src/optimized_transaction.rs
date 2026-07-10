use crate::error::{HeliusError, Result};
use crate::request_handler::SDK_USER_AGENT;
use crate::types::{
    CreateSmartTransactionConfig, CreateSmartTransactionSeedConfig, GetPriorityFeeEstimateOptions,
    GetPriorityFeeEstimateRequest, GetPriorityFeeEstimateResponse, PriorityLevel, SenderSendOptions, SmartTransaction,
    SmartTransactionConfig, Timeout,
};
use crate::Helius;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use bincode::{serialize, ErrorKind};
use phf::phf_map;
use rand::Rng;
use reqwest::StatusCode;
use serde_json::json;
use solana_client::{
    rpc_client::SerializableTransaction,
    rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig},
    rpc_response::{Response, RpcSimulateTransactionResult},
};
use solana_commitment_config::CommitmentConfig;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::signature::keypair_from_seed;
use solana_sdk::{
    bs58::encode,
    hash::Hash,
    instruction::Instruction,
    message::AddressLookupTableAccount,
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::{Signature, Signer},
    signer::keypair::Keypair,
    transaction::{Transaction, VersionedTransaction},
};
use solana_system_interface::instruction as system_instruction;
use solana_transaction_status::TransactionConfirmationStatus;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Default compute unit buffer multiplier for transaction simulation.
///
/// When simulating transactions to estimate compute units, multiply the
/// simulated value by 1.25 (125%) to account for:
/// - Simulation environment differences from actual execution
/// - Edge cases and worst-case execution paths
/// - Safety margin to prevent out-of-compute-units failures
///
/// This 25% buffer balances between preventing transaction failures (too little buffer)
/// and minimizing wasted compute unit fees (too much buffer).
const CU_BUFFER_MULTIPLIER_DEFAULT: f32 = 1.25;

/// Minimum tip in lamports for **Sender Max** (`swqos_only = false`).
///
/// Sender Max routes a transaction across multiple high-speed pathways and
/// enters it into a priority auction; tip more to land first. The same tier
/// handles both single transactions and bundles — a bundle simply carries this
/// minimum tip in at least one of its transactions.
/// Minimum tip: 0.001 SOL (1,000,000 lamports).
pub const MIN_TIP_LAMPORTS_MAX: u64 = 1_000_000; // 0.001 SOL

/// Deprecated alias for [`MIN_TIP_LAMPORTS_MAX`].
///
/// The non-SWQOS tier is now branded **Sender Max**. The previous 0.0002 SOL
/// minimum has been removed; this alias now resolves to the Sender Max minimum
/// (0.001 SOL) for backward compatibility.
#[deprecated(
    since = "1.2.0",
    note = "renamed to MIN_TIP_LAMPORTS_MAX (Sender Max); value is now 0.001 SOL"
)]
pub const MIN_TIP_LAMPORTS_DUAL: u64 = MIN_TIP_LAMPORTS_MAX;

/// Minimum tip in lamports for SWQOS-only mode (`swqos_only = true`).
///
/// SWQOS (Stake Weighted Quality of Service) mode prioritizes transactions
/// based on the sender's stake weight and tip amount.
/// Minimum tip: 0.000005 SOL (5,000 lamports).
pub const MIN_TIP_LAMPORTS_SWQOS: u64 = 5_000; // 0.000005 SOL

fn collect_unique_signers(signers: &[Arc<dyn Signer>], fee_payer: Option<&Arc<dyn Signer>>) -> Vec<Arc<dyn Signer>> {
    let mut all_signers: Vec<Arc<dyn Signer>> = Vec::with_capacity(signers.len() + usize::from(fee_payer.is_some()));
    let mut seen: HashSet<Pubkey> = HashSet::with_capacity(all_signers.capacity());

    if let Some(fee_payer) = fee_payer {
        if seen.insert(fee_payer.pubkey()) {
            all_signers.push(fee_payer.clone());
        }
    }

    for signer in signers {
        if seen.insert(signer.pubkey()) {
            all_signers.push(signer.clone());
        }
    }

    all_signers
}

fn collect_unique_keypair_refs<'a>(signers: &'a [Keypair], fee_payer: &'a Keypair) -> Vec<&'a Keypair> {
    let mut all_signers: Vec<&Keypair> = Vec::with_capacity(signers.len() + 1);
    let mut seen: HashSet<Pubkey> = HashSet::with_capacity(all_signers.capacity());

    if seen.insert(fee_payer.pubkey()) {
        all_signers.push(fee_payer);
    }

    for signer in signers {
        if seen.insert(signer.pubkey()) {
            all_signers.push(signer);
        }
    }

    all_signers
}

fn is_retryable_confirmation_error(err: &HeliusError) -> bool {
    matches!(err, HeliusError::Timeout { .. })
}

/// URL to fetch current Jito bundle tip floor prices.
///
/// This endpoint returns the minimum tip amounts required for different
/// priority levels on Jito's block engine.
const TIP_FLOOR_URL: &str = "https://bundles.jito.wtf/api/v1/bundles/tip_floor";

/// Helius Sender tip account addresses for mainnet-beta.
///
/// # What is Helius Sender?
///
/// Helius Sender is an ultra-low latency transaction submission service that
/// optimizes transaction landing by routing across multiple high-speed pathways
/// and entering a priority auction — tip more to land first. It offers two tiers:
/// - **Sender Max** (`swqos_only = false`): the multi-path tier. Routes across
///   multiple high-speed pathways and enters a priority auction. Handles both
///   single transactions and bundles over the same paths.
/// - **SWQOS-only** (`swqos_only = true`): Stake Weighted Quality of Service only,
///   with a lower minimum tip.
///
/// # Why multiple tip accounts?
///
/// Sender uses a pool of 10 tip accounts to:
/// - **Load Balancing**: Distribute tips across accounts for better throughput
/// - **Parallel Processing**: Enable concurrent transactions without account contention
///
/// # Requirements
///
/// Every transaction through Sender must include:
/// - **Tip**: Minimum 0.001 SOL for Sender Max (or 0.000005 SOL for SWQOS-only mode). This is the only hard requirement.
///
/// Recommended (but not required):
/// - **Priority Fee**: Via `ComputeBudgetProgram::set_compute_unit_price`. Recommended to improve landing, but not strictly required for Sender Max.
/// - **Skip Preflight**: `skip_preflight` is a caller-controlled passthrough (defaults to `true`); it is no longer required to be `true`.
///
/// Learn more: <https://www.helius.dev/docs/sending-transactions/sender>
const SENDER_TIP_ACCOUNTS: [&str; 10] = [
    "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE",
    "D2L6yPZ2FmmmTKPgzaMKdhu6EWZcTpLy1Vhx8uvZe7NZ",
    "9bnz4RShgq1hAnLnZbP8kbgBg1kEmcJBYQq3gQbmnSta",
    "5VY91ws6B2hMmBFRsXkoAAdsPHBJwRfBht4DXox3xkwn",
    "2nyhqdwKcJZR2vcqCyrYsaPVdAnFoJjiksCXJ7hfEYgD",
    "2q5pghRs6arqVjRvT5gfgWfWcHWmw1ZuCzphgd5KfWGJ",
    "wyvPkWjVZz1M8fHQnMMCDTQDbkManefNNhweYk5WkcF",
    "3KCKozbAaF75qEU33jtzozcJ29yJuaLJTy2jFdzUY8bT",
    "4vieeGHPYPG2MmyPRcYjdiDmmhN3ww7hsFNap8pVN3Ey",
    "4TQLFNWK8AovT1gFvda5jfw2oJeRMKEmw7aH6MGBJ3or",
];

/// Helius Sender regional endpoints for ultra-low latency transaction submission.
///
/// # Endpoint Selection Strategy
///
/// **For Frontend/Browser Applications:**
/// - Use `https://sender.helius-rpc.com/fast` (resolves CORS issues)
/// - Automatically routes to nearest location
///
/// **For Backend/Server Applications:**
/// - Choose regional HTTP endpoint closest to your infrastructure
/// - Minimizes network latency for server-to-server communication
///
/// # Regional Endpoints
///
/// - **US_SLC**: Salt Lake City, Utah (closest to core Solana validators)
/// - **US_EAST**: Newark, New Jersey (East Coast US)
/// - **EU_WEST**: London, UK (Western Europe)
/// - **EU_CENTRAL**: Frankfurt, Germany (Central Europe)
/// - **EU_NORTH**: Amsterdam, Netherlands (Northern Europe)
/// - **AP_SINGAPORE**: Singapore (Southeast Asia)
/// - **AP_TOKYO**: Tokyo, Japan (East Asia)
///
/// # Performance Tips
///
/// - Co-locate your infrastructure in FRA or EWR for optimal Helius routing
/// - Use connection warming via `/ping` endpoint during idle periods
/// - Avoid regions far from validator network (e.g., LATAM, South Africa)
///
/// Learn more: <https://www.helius.dev/docs/sending-transactions/sender>
pub static SENDER_ENDPOINTS: phf::Map<&'static str, &'static str> = phf_map! {
    "Default"      => "http://sender.helius-rpc.com",
    "US_SLC"       => "http://slc-sender.helius-rpc.com",
    "US_EAST"      => "http://ewr-sender.helius-rpc.com",
    "EU_WEST"      => "http://lon-sender.helius-rpc.com",
    "EU_CENTRAL"   => "http://fra-sender.helius-rpc.com",
    "EU_NORTH"     => "http://ams-sender.helius-rpc.com",
    "AP_SINGAPORE" => "http://sg-sender.helius-rpc.com",
    "AP_TOKYO"     => "http://tyo-sender.helius-rpc.com",
};

pub static SENDER_REGION_ALIASES: phf::Map<&'static str, &'static str> = phf_map! {
    "US-EAST"      => "US_EAST",
    "US-SLC"       => "US_SLC",
    "EU-WEST"      => "EU_WEST",
    "EU-CENTRAL"   => "EU_CENTRAL",
    "EU-NORTH"     => "EU_NORTH",
    "AP-SINGAPORE" => "AP_SINGAPORE",
    "AP-TOKYO"     => "AP_TOKYO",
};

const SENDER_DEFAULT_BASE: &str = "http://slc-sender.helius-rpc.com";

#[inline]
fn normalize_region(region: &str) -> &str {
    SENDER_REGION_ALIASES.get(region).copied().unwrap_or(region)
}

#[inline]
fn sender_base_url(region: &str) -> &'static str {
    let key: &str = normalize_region(region);
    SENDER_ENDPOINTS.get(key).copied().unwrap_or(SENDER_DEFAULT_BASE)
}

/// `/fast` endpoint used for sending transactions
#[inline]
pub fn sender_fast_url(region: &str) -> String {
    format!("{}/fast", sender_base_url(region))
}

/// `/ping` endpoint used for connection warming
#[inline]
pub fn sender_ping_url(region: &str) -> String {
    format!("{}/ping", sender_base_url(region))
}

/// POST base64 wire-transaction to Sender via `/fast`.
async fn post_to_sender(tx64: &str, opts: &SenderSendOptions) -> Result<Signature> {
    let mut endpoint: String = sender_fast_url(&opts.region);
    if opts.swqos_only {
        endpoint.push_str("?swqos_only=true");
    }

    let body = json!({
        "jsonrpc": "2.0",
        "id": format!("helius-rust-{}", std::time::SystemTime::now()
             .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
        "method": "sendTransaction",
        "params": [
            tx64,
            { "encoding": "base64", "skipPreflight": opts.skip_preflight, "maxRetries": 0 }
        ]
    });

    let res = reqwest::Client::new()
        .post(&endpoint)
        .header("User-Agent", SDK_USER_AGENT)
        .json(&body)
        .send()
        .await
        .map_err(|e| HeliusError::InvalidInput(format!("Sender request error: {e}")))?;

    let status = res.status();
    if !status.is_success() {
        // `text()` consumes `res` in this branch, and we return immediately.
        let text = res.text().await.unwrap_or_default();
        Err(HeliusError::InvalidInput(format!(
            "Sender HTTP {}: {}",
            status,
            text.chars().take(200).collect::<String>()
        )))
    } else {
        // Success path: `json()` consumes `res` *here*, not above.
        let val: serde_json::Value = res
            .json()
            .await
            .map_err(|e| HeliusError::InvalidInput(format!("Sender JSON parse error: {e}")))?;

        if let Some(s) = val.as_str() {
            return Signature::from_str(s)
                .map_err(|e| HeliusError::InvalidInput(format!("Invalid signature from Sender: {e}")));
        }
        if let Some(err) = val.get("error") {
            return Err(HeliusError::InvalidInput(format!("Sender error: {err}")));
        }
        if let Some(result) = val.get("result").and_then(|r| r.as_str()) {
            return Signature::from_str(result)
                .map_err(|e| HeliusError::InvalidInput(format!("Invalid signature from Sender: {e}")));
        }

        Err(HeliusError::InvalidInput(format!(
            "Unexpected Sender response: {}",
            val.to_string().chars().take(200).collect::<String>()
        )))
    }
}

impl Helius {
    // Builds a minimal, unsigned transaction for fee estimation: v0 if LUTs included, legacy otherwise
    fn build_unsigned_preflight_tx(
        payer: &Pubkey,
        instructions: &[Instruction],
        lookup_tables: Option<&[AddressLookupTableAccount]>,
        recent_blockhash: Hash,
    ) -> Result<Vec<u8>> {
        if let Some(luts) = lookup_tables {
            // Versioned v0 with LUT compression
            let v0_message: v0::Message = v0::Message::try_compile(payer, instructions, luts, recent_blockhash)?;
            let versioned_tx: VersionedTransaction = VersionedTransaction {
                signatures: vec![],
                message: VersionedMessage::V0(v0_message),
            };
            serialize(&versioned_tx).map_err(|e: Box<ErrorKind>| crate::error::HeliusError::InvalidInput(e.to_string()))
        } else {
            // Legacy unsigned, but include the recent blockhash to mirror final layout
            let mut tx: Transaction = Transaction::new_with_payer(instructions, Some(payer));
            tx.message.recent_blockhash = recent_blockhash;
            serialize(&tx).map_err(|e: Box<ErrorKind>| crate::error::HeliusError::InvalidInput(e.to_string()))
        }
    }

    /// Simulates a transaction to get the total compute units consumed
    ///
    /// # Arguments
    /// * `instructions` - The transaction instructions
    /// * `payer` - The public key of the payer
    /// * `lookup_tables` - The address lookup tables
    /// * `signers` - The signers for the transaction
    ///
    /// # Returns
    /// The compute units consumed, or None if unsuccessful
    pub async fn get_compute_units(
        &self,
        instructions: Vec<Instruction>,
        payer: Pubkey,
        lookup_tables: Vec<AddressLookupTableAccount>,
        signers: Option<&[Arc<dyn Signer>]>,
    ) -> Result<Option<u64>> {
        // Set the compute budget limit
        let test_instructions: Vec<Instruction> = vec![ComputeBudgetInstruction::set_compute_unit_limit(1_400_000)]
            .into_iter()
            .chain(instructions)
            .collect::<Vec<_>>();

        // Fetch the latest blockhash
        let recent_blockhash: Hash = self.connection().get_latest_blockhash()?;

        // Create a v0::Message
        let v0_message: v0::Message =
            v0::Message::try_compile(&payer, &test_instructions, &lookup_tables, recent_blockhash)?;
        let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message);

        // Create a VersionedTransaction (signed or unsigned)
        let transaction: VersionedTransaction = if let Some(signers) = signers {
            VersionedTransaction::try_new(versioned_message, signers)
                .map_err(|e| HeliusError::InvalidInput(format!("Signing error: {:?}", e)))?
        } else {
            VersionedTransaction {
                signatures: vec![],
                message: versioned_message,
            }
        };

        // Simulate the transaction
        let config: RpcSimulateTransactionConfig = RpcSimulateTransactionConfig {
            sig_verify: signers.is_some(),
            ..Default::default()
        };
        let result: Response<RpcSimulateTransactionResult> = self
            .connection()
            .simulate_transaction_with_config(&transaction, config)?;

        // Return the units consumed or None if not available
        Ok(result.value.units_consumed)
    }

    /// Poll a transaction to check whether it has been confirmed
    ///
    /// * `txt-sig` - The transaction signature to check
    ///
    /// # Returns
    /// The confirmed transaction signature or an error if the confirmation times out
    pub async fn poll_transaction_confirmation(&self, txt_sig: Signature) -> Result<Signature> {
        // 15 second timeout
        let timeout: Duration = Duration::from_secs(15);
        // 5 second retry interval
        let interval: Duration = Duration::from_secs(5);
        let start: Instant = Instant::now();

        loop {
            if start.elapsed() >= timeout {
                return Err(HeliusError::Timeout {
                    code: StatusCode::REQUEST_TIMEOUT,
                    text: format!("Transaction {}'s confirmation timed out", txt_sig),
                });
            }

            let status = self.connection().get_signature_statuses(&[txt_sig])?;

            match status.value[0].clone() {
                Some(status) => {
                    if status.err.is_none()
                        && (status.confirmation_status == Some(TransactionConfirmationStatus::Confirmed)
                            || status.confirmation_status == Some(TransactionConfirmationStatus::Finalized))
                    {
                        return Ok(txt_sig);
                    }
                    if let Some(err) = status.err {
                        return Err(HeliusError::TransactionError(err));
                    }
                }
                None => {
                    sleep(interval).await;
                }
            }
        }
    }

    /// Creates an optimized transaction based on the provided configuration
    ///
    /// # Arguments
    /// * `config` - The configuration for the smart transaction, which includes the transaction's instructions, signers, and lookup tables, depending on
    ///   whether it's a legacy or versioned smart transaction. The transaction's send configuration can also be changed, if provided
    ///
    /// # Returns
    /// An optimized `SmartTransaction` (i.e., `Transaction` or `VersionedTransaction`) and the `last_valid_block_height`
    pub async fn create_smart_transaction(
        &self,
        config: &CreateSmartTransactionConfig,
    ) -> Result<(SmartTransaction, u64)> {
        if config.signers.is_empty() {
            return Err(HeliusError::InvalidInput(
                "The fee payer must sign the transaction".to_string(),
            ));
        }

        let payer_pubkey: Pubkey = config
            .fee_payer
            .as_ref()
            .map_or(config.signers[0].pubkey(), |signer| signer.pubkey());
        let (recent_blockhash, last_valid_block_hash) = self
            .connection()
            .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())?;
        let mut final_instructions: Vec<Instruction> = vec![];

        // Check if any of the instructions provided set the compute unit price and/or limit, and throw an error if `true`
        let existing_compute_budget_instructions: bool = config.instructions.iter().any(|instruction| {
            instruction.program_id == ComputeBudgetInstruction::set_compute_unit_limit(0).program_id
                || instruction.program_id == ComputeBudgetInstruction::set_compute_unit_price(0).program_id
        });

        if existing_compute_budget_instructions {
            return Err(HeliusError::InvalidInput(
                "Cannot provide instructions that set the compute unit price and/or limit".to_string(),
            ));
        }

        // Determine if we need to use a versioned transaction
        let is_versioned: bool = config.lookup_tables.is_some();
        let preflight_bytes: Vec<u8> = Helius::build_unsigned_preflight_tx(
            &payer_pubkey,
            &config.instructions,
            config.lookup_tables.as_deref(),
            recent_blockhash,
        )?;

        // Encode the transaction
        let transaction_base58: String = encode(&preflight_bytes).into_string();

        // Get the priority fee estimate based on the serialized transaction
        let priority_fee_request: GetPriorityFeeEstimateRequest = GetPriorityFeeEstimateRequest {
            transaction: Some(transaction_base58),
            account_keys: None,
            options: Some(GetPriorityFeeEstimateOptions {
                priority_level: Some(PriorityLevel::High),
                ..Default::default()
            }),
        };

        let priority_fee_estimate: GetPriorityFeeEstimateResponse =
            self.rpc().get_priority_fee_estimate(priority_fee_request).await?;

        let priority_fee_recommendation: u64 =
            priority_fee_estimate
                .priority_fee_estimate
                .ok_or(HeliusError::InvalidInput(
                    "Priority fee estimate not available".to_string(),
                ))? as u64;

        let priority_fee: u64 = if let Some(provided_fee) = config.priority_fee_cap {
            // Take the minimum between the estimate and the user-provided cap
            std::cmp::min(priority_fee_recommendation, provided_fee)
        } else {
            priority_fee_recommendation
        };

        // Add the compute unit price instruction with the estimated fee
        let compute_budget_ix: Instruction = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
        let mut updated_instructions: Vec<Instruction> = config.instructions.clone();
        updated_instructions.push(compute_budget_ix.clone());
        final_instructions.push(compute_budget_ix);

        // Get the optimal compute units
        let all_signers: Vec<Arc<dyn Signer>> = collect_unique_signers(&config.signers, config.fee_payer.as_ref());
        let units: Option<u64> = self
            .get_compute_units(
                updated_instructions,
                payer_pubkey,
                config.lookup_tables.clone().unwrap_or_default(),
                Some(&all_signers),
            )
            .await?;

        if units.is_none() {
            return Err(HeliusError::InvalidInput(
                "Error fetching compute units for the instructions provided".to_string(),
            ));
        }

        let compute_units: u64 = units.ok_or(HeliusError::InvalidInput(
            "Error fetching compute units for the instructions provided".to_string(),
        ))?;

        let multiplier: f32 = config.cu_buffer_multiplier.unwrap_or(CU_BUFFER_MULTIPLIER_DEFAULT);

        let customers_cu: u32 = if compute_units < 1000 {
            1000
        } else {
            (compute_units as f64 * multiplier as f64).ceil() as u32
        };

        // Add the compute unit limit instruction with a margin
        let compute_units_ix: Instruction = ComputeBudgetInstruction::set_compute_unit_limit(customers_cu);
        final_instructions.push(compute_units_ix);

        // Add the original instructions back
        final_instructions.extend(config.instructions.clone());

        // Rebuild the transaction with the final instructions
        if is_versioned {
            let lookup_tables: &[AddressLookupTableAccount] = config.lookup_tables.as_deref().unwrap_or(&[]);
            let v0_message: v0::Message =
                v0::Message::try_compile(&payer_pubkey, &final_instructions, lookup_tables, recent_blockhash)?;
            let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message);
            let versioned_transaction: VersionedTransaction =
                VersionedTransaction::try_new(versioned_message, all_signers.as_slice())
                    .map_err(|e| HeliusError::InvalidInput(format!("Signing error: {:?}", e)))?;

            Ok((
                SmartTransaction::Versioned(versioned_transaction),
                last_valid_block_hash,
            ))
        } else {
            let mut tx: Transaction = Transaction::new_with_payer(&final_instructions, Some(&payer_pubkey));
            tx.try_partial_sign(&all_signers, recent_blockhash)?;

            Ok((SmartTransaction::Legacy(tx), last_valid_block_hash))
        }
    }

    /// Builds and sends an optimized transaction, and handles its confirmation status
    ///
    /// # Arguments
    /// * `config` - The configuration for the smart transaction, which includes the transaction's instructions, signers, and lookup tables, depending on
    ///   whether it's a legacy or versioned smart transaction. The transaction's send configuration can also be changed, if provided
    ///
    /// # Returns
    /// The transaction signature, if successful
    pub async fn send_smart_transaction(&self, config: SmartTransactionConfig) -> Result<Signature> {
        let (transaction, last_valid_block_height) = self.create_smart_transaction(&config.create_config).await?;

        match transaction {
            SmartTransaction::Legacy(tx) => {
                self.send_and_confirm_transaction(
                    &tx,
                    config.send_options,
                    last_valid_block_height,
                    Some(config.timeout.into()),
                )
                .await
            }
            SmartTransaction::Versioned(tx) => {
                self.send_and_confirm_transaction(
                    &tx,
                    config.send_options,
                    last_valid_block_height,
                    Some(config.timeout.into()),
                )
                .await
            }
        }
    }

    /// Sends a transaction and handles its confirmation status
    ///
    /// # Arguments
    /// * `transaction` - The transaction to be sent, which implements `SerializableTransaction`
    /// * `send_transaction_config` - Configuration options for sending the transaction
    /// * `last_valid_block_height` - The last block height at which the transaction is valid
    /// * `timeout` - Optional duration for polling transaction confirmation, defaults to 60 seconds
    ///
    /// # Returns
    /// The transaction signature, if successful
    pub async fn send_and_confirm_transaction(
        &self,
        transaction: &impl SerializableTransaction,
        send_transaction_config: RpcSendTransactionConfig,
        last_valid_block_height: u64,
        timeout: Option<Duration>,
    ) -> Result<Signature> {
        // Retry logic with a timeout
        let timeout: Duration = timeout.unwrap_or(Duration::from_secs(60));
        let start_time: Instant = Instant::now();

        // Keep retrying only while both conditions hold: there is time left in the timeout
        // budget AND the blockhash is still valid. Using `&&` stops as soon as either expires;
        // `||` would keep looping until both elapsed, defeating the timeout.
        while Instant::now().duration_since(start_time) < timeout
            && self.connection().get_block_height()? <= last_valid_block_height
        {
            let result = self
                .connection()
                .send_transaction_with_config(transaction, send_transaction_config);

            match result {
                Ok(signature) => {
                    // Poll for transaction confirmation
                    match self.poll_transaction_confirmation(signature).await {
                        Ok(sig) => return Ok(sig),
                        Err(err) if is_retryable_confirmation_error(&err) => continue,
                        Err(err) => return Err(err),
                    }
                }
                // Retry on send failure
                Err(_) => continue,
            }
        }

        Err(HeliusError::Timeout {
            code: StatusCode::REQUEST_TIMEOUT,
            text: format!(
                "Transaction failed to confirm within {}s or the blockhash expired",
                timeout.as_secs()
            ),
        })
    }

    /// Thread safe version of get_compute_units to simulate a transaction to get the total compute units consumed
    ///
    /// # Arguments
    /// * `instructions` - The transaction instructions
    /// * `payer` - The public key of the payer
    /// * `lookup_tables` - The address lookup tables
    /// * `keypairs` - The keypairs for the transaction
    ///
    /// # Returns
    /// The compute units consumed, or None if unsuccessful
    pub async fn get_compute_units_thread_safe(
        &self,
        instructions: Vec<Instruction>,
        payer: Pubkey,
        lookup_tables: Vec<AddressLookupTableAccount>,
        keypairs: Option<&[&Keypair]>,
    ) -> Result<Option<u64>> {
        let test_instructions: Vec<Instruction> = vec![ComputeBudgetInstruction::set_compute_unit_limit(1_400_000)]
            .into_iter()
            .chain(instructions)
            .collect::<Vec<_>>();

        let recent_blockhash: Hash = self.connection().get_latest_blockhash()?;
        let v0_message: v0::Message =
            v0::Message::try_compile(&payer, &test_instructions, &lookup_tables, recent_blockhash)?;
        let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message);

        let transaction: VersionedTransaction = if let Some(keypairs) = keypairs {
            VersionedTransaction::try_new(versioned_message, keypairs)
                .map_err(|e| HeliusError::InvalidInput(format!("Signing error: {:?}", e)))?
        } else {
            VersionedTransaction {
                signatures: vec![],
                message: versioned_message,
            }
        };

        let config: RpcSimulateTransactionConfig = RpcSimulateTransactionConfig {
            sig_verify: keypairs.is_some(),
            ..Default::default()
        };

        let result: Response<RpcSimulateTransactionResult> = self
            .connection()
            .simulate_transaction_with_config(&transaction, config)?;

        Ok(result.value.units_consumed)
    }

    /// Creates a smart transaction using seed bytes for thread-safe transaction creation
    ///
    /// Creates an optimized transaction using seed bytes instead of `Signers`` for thread-safe operations.
    ///
    /// # Arguments
    /// * `create_config` - Transaction configuration containing:
    ///   - `instructions`: Instructions to execute
    ///   - `signer_seeds`: Seed bytes for generating keypairs
    ///   - `fee_payer_seed`: Optional fee payer seed (defaults to first signer)
    ///   - `lookup_tables`: Optional address lookup tables for versioned transactions
    ///   - `priority_fee_cap`: Optional maximum priority fee
    ///
    /// # Returns
    /// A tuple containing:
    /// - `SmartTransaction`: The created transaction (legacy or versioned)
    /// - `u64`: Last valid block height for the transaction
    ///
    /// # Errors
    /// Returns `HeliusError` if:
    /// - No signer seeds provided
    /// - Failed to create keypairs from seeds
    /// - Failed to get compute units
    /// - Failed to estimate priority fees
    /// - Transaction creation fails
    pub async fn create_smart_transaction_with_seeds(
        &self,
        create_config: &CreateSmartTransactionSeedConfig,
    ) -> Result<(SmartTransaction, u64)> {
        if create_config.signer_seeds.is_empty() {
            return Err(HeliusError::InvalidInput(
                "At least one signer seed must be provided".to_string(),
            ));
        }

        let keypairs: Vec<Keypair> = create_config
            .signer_seeds
            .iter()
            .map(|seed| {
                keypair_from_seed(seed)
                    .map_err(|e| HeliusError::InvalidInput(format!("Failed to create keypair from seed: {e}")))
            })
            .collect::<Result<Vec<Keypair>>>()?;

        // Create the fee payer keypair if provided. Otherwise, we default to the first signer
        let fee_payer: Keypair = if let Some(fee_payer_seed) = create_config.fee_payer_seed {
            keypair_from_seed(&fee_payer_seed)
                .map_err(|e| HeliusError::InvalidInput(format!("Failed to create fee payer keypair from seed: {e}")))?
        } else {
            keypairs[0].insecure_clone()
        };

        let (recent_blockhash, last_valid_block_hash) = self
            .connection()
            .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())?;

        let mut final_instructions: Vec<Instruction> = vec![];

        // Get priority fee estimate (unsigned v0 if LUTs, legacy otherwise)
        let preflight_bytes = Self::build_unsigned_preflight_tx(
            &fee_payer.pubkey(),
            &create_config.instructions,
            create_config.lookup_tables.as_deref(),
            recent_blockhash,
        )?;
        let transaction_base58: String = encode(&preflight_bytes).into_string();

        let priority_fee_request: GetPriorityFeeEstimateRequest = GetPriorityFeeEstimateRequest {
            transaction: Some(transaction_base58),
            account_keys: None,
            options: Some(GetPriorityFeeEstimateOptions {
                priority_level: Some(PriorityLevel::High),
                ..Default::default()
            }),
        };

        let priority_fee_estimate: GetPriorityFeeEstimateResponse =
            self.rpc().get_priority_fee_estimate(priority_fee_request).await?;
        let priority_fee_recommendation: u64 =
            priority_fee_estimate
                .priority_fee_estimate
                .ok_or(HeliusError::InvalidInput(
                    "Priority fee estimate not available".to_string(),
                ))? as u64;

        let priority_fee: u64 = if let Some(provided_fee) = create_config.priority_fee_cap {
            std::cmp::min(priority_fee_recommendation, provided_fee)
        } else {
            priority_fee_recommendation
        };

        // Add compute budget instructions
        final_instructions.push(ComputeBudgetInstruction::set_compute_unit_price(priority_fee));

        // Get optimal compute units
        let mut test_instructions: Vec<Instruction> = final_instructions.clone();
        test_instructions.extend(create_config.instructions.clone());

        let all_signers: Vec<&Keypair> = collect_unique_keypair_refs(&keypairs, &fee_payer);

        let units: Option<u64> = self
            .get_compute_units_thread_safe(
                test_instructions,
                fee_payer.pubkey(),
                create_config.lookup_tables.clone().unwrap_or_default(),
                Some(&all_signers),
            )
            .await?;

        let compute_units: u64 = units.ok_or(HeliusError::InvalidInput(
            "Error fetching compute units for the instructions provided".to_string(),
        ))?;

        let multiplier: f32 = create_config
            .cu_buffer_multiplier
            .unwrap_or(CU_BUFFER_MULTIPLIER_DEFAULT);

        let customers_cu: u32 = if compute_units < 1000 {
            1000
        } else {
            (compute_units as f64 * multiplier as f64).ceil() as u32
        };

        final_instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(customers_cu));
        final_instructions.extend(create_config.instructions.clone());

        // Create the final transaction
        let transaction: SmartTransaction = if let Some(lookup_tables) = &create_config.lookup_tables {
            let message: v0::Message = v0::Message::try_compile(
                &fee_payer.pubkey(),
                &final_instructions,
                lookup_tables,
                recent_blockhash,
            )?;

            let versioned_message: VersionedMessage = VersionedMessage::V0(message);
            let tx: VersionedTransaction = VersionedTransaction::try_new(versioned_message, all_signers.as_slice())
                .map_err(|e| HeliusError::InvalidInput(format!("Signing error: {:?}", e)))?;

            SmartTransaction::Versioned(tx)
        } else {
            let mut tx: Transaction = Transaction::new_with_payer(&final_instructions, Some(&fee_payer.pubkey()));
            tx.sign(&all_signers, recent_blockhash);

            SmartTransaction::Legacy(tx)
        };

        Ok((transaction, last_valid_block_hash))
    }

    /// Sends a smart transaction using seed bytes
    ///
    /// This method allows for sending smart transactions in asynchronous contexts
    /// where the Signer trait's lack of Send + Sync would otherwise cause issues.
    /// It creates Keypairs from the provided seed bytes and uses them to sign the transaction.
    ///
    /// # Arguments
    ///
    /// * `create_config` - A `CreateSmartTransactionSeedConfig` containing:
    ///   - `instructions`: The instructions to be executed in the transaction.
    ///   - `signer_seeds`: Seed bytes for generating signer keypairs.
    ///   - `fee_payer_seed`: Optional seed bytes for generating the fee payer keypair.
    ///   - `lookup_tables`: Optional address lookup tables for the transaction.
    /// * `send_options` - Optional `RpcSendTransactionConfig` for sending the transaction.
    /// * `timeout` - Optional `Timeout` wait time for polling transaction confirmation.
    ///
    /// # Returns
    ///
    /// A `Result<Signature>` containing the transaction signature if successful, or an error if not.
    ///
    /// # Errors
    ///
    /// This function will return an error if keypair creation from seeds fails, the transaction sending fails,
    /// or no signer seeds are provided
    ///
    /// # Notes
    ///
    /// If no `fee_payer_seed` is provided, the first signer (i.e., derived from the first seed in `signer_seeds`) will be used as the fee payer
    pub async fn send_smart_transaction_with_seeds(
        &self,
        create_config: CreateSmartTransactionSeedConfig,
        send_options: Option<RpcSendTransactionConfig>,
        timeout: Option<Timeout>,
    ) -> Result<Signature> {
        if create_config.signer_seeds.is_empty() {
            return Err(HeliusError::InvalidInput(
                "At least one signer seed required".to_string(),
            ));
        }

        let (transaction, last_valid_block_hash) = self.create_smart_transaction_with_seeds(&create_config).await?;

        match transaction {
            SmartTransaction::Legacy(tx) => {
                self.send_and_confirm_transaction(
                    &tx,
                    send_options.unwrap_or_default(),
                    last_valid_block_hash,
                    Some(timeout.unwrap_or_default().into()),
                )
                .await
            }
            SmartTransaction::Versioned(tx) => {
                self.send_and_confirm_transaction(
                    &tx,
                    send_options.unwrap_or_default(),
                    last_valid_block_hash,
                    Some(timeout.unwrap_or_default().into()),
                )
                .await
            }
        }
    }

    /// Creates an optimized transaction without requiring any signers
    ///
    /// This version builds the transaction (legacy or versioned) without signing,
    /// and requires that a fee payer is provided
    ///
    /// # Arguments
    /// * `config` - The configuration for the smart transaction. Note that the `fee_payer` field must be provided.
    ///
    /// # Returns
    /// An unsigned `SmartTransaction` (i.e., `Transaction` or `VersionedTransaction`) and the `last_valid_block_height`
    pub async fn create_smart_transaction_without_signers(
        &self,
        config: &CreateSmartTransactionConfig,
    ) -> Result<(SmartTransaction, u64)> {
        // The payer must be provided
        let fee_payer: &Arc<dyn Signer> = config.fee_payer.as_ref().ok_or_else(|| {
            HeliusError::InvalidInput("Fee payer must be provided for unsigned transactions".to_string())
        })?;
        let payer_pubkey: Pubkey = fee_payer.pubkey();

        let (recent_blockhash, last_valid_block_hash) = self
            .connection()
            .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())?;

        let mut final_instructions: Vec<Instruction> = vec![];

        // Ensure that no compute budget ixs are included in the input
        let existing_compute_budget_instructions: bool = config.instructions.iter().any(|instruction| {
            instruction.program_id == ComputeBudgetInstruction::set_compute_unit_limit(0).program_id
                || instruction.program_id == ComputeBudgetInstruction::set_compute_unit_price(0).program_id
        });

        if existing_compute_budget_instructions {
            return Err(HeliusError::InvalidInput(
                "Cannot provide instructions that set the compute unit price and/or limit".to_string(),
            ));
        }

        // Determine if we need to build a versioned tx based on lookup tables
        let is_versioned: bool = config.lookup_tables.is_some();

        // Build the initial unsigned tx
        let preflight_bytes: Vec<u8> = Self::build_unsigned_preflight_tx(
            &payer_pubkey,
            &config.instructions,
            config.lookup_tables.as_deref(),
            recent_blockhash,
        )?;
        let transaction_base58: String = encode(&preflight_bytes).into_string();

        let priority_fee_request = GetPriorityFeeEstimateRequest {
            transaction: Some(transaction_base58),
            account_keys: None,
            options: Some(GetPriorityFeeEstimateOptions {
                recommended: Some(true),
                ..Default::default()
            }),
        };

        let priority_fee_estimate: GetPriorityFeeEstimateResponse =
            self.rpc().get_priority_fee_estimate(priority_fee_request).await?;

        let priority_fee_recommendation: u64 = priority_fee_estimate
            .priority_fee_estimate
            .ok_or_else(|| HeliusError::InvalidInput("Priority fee estimate not available".to_string()))?
            as u64;

        let priority_fee: u64 = if let Some(provided_fee) = config.priority_fee_cap {
            std::cmp::min(priority_fee_recommendation, provided_fee)
        } else {
            priority_fee_recommendation
        };

        // Add the compute unit price ix with the estimated fee at the start
        let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
        final_instructions.push(compute_budget_ix);

        // Get the optimal CUs
        let units: Option<u64> = self
            .get_compute_units(
                config.instructions.clone(),
                payer_pubkey,
                config.lookup_tables.clone().unwrap_or_default(),
                None,
            )
            .await?;

        if units.is_none() {
            return Err(HeliusError::InvalidInput(
                "Error fetching compute units for the provided instructions".to_string(),
            ));
        }

        let compute_units: u64 = units.ok_or(HeliusError::InvalidInput(
            "Error fetching compute units for the instructions provided".to_string(),
        ))?;

        let multiplier: f32 = config.cu_buffer_multiplier.unwrap_or(CU_BUFFER_MULTIPLIER_DEFAULT);

        let customers_cu: u32 = if compute_units < 1000 {
            1000
        } else {
            (compute_units as f64 * multiplier as f64).ceil() as u32
        };

        // Add the compute unit limit ix at the start
        let compute_units_ix = ComputeBudgetInstruction::set_compute_unit_limit(customers_cu);
        final_instructions.push(compute_units_ix);

        // Append the original ixs back
        final_instructions.extend(config.instructions.clone());

        // Rebuild the final unsigned tx with the updated ixs
        if is_versioned {
            let lookup_tables: &[AddressLookupTableAccount] = config.lookup_tables.as_deref().unwrap_or(&[]);
            let v0_message: v0::Message =
                v0::Message::try_compile(&payer_pubkey, &final_instructions, lookup_tables, recent_blockhash)?;
            let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message);

            let versioned_transaction: VersionedTransaction = VersionedTransaction {
                signatures: vec![],
                message: versioned_message,
            };

            Ok((
                SmartTransaction::Versioned(versioned_transaction),
                last_valid_block_hash,
            ))
        } else {
            let mut tx: Transaction = Transaction::new_with_payer(&final_instructions, Some(&payer_pubkey));
            tx.message.recent_blockhash = recent_blockhash;

            Ok((SmartTransaction::Legacy(tx), last_valid_block_hash))
        }
    }

    /// Fetches the 75th percentile landed tip floor from Jito's endpoint (in SOL).
    /// Returns `None` if the fetch fails or the response is malformed.
    pub async fn fetch_tip_floor_75th(&self) -> Result<Option<u64>> {
        let res = reqwest::Client::new()
            .get(TIP_FLOOR_URL)
            .header("User-Agent", SDK_USER_AGENT)
            .send()
            .await
            .map_err(|e| HeliusError::InvalidInput(format!("Tip floor fetch error: {e}")))?;

        if !res.status().is_success() {
            return Ok(None);
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| HeliusError::InvalidInput(format!("Tip floor JSON parse error: {e}")))?;

        let val_sol = json
            .get(0)
            .and_then(|o| o.get("landed_tips_75th_percentile"))
            .and_then(|v| v.as_f64());

        Ok(val_sol.map(|sol| (sol * 1_000_000_000.0) as u64))
    }

    /// Determines the tip amount in lamports using the 75th percentile floor or falling back to the minimum.
    pub async fn determine_tip_lamports(&self, swqos_only: bool) -> Result<u64> {
        let min_lamports: u64 = if swqos_only {
            MIN_TIP_LAMPORTS_SWQOS
        } else {
            MIN_TIP_LAMPORTS_MAX
        };
        let floor_lamports: u64 = self.fetch_tip_floor_75th().await?.unwrap_or(min_lamports);

        Ok(floor_lamports.max(min_lamports))
    }

    /// Creates an optimized smart transaction with an appended tip transfer instruction for Sender
    /// Will rename once Jito functions are removed.
    pub async fn create_smart_transaction_with_tip_for_sender(
        &self,
        mut config: CreateSmartTransactionConfig,
        tip_amount: u64,
    ) -> Result<(SmartTransaction, u64)> {
        if config.signers.is_empty() {
            return Err(HeliusError::InvalidInput(
                "The fee payer must sign the transaction".to_string(),
            ));
        }

        let payer_pubkey: Pubkey = config
            .fee_payer
            .as_ref()
            .map_or(config.signers[0].pubkey(), |signer| signer.pubkey());

        if tip_amount > 0 {
            let mut rng = rand::rng();
            let idx = rng.random_range(0..SENDER_TIP_ACCOUNTS.len());
            let tip_pubkey = Pubkey::from_str(SENDER_TIP_ACCOUNTS[idx])
                .map_err(|e| HeliusError::InvalidInput(format!("Invalid tip account: {e}")))?;

            let tip_ix = system_instruction::transfer(&payer_pubkey, &tip_pubkey, tip_amount);
            config.instructions.push(tip_ix);
        }

        self.create_smart_transaction(&config).await
    }

    /// Warms Sender connection by hitting `/ping`.
    pub async fn warm_sender_connection(&self, region: &str) -> Result<()> {
        let url = sender_ping_url(region);
        let res = reqwest::Client::new()
            .get(&url)
            .header("User-Agent", SDK_USER_AGENT)
            .send()
            .await
            .map_err(|e| HeliusError::InvalidInput(format!("Sender ping error: {e}")))?;
        if !res.status().is_success() {
            return Err(HeliusError::InvalidInput(format!("Sender ping HTTP {}", res.status())));
        }
        Ok(())
    }

    /// Sends a signed tx via Sender `/fast` and polls until confirmed (or until timeout/last valid blockhash expiry).
    /// NOTE: `skipPreflight` follows `opts.skip_preflight` (defaults to `true`); `maxRetries = 0`.
    pub async fn send_and_confirm_via_sender<T>(
        &self,
        transaction: &T,
        last_valid_block_height: u64,
        opts: SenderSendOptions,
    ) -> Result<Signature>
    where
        T: SerializableTransaction + serde::Serialize + ?Sized,
    {
        let wire: Vec<u8> =
            bincode::serialize(transaction).map_err(|e: Box<ErrorKind>| HeliusError::InvalidInput(e.to_string()))?;

        // Base64 encode the wire transaction for Sender
        let tx64: String = B64.encode(&wire);

        // Send to Sender
        let sig: Signature = post_to_sender(&tx64, &opts).await?;

        // Poll until confirmed (or timeout/last valid blockhash expiry)
        let start: Instant = Instant::now();
        let timeout: Duration = Duration::from_millis(opts.poll_timeout_ms);
        let interval: Duration = Duration::from_millis(opts.poll_interval_ms);

        loop {
            if start.elapsed() >= timeout {
                return Err(HeliusError::Timeout {
                    code: StatusCode::REQUEST_TIMEOUT,
                    text: format!("Transaction {}'s confirmation timed out", sig),
                });
            }

            if self.connection().get_block_height()? > last_valid_block_height {
                return Err(HeliusError::Timeout {
                    code: StatusCode::REQUEST_TIMEOUT,
                    text: format!(
                        "Transaction {} expired (last_valid_block_height={})",
                        sig, last_valid_block_height
                    ),
                });
            }

            match self.poll_transaction_confirmation(sig).await {
                Ok(confirmed) => return Ok(confirmed),
                Err(err) if is_retryable_confirmation_error(&err) => sleep(interval).await,
                Err(err) => return Err(err),
            }
        }
    }

    /// Builds an optimized tx and sends via Sender.
    /// If you need a tip transfer, prepend it to `config.create_config.instructions` before calling.
    pub async fn send_smart_transaction_with_sender(
        &self,
        config: SmartTransactionConfig,
        sender_opts: SenderSendOptions,
    ) -> Result<Signature> {
        if sender_opts.region.trim().is_empty() {
            return Err(HeliusError::InvalidInput("Sender region must be specified".to_string()));
        }

        // Determine tip and enforce floor (all in lamports)
        let mut tip_lamports = self.determine_tip_lamports(sender_opts.swqos_only).await?;
        let floor = if sender_opts.swqos_only {
            MIN_TIP_LAMPORTS_SWQOS
        } else {
            MIN_TIP_LAMPORTS_MAX
        };

        if tip_lamports < floor {
            tip_lamports = floor;
        }

        let create_cfg: CreateSmartTransactionConfig = config.create_config;

        let (transaction, last_valid_block_height) = self
            .create_smart_transaction_with_tip_for_sender(create_cfg, tip_lamports)
            .await?;

        match transaction {
            SmartTransaction::Legacy(tx) => {
                self.send_and_confirm_via_sender(&tx, last_valid_block_height, sender_opts)
                    .await
            }
            SmartTransaction::Versioned(tx) => {
                self.send_and_confirm_via_sender(&tx, last_valid_block_height, sender_opts)
                    .await
            }
        }
    }

    /// Submits a **bundle** of up to 5 transactions to Sender Max via `sendBundle`.
    ///
    /// Sender Max handles single transactions and bundles over the same paths and
    /// priority auction. The caller only needs to include the **0.001 SOL Sender
    /// tip** in at least one transaction of the bundle — Helius adds any pathway
    /// tips on your behalf. Do **not** add a separate pathway-specific tip or set
    /// a pathway-region header.
    ///
    /// Landing is tracked via each transaction's **signature**
    /// (`getSignatureStatuses`), not bundle IDs / `getBundleStatuses`.
    ///
    /// # Arguments
    /// * `transactions` - 1..=5 signed transactions to submit as a bundle.
    /// * `opts` - Sender options (region, polling, etc.). `swqos_only` is ignored
    ///   for bundles (Sender Max only).
    ///
    /// # Returns
    /// The signatures of every transaction in the bundle, in submission order,
    /// once all have confirmed (or an error/timeout).
    pub async fn send_bundle_with_sender<T>(
        &self,
        transactions: &[T],
        last_valid_block_height: u64,
        opts: SenderSendOptions,
    ) -> Result<Vec<Signature>>
    where
        T: SerializableTransaction + serde::Serialize,
    {
        if transactions.is_empty() {
            return Err(HeliusError::InvalidInput(
                "Bundle must contain at least one transaction".into(),
            ));
        }
        if transactions.len() > 5 {
            return Err(HeliusError::InvalidInput(format!(
                "Bundle supports at most 5 transactions, got {}",
                transactions.len()
            )));
        }
        if opts.region.trim().is_empty() {
            return Err(HeliusError::InvalidInput("Sender region must be specified".to_string()));
        }

        // Base64-encode each wire transaction.
        let mut encoded: Vec<String> = Vec::with_capacity(transactions.len());
        let mut signatures: Vec<Signature> = Vec::with_capacity(transactions.len());
        for tx in transactions {
            let wire: Vec<u8> =
                bincode::serialize(tx).map_err(|e: Box<ErrorKind>| HeliusError::InvalidInput(e.to_string()))?;
            encoded.push(B64.encode(&wire));
            signatures.push(*tx.get_signature());
        }

        // Bundles always go through Sender Max (no `?swqos_only=true`).
        //
        // Wire format verified against the Sender backend `/fast` handler
        // (`atlas-txn-sender/src/http_server/server.rs`): the `sendBundle` method
        // parses `params` as `[[base64Tx, ...], { "encoding": "base64" }]`.
        let endpoint = sender_fast_url(&opts.region);
        let body = json!({
            "jsonrpc": "2.0",
            "id": format!("helius-rust-bundle-{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis()),
            "method": "sendBundle",
            "params": [ encoded, { "encoding": "base64" } ]
        });

        let res = reqwest::Client::new()
            .post(&endpoint)
            .header("User-Agent", SDK_USER_AGENT)
            .json(&body)
            .send()
            .await
            .map_err(|e| HeliusError::InvalidInput(format!("Sender bundle request error: {e}")))?;

        let status = res.status();
        if !status.is_success() {
            let text = res.text().await.unwrap_or_default();
            return Err(HeliusError::InvalidInput(format!(
                "Sender bundle HTTP {}: {}",
                status,
                text.chars().take(200).collect::<String>()
            )));
        }

        // Parse the response for an explicit error; the result (bundle id, etc.) is
        // intentionally ignored — we track landing by signature, not bundle id.
        let val: serde_json::Value = res
            .json()
            .await
            .map_err(|e| HeliusError::InvalidInput(format!("Sender bundle JSON parse error: {e}")))?;
        if let Some(err) = val.get("error") {
            return Err(HeliusError::InvalidInput(format!("Sender bundle error: {err}")));
        }

        // Track landing via each transaction's signature.
        let start: Instant = Instant::now();
        let timeout: Duration = Duration::from_millis(opts.poll_timeout_ms);
        let interval: Duration = Duration::from_millis(opts.poll_interval_ms);

        for sig in &signatures {
            loop {
                if start.elapsed() >= timeout {
                    return Err(HeliusError::Timeout {
                        code: StatusCode::REQUEST_TIMEOUT,
                        text: format!("Bundle transaction {sig}'s confirmation timed out"),
                    });
                }

                if self.connection().get_block_height()? > last_valid_block_height {
                    return Err(HeliusError::Timeout {
                        code: StatusCode::REQUEST_TIMEOUT,
                        text: format!(
                            "Bundle transaction {sig} expired (last_valid_block_height={last_valid_block_height})"
                        ),
                    });
                }

                match self.poll_transaction_confirmation(*sig).await {
                    Ok(_) => break,
                    Err(err) if is_retryable_confirmation_error(&err) => sleep(interval).await,
                    Err(err) => return Err(err),
                }
            }
        }

        Ok(signatures)
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_unique_keypair_refs, collect_unique_signers, is_retryable_confirmation_error};
    use crate::error::HeliusError;
    use reqwest::StatusCode;
    use solana_sdk::{
        hash::Hash,
        instruction::{AccountMeta, Instruction, InstructionError},
        message::{v0, VersionedMessage},
        pubkey::Pubkey,
        signature::{Keypair, Signature, Signer},
        transaction::{Transaction, TransactionError, VersionedTransaction},
    };
    use std::sync::Arc;

    fn build_versioned_message(
        payer: &Keypair,
        writable_signer: &Keypair,
        readonly_signer: &Keypair,
    ) -> VersionedMessage {
        let instruction = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: vec![
                AccountMeta::new(writable_signer.pubkey(), true),
                AccountMeta::new_readonly(readonly_signer.pubkey(), true),
            ],
            data: vec![],
        };

        VersionedMessage::V0(
            v0::Message::try_compile(&payer.pubkey(), &[instruction], &[], Hash::new_unique()).unwrap(),
        )
    }

    #[test]
    fn collect_unique_signers_includes_fee_payer_once() {
        let fee_payer: Arc<dyn Signer> = Arc::new(Keypair::new());
        let signer: Arc<dyn Signer> = Arc::new(Keypair::new());

        let signers: Vec<Arc<dyn Signer>> = vec![signer.clone(), fee_payer.clone(), signer.clone()];
        let all_signers = collect_unique_signers(&signers, Some(&fee_payer));

        let signer_pubkeys: Vec<Pubkey> = all_signers.iter().map(|signer| signer.pubkey()).collect();
        assert_eq!(signer_pubkeys, vec![fee_payer.pubkey(), signer.pubkey()]);
    }

    #[test]
    fn collect_unique_keypair_refs_includes_fee_payer_once() {
        let fee_payer = Keypair::new();
        let signer = Keypair::new();
        let signers = vec![
            signer.insecure_clone(),
            fee_payer.insecure_clone(),
            signer.insecure_clone(),
        ];

        let all_signers = collect_unique_keypair_refs(&signers, &fee_payer);
        let signer_pubkeys: Vec<Pubkey> = all_signers.iter().map(|signer| signer.pubkey()).collect();

        assert_eq!(signer_pubkeys, vec![fee_payer.pubkey(), signer.pubkey()]);
    }

    #[test]
    fn versioned_try_new_reorders_arc_signers_to_match_message() {
        let fee_payer = Keypair::new();
        let writable_signer = Keypair::new();
        let readonly_signer = Keypair::new();
        let fee_payer_signer: Arc<dyn Signer> = Arc::new(fee_payer.insecure_clone());
        let writable_signer_arc: Arc<dyn Signer> = Arc::new(writable_signer.insecure_clone());
        let readonly_signer_arc: Arc<dyn Signer> = Arc::new(readonly_signer.insecure_clone());

        let message = build_versioned_message(&fee_payer, &writable_signer, &readonly_signer);
        let signers: Vec<Arc<dyn Signer>> = vec![readonly_signer_arc.clone(), writable_signer_arc.clone()];
        let all_signers = collect_unique_signers(&signers, Some(&fee_payer_signer));
        let tx = VersionedTransaction::try_new(message.clone(), all_signers.as_slice()).unwrap();
        let message_bytes = message.serialize();

        assert_eq!(
            tx.signatures,
            vec![
                Signature::from(fee_payer.sign_message(&message_bytes)),
                Signature::from(writable_signer.sign_message(&message_bytes)),
                Signature::from(readonly_signer.sign_message(&message_bytes)),
            ]
        );
    }

    #[test]
    fn manual_fee_payer_appended_signature_order_fails_verification() {
        let fee_payer = Keypair::new();
        let writable_signer = Keypair::new();
        let readonly_signer = Keypair::new();
        let message = build_versioned_message(&fee_payer, &writable_signer, &readonly_signer);
        let message_bytes = message.serialize();

        // This mirrors the pre-fix separate fee payer path: caller signers first, fee payer appended last.
        let manual_signatures = [
            readonly_signer.sign_message(&message_bytes),
            writable_signer.sign_message(&message_bytes),
            fee_payer.sign_message(&message_bytes),
        ];

        let verification_results: Vec<bool> = manual_signatures
            .iter()
            .zip(message.static_account_keys().iter())
            .map(|(signature, pubkey)| signature.verify(pubkey.as_ref(), &message_bytes))
            .collect();

        assert_eq!(verification_results, vec![false, true, false]);
    }

    #[test]
    fn manual_non_payer_caller_order_can_fail_verification() {
        let fee_payer = Keypair::new();
        let writable_signer = Keypair::new();
        let readonly_signer = Keypair::new();
        let message = build_versioned_message(&fee_payer, &writable_signer, &readonly_signer);
        let message_bytes = message.serialize();

        // This mirrors the pre-fix seed path: fee payer first, remaining signers left in caller order.
        let manual_signatures = [
            fee_payer.sign_message(&message_bytes),
            readonly_signer.sign_message(&message_bytes),
            writable_signer.sign_message(&message_bytes),
        ];

        let verification_results: Vec<bool> = manual_signatures
            .iter()
            .zip(message.static_account_keys().iter())
            .map(|(signature, pubkey)| signature.verify(pubkey.as_ref(), &message_bytes))
            .collect();

        assert_eq!(verification_results, vec![true, false, false]);
    }

    #[test]
    fn versioned_try_new_reorders_keypair_signers_to_match_message() {
        let fee_payer = Keypair::new();
        let writable_signer = Keypair::new();
        let readonly_signer = Keypair::new();

        let message = build_versioned_message(&fee_payer, &writable_signer, &readonly_signer);
        let signers = vec![readonly_signer.insecure_clone(), writable_signer.insecure_clone()];
        let all_signers = collect_unique_keypair_refs(&signers, &fee_payer);
        let tx = VersionedTransaction::try_new(message.clone(), all_signers.as_slice()).unwrap();
        let message_bytes = message.serialize();

        assert_eq!(
            tx.signatures,
            vec![
                Signature::from(fee_payer.sign_message(&message_bytes)),
                Signature::from(writable_signer.sign_message(&message_bytes)),
                Signature::from(readonly_signer.sign_message(&message_bytes)),
            ]
        );
    }

    #[test]
    fn legacy_try_partial_sign_reorders_keypairs_to_match_message() {
        let fee_payer = Keypair::new();
        let writable_signer = Keypair::new();
        let readonly_signer = Keypair::new();
        let recent_blockhash = Hash::new_unique();
        let instruction = Instruction {
            program_id: Pubkey::new_unique(),
            accounts: vec![
                AccountMeta::new(writable_signer.pubkey(), true),
                AccountMeta::new_readonly(readonly_signer.pubkey(), true),
            ],
            data: vec![],
        };
        let mut tx = Transaction::new_with_payer(&[instruction], Some(&fee_payer.pubkey()));
        let signers = vec![readonly_signer.insecure_clone(), writable_signer.insecure_clone()];
        let all_signers = collect_unique_keypair_refs(&signers, &fee_payer);

        tx.try_partial_sign(&all_signers, recent_blockhash).unwrap();

        let message_bytes = tx.message_data();
        assert_eq!(
            tx.signatures,
            vec![
                fee_payer.sign_message(&message_bytes),
                writable_signer.sign_message(&message_bytes),
                readonly_signer.sign_message(&message_bytes),
            ]
        );
    }

    #[test]
    fn confirmation_retries_only_on_timeout() {
        let timeout = HeliusError::Timeout {
            code: StatusCode::REQUEST_TIMEOUT,
            text: "pending".to_string(),
        };
        let tx_error =
            HeliusError::TransactionError(TransactionError::InstructionError(0, InstructionError::Custom(1)));
        let invalid_input = HeliusError::InvalidInput("bad config".to_string());

        assert!(is_retryable_confirmation_error(&timeout));
        assert!(!is_retryable_confirmation_error(&tx_error));
        assert!(!is_retryable_confirmation_error(&invalid_input));
    }
}
