use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::error::{HeliusError, Result};
use helius::rpc_client::RpcClient;
use helius::types::inner::RpcRequest;
use helius::types::*;

use mockito::{self, Server};
use reqwest::Client;
use serde_json::json;
use solana_commitment_config::CommitmentLevel;

fn test_helius(url: &str) -> Helius {
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

    Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    }
}

#[test]
fn test_get_transfers_by_address_request_serialization() {
    let config = GetTransfersByAddressConfig {
        with: Some("CounterpartyAddress".to_string()),
        direction: Some(GetTransfersByAddressDirection::Out),
        mint: Some("So11111111111111111111111111111111111111112".to_string()),
        sol_mode: Some(GetTransfersByAddressSolMode::Separate),
        filters: Some(GetTransfersByAddressFilters {
            amount: Some(TransferAmountFilter {
                gte: Some(1),
                ..Default::default()
            }),
            block_time: Some(TransferBlockTimeFilter {
                gt: Some(1_701_234_567),
                ..Default::default()
            }),
            slot: Some(TransferSlotFilter {
                lte: Some(250_000_000),
                ..Default::default()
            }),
        }),
        limit: Some(25),
        pagination_token: Some("250000000:3".to_string()),
        commitment: Some(CommitmentLevel::Confirmed),
        sort_order: Some(SortOrder::Asc),
    };

    let request = RpcRequest::new(
        "getTransfersByAddress".to_string(),
        GetTransfersByAddressRequest::new("SomeAddress".to_string(), Some(config)),
    );

    let value = serde_json::to_value(request).unwrap();
    assert_eq!(
        value,
        json!({
            "jsonrpc": "2.0",
            "id": "helius-rust-sdk",
            "method": "getTransfersByAddress",
            "params": [
                "SomeAddress",
                {
                    "with": "CounterpartyAddress",
                    "direction": "out",
                    "mint": "So11111111111111111111111111111111111111112",
                    "solMode": "separate",
                    "filters": {
                        "amount": { "gte": 1 },
                        "blockTime": { "gt": 1701234567 },
                        "slot": { "lte": 250000000 }
                    },
                    "limit": 25,
                    "paginationToken": "250000000:3",
                    "commitment": "confirmed",
                    "sortOrder": "asc"
                }
            ]
        })
    );
}

#[test]
fn test_get_transfers_by_address_omits_optional_config() {
    let request = RpcRequest::new(
        "getTransfersByAddress".to_string(),
        GetTransfersByAddressRequest::new("SomeAddress".to_string(), None),
    );

    let value = serde_json::to_value(request).unwrap();
    assert_eq!(
        value,
        json!({
            "jsonrpc": "2.0",
            "id": "helius-rust-sdk",
            "method": "getTransfersByAddress",
            "params": ["SomeAddress"]
        })
    );
}

#[tokio::test]
async fn test_get_transfers_by_address_success() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url = server.url();

    let mock_response = json!({
        "jsonrpc": "2.0",
        "id": "helius-rust-sdk",
        "result": {
            "data": [
                {
                    "signature": "5h6xBEauJ3PK6SWCZ1PGjBvj8vDdWG3KpwATGy1ARAXFSDwt8GFXM7W5Ncn16wmqokgpiKRLuS83KUxyZyv2sUYv",
                    "slot": 250000000,
                    "blockTime": 1701234567,
                    "type": "transfer",
                    "fromUserAccount": "FromWallet",
                    "toUserAccount": "ToWallet",
                    "fromTokenAccount": "FromTokenAccount",
                    "toTokenAccount": "ToTokenAccount",
                    "mint": "TokenMint",
                    "amount": "1000000",
                    "decimals": 6,
                    "uiAmount": "1",
                    "confirmationStatus": "finalized",
                    "transactionIdx": 2,
                    "instructionIdx": 3,
                    "innerInstructionIdx": 0
                },
                {
                    "signature": "4m7xBEauJ3PK6SWCZ1PGjBvj8vDdWG3KpwATGy1ARAXFSDwt8GFXM7W5Ncn16wmqokgpiKRLuS83KUxyZyv2sUYv",
                    "slot": 250000001,
                    "blockTime": 1701234600,
                    "type": "transfer",
                    "fromUserAccount": null,
                    "toUserAccount": "FeeCollector",
                    "mint": "So11111111111111111111111111111111111111111",
                    "amount": "5000",
                    "feeAmount": "5000",
                    "decimals": 9,
                    "uiAmount": "0.000005",
                    "feeUiAmount": "0.000005",
                    "confirmationStatus": "confirmed",
                    "transactionIdx": 4,
                    "instructionIdx": 1,
                    "innerInstructionIdx": 0
                }
            ],
            "paginationToken": "250000001:4"
        }
    });

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response.to_string())
        .create();

    let helius = test_helius(&url);
    let config = GetTransfersByAddressConfig {
        limit: Some(2),
        direction: Some(GetTransfersByAddressDirection::Any),
        ..Default::default()
    };

    let response = helius
        .rpc()
        .get_transfers_by_address("SomeAddress".to_string(), Some(config))
        .await;

    assert!(response.is_ok(), "API call failed with error: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.data.len(), 2);
    assert_eq!(result.pagination_token, Some("250000001:4".to_string()));

    let token_transfer = &result.data[0];
    assert_eq!(
        token_transfer.transfer_type,
        GetTransfersByAddressTransferType::Transfer
    );
    assert_eq!(token_transfer.amount, "1000000");
    assert_eq!(token_transfer.ui_amount, "1");
    assert_eq!(token_transfer.from_token_account, Some("FromTokenAccount".to_string()));
    assert_eq!(token_transfer.to_token_account, Some("ToTokenAccount".to_string()));

    let sol_transfer = &result.data[1];
    assert_eq!(sol_transfer.transfer_type, GetTransfersByAddressTransferType::Transfer);
    assert_eq!(sol_transfer.mint, "So11111111111111111111111111111111111111111");
    assert!(sol_transfer.from_token_account.is_none());
    assert!(sol_transfer.to_token_account.is_none());
    assert_eq!(sol_transfer.fee_amount, Some("5000".to_string()));
    assert_eq!(sol_transfer.confirmation_status, TransferConfirmationStatus::Confirmed);
}

#[tokio::test]
async fn test_get_transfers_by_address_failure() {
    let mut server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let helius = test_helius(&url);
    let response: Result<GetTransfersByAddressResponse> = helius
        .rpc()
        .get_transfers_by_address("SomeAddress".to_string(), None)
        .await;

    assert!(response.is_err(), "Expected an error but got success");
    assert!(
        matches!(response.unwrap_err(), HeliusError::InternalError { .. }),
        "Expected InternalError"
    );
}
