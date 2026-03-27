use solana_sdk::pubkey::Pubkey;
use solana_stake_interface::program::id as stake_program_id;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_withdraw_instruction_returns_withdraw() {
    let (_server, helius) = setup_mock().await;

    let owner = Pubkey::new_unique();
    let stake_account = Pubkey::new_unique();
    let destination = Pubkey::new_unique();
    let lamports: u64 = 5_000_000_000;

    let ix = helius.get_withdraw_instruction(owner, stake_account, destination, lamports);

    assert_eq!(ix.program_id, stake_program_id());
    assert!(
        ix.accounts.iter().any(|a| a.pubkey == stake_account),
        "Instruction should reference the stake account"
    );
    assert!(
        ix.accounts.iter().any(|a| a.pubkey == destination),
        "Instruction should reference the destination"
    );
    assert!(
        ix.accounts.iter().any(|a| a.pubkey == owner),
        "Instruction should reference the owner"
    );
}

#[tokio::test]
async fn test_get_withdraw_instruction_owner_is_signer() {
    let (_server, helius) = setup_mock().await;

    let owner = Pubkey::new_unique();
    let stake_account = Pubkey::new_unique();
    let destination = Pubkey::new_unique();

    let ix = helius.get_withdraw_instruction(owner, stake_account, destination, 1_000_000);

    let owner_account = ix.accounts.iter().find(|a| a.pubkey == owner).unwrap();
    assert!(owner_account.is_signer, "Owner must be a signer on withdraw");
}

#[tokio::test]
async fn test_get_withdraw_instruction_zero_lamports() {
    let (_server, helius) = setup_mock().await;

    let owner = Pubkey::new_unique();
    let stake_account = Pubkey::new_unique();
    let destination = Pubkey::new_unique();

    // Zero lamports is technically valid at the instruction level
    let ix = helius.get_withdraw_instruction(owner, stake_account, destination, 0);
    assert_eq!(ix.program_id, stake_program_id());
}
