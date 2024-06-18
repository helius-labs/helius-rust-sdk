#![allow(unused_imports)]
/// Jito Smart Transactions
/// 
/// This module allows the creation and sending of smart transactions with Jito tips.
/// It includes methods to add tips to transactions, create smart transactions with tips, and 
/// send these smart transactions as bundles. Additionally, it provides the ability to check
/// the status of sent bundles  

use crate::error::{HeliusError, Result};
use crate::types::{
    CreateSmartTransactionConfig, GetPriorityFeeEstimateOptions, GetPriorityFeeEstimateRequest,
    GetPriorityFeeEstimateResponse, SmartTransaction, SmartTransactionConfig,
};
use crate::Helius;

use bincode::{serialize, ErrorKind};
use reqwest::StatusCode;
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
use tokio::time::sleep;
use phf::phf_map;

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
        let tip_instruction: Instruction = system_instruction::transfer(&fee_payer, &Pubkey::from_str(tip_account).unwrap(), tip_amount);
        instructions.push(tip_instruction);
    }
}