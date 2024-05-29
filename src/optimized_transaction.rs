use crate::error::{HeliusError, Result};
use crate::types::{
    GetPriorityFeeEstimateOptions, GetPriorityFeeEstimateRequest, GetPriorityFeeEstimateResponse,
    SmartTransactionConfig,
};
use crate::Helius;

use bincode::{serialize, ErrorKind};
use reqwest::StatusCode;
use solana_client::rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig};
use solana_client::rpc_response::{Response, RpcSimulateTransactionResult};
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    bs58::encode,
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    instruction::Instruction,
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

impl Helius {
    /// Simulates a transaction to get the total compute units consumed
    ///
    /// # Arguments
    /// * `instructions` - The transaction instructions
    /// * `payer` - The public key of the payer
    /// * `lookup_tables` - The address lookup tables
    /// * `from_keypair` - The keypair signing the transaction (needed to simulate the transaction)
    ///
    /// # Returns
    /// The compute units consumed, or None if unsuccessful
    pub async fn get_compute_units(
        &self,
        instructions: Vec<Instruction>,
        payer: Pubkey,
        lookup_tables: Vec<AddressLookupTableAccount>,
        from_keypair: &Keypair,
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

        // Create a signed VersionedTransaction
        let transaction: VersionedTransaction = VersionedTransaction::try_new(versioned_message, &[from_keypair])
            .map_err(|e| HeliusError::InvalidInput(format!("Signing error: {:?}", e)))?;

        // Simulate the transaction
        let config: RpcSimulateTransactionConfig = RpcSimulateTransactionConfig {
            sig_verify: true,
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

        let commitment_config: CommitmentConfig = CommitmentConfig::confirmed();

        loop {
            if start.elapsed() >= timeout {
                return Err(HeliusError::Timeout {
                    code: StatusCode::REQUEST_TIMEOUT,
                    text: format!("Transaction {}'s confirmation timed out", txt_sig),
                });
            }

            match self
                .connection()
                .get_signature_status_with_commitment(&txt_sig, commitment_config)
            {
                Ok(Some(Ok(()))) => return Ok(txt_sig),
                Ok(Some(Err(err))) => return Err(HeliusError::TransactionError(err)),
                Ok(None) => {
                    sleep(interval).await;
                }
                Err(err) => return Err(HeliusError::ClientError(err)),
            }
        }
    }

    /// Builds and sends an optimized transaction, and handles its confirmation status
    ///
    /// # Arguments
    /// * `config` - The configuration for the smart transaction, which includes the transaction's instructions, and the user's keypair. If provided, it also
    /// includes whether preflight checks should be skipped, how many times to retry the transaction, and any address lookup tables to be included in the transaction
    ///
    /// # Returns
    /// The transaction signature, if successful
    pub async fn send_smart_transaction(&self, config: SmartTransactionConfig<'_>) -> Result<Signature> {
        let pubkey: Pubkey = config.from_keypair.pubkey();
        let recent_blockhash: Hash = self.connection().get_latest_blockhash()?;
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

        // Get the optimal compute units
        let units: Option<u64> = self
            .get_compute_units(
                config.instructions.clone(),
                pubkey,
                config.lookup_tables.clone().unwrap_or_default(),
                &config.from_keypair,
            )
            .await?;

        if units.is_none() {
            return Err(HeliusError::InvalidInput(
                "Error fetching compute units for the instructions provided".to_string(),
            ));
        }

        let compute_units: u64 = units.unwrap();
        let customers_cu: u32 = if compute_units < 1000 {
            1000
        } else {
            (compute_units as f64 * 1.5).ceil() as u32
        };

        // Add the compute unit limit instruction with a margin
        let compute_units_ix: Instruction = ComputeBudgetInstruction::set_compute_unit_limit(customers_cu);
        final_instructions.push(compute_units_ix);

        // Determine if we need to use a versioned transaction
        let is_versioned: bool = false; //config.lookup_tables.is_some();
        let mut legacy_transaction: Option<Transaction> = None;
        let mut versioned_transaction: Option<VersionedTransaction> = None;

        // Build the initial transaction based on whether lookup tables are present
        if is_versioned {
            let lookup_tables: &[AddressLookupTableAccount] = config.lookup_tables.as_deref().unwrap_or_default();
            let v0_message: v0::Message =
                v0::Message::try_compile(&pubkey, &config.instructions, lookup_tables, recent_blockhash)?;
            let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message);

            // Sign the versioned transaction
            let signers: Vec<&dyn Signer> = vec![config.from_keypair];
            let signatures: Vec<Signature> = signers
                .iter()
                .map(|signer| signer.try_sign_message(versioned_message.serialize().as_slice()))
                .collect::<std::result::Result<Vec<_>, _>>()?;

            versioned_transaction = Some(VersionedTransaction {
                signatures,
                message: versioned_message,
            });
        } else {
            // If no lookup tables are present, we build a regular transaction
            let mut tx: Transaction = Transaction::new_with_payer(&config.instructions, Some(&pubkey));
            tx.try_sign(&[config.from_keypair], recent_blockhash)?;
            legacy_transaction = Some(tx);
        }

        // Serialize the transaction
        let serialized_tx: Vec<u8> = if let Some(tx) = &legacy_transaction {
            serialize(&tx).map_err(|e: Box<ErrorKind>| HeliusError::InvalidInput(e.to_string()))?
        } else if let Some(tx) = &versioned_transaction {
            serialize(&tx).map_err(|e: Box<ErrorKind>| HeliusError::InvalidInput(e.to_string()))?
        } else {
            return Err(HeliusError::InvalidInput("No transaction available".to_string()));
        };

        // Encode the transaction
        let transaction_base58: String = encode(&serialized_tx).into_string();

        // Get the priority fee estimate based on the serialized transaction
        let priority_fee_request: GetPriorityFeeEstimateRequest = GetPriorityFeeEstimateRequest {
            transaction: Some(transaction_base58),
            account_keys: None,
            options: Some(GetPriorityFeeEstimateOptions {
                recommended: Some(true),
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

        let lamports_to_micro_lamports: u64 = 10_u64.pow(6);
        let minimum_total_pfee_lamports: u64 = 10_000;
        let microlamports_per_cu: u64 = std::cmp::max(
            priority_fee_recommendation,
            ((minimum_total_pfee_lamports as f64 / customers_cu as f64) * lamports_to_micro_lamports as f64).round()
                as u64,
        );

        // Add the compute unit price instruction with the estimated fee
        let compute_budget_ix: Instruction = ComputeBudgetInstruction::set_compute_unit_price(microlamports_per_cu);
        final_instructions.push(compute_budget_ix);

        // Add the original instructions back
        final_instructions.extend(config.instructions.clone());

        // Rebuild the transaction with the final instructions
        if is_versioned {
            let lookup_tables: &[AddressLookupTableAccount] = config.lookup_tables.as_deref().unwrap();
            let v0_message: v0::Message =
                v0::Message::try_compile(&pubkey, &final_instructions, lookup_tables, recent_blockhash)?;
            let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message);
            let signers: Vec<&dyn Signer> = vec![config.from_keypair];
            let signatures: Vec<Signature> = signers
                .iter()
                .map(|signer| signer.try_sign_message(versioned_message.serialize().as_slice()))
                .collect::<std::result::Result<Vec<_>, _>>()?;

            versioned_transaction = Some(VersionedTransaction {
                signatures,
                message: versioned_message,
            });
        } else {
            let mut tx: Transaction = Transaction::new_with_payer(&final_instructions, Some(&pubkey));
            tx.try_sign(&[config.from_keypair], recent_blockhash)?;
            legacy_transaction = Some(tx);
        }

        // Common logic for sending transactions
        let send_transaction_config: RpcSendTransactionConfig = RpcSendTransactionConfig {
            skip_preflight: config.send_options.skip_preflight,
            preflight_commitment: config.send_options.preflight_commitment,
            encoding: config.send_options.encoding,
            max_retries: config.send_options.max_retries,
            min_context_slot: config.send_options.min_context_slot,
        };

        let send_result = |transaction: &Transaction| {
            self.connection()
                .send_transaction_with_config(transaction, send_transaction_config)
        };
        let send_versioned_result = |transaction: &VersionedTransaction| {
            self.connection()
                .send_transaction_with_config(transaction, send_transaction_config)
        };

        // Retry logic with a timeout of 60 seconds
        let timeout: Duration = Duration::from_secs(60);
        let start_time: Instant = Instant::now();

        while Instant::now().duration_since(start_time) < timeout {
            let result = if is_versioned {
                send_versioned_result(versioned_transaction.as_ref().unwrap())
            } else {
                send_result(legacy_transaction.as_ref().unwrap())
            };

            match result {
                Ok(signature) => return self.poll_transaction_confirmation(signature).await,
                Err(_) => continue,
            }
        }

        Err(HeliusError::Timeout {
            code: StatusCode::REQUEST_TIMEOUT,
            text: "Transaction failed to confirm in 60s".to_string(),
        })
    }
}
