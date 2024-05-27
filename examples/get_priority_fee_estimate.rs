use helius::error::Result;
use helius::types::*;
use helius::Helius;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let request: GetPriorityFeeEstimateRequest = GetPriorityFeeEstimateRequest {
        transaction: None,
        account_keys: Some(vec!["JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string()]),
        options: Some(GetPriorityFeeEstimateOptions {
            priority_level: Some(PriorityLevel::High),
            include_all_priority_fee_levels: None,
            transaction_encoding: None,
            lookback_slots: None,
        }),
    };

    let response: Result<GetPriorityFeeEstimateResponse> = helius.rpc().get_priority_fee_estimate(request).await;
    println!("Assets: {:?}", response);

    Ok(())
}
