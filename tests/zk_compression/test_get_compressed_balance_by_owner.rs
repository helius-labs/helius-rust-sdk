use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_compressed_balance_by_owner_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetCompressedBalanceResponse {
            context: ZkCompressedContext { slot: 100 },
            value: 12000,
        },
        id: "1".to_string(),
    };

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let request = GetCompressedBalanceByOwnerRequest {
        owner: "11111113R2cuenjG5nFubqX9Wzuukdin2YfGQVzu5".to_string(),
    };

    let response = helius.get_compressed_balance_by_owner(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.value, 12000);
}

#[tokio::test]
async fn test_get_compressed_balance_by_owner_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetCompressedBalanceByOwnerRequest {
        owner: "11111113R2cuenjG5nFubqX9Wzuukdin2YfGQVzu5".to_string(),
    };

    let response = helius.get_compressed_balance_by_owner(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
