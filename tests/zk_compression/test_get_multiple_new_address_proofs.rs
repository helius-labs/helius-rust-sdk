use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_multiple_new_address_proofs_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetMultipleNewAddressProofsResponse {
            context: ZkCompressedContext { slot: 100 },
            value: vec![MerkleContextWithNewAddressProof {
                address: "addr1".to_string(),
                higher_range_address: "higher1".to_string(),
                low_element_leaf_index: 3,
                lower_range_address: "lower1".to_string(),
                merkle_tree: "tree1".to_string(),
                next_index: 10,
                proof: vec!["proof1".to_string()],
                root: "root1".to_string(),
                root_seq: 5,
            }],
        },
        id: "1".to_string(),
    };

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let addresses = vec!["addr1".to_string()];

    let response = helius.get_multiple_new_address_proofs(addresses).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.value.len(), 1);
    assert_eq!(result.value[0].address, "addr1");
    assert_eq!(result.value[0].merkle_tree, "tree1");
    assert_eq!(result.value[0].next_index, 10);
}

#[tokio::test]
async fn test_get_multiple_new_address_proofs_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let addresses = vec!["addr1".to_string()];

    let response = helius.get_multiple_new_address_proofs(addresses).await;
    assert!(response.is_err(), "Expected an error but got success");
}
