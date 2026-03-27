use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::Server;
use reqwest::Client;

/// Creates a mockito server and a Helius client wired to it.
///
/// The Solana RPC client inside Helius also points at the mockito server,
/// so Solana JSON-RPC calls (e.g., `getMinimumBalanceForRentExemption`,
/// `getLatestBlockhash`) can be mocked with standard mockito matchers.
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

/// Mocks `getLatestBlockhash` Solana JSON-RPC call.
pub fn mock_latest_blockhash(server: &mut Server) {
    server
        .mock("POST", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Regex("getLatestBlockhash".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            r#"{"jsonrpc":"2.0","result":{"context":{"slot":1},"value":{"blockhash":"EkSnNWid2cvwEVnVx9aBqawnmiCNiDgp3gUdkDPTKN1N","lastValidBlockHeight":100}},"id":1}"#,
        )
        .create();
}

/// Mocks `getMinimumBalanceForRentExemption` Solana JSON-RPC call.
pub fn mock_rent_exempt(server: &mut Server) {
    server
        .mock("POST", mockito::Matcher::Any)
        .match_body(mockito::Matcher::Regex("getMinimumBalanceForRentExemption".to_string()))
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"jsonrpc":"2.0","result":2282880,"id":1}"#)
        .create();
}
