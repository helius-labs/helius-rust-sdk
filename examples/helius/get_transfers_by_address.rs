use helius::error::Result;
use helius::types::{
    Cluster, GetTransfersByAddressConfig, GetTransfersByAddressDirection, GetTransfersByAddressFilters,
    GetTransfersByAddressSolMode, SortOrder, TransferBlockTimeFilter,
};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = "your_api_key";
    let helius = Helius::new(api_key, Cluster::MainnetBeta)?;

    let address = "wallet_address";
    let config = GetTransfersByAddressConfig {
        limit: Some(10),
        direction: Some(GetTransfersByAddressDirection::Any),
        sol_mode: Some(GetTransfersByAddressSolMode::Merged),
        sort_order: Some(SortOrder::Desc),
        filters: Some(GetTransfersByAddressFilters {
            block_time: Some(TransferBlockTimeFilter {
                gte: Some(1_704_067_200),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = helius
        .rpc()
        .get_transfers_by_address(address.to_string(), Some(config))
        .await?;

    println!("Fetched {} transfers", result.data.len());
    for transfer in &result.data {
        println!(
            "{}: {} {} from {:?} to {:?}",
            transfer.signature, transfer.ui_amount, transfer.mint, transfer.from_user_account, transfer.to_user_account
        );
    }

    if let Some(token) = result.pagination_token {
        println!("Pagination token for next page: {}", token);
    }

    Ok(())
}
