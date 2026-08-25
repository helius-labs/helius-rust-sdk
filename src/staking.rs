use std::str::FromStr;

use crate::{
    error::{HeliusError, Result},
    Helius,
};

use bincode;
use once_cell::sync::Lazy;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_commitment_config::CommitmentConfig;
use solana_sdk::account::Account;
use solana_sdk::hash::Hash;
use solana_sdk::{
    bs58,
    instruction::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use solana_stake_interface::{
    instruction as stake_instruction,
    state::{Authorized, StakeStateV2},
};

pub static HELIUS_VALIDATOR_PUBKEY: Lazy<Pubkey> =
    Lazy::new(|| Pubkey::from_str("he1iusunGwqrNtafDtLdhsUQDFvo13z9sUa36PauBtk").expect("Invalid Pubkey"));

/// Byte offset of `Meta::authorized.staker` within the bincode encoding of [`StakeStateV2`].
///
/// Layout: a 4-byte bincode enum discriminant, then `Meta::rent_exempt_reserve` (`u64`, 8 bytes),
/// putting the staker at 12 and the withdrawer at 44. Pinned by `stake_state_authority_offsets`
/// in the unit tests below.
const STAKER_AUTHORITY_OFFSET: usize = 12;

/// Byte offset of `Meta::authorized.withdrawer` within the bincode encoding of [`StakeStateV2`].
///
/// Not used to build a filter today; it exists so the layout test can prove offset 12 is the
/// staker rather than merely that *something* sits there. See [`STAKER_AUTHORITY_OFFSET`].
#[cfg(test)]
const WITHDRAWER_AUTHORITY_OFFSET: usize = 44;

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
    pub async fn create_stake_transaction(&self, owner: Pubkey, amount_sol: f64) -> Result<(String, Pubkey)> {
        let rent_exempt: u64 = self
            .connection()
            .get_minimum_balance_for_rent_exemption(StakeStateV2::size_of())?;
        if !amount_sol.is_finite() || amount_sol <= 0.0 {
            return Err(HeliusError::InvalidInput(
                "Stake amount must be a positive finite number".into(),
            ));
        }

        let stake_lamports_f = (amount_sol * LAMPORTS_PER_SOL as f64).round();

        if stake_lamports_f < 0.0 || stake_lamports_f > u64::MAX as f64 {
            return Err(HeliusError::InvalidInput(
                "Stake amount is out of valid lamports range".into(),
            ));
        }

        let lamports = (stake_lamports_f as u64)
            .checked_add(rent_exempt)
            .ok_or_else(|| HeliusError::InvalidInput("Lamports overflow".into()))?;

        let stake_account: Keypair = Keypair::new();

        let authorized: Authorized = Authorized {
            staker: owner,
            withdrawer: owner,
        };

        let create_ix: Vec<Instruction> = stake_instruction::create_account(
            &owner,
            &stake_account.pubkey(),
            &authorized,
            &solana_stake_interface::state::Lockup::default(),
            lamports,
        );

        let delegate_ix: Instruction =
            stake_instruction::delegate_stake(&stake_account.pubkey(), &owner, &HELIUS_VALIDATOR_PUBKEY);

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

    /// Generate an unsigned, base58-encoded transaction to deactivate a stake account
    ///
    /// This transaction must be signed by the wallet that authorized the stake before broadcasting
    /// After deactivation, the stake must cool down (~2 epochs) before it can be withdrawn
    ///
    /// # Arguments
    ///
    /// * `owner` - The public key of the wallet that authorized the original stake
    /// * `stake_account` - The public key of the stake account to deactivate
    ///
    /// # Returns
    ///
    /// * `String` - A base58-encoded unsigned serialized transaction
    ///
    /// # Errors
    ///
    /// Returns an error if fetching the latest blockhash or serializing the transaction fails
    pub async fn create_unstake_transaction(&self, owner: Pubkey, stake_account: Pubkey) -> Result<String> {
        let deactivate_ix: Instruction = stake_instruction::deactivate_stake(&stake_account, &owner);

        let blockhash: Hash = self.connection().get_latest_blockhash()?;

        let mut tx: Transaction = Transaction::new_with_payer(&[deactivate_ix], Some(&owner));

        tx.message.recent_blockhash = blockhash;

        let serialized: Vec<u8> = bincode::serialize(&tx)
            .map_err(|e| HeliusError::InvalidInput(format!("Failed to serialize transaction: {e}")))?;

        let encoded: String = bs58::encode(serialized).into_string();

        Ok(encoded)
    }

    /// Generate an unsigned, base58-encoded transaction to withdraw lamports from a stake account
    ///
    /// This must only be called **after** the stake account has been deactivated and fully cooled down
    ///
    /// # Arguments
    ///
    /// * `owner` - The wallet that authorized the stake and can withdraw from it
    /// * `stake_account` - The public key of the stake account to withdraw from
    /// * `destination` - The wallet to receive the withdrawn SOL
    /// * `lamports` - The number of lamports to withdraw
    ///
    /// # Returns
    ///
    /// * `String` - A base58-encoded unsigned serialized transaction
    ///
    /// # Errors
    ///
    /// Returns an error if the blockhash cannot be fetched or if serialization fails
    pub async fn create_withdraw_transaction(
        &self,
        owner: Pubkey,
        stake_account: Pubkey,
        destination: Pubkey,
        lamports: u64,
    ) -> Result<String> {
        let withdraw_ix: Instruction = stake_instruction::withdraw(
            &stake_account,
            &owner,
            &destination,
            lamports,
            None, // Custodian
        );

        let blockhash: Hash = self.connection().get_latest_blockhash()?;

        let mut tx: Transaction = Transaction::new_with_payer(&[withdraw_ix], Some(&owner));
        tx.message.recent_blockhash = blockhash;

        let serialized: Vec<u8> = bincode::serialize(&tx)
            .map_err(|e| HeliusError::InvalidInput(format!("Failed to serialize transaction: {e}")))?;

        let encoded: String = bs58::encode(serialized).into_string();

        Ok(encoded)
    }

    /// Generate the instructions to create and delegate a new stake account with Helius
    ///
    /// This method only returns the `Vec<Instruction>` and the newly generated `Keypair` for the stake account
    /// Note that **you** are responsible for building, signing, and sending the transaction. We recommend
    /// using this method with our smart transactions
    ///
    /// # Arguments
    ///
    /// * `owner` - The public key of the wallet funding and authorizing the stake
    /// * `amount_sol` - The amount of SOL to stake, **excluding** the rent-exempt minimum
    ///
    /// # Returns
    ///
    /// * A tuple:
    ///   - `Vec<Instruction>` - Instructions to create and delegate the stake account
    ///   - `Keypair` - The newly generated stake account keypair
    ///
    /// # Errors
    ///
    /// Returns an error if fetching the rent-exempt minimum balance fails
    pub async fn get_stake_instructions(&self, owner: Pubkey, amount_sol: f64) -> Result<(Vec<Instruction>, Keypair)> {
        let rent_exempt: u64 = self
            .connection()
            .get_minimum_balance_for_rent_exemption(StakeStateV2::size_of())?;

        if !amount_sol.is_finite() || amount_sol <= 0.0 {
            return Err(HeliusError::InvalidInput(
                "Stake amount must be a positive finite number".into(),
            ));
        }

        let stake_lamports_f = (amount_sol * LAMPORTS_PER_SOL as f64).round();

        if stake_lamports_f < 0.0 || stake_lamports_f > u64::MAX as f64 {
            return Err(HeliusError::InvalidInput(
                "Stake amount is out of valid lamports range".into(),
            ));
        }

        let lamports = (stake_lamports_f as u64)
            .checked_add(rent_exempt)
            .ok_or_else(|| HeliusError::InvalidInput("Lamports overflow".into()))?;

        let stake_account: Keypair = Keypair::new();

        let authorized: Authorized = Authorized {
            staker: owner,
            withdrawer: owner,
        };

        let mut instructions: Vec<Instruction> = stake_instruction::create_account(
            &owner,
            &stake_account.pubkey(),
            &authorized,
            &solana_stake_interface::state::Lockup::default(),
            lamports,
        );

        instructions.push(stake_instruction::delegate_stake(
            &stake_account.pubkey(),
            &owner,
            &HELIUS_VALIDATOR_PUBKEY,
        ));

        Ok((instructions, stake_account))
    }

    /// Generates an instruction to deactivate a given stake account
    ///
    /// This instruction deactivates the stake account, signaling the validator
    /// to remove it at the next epoch boundary. After two epochs (~2-4 days),
    /// the stake can be withdrawn
    ///
    /// # Arguments
    ///
    /// * `owner` - The public key that authorized the original stake
    /// * `stake_account` - The public key of the stake account to deactivate
    ///
    /// # Returns
    ///
    /// * `Instruction` - The `deactivate_stake` instruction
    pub fn get_unstake_instruction(&self, owner: Pubkey, stake_account: Pubkey) -> Instruction {
        stake_instruction::deactivate_stake(&stake_account, &owner)
    }

    /// Generates an instruction to withdraw lamports from a given stake account
    ///
    /// This should be called **after** the stake account has been deactivated and fully cooled down
    /// If the entire balance is withdrawn (including rent-exempt minimum), the stake account will
    /// be closed
    ///
    /// # Arguments
    ///
    /// * `owner` - The public key that authorized the withdrawal
    /// * `stake_account` - The public key of the stake account to withdraw from
    /// * `destination` - The public key of the wallet to receive the withdrawn lamports
    /// * `lamports` - The amount of lamports to withdraw
    ///
    /// # Returns
    ///
    /// * `Instruction` - The `withdraw` instruction
    pub fn get_withdraw_instruction(
        &self,
        owner: Pubkey,
        stake_account: Pubkey,
        destination: Pubkey,
        lamports: u64,
    ) -> Instruction {
        stake_instruction::withdraw(&stake_account, &owner, &destination, lamports, None)
    }

    /// Determine how many lamports are withdrawable from a stake account
    ///
    /// This checks whether the stake account is fully deactivated and cooled down,
    /// and subtracts the rent-exempt minimum unless explicitly included.
    ///
    /// # Arguments
    ///
    /// * `stake_account` - The public key of the stake account to inspect
    /// * `include_rent_exempt` - Whether to include the rent-exempt minimum in the returned amount
    ///
    /// # Returns
    ///
    /// * `u64` - The number of lamports that can be withdrawn (0 if none)
    ///
    /// # Errors
    ///
    /// Returns an error if the account cannot be found or isn't a valid stake account
    pub async fn get_withdrawable_amount(&self, stake_account: Pubkey, include_rent_exempt: bool) -> Result<u64> {
        let account = self
            .connection()
            .get_account_with_commitment(&stake_account, CommitmentConfig::confirmed())?
            .value
            .ok_or_else(|| HeliusError::NotFound {
                text: format!("Stake account {} not found", stake_account),
            })?;

        let lamports = account.lamports;

        let state: StakeStateV2 = bincode::deserialize(&account.data)
            .map_err(|_| HeliusError::InvalidInput("Failed to parse stake account".into()))?;

        let deactivation_epoch = match state {
            StakeStateV2::Stake(_, stake, _) => stake.delegation.deactivation_epoch,
            _ => {
                return Err(HeliusError::InvalidInput(
                    "Account is not a valid delegated stake account".into(),
                ));
            }
        };

        let current_epoch = self.connection().get_epoch_info()?.epoch;

        // A stake deactivated in epoch N is only withdrawable once epoch N has fully passed
        // (i.e. current_epoch > deactivation_epoch). During the deactivation epoch itself it is
        // still cooling down, so `>=` is required here. Active stakes use a sentinel
        // deactivation_epoch of u64::MAX and are likewise reported as not withdrawable.
        if deactivation_epoch >= current_epoch {
            return Ok(0); // Still active or cooling down
        }

        if include_rent_exempt {
            return Ok(lamports);
        }

        let rent_exempt = self
            .connection()
            .get_minimum_balance_for_rent_exemption(StakeStateV2::size_of())?;

        Ok(lamports.saturating_sub(rent_exempt))
    }

    /// Return every stake-program account whose `Authorized::staker` (offset 12)
    /// matches `wallet`.
    ///
    /// The filter is a `memcmp` against the bincode encoding of [`StakeStateV2`],
    /// whose `Meta` prefix lays out as:
    ///
    /// ```text
    /// offset 0  –   enum discriminant (bincode u32, 4 bytes)
    /// offset 4  –   Meta::rent_exempt_reserve (u64, 8 bytes)
    /// offset 12 –   Meta::authorized.staker (Pubkey, 32 bytes)
    /// offset 44 –   Meta::authorized.withdrawer (Pubkey, 32 bytes)
    /// offset 76 –   Meta::lockup ...
    /// ```
    ///
    /// Note this filters on the **staker** authority (the one that can delegate and
    /// deactivate), not the withdrawer. For accounts where the two differ — custodial
    /// and liquid-staking setups, delegated management — these are different sets.
    ///
    /// # Arguments
    /// * `wallet` – the Pubkey we filter for
    ///
    /// # Returns
    ///
    /// `Vec<(Pubkey, Account)>` – keyed raw accounts.  You can deserialize them with
    /// `StakeStateV2::deserialize()` if you need to
    pub async fn get_stake_accounts(&self, wallet: Pubkey) -> Result<Vec<(Pubkey, Account)>> {
        let filters: Option<Vec<RpcFilterType>> = Some(vec![RpcFilterType::Memcmp(Memcmp::new(
            STAKER_AUTHORITY_OFFSET,
            MemcmpEncodedBytes::Base58(wallet.to_string()),
        ))]);

        let acct_cfg: RpcAccountInfoConfig = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..Default::default()
        };

        let cfg: RpcProgramAccountsConfig = RpcProgramAccountsConfig {
            filters,
            account_config: acct_cfg,
            with_context: None,
            ..Default::default()
        };

        // `get_program_accounts_with_config` was removed in solana-client 4.x; its replacement
        // returns UI-encoded accounts, so decode each back into an `Account`.
        let accounts: Vec<(Pubkey, Account)> = self
            .connection()
            .get_program_ui_accounts_with_config(&solana_stake_interface::program::id(), cfg)
            .map_err(|e| HeliusError::InvalidInput(e.to_string()))?
            .into_iter()
            .map(|(pubkey, ui_account)| {
                ui_account
                    .to_account()
                    .map(|account| (pubkey, account))
                    .ok_or_else(|| HeliusError::InvalidInput(format!("Failed to decode stake account {}", pubkey)))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(accounts)
    }
}

#[cfg(test)]
mod tests {
    use super::{Pubkey, STAKER_AUTHORITY_OFFSET, WITHDRAWER_AUTHORITY_OFFSET};

    use solana_stake_interface::stake_flags::StakeFlags;
    use solana_stake_interface::state::{Authorized, Delegation, Lockup, Meta, Stake, StakeStateV2};

    /// Pins the byte offsets of both authorities within the bincode encoding of `StakeStateV2`.
    ///
    /// `get_stake_accounts` filters with a `memcmp` at a hard-coded offset, so a layout change
    /// upstream (or a transposed constant) would otherwise silently return the wrong accounts
    /// rather than fail. Serializing a value whose staker and withdrawer differ is the only way
    /// to tell the two apart: with equal keys, offset 44 looks just as correct as offset 12.
    #[test]
    #[allow(deprecated)] // `Meta::rent_exempt_reserve` is deprecated but still occupies the bytes
    fn stake_state_authority_offsets() {
        let staker = Pubkey::new_from_array([0x11; 32]);
        let withdrawer = Pubkey::new_from_array([0x22; 32]);

        let meta = Meta {
            rent_exempt_reserve: 42,
            authorized: Authorized { staker, withdrawer },
            lockup: Lockup::default(),
        };
        let state = StakeStateV2::Stake(
            meta,
            Stake {
                delegation: Delegation::default(),
                credits_observed: 7,
            },
            StakeFlags::empty(),
        );

        let bytes = bincode::serialize(&state).expect("StakeStateV2 should serialize");

        // 4-byte bincode enum discriminant, then `Meta::rent_exempt_reserve` (u64).
        assert_eq!(&bytes[0..4], &[2, 0, 0, 0], "expected the `Stake` variant discriminant");
        assert_eq!(
            &bytes[4..12],
            &42u64.to_le_bytes(),
            "rent_exempt_reserve should directly follow the discriminant"
        );

        assert_eq!(
            &bytes[STAKER_AUTHORITY_OFFSET..STAKER_AUTHORITY_OFFSET + 32],
            staker.as_ref(),
            "STAKER_AUTHORITY_OFFSET does not point at Authorized::staker"
        );
        assert_eq!(
            &bytes[WITHDRAWER_AUTHORITY_OFFSET..WITHDRAWER_AUTHORITY_OFFSET + 32],
            withdrawer.as_ref(),
            "WITHDRAWER_AUTHORITY_OFFSET does not point at Authorized::withdrawer"
        );
    }
}
