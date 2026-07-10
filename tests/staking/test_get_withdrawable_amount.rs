use base64::Engine;
use solana_sdk::pubkey::Pubkey;
use solana_stake_interface::stake_flags::StakeFlags;
use solana_stake_interface::state::{Meta, Stake, StakeStateV2};

use super::helpers::setup_mock;

use mockito::{Matcher, Server};

/// Base64-encoded bincode of a delegated stake account with the given `deactivation_epoch`.
fn stake_account_data(deactivation_epoch: u64) -> String {
    let mut stake = Stake::default();
    stake.delegation.deactivation_epoch = deactivation_epoch;
    let state = StakeStateV2::Stake(Meta::default(), stake, StakeFlags::default());
    let bytes = bincode::serialize(&state).unwrap();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn mock_account(server: &mut Server, deactivation_epoch: u64, lamports: u64) {
    let body = format!(
        r#"{{"jsonrpc":"2.0","result":{{"context":{{"slot":1}},"value":{{"data":["{}","base64"],"executable":false,"lamports":{},"owner":"Stake11111111111111111111111111111111111111","rentEpoch":18446744073709551615,"space":200}}}},"id":1}}"#,
        stake_account_data(deactivation_epoch),
        lamports
    );
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getAccountInfo".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create();
}

fn mock_epoch(server: &mut Server, epoch: u64) {
    let body = format!(
        r#"{{"jsonrpc":"2.0","result":{{"absoluteSlot":100,"blockHeight":100,"epoch":{},"slotIndex":1,"slotsInEpoch":432000,"transactionCount":1}},"id":1}}"#,
        epoch
    );
    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getEpochInfo".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create();
}

/// A stake deactivated in epoch N is still cooling down during epoch N itself, so a query at
/// current_epoch == deactivation_epoch must report 0 withdrawable (the off-by-one boundary).
#[tokio::test(flavor = "multi_thread")]
async fn test_withdrawable_zero_during_deactivation_epoch() {
    let (mut server, helius) = setup_mock().await;
    mock_account(&mut server, 10, 5_000_000_000);
    mock_epoch(&mut server, 10);

    let stake_account = Pubkey::new_unique();
    let amount = helius.get_withdrawable_amount(stake_account, true).await.unwrap();

    assert_eq!(
        amount, 0,
        "Stake in its deactivation epoch must not be withdrawable yet"
    );
}

/// Once the deactivation epoch has fully passed (current_epoch > deactivation_epoch), the full
/// balance is withdrawable when rent-exempt lamports are included.
#[tokio::test(flavor = "multi_thread")]
async fn test_withdrawable_after_cooldown() {
    let (mut server, helius) = setup_mock().await;
    mock_account(&mut server, 10, 5_000_000_000);
    mock_epoch(&mut server, 11);

    let stake_account = Pubkey::new_unique();
    let amount = helius.get_withdrawable_amount(stake_account, true).await.unwrap();

    assert_eq!(
        amount, 5_000_000_000,
        "Cooled-down stake should report its full balance"
    );
}

/// An active (never-deactivated) stake uses the u64::MAX sentinel and is never withdrawable.
#[tokio::test(flavor = "multi_thread")]
async fn test_withdrawable_zero_for_active_stake() {
    let (mut server, helius) = setup_mock().await;
    mock_account(&mut server, u64::MAX, 5_000_000_000);
    mock_epoch(&mut server, 300);

    let stake_account = Pubkey::new_unique();
    let result = helius.get_withdrawable_amount(stake_account, true).await;

    assert!(matches!(result, Ok(0)), "Active stake must report 0, got {result:?}");
}
