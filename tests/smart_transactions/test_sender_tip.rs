use helius::error::HeliusError;
use helius::optimized_transaction::{DEFAULT_MAX_TIP_LAMPORTS, MIN_TIP_LAMPORTS_MAX, MIN_TIP_LAMPORTS_SWQOS};
use helius::types::{SenderSendOptions, SmartTransactionConfig, Timeout};

use std::sync::Arc;

use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_system_interface::instruction as system_instruction;

use super::helpers::setup_mock;

/// A ceiling below the tier minimum is unsatisfiable: no tip is both at least the minimum and at
/// most the ceiling. It fails fast rather than silently paying above the stated ceiling, and it
/// fails *before* the tip-floor feed is contacted, so a misconfiguration costs no network call.
#[tokio::test(flavor = "multi_thread")]
async fn test_ceiling_below_tier_minimum_is_rejected() {
    let (_server, helius) = setup_mock().await;

    let result = helius
        .determine_tip_lamports_with_cap(false, MIN_TIP_LAMPORTS_MAX - 1)
        .await;

    let err = result.expect_err("a ceiling below the Sender Max minimum should be rejected");
    assert!(
        matches!(err, HeliusError::InvalidInput(_)),
        "expected InvalidInput, got {err:?}"
    );
    assert!(
        err.to_string().contains("max_tip_lamports"),
        "the error should name the offending option, got {err}"
    );
    assert!(
        err.to_string().contains("Sender Max"),
        "the error should name the tier whose minimum was violated, got {err}"
    );
}

/// The SWQOS tier has its own, much lower minimum, and the same rule applies against it.
#[tokio::test(flavor = "multi_thread")]
async fn test_ceiling_below_swqos_minimum_is_rejected() {
    let (_server, helius) = setup_mock().await;

    let result = helius
        .determine_tip_lamports_with_cap(true, MIN_TIP_LAMPORTS_SWQOS - 1)
        .await;

    let err = result.expect_err("a ceiling below the SWQOS minimum should be rejected");
    assert!(
        matches!(err, HeliusError::InvalidInput(_)),
        "expected InvalidInput, got {err:?}"
    );
    assert!(
        err.to_string().contains("SWQOS-only"),
        "the error should name the SWQOS tier, not Sender Max, got {err}"
    );

    // The bound is per-tier: a ceiling that clears the SWQOS minimum still fails on Sender Max.
    // Asserted through the rejection rather than the accepting path, because a passing validation
    // continues on to the live tip-floor feed and this suite does not make network calls.
    let on_sender_max = helius
        .determine_tip_lamports_with_cap(false, MIN_TIP_LAMPORTS_SWQOS)
        .await;
    assert!(
        matches!(on_sender_max, Err(HeliusError::InvalidInput(_))),
        "the SWQOS minimum is far below the Sender Max minimum and must be rejected there, got {on_sender_max:?}"
    );
}

/// Callers who never touch the new option still get a bounded tip: this is the safe-by-default
/// half of the fix, and the reason `determine_tip_lamports` keeps its signature.
#[test]
fn test_default_options_carry_a_tip_ceiling() {
    let options = SenderSendOptions::default();

    assert_eq!(
        options.max_tip_lamports, DEFAULT_MAX_TIP_LAMPORTS,
        "default options must carry the default ceiling"
    );
    assert!(
        options.max_tip_lamports >= MIN_TIP_LAMPORTS_MAX,
        "the default ceiling must be satisfiable on the default (Sender Max) tier"
    );
}

/// The builder sets the ceiling and leaves the rest of the defaults alone.
#[test]
fn test_builder_sets_the_tip_ceiling() {
    let options = SenderSendOptions::new()
        .with_swqos_only(true)
        .with_max_tip_lamports(50_000);

    assert_eq!(options.max_tip_lamports, 50_000);
    assert!(options.swqos_only);
    assert_eq!(
        options.poll_timeout_ms,
        SenderSendOptions::default().poll_timeout_ms,
        "unrelated defaults should be untouched"
    );
}

/// The ceiling has to actually reach the clamp. `send_smart_transaction_with_sender` reads it off
/// `SenderSendOptions`, so an unsatisfiable ceiling must surface from the public send path — which
/// it can only do if the option is threaded through rather than dropped.
///
/// This also pins the ordering: validation happens before the tip-floor feed is contacted and
/// before anything is signed or submitted, so a misconfiguration costs no network call and cannot
/// half-send a transaction.
#[tokio::test(flavor = "multi_thread")]
async fn test_send_path_threads_the_ceiling_through() {
    let (_server, helius) = setup_mock().await;

    let payer = Keypair::new();
    let instructions = vec![system_instruction::transfer(&payer.pubkey(), &Pubkey::new_unique(), 1)];
    let config = SmartTransactionConfig::new(
        instructions,
        vec![Arc::new(payer) as Arc<dyn Signer>],
        Timeout::default(),
    );

    let options = SenderSendOptions::new().with_max_tip_lamports(MIN_TIP_LAMPORTS_MAX - 1);

    let result = helius.send_smart_transaction_with_sender(config, options).await;

    let err = result.expect_err("an unsatisfiable ceiling should fail the send");
    assert!(
        matches!(err, HeliusError::InvalidInput(_)),
        "expected InvalidInput, got {err:?}"
    );
    assert!(
        err.to_string().contains("max_tip_lamports"),
        "the ceiling from SenderSendOptions should be the one that was validated, got {err}"
    );
}
