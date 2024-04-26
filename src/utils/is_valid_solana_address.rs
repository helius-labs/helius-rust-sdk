use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Returns whether a given string slice is a valid Solana address
pub fn is_valid_solana_address(address: &str) -> bool {
    address.len() >= 32 && address.len() <= 44 && Pubkey::from_str(address).is_ok()
}
