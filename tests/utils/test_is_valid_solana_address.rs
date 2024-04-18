use helius_sdk::utils::is_valid_solana_address;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_addresses() {
        let valid_addresses = [
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
            "9AhKqLR67hwapvG8SA2JFXaCshXc9nALJjpKaHZrsbkw",
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        ];

        for addr in valid_addresses.iter() {
            assert!(
                is_valid_solana_address(addr),
                "Address {} is a valid Solana address",
                addr
            );
        }
    }
}
