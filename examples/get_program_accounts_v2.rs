use helius::error::Result;
use helius::types::*;
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let token_program_id: String = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string();

    // Basic query with pagination
    let config1: GetProgramAccountsV2Config = GetProgramAccountsV2Config {
        encoding: Some(Encoding::JsonParsed),
        limit: Some(10),
        ..Default::default()
    };
    let first_page: GetProgramAccountsV2Response = helius
        .rpc()
        .get_program_accounts_v2(token_program_id.clone(), config1)
        .await?;

    println!("Fetched {} accounts", first_page.accounts.len());
    println!("Pagination key: {:?}", first_page.pagination_key);
    if !first_page.accounts.is_empty() {
        println!("First account: {}", first_page.accounts[0].pubkey);
    }

    // Continue pagination if more results exist
    if let Some(pagination_key) = first_page.pagination_key {
        let config2: GetProgramAccountsV2Config = GetProgramAccountsV2Config {
            encoding: Some(Encoding::JsonParsed),
            limit: Some(10),
            pagination_key: Some(pagination_key),
            ..Default::default()
        };
        let second_page: GetProgramAccountsV2Response = helius
            .rpc()
            .get_program_accounts_v2(token_program_id.clone(), config2)
            .await?;

        println!("Fetched {} more accounts", second_page.accounts.len());
        println!("Next pagination key: {:?}", second_page.pagination_key);
    }

    // Using filters
    let config3: GetProgramAccountsV2Config = GetProgramAccountsV2Config {
        encoding: Some(Encoding::JsonParsed),
        limit: Some(5),
        filters: Some(vec![GpaFilter::DataSize { data_size: 165 }]),
        ..Default::default()
    };
    let token_accounts: GetProgramAccountsV2Response = helius
        .rpc()
        .get_program_accounts_v2(token_program_id.clone(), config3)
        .await?;

    println!("Found {} token accounts", token_accounts.accounts.len());
    for account in &token_accounts.accounts {
        if let Some(parsed) = &account.account.data.as_object() {
            if let Some(info) = parsed.get("parsed").and_then(|p| p.get("info")) {
                let mint = info.get("mint").and_then(|m| m.as_str()).unwrap_or("Unknown");
                println!(
                    "  Token account {}... with mint: {}...",
                    &account.pubkey[0..8],
                    &mint[0..8]
                );
            }
        }
    }

    // Incremental updates (change to a more recent slot)
    let recent_slot: u64 = 363340000;
    let config4: GetProgramAccountsV2Config = GetProgramAccountsV2Config {
        encoding: Some(Encoding::JsonParsed),
        limit: Some(5),
        changed_since_slot: Some(recent_slot),
        ..Default::default()
    };
    let changed_accounts: GetProgramAccountsV2Response = helius
        .rpc()
        .get_program_accounts_v2(token_program_id.clone(), config4)
        .await?;

    println!(
        "Found {} accounts changed since slot {}",
        changed_accounts.accounts.len(),
        recent_slot
    );

    // Using memcmp filter
    let specific_owner: String = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY".to_string();
    let config6: GetProgramAccountsV2Config = GetProgramAccountsV2Config {
        encoding: Some(Encoding::Base64),
        limit: Some(5),
        filters: Some(vec![
            GpaFilter::DataSize { data_size: 165 },
            GpaFilter::Memcmp {
                memcmp: GpaMemcmp {
                    offset: 32,
                    bytes: specific_owner.clone(),
                },
            },
        ]),
        ..Default::default()
    };
    let memcmp_accounts: GetProgramAccountsV2Response = helius
        .rpc()
        .get_program_accounts_v2(token_program_id.clone(), config6)
        .await?;

    println!(
        "Found {} accounts owned by {}...",
        memcmp_accounts.accounts.len(),
        &specific_owner[0..8]
    );

    Ok(())
}
