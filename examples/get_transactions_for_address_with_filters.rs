use helius::error::Result;
use helius::types::inner::{GetTransactionsFilters, SlotFilter, TransactionDetails, TransactionStatusFilter};
use helius::types::{Cluster, GetTransactionsForAddressOptions, SortOrder};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;
    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

    println!("=== Example 1: Recent successful transactions ===");
    let options = GetTransactionsForAddressOptions {
        limit: Some(10),
        transaction_details: Some(TransactionDetails::Full),
        sort_order: Some(SortOrder::Desc),
        filters: Some(GetTransactionsFilters {
            status: Some(TransactionStatusFilter::Succeeded),
            ..Default::default()
        }),
        ..Default::default()
    };

    match helius
        .rpc()
        .get_transactions_for_address(address.to_string(), options)
        .await
    {
        Ok(result) => {
            println!("Fetched {} successful transactions", result.data.len());
            println!("Pagination token: {:?}", result.pagination_token);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    println!();

    println!("=== Example 2: Transactions in slot range ===");
    let options = GetTransactionsForAddressOptions {
        limit: Some(5),
        transaction_details: Some(TransactionDetails::Signatures),
        filters: Some(GetTransactionsFilters {
            slot: Some(SlotFilter {
                gte: Some(250000000),
                lte: Some(260000000),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    match helius
        .rpc()
        .get_transactions_for_address(address.to_string(), options)
        .await
    {
        Ok(result) => {
            println!("Fetched {} transactions in slot range", result.data.len());
            for (i, tx) in result.data.iter().enumerate() {
                println!("  {}. {:?}", i + 1, tx);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    println!();

    println!("=== Example 3: Failed transactions ===");
    let options = GetTransactionsForAddressOptions {
        limit: Some(5),
        transaction_details: Some(TransactionDetails::Signatures),
        filters: Some(GetTransactionsFilters {
            status: Some(TransactionStatusFilter::Failed),
            ..Default::default()
        }),
        ..Default::default()
    };

    match helius
        .rpc()
        .get_transactions_for_address(address.to_string(), options)
        .await
    {
        Ok(result) => {
            println!("Fetched {} failed transactions", result.data.len());
        }
        Err(e) => println!("Error: {:?}", e),
    }

    println!();

    println!("=== Example 4: Include token account transactions ===");
    let options = GetTransactionsForAddressOptions {
        limit: Some(5),
        transaction_details: Some(TransactionDetails::Signatures),
        filters: Some(GetTransactionsFilters {
            include_token_accounts: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    };

    match helius
        .rpc()
        .get_transactions_for_address(address.to_string(), options)
        .await
    {
        Ok(result) => {
            println!("Fetched {:?} transactions (including token accounts)", result.data);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    println!();

    println!("=== Example 5: Pagination ===");
    let mut page = 1;
    let mut pagination_token: Option<String> = None;

    loop {
        let options = GetTransactionsForAddressOptions {
            limit: Some(3),
            transaction_details: Some(TransactionDetails::Signatures),
            pagination_token: pagination_token.clone(),
            ..Default::default()
        };

        match helius
            .rpc()
            .get_transactions_for_address(address.to_string(), options)
            .await
        {
            Ok(result) => {
                println!("Page {}: {} transactions", page, result.data.len());

                if let Some(token) = result.pagination_token {
                    pagination_token = Some(token);
                    page += 1;

                    // Stop after 3 pages as an example
                    if page > 3 {
                        println!("Stopping after 3 pages...");
                        break;
                    }
                } else {
                    println!("No more pages");
                    break;
                }
            }
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
