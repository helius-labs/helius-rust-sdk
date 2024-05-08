use helius::config::Config;
use helius::error::HeliusError;
use helius::rpc_client::RpcClient;
use helius::types::*;
use helius::Helius;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let config: Arc<Config> = Arc::new(Config::new(api_key, cluster)?);
    let client: reqwest::Client = reqwest::Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());

    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
    };

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

    let response: Result<GetPriorityFeeEstimateResponse, HeliusError> =
        helius.rpc().get_priority_fee_estimate(request).await;
    println!("Assets: {:?}", response);

    Ok(())
}
