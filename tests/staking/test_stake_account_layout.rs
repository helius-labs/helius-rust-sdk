use super::helpers::setup_mock;
use mockito::Matcher;
use solana_sdk::pubkey::Pubkey;

const STAKE_PROGRAM: &str = "Stake11111111111111111111111111111111111111";

/// `get_stake_accounts` must filter on the staker authority (offset 12), not the withdrawer
/// (offset 44). Asserting on the outgoing request body is what actually catches a regression:
/// a wrong offset still returns a well-formed response, just for the wrong set of accounts.
#[tokio::test(flavor = "multi_thread")]
async fn test_get_stake_accounts_filters_on_staker_offset() {
    let (mut server, helius) = setup_mock().await;
    let wallet = Pubkey::new_unique();

    let mock = server
        .mock("POST", Matcher::Any)
        .match_body(Matcher::AllOf(vec![
            Matcher::Regex("getProgramAccounts".to_string()),
            // Literal 12, not the constant: deriving the expectation from the value under
            // test would make this pass no matter what the constant said.
            Matcher::Regex(r#""offset"\s*:\s*12"#.to_string()),
            Matcher::Regex(format!(r#""bytes"\s*:\s*"{}""#, wallet)),
        ]))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(format!(
            r#"{{"jsonrpc":"2.0","result":[{{"pubkey":"{STAKE_PROGRAM}","account":{{"lamports":1,"owner":"{STAKE_PROGRAM}","data":["","base64"],"executable":false,"rentEpoch":0,"space":0}}}}],"id":1}}"#
        ))
        .create();

    helius
        .get_stake_accounts(wallet)
        .await
        .expect("get_stake_accounts should succeed");

    // The mock only matches if the memcmp offset in the request body is the staker offset.
    mock.assert();
}
