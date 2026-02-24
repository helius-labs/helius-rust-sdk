use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_compressed_account_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetCompressedAccountResponse {
            context: ZkCompressedContext { slot: 100 },
            value: Some(CompressedAccount {
                address: Some("11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3".to_string()),
                data: None,
                hash: "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
                lamports: 1000,
                leaf_index: 0,
                owner: "11111114d3RrygbPdAtMuFnDmzsN8T5fYKVQ7FVr7".to_string(),
                seq: 1,
                slot_created: 50,
                tree: "11111117qkFjr4u54stuNNUR8fRF8dNhaP35yvANs".to_string(),
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

    let request = GetCompressedAccountRequest {
        address: Some("11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3".to_string()),
        ..Default::default()
    };

    let response = helius.get_compressed_account(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.context.slot, 100);
    assert!(result.value.is_some());

    let account = result.value.unwrap();
    assert_eq!(account.lamports, 1000);
    assert_eq!(account.owner, "11111114d3RrygbPdAtMuFnDmzsN8T5fYKVQ7FVr7");
}

#[tokio::test]
async fn test_get_compressed_account_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetCompressedAccountRequest {
        hash: Some("11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string()),
        ..Default::default()
    };

    let response = helius.get_compressed_account(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
