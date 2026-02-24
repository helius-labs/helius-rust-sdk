use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_compression_signatures_for_token_owner_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetCompressionSignaturesForTokenOwnerResponse {
            context: ZkCompressedContext { slot: 100 },
            value: PaginatedSignatureInfoList {
                items: vec![SignatureInfo {
                    signature: "token_sig1".to_string(),
                    slot: 88,
                    block_time: 1700000000,
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

    let request = GetCompressionSignaturesForOwnerRequest {
        owner: "owner1".to_string(),
        ..Default::default()
    };

    let response = helius.get_compression_signatures_for_token_owner(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.value.items.len(), 1);
    assert_eq!(result.value.items[0].signature, "token_sig1");
}

#[tokio::test]
async fn test_get_compression_signatures_for_token_owner_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetCompressionSignaturesForOwnerRequest {
        owner: "owner1".to_string(),
        ..Default::default()
    };

    let response = helius.get_compression_signatures_for_token_owner(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
