use helius::error::Result;
use helius::types::{
    Cluster, Encoding, GetHistoricalAccountInfoConfig, GetHistoricalTokenAccountsByOwnerConfig, HistoricalAccountQuery,
    TokenAccountsOwnerFilter, HISTORICAL_ACCOUNT_COVERAGE_START_SLOT,
};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = "your_api_key";
    let helius = Helius::new(api_key, Cluster::MainnetBeta)?;

    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    let wallet = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY";

    // What one transaction changed. getTransactionsForAddress returns each transaction's slot and index.
    let (slot, transaction_index) = (445_000_000, 10);
    let config = GetHistoricalAccountInfoConfig {
        encoding: Some(Encoding::JsonParsed),
        ..Default::default()
    };
    for query in [
        HistoricalAccountQuery::before_transaction(slot, transaction_index),
        HistoricalAccountQuery::after_transaction(slot, transaction_index),
    ] {
        let response = helius
            .rpc()
            .get_historical_account_info(usdc_mint.to_string(), query, Some(config.clone()))
            .await?;
        println!("{:?}: {:?}", query, response.value.map(|account| account.data));
    }

    // The account's five most recent states as of a slot, newest first
    let mut query = HistoricalAccountQuery::at_slot(slot);
    for _ in 0..5 {
        let response = helius
            .rpc()
            .get_historical_account_info(usdc_mint.to_string(), query, None)
            .await?;
        let Some(account) = response.value else {
            break;
        };
        let written_at = account.last_updated_at;
        println!("{} lamports, written at {:?}", account.lamports, written_at);
        if written_at.slot <= HISTORICAL_ACCOUNT_COVERAGE_START_SLOT {
            break;
        }
        query = HistoricalAccountQuery::before(&written_at);
    }

    // The wallet's USDC token accounts at the slot
    let token_accounts = helius
        .rpc()
        .get_all_historical_token_accounts_by_owner(
            wallet.to_string(),
            HistoricalAccountQuery::at_slot(slot),
            GetHistoricalTokenAccountsByOwnerConfig {
                filter: Some(TokenAccountsOwnerFilter::Mint {
                    mint: usdc_mint.to_string(),
                }),
                encoding: Some(Encoding::JsonParsed),
                ..Default::default()
            },
        )
        .await?;
    for token_account in &token_accounts {
        println!("{}: {}", token_account.pubkey, token_account.account.data);
    }

    Ok(())
}
