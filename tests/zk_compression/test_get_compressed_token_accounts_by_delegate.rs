use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_compressed_token_accounts_by_delegate_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetCompressedTokenAccountsByDelegateResponse {
            context: ZkCompressedContext { slot: 100 },
            value: TokenAccountList {
                items: vec![CompressedTokenAccount {
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
                    token_data: TokenData {
                        mint: "mint1".to_string(),
                        owner: "owner1".to_string(),
                        amount: 100,
                        state: AccountState::Initialized,
                        delegate: Some("delegate1".to_string()),
                        tlv: None,
                    },
                }],
                cursor: None,
            },
        },
        id: "1".to_string(),
    };

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let request = GetCompressedTokenAccountsByDelegateRequest {
        delegate: "delegate1".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_token_accounts_by_delegate(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.value.items.len(), 1);
    assert_eq!(result.value.items[0].token_data.amount, 100);
}

#[tokio::test]
async fn test_get_compressed_token_accounts_by_delegate_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetCompressedTokenAccountsByDelegateRequest {
        delegate: "delegate1".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_token_accounts_by_delegate(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
