use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_multiple_compressed_accounts_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetMultipleCompressedAccountsResponse {
            context: ZkCompressedContext { slot: 100 },
            value: AccountList {
                items: vec![
                    Some(CompressedAccount {
                        address: None,
                        data: None,
                        hash: "hash1".to_string(),
                        lamports: 1000,
                        leaf_index: 0,
                        owner: "owner1".to_string(),
                        seq: 1,
                        slot_created: 50,
                        tree: "tree1".to_string(),
                    }),
                    None,
                ],
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

    let request = GetMultipleCompressedAccountsRequest {
        addresses: Some(vec!["addr1".to_string(), "addr2".to_string()]),
        ..Default::default()
    };

    let response = helius.get_multiple_compressed_accounts(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.value.items.len(), 2);
    assert!(result.value.items[0].is_some());
    assert!(result.value.items[1].is_none());
    assert_eq!(result.value.items[0].as_ref().unwrap().lamports, 1000);
}

#[tokio::test]
async fn test_get_multiple_compressed_accounts_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetMultipleCompressedAccountsRequest {
        hashes: Some(vec!["hash1".to_string()]),
        ..Default::default()
    };

    let response = helius.get_multiple_compressed_accounts(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
