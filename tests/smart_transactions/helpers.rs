use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::Server;
use reqwest::Client;

/// Creates a mockito server and a Helius client wired to it.
/// Both the Helius HTTP handler and the Solana RPC client point at the mockito server.
pub async fn setup_mock() -> (Server, Helius) {
    let server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url = server.url();

    let config = Arc::new(Config {
        api_key: Some(ApiKey::new("fake_api_key").unwrap()),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
        custom_url: None,
    });

    let client = Client::new();
    let rpc_client = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };

    (server, helius)
}

/// Mocks `getLatestBlockhash` with commitment support (returns blockhash + lastValidBlockHeight).
pub fn mock_latest_blockhash(server: &mut Server) {
    server
        .mock("POST", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Regex("getLatestBlockhash".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":{"blockhash":"EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N","lastValidBlockHeight":200}},"id":1}"#,
        )
        .create();
}

/// Mocks `getPriorityFeeEstimate` JSON-RPC call (Helius HTTP endpoint).
pub fn mock_priority_fee_estimate(server: &mut Server) {
    server
        .mock("POST", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Regex("getPriorityFeeEstimate".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"jsonrpc":"2.0","result":{"priorityFeeEstimate":1000.0},"id":"1"}"#)
        .create();
}

/// Mocks `simulateTransaction` to return a successful result with compute units consumed.
pub fn mock_simulate_transaction(server: &mut Server, units_consumed: u64) {
    server
        .mock("POST", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Regex("simulateTransaction".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(format!(
            r#"{{"jsonrpc":"2.0","result":{{"context":{{"slot":1}},"value":{{"err":null,"logs":[],"accounts":null,"unitsConsumed":{},"returnData":null}}}},"id":1}}"#,
            units_consumed
        ))
        .create();
}

/// Mocks `simulateTransaction` returning a **failed** simulation (`err` set, `unitsConsumed: 0`),
/// mirroring what the RPC returns for e.g. an unsupported transaction version.
pub fn mock_simulate_transaction_failed(server: &mut Server) {
    server
        .mock("POST", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Regex("simulateTransaction".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":{"err":"UnsupportedVersion","logs":[],"accounts":null,"unitsConsumed":0,"returnData":null}},"id":1}"#,
        )
        .create();
}
