use crate::error::{HeliusError, Result};
use crate::types::{
    GetPriorityFeeEstimateOptions, GetPriorityFeeEstimateRequest, GetPriorityFeeEstimateResponse,
    SmartTransactionConfig,
};
use crate::Helius;

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use bincode::{serialize, ErrorKind};
use reqwest::StatusCode;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_client::rpc_response::{Response, RpcSimulateTransactionResult};
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    instruction::Instruction,
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::{Signature, Signer},
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
    ///
    /// # Returns
    /// The compute units consumed, or None if unsuccessful
    pub async fn get_compute_units(
        &self,
        instructions: Vec<Instruction>,
        payer: Pubkey,
        lookup_tables: Vec<AddressLookupTableAccount>,
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

        // Create an unsigned VersionedTransaction
        let transaction: VersionedTransaction = VersionedTransaction {
            signatures: vec![],
            message: versioned_message,
        };

        // Simulate the transaction
        let result: Response<RpcSimulateTransactionResult> = self.connection().simulate_transaction(&transaction)?;

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
    /// * `config` - The configuration for the smart transaction, which includes the transaction's instructions, the user's keypair, whether preflight checks
    /// should be skipped, and how many times to retry the transaction, if provided
    ///
    /// # Returns
    /// The transaction signature, if successful
    pub async fn send_smart_transaction(&self, config: SmartTransactionConfig<'_>) -> Result<Signature> {
        let pubkey: Pubkey = config.from_keypair.pubkey();
        let mut recent_blockhash: Hash = self.connection().get_latest_blockhash()?;

        // Build the initial transaction and estimate the priority fee
        let mut transaction: Transaction = Transaction::new_with_payer(&config.instructions, Some(&pubkey));
        transaction.try_sign(&[config.from_keypair], recent_blockhash)?;

        // Serialize the transaction
        let serialized_transaction: Vec<u8> =
            serialize(&transaction).map_err(|e: Box<ErrorKind>| HeliusError::InvalidInput(e.to_string()))?;

        // Convert the serialized transaction to a Base64 string
        let transaction_base64: String = STANDARD.encode(&serialized_transaction);

        // Get the priority fee estimate based on the serialized transaction
        let priority_fee_request: GetPriorityFeeEstimateRequest = GetPriorityFeeEstimateRequest {
            transaction: Some(transaction_base64),
            account_keys: None,
            options: Some(GetPriorityFeeEstimateOptions {
                recommended: Some(true),
                ..Default::default()
            }),
        };

        let priority_fee_estimate: GetPriorityFeeEstimateResponse =
            self.rpc().get_priority_fee_estimate(priority_fee_request).await?;

        let priority_fee_f64 = priority_fee_estimate
            .priority_fee_estimate
            .ok_or(HeliusError::InvalidInput(
                "Priority fee estimate not available".to_string(),
            ))?;

        // Directly cast as u64
        let priority_fee: u64 = priority_fee_f64 as u64;

        // Add the compute unit price instruction with the estimated fee
        let compute_budget_ix: Instruction = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
        let mut final_instructions: Vec<Instruction> = vec![compute_budget_ix];
        final_instructions.extend(config.instructions.clone());

        // Get the optimal compute units
        if let Some(units) = self
            .get_compute_units(final_instructions.clone(), pubkey, vec![])
            .await?
        {
            // Add some margin to the compute units to ensure the transaction does not fail
            let compute_units_ix: Instruction =
                ComputeBudgetInstruction::set_compute_unit_limit((units as f64 * 1.1).ceil() as u32);
            final_instructions.insert(0, compute_units_ix);
        }

        // Build the optimized transaction
        let mut optimized_transaction: Transaction = Transaction::new_with_payer(&final_instructions, Some(&pubkey));
        optimized_transaction.try_sign(&[config.from_keypair], recent_blockhash)?;

        // Re-fetch the blockhash every 4 retries, or roughly once every minute
        let blockhash_validity_threshold: usize = 4;

        let mut retry_count: usize = 0;
        let txt_sig: Signature;

        let skip_preflight_checks: bool = config.skip_preflight_checks.unwrap_or(true);
        let send_transaction_config: RpcSendTransactionConfig = RpcSendTransactionConfig {
            skip_preflight: skip_preflight_checks,
            ..Default::default()
        };
        let max_retries: usize = config.max_retries.unwrap_or(6);

        // Send the transaction with configurable retries and preflight checks
        while retry_count <= max_retries {
            if retry_count > 0 && retry_count % blockhash_validity_threshold == 0 {
                recent_blockhash = self.connection().get_latest_blockhash()?;
                optimized_transaction.try_sign(&[config.from_keypair], recent_blockhash)?;
            }

            match self
                .connection()
                .send_transaction_with_config(&optimized_transaction, send_transaction_config)
            {
                Ok(signature) => {
                    txt_sig = signature;
                    return self.poll_transaction_confirmation(txt_sig).await;
                }
                Err(error) => {
                    retry_count += 1;

                    if retry_count > max_retries {
                        return Err(HeliusError::ClientError(error));
                    }

                    continue;
                }
            }
        }

        Err(HeliusError::Timeout {
            code: StatusCode::REQUEST_TIMEOUT,
            text: "Reached an unexpected point in send_smart_transaction".to_string(),
        })
    }
}
