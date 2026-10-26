use futures_util::StreamExt;
use helius::error::{HeliusError, Result};
use helius::parsed_streams::{ParsedStreamsClient, ParsedStreamsOptions};
use helius::types::parsed_streams::{ParsedStreamDetails, ParsedTransactionFilter, ParsedTransactionSubscribeOptions};

const JUPITER: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

/// Stream decoded Jupiter swaps with Helius **Parsed Streams**.
///
/// Flow: `describe_program` to get the exact instruction names, build a filter with them, then
/// subscribe. The client reconnects and resubscribes on its own when the server closes the
/// connection (idle after 10 minutes without a match, deploys, network edge), and
/// `on_reconnect` reports the last slot it delivered so you can backfill the gap from RPC.
/// 1 credit per delivered event.
#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";

    let options = ParsedStreamsOptions::default().with_on_reconnect(|info| {
        eprintln!(
            "Reconnected after close {}; transactions after slot {:?} may have been missed",
            info.close_code, info.last_slot
        )
    });
    let streams = ParsedStreamsClient::with_options(api_key, options)?;

    // Check the names against the catalog first: a guessed name is valid but matches nothing
    let instruction_names: Vec<String> = vec!["route".to_string(), "shared_accounts_route".to_string()];
    let jupiter = streams.describe_program(JUPITER).await?;
    let unknown: Vec<&String> = instruction_names
        .iter()
        .filter(|name| !jupiter.instructions.contains(name))
        .collect();
    if !unknown.is_empty() {
        return Err(HeliusError::InvalidInput(format!(
            "Not in {}'s catalog: {unknown:?}",
            jupiter.name
        )));
    }

    let mut subscription = streams
        .parsed_transaction_subscribe(
            ParsedTransactionFilter {
                programs: Some(vec![JUPITER.to_string()]),
                instruction_names: Some(instruction_names),
                include_cpi: Some(false), // Only instructions the user signed for
                ..Default::default()
            },
            Some(ParsedTransactionSubscribeOptions {
                details: Some(ParsedStreamDetails::Full),
                ..Default::default()
            }),
        )
        .await?;

    let mut count = 0;
    while let Some(notification) = subscription.next().await {
        let notification = notification?;
        let value = notification.value;
        for &i in value.matched_indexes.as_deref().unwrap_or_default() {
            let instruction = &value.instructions[i];
            println!(
                "slot {} {} {:?} {}",
                notification.context.slot,
                value.transaction.signature,
                instruction.instruction_name,
                instruction
                    .summary
                    .as_ref()
                    .map(|summary| summary.description.clone())
                    .unwrap_or_else(|| format!("{:?}", instruction.decoded.as_ref().map(|decoded| &decoded.args))),
            );
        }

        count += 1;
        if count >= 10 {
            break;
        }
    }

    // Dropping the subscription unsubscribes; close ends the connection
    drop(subscription);
    streams.close();
    Ok(())
}
