use std::str::FromStr;

use crate::{Helius, error::{HeliusError, Result}};

use bincode;
use once_cell::sync::Lazy;
use solana_sdk::{
    bs58, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signer::{keypair::Keypair, Signer}, stake::{
        self, instruction as stake_instruction, state::{Authorized, StakeStateV2}
    }, transaction::Transaction, instruction::Instruction
};
use solana_program::hash::Hash;

pub static HELIUS_VALIDATOR_PUBKEY: Lazy<Pubkey> = Lazy::new(|| {
    Pubkey::from_str("he1iusunGwqrNtafDtLdhsUQDFvo13z9sUa36PauBtk").expect("Invalid Pubkey")
});

impl Helius {
    /// Generate an unsigned, base58-encoded transaction that creates and delegates a new stake account
    ///
    /// This transaction must be signed by the funder's wallet before broadcasting. It delegates stake
    /// to the Helius validator, and includes enough lamports to cover both the specified stake amount
    /// and the rent-exempt minimum for a stake account
    ///
    /// # Arguments
    ///
    /// * `owner` - The public key of the wallet funding and authorizing the stake
    /// * `amount_sol` - The amount of SOL to stake, **excluding** the rent-exempt minimum
    ///
    /// # Returns
    ///
    /// * A tuple of:
    ///   - `String`: base58-encoded unsigned serialized transaction
    ///   - `Pubkey`: the new stake account's public key
    ///
    /// # Errors
    ///
    /// Returns an error if fetching the rent-exemption balance, blockhash, or serializing the transaction
    /// fails
    pub async fn create_stake_transaction(
        &self,
        owner: Pubkey,
        amount_sol: f64,
    ) -> Result<(String, Pubkey)> {
        let rent_exempt: u64 = self.connection().get_minimum_balance_for_rent_exemption(StakeStateV2::size_of())?;
        let lamports: u64 = ((amount_sol * LAMPORTS_PER_SOL as f64).round() as u64) + rent_exempt;

        let stake_account: Keypair = Keypair::new();

        let authorized: Authorized = Authorized {
            staker: owner,
            withdrawer: owner,
        };
    
        let create_ix: Vec<Instruction> = stake_instruction::create_account(
            &owner,
            &stake_account.pubkey(),
            &authorized,
            &stake::state::Lockup::default(),
            lamports,
        );

        let delegate_ix: Instruction = stake_instruction::delegate_stake(
            &stake_account.pubkey(),
            &owner,
            &HELIUS_VALIDATOR_PUBKEY,
        );
    
        let blockhash: Hash = self.connection().get_latest_blockhash()?;
        let mut instructions: Vec<Instruction> = create_ix;
        instructions.push(delegate_ix);
        
        let mut tx: Transaction = Transaction::new_with_payer(&instructions, Some(&owner));
        tx.partial_sign(&[&stake_account], blockhash);
    
        let serialized: Vec<u8> = bincode::serialize(&tx)
            .map_err(|e| HeliusError::InvalidInput(format!("Failed to serialize transaction: {e}")))?;
    
        let encoded: String = bs58::encode(serialized).into_string();
    
        Ok((encoded, stake_account.pubkey()))
    }
}