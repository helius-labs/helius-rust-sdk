use helius::types::*;

use super::helpers::setup_mock;

#[tokio::test]
async fn test_get_compressed_mint_token_holders_success() {
    let (mut server, helius) = setup_mock().await;

    let mock_response = ApiResponse {
        jsonrpc: "2.0".to_string(),
        result: GetCompressedMintTokenHoldersResponse {
            context: ZkCompressedContext { slot: 100 },
            value: OwnerBalanceList {
                items: vec![
                    OwnerBalance {
                        owner: "owner1".to_string(),
                        balance: 1000,
                    },
                    OwnerBalance {
                        owner: "owner2".to_string(),
                        balance: 2000,
                    },
                ],
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

    let request = GetCompressedMintTokenHoldersRequest {
        mint: "111111152P2r5yt6odmBLPsFCLBrFisJ3aS7LqLAT".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_mint_token_holders(request).await;
    assert!(response.is_ok(), "API call failed: {:?}", response.err());

    let result = response.unwrap();
    assert_eq!(result.value.items.len(), 2);
    assert_eq!(result.value.items[0].balance, 1000);
}

#[tokio::test]
async fn test_get_compressed_mint_token_holders_failure() {
    let (mut server, helius) = setup_mock().await;

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Internal Server Error"}"#)
        .create();

    let request = GetCompressedMintTokenHoldersRequest {
        mint: "111111152P2r5yt6odmBLPsFCLBrFisJ3aS7LqLAT".to_string(),
        ..Default::default()
    };

    let response = helius.get_compressed_mint_token_holders(request).await;
    assert!(response.is_err(), "Expected an error but got success");
}
