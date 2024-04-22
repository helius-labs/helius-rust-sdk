use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub fn is_valid_solana_address(address: &str) -> bool {
    address.len() >= 32 && address.len() <= 44 && Pubkey::from_str(address).is_ok()
}
