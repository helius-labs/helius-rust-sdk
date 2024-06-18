#![allow(unused_imports)]
/// Jito Smart Transactions
///
/// This module allows the creation and sending of smart transactions with Jito tips.
/// It includes methods to add tips to transactions, create smart transactions with tips, and
/// send these smart transactions as bundles. Additionally, it provides the ability to check
/// the status of sent bundles  
use crate::error::{HeliusError, Result};
use crate::types::{
    BasicRequest, CreateSmartTransactionConfig, GetPriorityFeeEstimateOptions, GetPriorityFeeEstimateRequest,
    GetPriorityFeeEstimateResponse, SmartTransaction, SmartTransactionConfig,
};
use crate::Helius;

use bincode::{serialize, ErrorKind};
use phf::phf_map;
use rand::seq::SliceRandom;
use reqwest::{Method, StatusCode, Url};
use serde::Serialize;
use solana_client::rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig};
use solana_client::rpc_response::{Response, RpcSimulateTransactionResult};
use solana_sdk::system_instruction;
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
    transaction::{Transaction, VersionedTransaction},
};
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::time::{sleep, timeout_at};

/// Jito tip accounts
pub const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

/// Jito API URLs for different regions
pub static JITO_API_URLS: phf::Map<&'static str, &'static str> = phf_map! {
    "Default" => "https://mainnet.block-engine.jito.wtf",
    "NY" => "https://ny.mainnet.block-engine.jito.wtf",
    "Amsterdam" => "https://amsterdam.mainnet.block-engine.jito.wtf",
    "Frankfurt" => "https://frankfurt.mainnet.block-engine.jito.wtf",
    "Tokyo" => "https://tokyo.mainnet.block-engine.jito.wtf",
};

/// Type alias for Jito regions
pub type JitoRegion = &'static str;

impl Helius {
    /// Adds a tip instruction to the provided instructions
    ///
    /// # Arguments
    /// * `instructions` - The transaction instructions to which the tip_instruction will be added
    /// * `fee_payer` - The public key of the fee payer
    /// * `tip_account` - The public key of the tip account as a string
    /// * `tip_amount` - The amount of lamports to tip
    pub fn add_tip_instruction(
        &self,
        instructions: &mut Vec<Instruction>,
        fee_payer: Pubkey,
        tip_account: &str,
        tip_amount: u64,
    ) {
        let tip_instruction: Instruction =
            system_instruction::transfer(&fee_payer, &Pubkey::from_str(tip_account).unwrap(), tip_amount);
        instructions.push(tip_instruction);
    }

    /// Creates a smart transaction with a Jito tip
    ///
    /// # Arguments
    /// * `config` - The configuration for creating the smart transaction
    /// * `tip_amount` - The amount of lamports to tip. Defaults to `1000`
    ///
    /// # Returns
    /// A `Result` containing the serialized transaction as a base58-encoded string
    pub async fn create_smart_transaction_with_tip(
        &self,
        mut config: CreateSmartTransactionConfig<'_>,
        tip_amount: Option<u64>,
    ) -> Result<String> {
        if config.signers.is_empty() {
            return Err(HeliusError::InvalidInput(
                "The transaction must have at least one signer".to_string(),
            ));
        }

        let tip_amount: u64 = tip_amount.unwrap_or(1000);
        let random_tip_account = JITO_TIP_ACCOUNTS.choose(&mut rand::thread_rng()).unwrap();
        let payer_key = config
            .fee_payer
            .map_or_else(|| config.signers[0].pubkey(), |signer| signer.pubkey());

        self.add_tip_instruction(&mut config.instructions, payer_key, random_tip_account, tip_amount);

        let smart_transaction: SmartTransaction = self.create_smart_transaction(&config).await?;
        let serialized_transaction: Vec<u8> =
            serialize(&smart_transaction).map_err(|e: Box<ErrorKind>| HeliusError::InvalidInput(e.to_string()))?;
        let transaction_base58: String = encode(&serialized_transaction).into_string();

        Ok(transaction_base58)
    }

    /// Sends a bundle of transactions to the Jito Block Engine
    ///
    /// # Arguments
    /// * `serialized_transactions` - The serialized transactions in the bundle
    /// * `jito_api_url` - The Jito Block Engine API URL
    ///
    /// # Returns
    /// A `Result` containing the bundle ID
    pub async fn send_jito_bundle(&self, serialized_transactions: Vec<String>, jito_api_url: &str) -> Result<String> {
        let request: BasicRequest = BasicRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "sendBundle".to_string(),
            params: vec![serialized_transactions],
        };

        let parsed_url: Url = Url::parse(&jito_api_url).expect("Failed to parse URL");

        self.rpc_client.handler.send(Method::POST, parsed_url, Some(&request)).await
    }

}
