use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_transaction_with_compression_info_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetTransactionWithCompressionInfoResponse {
            compression_info: Some(CompressionInfo {
                closed_accounts: vec![],
                opened_accounts: vec![AccountWithOptionalTokenData {
                    account: CompressedAccount {
                        address: None,
                        data: None,
                        hash: "hash1".to_string(),
                        lamports: 0,
                        leaf_index: 0,
                        owner: "owner1".to_string(),
                        seq: 1,
                        slot_created: 50,
                        tree: "tree1".to_string(),
                    },
                    optional_token_data: Some(TokenData {
                        mint: "mint1".to_string(),
                        owner: "owner1".to_string(),
                        amount: 500,
                        state: AccountState::Initialized,
                        delegate: None,
                        tlv: None,
                    }),
                }],
            }),
            transaction: Some(serde_json::json!({"slot": 100})),
        },
        id: "1".to_string(),
    };

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let request = GetTransactionWithCompressionInfoRequest {
        signature: "5J8H5sTvEhnGcB4R8K1n7mfoiWUD9RzPVGES7e3WxC7c".to_string(),
    };

    let response = helius.get_transaction_with_compression_info(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert!(result.compression_info.is_some());
    let info = result.compression_info.unwrap();
    assert!(info.closed_accounts.is_empty());
    assert_eq!(info.opened_accounts.len(), 1);
    assert_eq!(info.opened_accounts[0].account.hash, "hash1");
    assert!(info.opened_accounts[0].optional_token_data.is_some());
    assert!(result.transaction.is_some());
}

#[tokio::test]
async fn test_get_transaction_with_compression_info_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetTransactionWithCompressionInfoRequest {
        signature: "5J8H5sTvEhnGcB4R8K1n7mfoiWUD9RzPVGES7e3WxC7c".to_string(),
    };

    let response = helius.get_transaction_with_compression_info(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
