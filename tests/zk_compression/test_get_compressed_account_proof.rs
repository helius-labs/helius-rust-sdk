use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_compressed_account_proof_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetCompressedAccountProofResponse {
            context: ZkCompressedContext { slot: 100 },
            value: MerkleProofWithContext {
                hash: "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
                leaf_index: 5,
                merkle_tree: "11111117qkFjr4u54stuNNUR8fRF8dNhaP35yvANs".to_string(),
                proof: vec!["proof_hash_1".to_string(), "proof_hash_2".to_string()],
                root: "root_hash".to_string(),
                root_seq: 10,
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

    let request = GetCompressedAccountProofRequest {
        hash: "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
    };

    let response = helius.get_compressed_account_proof(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.value.leaf_index, 5);
    assert_eq!(result.value.proof.len(), 2);
    assert_eq!(result.value.root, "root_hash");
}

#[tokio::test]
async fn test_get_compressed_account_proof_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetCompressedAccountProofRequest {
        hash: "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
    };

    let response = helius.get_compressed_account_proof(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
