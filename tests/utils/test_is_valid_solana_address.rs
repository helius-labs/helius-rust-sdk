use helius_sdk::utils::is_valid_solana_address;

#[test]
fn test_valid_addresses() {
    let valid_addresses: [&str; 3] = [
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

#[test]
fn test_invalid_addresses() {
    let invalid_addresses: [&str; 5] = [
        "DefinitelyNotASolanaAddress",
        "12345",
        "",
        "TooShort",
        "12345678901234567890123456789012345678901234",
    ];

    for addr in invalid_addresses.iter() {
        assert!(
            !is_valid_solana_address(addr),
            "Address {} is not a valid Solana address",
            addr
        );
    }
}
