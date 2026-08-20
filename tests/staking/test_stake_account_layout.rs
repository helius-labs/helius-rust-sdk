use mockito::Matcher;
use solana_sdk::pubkey::Pubkey;
use solana_stake_interface::stake_flags::StakeFlags;
use solana_stake_interface::state::{Authorized, Delegation, Lockup, Meta, Stake, StakeStateV2};

use helius::staking::{STAKER_AUTHORITY_OFFSET, WITHDRAWER_AUTHORITY_OFFSET};

use super::helpers::setup_mock;

const STAKE_PROGRAM: &str = "Stake11111111111111111111111111111111111111";

/// Pins the byte offsets of both authorities within the bincode encoding of `StakeStateV2`.
///
/// `get_stake_accounts` filters with a `memcmp` at a hard-coded offset, so a layout change
/// upstream (or a transposed constant) would otherwise silently return the wrong accounts
/// rather than fail. Serializing a value whose staker and withdrawer differ is the only way
/// to tell the two apart.
#[test]
#[allow(deprecated)] // `Meta::rent_exempt_reserve` is deprecated but still occupies the bytes
fn stake_state_authority_offsets_are_stable() {
    let staker = Pubkey::new_from_array([0x11; 32]);
    let withdrawer = Pubkey::new_from_array([0x22; 32]);

    let meta = Meta {
        rent_exempt_reserve: 42,
        authorized: Authorized { staker, withdrawer },
        lockup: Lockup::default(),
    };
    let state = StakeStateV2::Stake(
        meta,
        Stake {
            delegation: Delegation::default(),
            credits_observed: 7,
        },
        StakeFlags::empty(),
    );

    let bytes = bincode::serialize(&state).expect("StakeStateV2 should serialize");

    // 4-byte bincode enum discriminant, then `Meta::rent_exempt_reserve` (u64).
    assert_eq!(&bytes[0..4], &[2, 0, 0, 0], "expected the `Stake` variant discriminant");
    assert_eq!(
        &bytes[4..12],
        &42u64.to_le_bytes(),
        "rent_exempt_reserve should directly follow the discriminant"
    );

    assert_eq!(
        &bytes[STAKER_AUTHORITY_OFFSET..STAKER_AUTHORITY_OFFSET + 32],
        staker.as_ref(),
        "STAKER_AUTHORITY_OFFSET does not point at Authorized::staker"
    );
    assert_eq!(
        &bytes[WITHDRAWER_AUTHORITY_OFFSET..WITHDRAWER_AUTHORITY_OFFSET + 32],
        withdrawer.as_ref(),
        "WITHDRAWER_AUTHORITY_OFFSET does not point at Authorized::withdrawer"
    );
}

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
