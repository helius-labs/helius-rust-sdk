use helius::error::Result;
use helius::types::*;
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let token_program_id: String = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string();

    let config: GetProgramAccountsV2Config = GetProgramAccountsV2Config {
        encoding: Some(enums::Encoding::JsonParsed),
        filters: Some(vec![enums::GpaFilter::DataSize { data_size: 82 }]), // Mint accounts
        ..Default::default()
    };

    let all_accounts: Vec<GpaAccount> = helius.rpc().get_all_program_accounts(token_program_id, config).await?;

    println!("Total accounts fetched: {}", all_accounts.len());

    Ok(())
}
