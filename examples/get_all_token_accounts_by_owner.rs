use helius::error::Result;
use helius::types::*;
use helius::Helius;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let owner_address: String = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY".to_string();

    let filter: TokenAccountsOwnerFilter = TokenAccountsOwnerFilter::Program {
        program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
    };
    let config: GetTokenAccountsByOwnerV2Config = GetTokenAccountsByOwnerV2Config {
        encoding: Some(Encoding::JsonParsed),
        ..Default::default()
    };

    let all_token_accounts: Vec<TokenAccountRecord> = helius
        .rpc()
        .get_all_token_accounts_by_owner(owner_address, filter, config)
        .await?;

    println!("Total token accounts: {}", all_token_accounts.len());

    // Group by mint
    let mut tokens_by_mint: HashMap<String, f64> = HashMap::new();
    for account in &all_token_accounts {
        if let Some(parsed) = &account.account.data.as_object() {
            if let Some(info) = parsed.get("parsed").and_then(|p| p.get("info")) {
                if let Some(mint) = info.get("mint").and_then(|m| m.as_str()) {
                    let balance_str: &str = info
                        .get("tokenAmount")
                        .and_then(|ta| ta.get("uiAmountString"))
                        .and_then(|uas| uas.as_str())
                        .unwrap_or("0");
                    let balance: f64 = balance_str.parse().unwrap_or(0.0);
                    *tokens_by_mint.entry(mint.to_string()).or_insert(0.0) += balance;
                }
            }
        }
    }

    println!("\nToken holdings summary:");
    for (mint, balance) in tokens_by_mint {
        if balance > 0.0 {
            println!("  {}...: {}", &mint[0..8], balance);
        }
    }

    Ok(())
}
