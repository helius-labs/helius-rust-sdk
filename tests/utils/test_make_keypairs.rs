use helius_sdk::utils::make_keypairs;
use solana_sdk::{signature::Keypair, signer::Signer};
use std::collections::HashSet;

#[test]
fn test_make_keypairs_generates_correct_amount() {
    let amounts: Vec<usize> = vec![1, 5, 10];

    for amount in amounts {
        let keypairs: Vec<Keypair> = make_keypairs(amount);
        assert_eq!(keypairs.len(), amount, "Should generate the correct number of keypairs");
    }
}

#[test]
fn test_make_keypairs_are_unique() {
    let keypairs: Vec<Keypair> = make_keypairs(10);
    let mut seen: HashSet<_> = HashSet::new();

    for keypair in keypairs {
        let pk = keypair.pubkey().to_string();
        assert!(seen.insert(pk), "Each keypair should be unique");
    }
}
