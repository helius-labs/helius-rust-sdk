use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_compressed_accounts_by_owner_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetCompressedAccountsByOwnerResponse {
            context: ZkCompressedContext { slot: 100 },
            value: PaginatedAccountList {
                items: vec![CompressedAccount {
                    address: None,
                    data: None,
                    hash: "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
                    lamports: 500,
                    leaf_index: 0,
                    owner: "11111114d3RrygbPdAtMuFnDmzsN8T5fYKVQ7FVr7".to_string(),
                    seq: Some(1),
                    slot_created: 50,
                    tree: "11111117qkFjr4u54stuNNUR8fRF8dNhaP35yvANs".to_string(),
                }],
                cursor: Some("next_cursor".to_string()),
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

    let request = GetCompressedAccountsByOwnerRequest {
        owner: "11111114d3RrygbPdAtMuFnDmzsN8T5fYKVQ7FVr7".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_accounts_by_owner(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.value.items.len(), 1);
    assert_eq!(result.value.cursor, Some("next_cursor".to_string()));
}

/// Accounts in a V2 (batched) state tree come back with `"seq": null`; one of them must not
/// make the whole page fail to decode.
#[tokio::test]
async fn test_get_compressed_accounts_by_owner_accepts_null_seq() {
    let (mut server, helius) = setup_mock().await;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "1",
        "result": {
            "context": { "slot": 100 },
            "value": {
                "items": [
                    {
                        "hash": "2veKUPKieajFG7yzPNGARHq7jYS1FiXcXG8P9Txhvhcn",
                        "address": null,
                        "data": null,
                        "owner": "J6ULJViDpQTRntYTmfkpUZZ71KTVkZNJFSDcQhFCo5Ef",
                        "lamports": 999998000,
                        "tree": "smt1NamzXdq4AMqS2fS2F1i5KTYPZRhoHgWx38d8WsT",
                        "leafIndex": 3,
                        "seq": 4,
                        "slotCreated": 0
                    },
                    {
                        "hash": "JReC6h68m3EdCKP7S35e7BE4pBPwQz1HfjUkboeQy9r",
                        "address": null,
                        "data": null,
                        "owner": "J6ULJViDpQTRntYTmfkpUZZ71KTVkZNJFSDcQhFCo5Ef",
                        "lamports": 1000,
                        "tree": "bmt1LryLZUMmF7ZtqESaw7wifBXLfXHQYoE4GAmrahU",
                        "leafIndex": 6,
                        "seq": null,
                        "slotCreated": 0
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

    let request = GetCompressedAccountsByOwnerRequest {
        owner: "J6ULJViDpQTRntYTmfkpUZZ71KTVkZNJFSDcQhFCo5Ef".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_accounts_by_owner(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let items = response.unwrap().value.items;
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].seq, Some(4));
    assert_eq!(items[1].seq, None);
}

#[tokio::test]
async fn test_get_compressed_accounts_by_owner_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetCompressedAccountsByOwnerRequest {
        owner: "11111114d3RrygbPdAtMuFnDmzsN8T5fYKVQ7FVr7".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_accounts_by_owner(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
