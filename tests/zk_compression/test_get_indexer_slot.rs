use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_indexer_slot_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: 250_000_000u64,
        id: "1".to_string(),
    };

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let response = helius.get_indexer_slot().await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result, 250_000_000);
}

#[tokio::test]
async fn test_get_indexer_slot_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let response = helius.get_indexer_slot().await;
    assert!(response.is_err(), "Expected an error but got success");
}
