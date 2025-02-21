use crate::error::{HeliusError, Result};
use crate::types::{
    CreateSmartTransactionConfig, CreateSmartTransactionSeedConfig, GetPriorityFeeEstimateOptions,
    GetPriorityFeeEstimateRequest, GetPriorityFeeEstimateResponse, SmartTransaction, SmartTransactionConfig, Timeout,
};
use crate::Helius;
use std::sync::Arc;

use bincode::{serialize, ErrorKind};
use reqwest::StatusCode;
use solana_client::{
    rpc_client::SerializableTransaction,
    rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig},
    rpc_response::{Response, RpcSimulateTransactionResult},
};
use solana_sdk::signature::keypair_from_seed;
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    bs58::encode,
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    instruction::Instruction,
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::{Signature, Signer},
    signer::keypair::Keypair,
    transaction::{Transaction, VersionedTransaction},
};
use solana_transaction_status::TransactionConfirmationStatus;
use std::time::{Duration, Instant};
use tokio::time::sleep;

impl Helius {
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
                    if status.err.is_some() {
                        return Err(HeliusError::TransactionError(status.err.unwrap()));
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
    /// whether it's a legacy or versioned smart transaction. The transaction's send configuration can also be changed, if provided
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
        let mut legacy_transaction: Option<Transaction> = None;
        let mut versioned_transaction: Option<VersionedTransaction> = None;

        // Build the initial transaction based on whether lookup tables are present
        if is_versioned {
            let lookup_tables: &[AddressLookupTableAccount] = config.lookup_tables.as_deref().unwrap_or_default();
            let v0_message: v0::Message =
                v0::Message::try_compile(&payer_pubkey, &config.instructions, lookup_tables, recent_blockhash)?;
            let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message);

            let all_signers: Vec<Arc<dyn Signer>> = if let Some(fee_payer) = &config.fee_payer {
                let mut all_signers = config.signers.clone();
                if !all_signers.iter().any(|signer| signer.pubkey() == fee_payer.pubkey()) {
                    all_signers.push(fee_payer.clone());
                }

                all_signers
            } else {
                config.signers.clone()
            };

            // Sign the versioned transaction
            let signatures: Vec<Signature> = all_signers
                .iter()
                .map(|signer| signer.try_sign_message(versioned_message.serialize().as_slice()))
                .collect::<std::result::Result<Vec<_>, _>>()?;

            versioned_transaction = Some(VersionedTransaction {
                signatures,
                message: versioned_message,
            });
        } else {
            // If no lookup tables are present, we build a regular transaction
            let mut tx: Transaction = Transaction::new_with_payer(&config.instructions, Some(&payer_pubkey));
            tx.try_partial_sign(&config.signers, recent_blockhash)?;

            if let Some(fee_payer) = config.fee_payer.as_ref() {
                tx.try_partial_sign(&[fee_payer], recent_blockhash)?;
            }

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
        let units: Option<u64> = self
            .get_compute_units(
                updated_instructions,
                payer_pubkey,
                config.lookup_tables.clone().unwrap_or_default(),
                Some(&config.signers),
            )
            .await?;

        if units.is_none() {
            return Err(HeliusError::InvalidInput(
                "Error fetching compute units for the instructions provided".to_string(),
            ));
        }

        let compute_units: u64 = units.unwrap();
        println!("{}", compute_units);
        let customers_cu: u32 = if compute_units < 1000 {
            1000
        } else {
            (compute_units as f64 * 1.1).ceil() as u32
        };

        // Add the compute unit limit instruction with a margin
        let compute_units_ix: Instruction = ComputeBudgetInstruction::set_compute_unit_limit(customers_cu);
        final_instructions.push(compute_units_ix);

        // Add the original instructions back
        final_instructions.extend(config.instructions.clone());

        // Rebuild the transaction with the final instructions
        if is_versioned {
            let lookup_tables: &[AddressLookupTableAccount] = config.lookup_tables.as_deref().unwrap();
            let v0_message: v0::Message =
                v0::Message::try_compile(&payer_pubkey, &final_instructions, lookup_tables, recent_blockhash)?;
            let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message);

            let all_signers: Vec<Arc<dyn Signer>> = if let Some(fee_payer) = config.fee_payer.as_ref() {
                let mut all_signers = config.signers.clone();
                if !all_signers.iter().any(|signer| signer.pubkey() == fee_payer.pubkey()) {
                    all_signers.push(fee_payer.clone());
                }
                all_signers
            } else {
                config.signers.clone()
            };

            let signatures: Vec<Signature> = all_signers
                .iter()
                .map(|signer| signer.try_sign_message(versioned_message.serialize().as_slice()))
                .collect::<std::result::Result<Vec<_>, _>>()?;

            versioned_transaction = Some(VersionedTransaction {
                signatures,
                message: versioned_message,
            });

            Ok((
                SmartTransaction::Versioned(versioned_transaction.unwrap()),
                last_valid_block_hash,
            ))
        } else {
            let mut tx: Transaction = Transaction::new_with_payer(&final_instructions, Some(&payer_pubkey));
            tx.try_partial_sign(&config.signers, recent_blockhash)?;

            if let Some(fee_payer) = config.fee_payer.as_ref() {
                tx.try_partial_sign(&[fee_payer], recent_blockhash)?;
            }

            legacy_transaction = Some(tx);

            Ok((
                SmartTransaction::Legacy(legacy_transaction.unwrap()),
                last_valid_block_hash,
            ))
        }
    }

    /// Builds and sends an optimized transaction, and handles its confirmation status
    ///
    /// # Arguments
    /// * `config` - The configuration for the smart transaction, which includes the transaction's instructions, signers, and lookup tables, depending on
    /// whether it's a legacy or versioned smart transaction. The transaction's send configuration can also be changed, if provided
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

        while Instant::now().duration_since(start_time) < timeout
            || self.connection().get_block_height()? <= last_valid_block_height
        {
            let result = self
                .connection()
                .send_transaction_with_config(transaction, send_transaction_config);

            match result {
                Ok(signature) => {
                    // Poll for transaction confirmation
                    match self.poll_transaction_confirmation(signature).await {
                        Ok(sig) => return Ok(sig),
                        // Retry on polling failure
                        Err(_) => continue,
                    }
                }
                // Retry on send failure
                Err(_) => continue,
            }
        }

        Err(HeliusError::Timeout {
            code: StatusCode::REQUEST_TIMEOUT,
            text: "Transaction failed to confirm in 60s".to_string(),
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
            let mut tx = VersionedTransaction {
                signatures: vec![Signature::default(); keypairs.len()],
                message: versioned_message.clone(),
            };

            for (i, keypair) in keypairs.iter().enumerate() {
                tx.signatures[i] = keypair.sign_message(&versioned_message.serialize());
            }

            tx
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
                "At least one signer seed must be provided".to_string(),
            ));
        }

        let keypairs: Vec<Keypair> = create_config
            .signer_seeds
            .into_iter()
            .map(|seed| keypair_from_seed(&seed).expect("Failed to create keypair from seed"))
            .collect();

        // Create the fee payer keypair if provided. Otherwise, we default to the first signer
        let fee_payer: Keypair = if let Some(fee_payer_seed) = create_config.fee_payer_seed {
            keypair_from_seed(&fee_payer_seed).expect("Failed to create keypair from seed")
        } else {
            Keypair::from_bytes(&keypairs[0].to_bytes()).unwrap()
        };

        let (recent_blockhash, last_valid_block_hash) = self
            .connection()
            .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())?;

        let mut final_instructions: Vec<Instruction> = vec![];

        // Get priority fee estimate
        let transaction: Transaction = Transaction::new_signed_with_payer(
            &create_config.instructions,
            Some(&fee_payer.pubkey()),
            &[&fee_payer],
            recent_blockhash,
        );

        let serialized_tx: Vec<u8> = serialize(&transaction).map_err(|e| HeliusError::InvalidInput(e.to_string()))?;
        let transaction_base58: String = encode(&serialized_tx).into_string();

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

        let units: Option<u64> = self
            .get_compute_units_thread_safe(
                test_instructions,
                fee_payer.pubkey(),
                create_config.lookup_tables.clone().unwrap_or_default(),
                Some(&[&fee_payer]),
            )
            .await?;

        let compute_units: u64 = units.ok_or(HeliusError::InvalidInput(
            "Error fetching compute units for the instructions provided".to_string(),
        ))?;

        let customers_cu: u32 = if compute_units < 1000 {
            1000
        } else {
            (compute_units as f64 * 1.1).ceil() as u32
        };

        final_instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(customers_cu));
        final_instructions.extend(create_config.instructions);

        // Create the final transaction
        let transaction: SmartTransaction = if let Some(lookup_tables) = create_config.lookup_tables {
            let message: v0::Message = v0::Message::try_compile(
                &fee_payer.pubkey(),
                &final_instructions,
                &lookup_tables,
                recent_blockhash,
            )?;

            let versioned_message: VersionedMessage = VersionedMessage::V0(message);

            let fee_payer_copy: Keypair = Keypair::from_bytes(&fee_payer.to_bytes()).unwrap();
            let mut all_signers: Vec<Keypair> = vec![fee_payer_copy];
            all_signers.extend(keypairs.into_iter().filter(|k| k.pubkey() != fee_payer.pubkey()));

            let mut tx: VersionedTransaction = VersionedTransaction {
                signatures: vec![Signature::default(); all_signers.len()],
                message: versioned_message.clone(),
            };

            // Sign message with all keypairs
            for (i, keypair) in all_signers.iter().enumerate() {
                tx.signatures[i] = keypair.sign_message(&versioned_message.serialize());
            }

            SmartTransaction::Versioned(tx)
        } else {
            let mut tx: Transaction = Transaction::new_with_payer(&final_instructions, Some(&fee_payer.pubkey()));

            let mut signers: Vec<&Keypair> = vec![&fee_payer];
            signers.extend(keypairs.iter().filter(|k| k.pubkey() != fee_payer.pubkey()));

            tx.sign(&signers, recent_blockhash);

            SmartTransaction::Legacy(tx)
        };

        // Send and confirm the transaction
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
        let mut legacy_transaction: Option<Transaction> = None;
        let mut versioned_transaction: Option<VersionedTransaction> = None;

        // Build the initial unsigned tx
        if is_versioned {
            let lookup_tables: &[AddressLookupTableAccount] = config.lookup_tables.as_deref().unwrap_or_default();
            let v0_message: v0::Message =
                v0::Message::try_compile(&payer_pubkey, &config.instructions, lookup_tables, recent_blockhash)?;
            let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message);

            versioned_transaction = Some(VersionedTransaction {
                signatures: vec![],
                message: versioned_message,
            });
        } else {
            let mut tx: Transaction = Transaction::new_with_payer(&config.instructions, Some(&payer_pubkey));
            tx.message.recent_blockhash = recent_blockhash;
            legacy_transaction = Some(tx);
        }

        // Serialize the unsigned tx
        let serialized_tx: Vec<u8> = if let Some(tx) = &legacy_transaction {
            serialize(&tx).map_err(|e: Box<ErrorKind>| HeliusError::InvalidInput(e.to_string()))?
        } else if let Some(tx) = &versioned_transaction {
            serialize(&tx).map_err(|e: Box<ErrorKind>| HeliusError::InvalidInput(e.to_string()))?
        } else {
            return Err(HeliusError::InvalidInput("No transaction available".to_string()));
        };

        // Encode the tx to base58 for priority fee estimation
        let transaction_base58: String = encode(&serialized_tx).into_string();

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

        let compute_units = units.unwrap();
        let customers_cu: u32 = if compute_units < 1000 {
            1000
        } else {
            (compute_units as f64 * 1.1).ceil() as u32
        };

        // Add the compute unit limit ix at the start
        let compute_units_ix = ComputeBudgetInstruction::set_compute_unit_limit(customers_cu);
        final_instructions.push(compute_units_ix);

        // Append the original ixs back
        final_instructions.extend(config.instructions.clone());

        // Rebuild the final unsigned tx with the updated ixs
        if is_versioned {
            let lookup_tables: &[AddressLookupTableAccount] = config.lookup_tables.as_deref().unwrap();
            let v0_message: v0::Message =
                v0::Message::try_compile(&payer_pubkey, &final_instructions, lookup_tables, recent_blockhash)?;
            let versioned_message: VersionedMessage = VersionedMessage::V0(v0_message);

            versioned_transaction = Some(VersionedTransaction {
                signatures: vec![],
                message: versioned_message,
            });

            Ok((
                SmartTransaction::Versioned(versioned_transaction.unwrap()),
                last_valid_block_hash,
            ))
        } else {
            let mut tx: Transaction = Transaction::new_with_payer(&final_instructions, Some(&payer_pubkey));
            tx.message.recent_blockhash = recent_blockhash;
            legacy_transaction = Some(tx);

            Ok((
                SmartTransaction::Legacy(legacy_transaction.unwrap()),
                last_valid_block_hash,
            ))
        }
    }
}
