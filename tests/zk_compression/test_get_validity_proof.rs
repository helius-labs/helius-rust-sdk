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
                    a: vec![1; 32],
                    b: vec![2; 64],
                    c: vec![3; 32],
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
    assert_eq!(result.value.compressed_proof.a, vec![1; 32]);
    assert_eq!(result.value.compressed_proof.b, vec![2; 64]);
    assert_eq!(result.value.compressed_proof.c, vec![3; 32]);
    assert_eq!(result.value.leaves.len(), 1);
    assert_eq!(result.value.merkle_trees.len(), 1);
}

/// The indexer serializes each proof component as a JSON array of bytes, not as a string.
#[tokio::test]
async fn test_get_validity_proof_decodes_byte_array_components() {
    let (mut server, helius) = setup_mock().await;

    let a: Vec<u8> = (1..=32).collect();
    let b: Vec<u8> = (1..=64).collect();
    let c: Vec<u8> = (33..=64).collect();
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "1",
        "result": {
            "value": {
                "compressedProof": { "a": a, "b": b, "c": c },
                "roots": ["jQKhAHpGbSTJLimk3bGSX7KLi7CsX6nkosS9YeCFfVs"],
                "rootIndices": [7],
                "leafIndices": [3],
                "leaves": ["JsiSYnfKzmaxKvU8vhqsM6PiuPEFVVVwAsNcPUtDo64"],
                "merkleTrees": ["5bdFnXU47QjzGpzHfXnxcEi5WXyxzEAZzd1vrE39bf1W"]
            },
            "context": { "slot": 100 }
        }
    });

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body.to_string())
        .create();

    let request = GetValidityProofRequest {
        hashes: Some(vec!["JsiSYnfKzmaxKvU8vhqsM6PiuPEFVVVwAsNcPUtDo64".to_string()]),
        ..Default::default()
    };

    let response = helius.get_validity_proof(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let proof = response.unwrap().value;
    assert_eq!(proof.compressed_proof.a, a);
    assert_eq!(proof.compressed_proof.b, b);
    assert_eq!(proof.compressed_proof.c, c);
    assert_eq!(proof.root_indices, vec![7]);
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
