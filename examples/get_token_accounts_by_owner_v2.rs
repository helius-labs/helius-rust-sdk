use helius::error::Result;
use helius::types::*;
use helius::Helius;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let owner_address = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY".to_string();

    // Get all SPL token accounts
    let filter1: TokenAccountsOwnerFilter = TokenAccountsOwnerFilter::Program {
        program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
    };
    let config_t1: GetTokenAccountsByOwnerV2Config = GetTokenAccountsByOwnerV2Config {
        encoding: Some(Encoding::JsonParsed),
        limit: Some(10),
        ..Default::default()
    };
    let spl_token_accounts: GetTokenAccountsByOwnerV2Response = helius
        .rpc()
        .get_token_accounts_by_owner_v2(owner_address.clone(), filter1, config_t1)
        .await?;

    println!("Found {} token accounts", spl_token_accounts.value.accounts.len());
    if let Some(context) = spl_token_accounts.context {
        println!("Context slot: {}", context.slot);
        println!("API version: {:?}", context.api_version);
    }

    for account in &spl_token_accounts.value.accounts {
        if let Some(parsed) = &account.account.data.as_object() {
            if let Some(info) = parsed.get("parsed").and_then(|p| p.get("info")) {
                let mint: &str = info.get("mint").and_then(|m| m.as_str()).unwrap_or("Unknown");
                let token_amount: &str = info
                    .get("tokenAmount")
                    .and_then(|ta| ta.get("uiAmountString"))
                    .and_then(|uas| uas.as_str())
                    .unwrap_or("0");

                println!("  Account {}...", &account.pubkey[0..8]);
                println!("    Mint: {}", mint);
                println!("    Balance: {}", token_amount);
            }
        }
    }

    // Continue pagination if needed
    if let Some(pagination_key) = spl_token_accounts.value.pagination_key {
        let filter2: TokenAccountsOwnerFilter = TokenAccountsOwnerFilter::Program {
            program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
        };
        let config_t2: GetTokenAccountsByOwnerV2Config = GetTokenAccountsByOwnerV2Config {
            encoding: Some(Encoding::JsonParsed),
            limit: Some(10),
            pagination_key: Some(pagination_key),
            ..Default::default()
        };
        let next_page: GetTokenAccountsByOwnerV2Response = helius
            .rpc()
            .get_token_accounts_by_owner_v2(owner_address.clone(), filter2, config_t2)
            .await?;

        println!("Fetched {} more accounts", next_page.value.accounts.len());
    }

    // Get accounts for a specific mint (e.g., USDC)
    let usdc_mint: String = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string();
    let filter3: TokenAccountsOwnerFilter = TokenAccountsOwnerFilter::Mint {
        mint: usdc_mint.clone(),
    };
    let config_t3: GetTokenAccountsByOwnerV2Config = GetTokenAccountsByOwnerV2Config {
        encoding: Some(Encoding::JsonParsed),
        ..Default::default()
    };
    let usdc_accounts: GetTokenAccountsByOwnerV2Response = helius
        .rpc()
        .get_token_accounts_by_owner_v2(owner_address.clone(), filter3, config_t3)
        .await?;

    if !usdc_accounts.value.accounts.is_empty() {
        println!("Found {} USDC account(s)", usdc_accounts.value.accounts.len());

        let account = &usdc_accounts.value.accounts[0];

        if let Some(parsed) = &account.account.data.as_object() {
            if let Some(info) = parsed.get("parsed").and_then(|p| p.get("info")) {
                let token_amount = info
                    .get("tokenAmount")
                    .and_then(|ta| ta.get("uiAmountString"))
                    .and_then(|uas| uas.as_str())
                    .unwrap_or("0");
                println!("  USDC Balance: {}", token_amount);
            }
        }
    } else {
        println!("No USDC accounts found for this wallet");
    }

    // Get Token-2022 accounts
    let filter4: TokenAccountsOwnerFilter = TokenAccountsOwnerFilter::Program {
        program_id: "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".to_string(),
    };
    let config_t4: GetTokenAccountsByOwnerV2Config = GetTokenAccountsByOwnerV2Config {
        encoding: Some(Encoding::JsonParsed),
        limit: Some(5),
        ..Default::default()
    };
    let token2022_accounts: GetTokenAccountsByOwnerV2Response = helius
        .rpc()
        .get_token_accounts_by_owner_v2(owner_address.clone(), filter4, config_t4)
        .await?;

    println!("Found {} Token-2022 accounts", token2022_accounts.value.accounts.len());

    // Recently changed accounts (change to a more recent slot)
    let recent_slot_t: u64 = 363340000;
    let filter5: TokenAccountsOwnerFilter = TokenAccountsOwnerFilter::Program {
        program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
    };
    let config_t5: GetTokenAccountsByOwnerV2Config = GetTokenAccountsByOwnerV2Config {
        encoding: Some(Encoding::JsonParsed),
        limit: Some(5),
        changed_since_slot: Some(recent_slot_t),
        ..Default::default()
    };
    let recently_changed: GetTokenAccountsByOwnerV2Response = helius
        .rpc()
        .get_token_accounts_by_owner_v2(owner_address.clone(), filter5, config_t5)
        .await?;

    println!(
        "Found {} accounts changed since slot {}",
        recently_changed.value.accounts.len(),
        recent_slot_t
    );

    Ok(())
}
