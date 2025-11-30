use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::rpc_client::RpcClient;
use helius::types::inner::TransactionDetails;
use helius::types::*;

use mockito::{self, Server};
use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn test_get_transactions_for_address_success() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url = server.url();

    // Mock response data
    let mock_data = json!([
        {
            "signature": "5h6xBEauJ3PK6SWCZ1PGjBvj8vDdWG3KpwATGy1ARAXFSDwt8GFXM7W5Ncn16wmqokgpiKRLuS83KUxyZyv2sUYv",
            "slot": 1054,
            "err": null,
            "memo": null,
            "blockTime": 1641038400,
            "confirmationStatus": "finalized"
        }
    ]);

    let mock_response = json!({
        "jsonrpc": "2.0",
        "id": "helius-rust-sdk",
        "result": {
            "data": mock_data,
            "paginationToken": "1055:5"
        }
    });

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let config = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
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

    let options = GetTransactionsForAddressOptions {
        limit: Some(10),
        transaction_details: Some(TransactionDetails::Signatures),
        ..Default::default()
    };

    let response = helius
        .rpc()
        .get_transactions_for_address("SomeAddress".to_string(), options)
        .await;

    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.pagination_token, Some("1055:5".to_string()));

    let first_tx = &result.data[0];
    assert_eq!(first_tx["slot"], 1054);
}
