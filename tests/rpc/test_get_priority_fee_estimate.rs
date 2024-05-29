use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::error::HeliusError;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::{self, Server};
use reqwest::Client;

#[tokio::test]
async fn test_get_nft_editions_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let mock_response: ApiResponse<GetPriorityFeeEstimateResponse> = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetPriorityFeeEstimateResponse {
            priority_fee_estimate: Some(100.0),
            priority_fee_levels: Some(MicroLamportPriorityFeeLevels {
                none: 0.0,
                low: 10.0,
                medium: 100.0,
                high: 500.0,
                very_high: 1000.0,
                unsafe_max: 10000.0,
            }),
        },
        id: "1".to_string(),
    };

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });

    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
    };

    let request: GetPriorityFeeEstimateRequest = GetPriorityFeeEstimateRequest {
        transaction: None,
        account_keys: Some(vec!["JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string()]),
        options: Some(GetPriorityFeeEstimateOptions {
            priority_level: Some(PriorityLevel::High),
            include_all_priority_fee_levels: None,
            transaction_encoding: None,
            lookback_slots: None,
            recommended: None,
            include_vote: None,
        }),
    };

    let response: Result<GetPriorityFeeEstimateResponse, HeliusError> =
        helius.rpc().get_priority_fee_estimate(request).await;
    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let fee_estimate = response.unwrap();
    assert!(fee_estimate.priority_fee_estimate.is_some(), "No fee estimate returned");
    assert_eq!(
        fee_estimate.priority_fee_estimate.unwrap(),
        100.0,
        "Fee estimate does not match expected value"
    );
}

#[tokio::test]
async fn test_get_nft_editions_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });

    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
    };

    let request: GetPriorityFeeEstimateRequest = GetPriorityFeeEstimateRequest {
        transaction: None,
        account_keys: Some(vec!["JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string()]),
        options: Some(GetPriorityFeeEstimateOptions {
            priority_level: Some(PriorityLevel::High),
            include_all_priority_fee_levels: None,
            transaction_encoding: None,
            lookback_slots: None,
            recommended: None,
            include_vote: None,
        }),
    };

    let response: Result<GetPriorityFeeEstimateResponse, HeliusError> =
        helius.rpc().get_priority_fee_estimate(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
