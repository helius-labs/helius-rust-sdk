use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_compressed_token_accounts_by_owner_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetCompressedTokenAccountsByOwnerResponse {
            context: ZkCompressedContext { slot: 100 },
            value: TokenAccountList {
                items: vec![],
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

    let request = GetCompressedTokenAccountsByOwnerRequest {
        owner: "owner1".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_token_accounts_by_owner(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert!(result.value.items.is_empty());
}

/// A token account in a V2 (batched) state tree comes back with `"seq": null`.
#[tokio::test]
async fn test_get_compressed_token_accounts_by_owner_accepts_null_seq() {
    let (mut server, helius) = setup_mock().await;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "1",
        "result": {
            "context": { "slot": 100 },
            "value": {
                "items": [
                    {
                        "account": {
                            "hash": "JReC6h68m3EdCKP7S35e7BE4pBPwQz1HfjUkboeQy9r",
                            "address": null,
                            "data": {
                                "discriminator": 2,
                                "data": "Fci4U9QOCVoeSJNu52uQTHTZkN+jGh5G5VqySLzUdo/IF0SuAAU/8W2JwQ9eoK7yn7i6O/dx7aXfmnaBmwIYnAUAAAAAAAAAAAAA",
                                "dataHash": "4C8Wy6GxRZhiUfR6aSDejhYhnyixhtsNUFbKdPCUcNoq"
                            },
                            "owner": "cTokenmWW8bLPjZEBAUgYy3zKxQZW6VKi7bqNFEVv3m",
                            "lamports": 0,
                            "tree": "bmt1LryLZUMmF7ZtqESaw7wifBXLfXHQYoE4GAmrahU",
                            "leafIndex": 6,
                            "seq": null,
                            "slotCreated": 0
                        },
                        "tokenData": {
                            "mint": "2U35cKS3Cj2xs5EBdByXYU7LaKAitqjSZc1Jnvu4iPf4",
                            "owner": "EU57rQxcmFhJ24ApVdUy3y4MxFXcTUN3uiVeWvGgtWXu",
                            "amount": 5,
                            "delegate": null,
                            "state": "initialized",
                            "tlv": null
                        }
                    }
                ],
                "cursor": null
            }
        }
    });

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body.to_string())
        .create();

    let request = GetCompressedTokenAccountsByOwnerRequest {
        owner: "EU57rQxcmFhJ24ApVdUy3y4MxFXcTUN3uiVeWvGgtWXu".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_token_accounts_by_owner(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let items = response.unwrap().value.items;
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].account.seq, None);
    assert_eq!(items[0].token_data.amount, 5);
}

#[tokio::test]
async fn test_get_compressed_token_accounts_by_owner_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetCompressedTokenAccountsByOwnerRequest {
        owner: "owner1".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_token_accounts_by_owner(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
