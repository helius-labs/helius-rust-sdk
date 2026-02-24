use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_multiple_compressed_account_proofs_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetMultipleCompressedAccountProofsResponse {
            context: ZkCompressedContext { slot: 100 },
            value: vec![MerkleProofWithContext {
                hash: "hash1".to_string(),
                leaf_index: 5,
                merkle_tree: "tree1".to_string(),
                proof: vec!["proof_element1".to_string(), "proof_element2".to_string()],
                root: "root1".to_string(),
                root_seq: 10,
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

    let hashes = vec!["hash1".to_string()];

    let response = helius.get_multiple_compressed_account_proofs(hashes).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.value.len(), 1);
    assert_eq!(result.value[0].hash, "hash1");
    assert_eq!(result.value[0].leaf_index, 5);
    assert_eq!(result.value[0].proof.len(), 2);
}

#[tokio::test]
async fn test_get_multiple_compressed_account_proofs_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let hashes = vec!["hash1".to_string()];

    let response = helius.get_multiple_compressed_account_proofs(hashes).await;
    assert!(response.is_err(), "Expected an error but got success");
}
