use crate::error::{HeliusError, Result};
use crate::Helius;

use reqwest::StatusCode;
use solana_client::rpc_response::{Response, RpcSimulateTransactionResult};
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    hash::Hash,
    instruction::Instruction,
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    signature::Signature,
    transaction::VersionedTransaction,
};
use std::time::Duration;
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
        let mut elapsed = Duration::default();

        let commitment_config: CommitmentConfig = CommitmentConfig::confirmed();

        loop {
            if elapsed >= timeout {
                return Err(HeliusError::Timeout {
                    code: StatusCode::REQUEST_TIMEOUT,
                    text: format!("Transaction {}'s confirmation timed out", txt_sig),
                });
            }

            match self.connection().get_signature_status_with_commitment(&txt_sig, commitment_config) {
                Ok(Some(Ok(()))) => return Ok(txt_sig),
                Ok(Some(Err(err))) => return Err(HeliusError::TransactionError(err)),
                Ok(None) => {
                    sleep(interval).await;
                    elapsed += interval;
                }
                Err(err) => return Err(HeliusError::ClientError(err)),
            }
        }
    }
}
