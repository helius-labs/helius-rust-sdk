use mockito::Matcher;
use solana_sdk::pubkey::Pubkey;

use super::helpers::setup_mock;

const STAKE_PROGRAM: &str = "Stake11111111111111111111111111111111111111";
const STAKE_ACCOUNT: &str = "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin";

/// `get_stake_accounts` migrated from the removed `get_program_accounts_with_config` to
/// `get_program_ui_accounts_with_config`, which returns UI-encoded accounts. This asserts the
/// UI→`Account` decode is correct end to end: the base64 `data`, `lamports`, `owner`, and
/// `pubkey` all round-trip. (Data bytes `[1, 2, 3, 4]` encode to base64 `"AQIDBA=="`.)
#[tokio::test(flavor = "multi_thread")]
async fn test_get_stake_accounts_decodes_ui_accounts() {
    let (mut server, helius) = setup_mock().await;

    let body = format!(
        r#"{{"jsonrpc":"2.0","result":[{{"pubkey":"{STAKE_ACCOUNT}","account":{{"lamports":42,"owner":"{STAKE_PROGRAM}","data":["AQIDBA==","base64"],"executable":false,"rentEpoch":18446744073709551615,"space":4}}}}],"id":1}}"#
    );

    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getProgramAccounts".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create();

    let accounts = helius
        .get_stake_accounts(Pubkey::new_unique())
        .await
        .expect("get_stake_accounts should succeed");

    assert_eq!(accounts.len(), 1, "expected exactly one stake account");
    let (pubkey, account) = &accounts[0];
    assert_eq!(pubkey.to_string(), STAKE_ACCOUNT, "pubkey mismatch");
    assert_eq!(account.lamports, 42, "lamports mismatch");
    assert_eq!(account.owner.to_string(), STAKE_PROGRAM, "owner mismatch");
    assert_eq!(account.data, vec![1, 2, 3, 4], "base64 data did not decode correctly");
    assert!(!account.executable);
}

/// A UI account whose `data` cannot be decoded surfaces as an error rather than being silently
/// dropped. `"!!!"` is not valid base64.
#[tokio::test(flavor = "multi_thread")]
async fn test_get_stake_accounts_errors_on_undecodable_account() {
    let (mut server, helius) = setup_mock().await;

    let body = format!(
        r#"{{"jsonrpc":"2.0","result":[{{"pubkey":"{STAKE_ACCOUNT}","account":{{"lamports":42,"owner":"{STAKE_PROGRAM}","data":["!!!","base64"],"executable":false,"rentEpoch":0,"space":0}}}}],"id":1}}"#
    );

    server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::Regex("getProgramAccounts".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(body)
        .create();

    let result = helius.get_stake_accounts(Pubkey::new_unique()).await;
    assert!(result.is_err(), "an undecodable account should error, got {result:?}");
}
