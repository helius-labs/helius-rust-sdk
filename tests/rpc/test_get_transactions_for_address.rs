use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::rpc_client::RpcClient;
use helius::types::inner::{TransactionDetails, TransactionEntry};
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

    match &result.data[0] {
        TransactionEntry::Signature(entry) => {
            assert_eq!(entry.slot, 1054);
            assert_eq!(
                entry.signature,
                "5h6xBEauJ3PK6SWCZ1PGjBvj8vDdWG3KpwATGy1ARAXFSDwt8GFXM7W5Ncn16wmqokgpiKRLuS83KUxyZyv2sUYv"
            );
            assert_eq!(entry.block_time, Some(1641038400));
            assert_eq!(entry.confirmation_status, Some("finalized".to_string()));
            assert!(entry.err.is_none());
            assert!(entry.memo.is_none());
        }
        TransactionEntry::Full(_) => panic!("Expected Signature variant, got Full"),
        TransactionEntry::Unknown(_) => panic!("Expected Signature variant, got Unknown"),
    }
}

#[tokio::test]
async fn test_get_transactions_for_address_full_success() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url = server.url();

    // Mock a full transaction response (matches getTransactionsForAddress full mode)
    let mock_data = json!([
        {
            "slot": 1054,
            "transactionIndex": 42,
            "transaction": {
                "signatures": [
                    "5h6xBEauJ3PK6SWCZ1PGjBvj8vDdWG3KpwATGy1ARAXFSDwt8GFXM7W5Ncn16wmqokgpiKRLuS83KUxyZyv2sUYv"
                ],
                "message": {
                    "accountKeys": [
                        "83astBRguLMdt2h5U1Tpdq5tjFoJ6noeGwaY3mDLVcri",
                        "11111111111111111111111111111111"
                    ],
                    "header": {
                        "numReadonlySignedAccounts": 0,
                        "numReadonlyUnsignedAccounts": 1,
                        "numRequiredSignatures": 1
                    },
                    "instructions": [
                        {
                            "accounts": [0, 1],
                            "data": "3Bxs4ThwQbE4vyj5",
                            "programIdIndex": 1
                        }
                    ],
                    "recentBlockhash": "GeyAMqaSPnLV7LPY8ByVNvZmhFBvAsGagsby35sBvSfN"
                }
            },
            "meta": {
                "err": null,
                "status": { "Ok": null },
                "fee": 5000,
                "preBalances": [1000000000, 0],
                "postBalances": [999995000, 0],
                "innerInstructions": [],
                "logMessages": ["Program 11111111111111111111111111111111 invoke [1]"],
                "preTokenBalances": [],
                "postTokenBalances": [],
                "rewards": []
            },
            "blockTime": 1641038400
        }
    ]);

    let mock_response = json!({
        "jsonrpc": "2.0",
        "id": "helius-rust-sdk",
        "result": {
            "data": mock_data,
            "paginationToken": null
        }
    });

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

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

    let options = GetTransactionsForAddressOptions {
        limit: Some(10),
        transaction_details: Some(TransactionDetails::Full),
        ..Default::default()
    };

    let response = helius
        .rpc()
        .get_transactions_for_address("SomeAddress".to_string(), options)
        .await;

    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.data.len(), 1);
    assert!(result.pagination_token.is_none());

    match &result.data[0] {
        TransactionEntry::Full(tx) => {
            assert_eq!(tx.slot, 1054);
            assert_eq!(tx.transaction_index, Some(42));
            assert_eq!(tx.block_time, Some(1641038400));
            assert!(tx.meta.is_some());
            let meta = tx.meta.as_ref().unwrap();
            assert!(meta.err.is_none());
        }
        TransactionEntry::Signature(_) => panic!("Expected Full variant, got Signature"),
        TransactionEntry::Unknown(_) => panic!("Expected Full variant, got Unknown"),
    }
}

#[tokio::test]
async fn test_get_transactions_for_address_unknown_fallback() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url = server.url();

    // Mock a response with an unrecognized shape (e.g., a hypothetical new API mode)
    let mock_data = json!([
        {
            "someNewField": "unexpected_data",
            "anotherField": 42
        }
    ]);

    let mock_response = json!({
        "jsonrpc": "2.0",
        "id": "helius-rust-sdk",
        "result": {
            "data": mock_data,
            "paginationToken": null
        }
    });

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

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

    let options = GetTransactionsForAddressOptions {
        limit: Some(10),
        ..Default::default()
    };

    let response = helius
        .rpc()
        .get_transactions_for_address("SomeAddress".to_string(), options)
        .await;

    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.data.len(), 1);

    match &result.data[0] {
        TransactionEntry::Unknown(val) => {
            assert_eq!(val["someNewField"], "unexpected_data");
            assert_eq!(val["anotherField"], 42);
        }
        TransactionEntry::Signature(_) => panic!("Expected Unknown variant, got Signature"),
        TransactionEntry::Full(_) => panic!("Expected Unknown variant, got Full"),
    }
}
