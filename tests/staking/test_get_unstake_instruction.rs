use helius::staking::HELIUS_VALIDATOR_PUBKEY;
use solana_sdk::pubkey::Pubkey;
use solana_stake_interface::program::id as stake_program_id;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_unstake_instruction_returns_deactivate() {
    let (_server, helius) = setup_mock().await;

    let owner = Pubkey::new_unique();
    let stake_account = Pubkey::new_unique();

    let ix = helius.get_unstake_instruction(owner, stake_account);

    assert_eq!(ix.program_id, stake_program_id());
    assert!(
        ix.accounts.iter().any(|a| a.pubkey == stake_account),
        "Instruction should reference the stake account"
    );
    assert!(
        ix.accounts.iter().any(|a| a.pubkey == owner),
        "Instruction should reference the owner"
    );
}

#[tokio::test]
async fn test_get_unstake_instruction_owner_is_signer() {
    let (_server, helius) = setup_mock().await;

    let owner = Pubkey::new_unique();
    let stake_account = Pubkey::new_unique();

    let ix = helius.get_unstake_instruction(owner, stake_account);

    let owner_account = ix.accounts.iter().find(|a| a.pubkey == owner).unwrap();
    assert!(owner_account.is_signer, "Owner must be a signer on deactivate");
}

#[test]
fn test_helius_validator_pubkey_is_valid() {
    let expected = "he1iusunGwqrNtafDtLdhsUQDFvo13z9sUa36PauBtk";
    assert_eq!(HELIUS_VALIDATOR_PUBKEY.to_string(), expected);
}
