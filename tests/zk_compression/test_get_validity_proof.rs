use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_validity_proof_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetValidityProofResponse {
            context: ZkCompressedContext { slot: 100 },
            value: CompressedProofWithContext {
                compressed_proof: CompressedProof {
                    a: "proof_a".to_string(),
                    b: "proof_b".to_string(),
                    c: "proof_c".to_string(),
                },
                leaf_indices: vec![0],
                leaves: vec!["leaf1".to_string()],
                merkle_trees: vec!["tree1".to_string()],
                root_indices: vec![0],
                roots: vec!["root1".to_string()],
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

    let request = GetValidityProofRequest {
        hashes: Some(vec!["hash1".to_string()]),
        ..Default::default()
    };

    let response = helius.get_validity_proof(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.value.compressed_proof.a, "proof_a");
    assert_eq!(result.value.compressed_proof.b, "proof_b");
    assert_eq!(result.value.compressed_proof.c, "proof_c");
    assert_eq!(result.value.leaves.len(), 1);
    assert_eq!(result.value.merkle_trees.len(), 1);
}

#[tokio::test]
async fn test_get_validity_proof_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetValidityProofRequest {
        hashes: Some(vec!["hash1".to_string()]),
        ..Default::default()
    };

    let response = helius.get_validity_proof(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
