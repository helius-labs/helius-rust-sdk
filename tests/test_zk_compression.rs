use helius::types::{
    Asset, CompressedAccountData, CreatedAtFilter, GetCompressedAccountRequest, GetCompressedAccountResponse,
    GetCompressedAccountsResponse, SearchAssets, SystemInfo,
};
use helius::{Helius, HeliusFactory};
use serde_json::json;

/// Test deserialization of GetCompressedAccountResponse from JSON
/// This verifies our camelCase and Option<T> logic is correct
#[test]
fn test_compressed_account_response_deserialization() {
    let json_response = json!({
        "address": "CKfatsPMUf8SkiURsDXs7eK6GWb4Jsd6UDbs7twMCWxo",
        "data": {
            "discriminator": [1, 2, 3, 4],
            "data": [10, 20, 30, 40, 50],
            "dataHash": "ABC123DEF456"
        },
        "hash": "hash123",
        "tree": "tree456",
        "leafIndex": 42,
        "seq": 100,
        "slotCreated": 123456789
    });

    let result: Result<GetCompressedAccountResponse, _> = serde_json::from_value(json_response);
    assert!(result.is_ok(), "Failed to deserialize: {:?}", result.err());

    let response = result.unwrap();
    assert_eq!(
        response.address,
        Some("CKfatsPMUf8SkiURsDXs7eK6GWb4Jsd6UDbs7twMCWxo".to_string())
    );
    assert_eq!(response.hash, Some("hash123".to_string()));
    assert_eq!(response.tree, Some("tree456".to_string()));
    assert_eq!(response.leaf_index, Some(42));
    assert_eq!(response.seq, Some(100));
    assert_eq!(response.slot_created, Some(123456789));

    // Verify nested data structure
    assert!(response.data.is_some());
    let data = response.data.unwrap();
    assert_eq!(data.discriminator, Some(vec![1, 2, 3, 4]));
    assert_eq!(data.data, Some(vec![10, 20, 30, 40, 50]));
    assert_eq!(data.data_hash, Some("ABC123DEF456".to_string()));
}

/// Test deserialization with missing optional fields
#[test]
fn test_compressed_account_response_partial_deserialization() {
    let json_response = json!({
        "address": "CKfatsPMUf8SkiURsDXs7eK6GWb4Jsd6UDbs7twMCWxo",
        "hash": "hash123"
    });

    let result: Result<GetCompressedAccountResponse, _> = serde_json::from_value(json_response);
    assert!(result.is_ok(), "Failed to deserialize partial response");

    let response = result.unwrap();
    assert_eq!(
        response.address,
        Some("CKfatsPMUf8SkiURsDXs7eK6GWb4Jsd6UDbs7twMCWxo".to_string())
    );
    assert_eq!(response.hash, Some("hash123".to_string()));
    assert!(response.data.is_none());
    assert!(response.tree.is_none());
    assert!(response.leaf_index.is_none());
}

/// Test GetCompressedAccountsResponse with pagination
#[test]
fn test_compressed_accounts_response_with_cursor() {
    let json_response = json!({
        "items": [
            {
                "address": "Account1",
                "hash": "hash1"
            },
            {
                "address": "Account2",
                "hash": "hash2"
            }
        ],
        "cursor": "next_page_cursor_123"
    });

    let result: Result<GetCompressedAccountsResponse, _> = serde_json::from_value(json_response);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.items.len(), 2);
    assert_eq!(response.cursor, Some("next_page_cursor_123".to_string()));
    assert_eq!(response.items[0].address, Some("Account1".to_string()));
    assert_eq!(response.items[1].address, Some("Account2".to_string()));
}

/// Test GetCompressedAccountRequest serialization
#[test]
fn test_compressed_account_request_serialization() {
    let request = GetCompressedAccountRequest {
        address: "CKfatsPMUf8SkiURsDXs7eK6GWb4Jsd6UDbs7twMCWxo".to_string(),
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["address"], "CKfatsPMUf8SkiURsDXs7eK6GWb4Jsd6UDbs7twMCWxo");
}

/// Test that Asset struct can deserialize with new system field
#[test]
fn test_asset_with_system_info_deserialization() {
    let json_asset = json!({
        "interface": "V1_NFT",
        "id": "asset123",
        "ownership": {
            "frozen": false,
            "delegated": false,
            "delegate": null,
            "ownership_model": "single",
            "owner": "owner123"
        },
        "mutable": true,
        "burnt": false,
        "system": {
            "createdAt": 1703779200
        }
    });

    let result: Result<Asset, _> = serde_json::from_value(json_asset);
    assert!(
        result.is_ok(),
        "Failed to deserialize Asset with system field: {:?}",
        result.err()
    );

    let asset = result.unwrap();
    assert!(asset.system.is_some());
    let system_info = asset.system.unwrap();
    assert_eq!(system_info.created_at, Some(1703779200));
}

/// Test that SearchAssets can deserialize with created_at filter
#[test]
fn test_search_assets_with_created_at_filter() {
    let json_search = json!({
        "createdAt": {
            "after": 1703779200,
            "before": 1704038400
        }
    });

    let result: Result<SearchAssets, _> = serde_json::from_value(json_search);
    assert!(result.is_ok(), "Failed to deserialize SearchAssets with createdAt");

    let search = result.unwrap();
    assert!(search.created_at.is_some());
    let created_at = search.created_at.unwrap();
    assert_eq!(created_at.after, Some(1703779200));
    assert_eq!(created_at.before, Some(1704038400));
}

/// Test SystemInfo serialization/deserialization
#[test]
fn test_system_info_roundtrip() {
    let system_info = SystemInfo {
        created_at: Some(1703779200),
    };

    let json = serde_json::to_value(&system_info).unwrap();
    assert_eq!(json["createdAt"], 1703779200);

    let deserialized: SystemInfo = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.created_at, Some(1703779200));
}

/// Test CreatedAtFilter serialization/deserialization
#[test]
fn test_created_at_filter_roundtrip() {
    let filter = CreatedAtFilter {
        after: Some(1703779200),
        before: Some(1704038400),
    };

    let json = serde_json::to_value(&filter).unwrap();
    assert_eq!(json["after"], 1703779200);
    assert_eq!(json["before"], 1704038400);

    let deserialized: CreatedAtFilter = serde_json::from_value(json).unwrap();
    assert_eq!(deserialized.after, Some(1703779200));
    assert_eq!(deserialized.before, Some(1704038400));
}

/// Test that Helius client can be initialized and ZK Compression methods are callable
#[tokio::test]
async fn test_helius_client_zk_compression_methods_exist() {
    // Create a Helius client with a dummy API key using the factory
    let factory = HeliusFactory::new("dummy-api-key");
    let helius: Helius = factory.create(helius::types::Cluster::Devnet).unwrap();

    // Verify the client has the new methods by checking they compile and are callable
    // We can't actually call them without a real API key, but we can verify the signatures
    let request = GetCompressedAccountRequest {
        address: "test_address".to_string(),
    };

    // This will fail at runtime due to invalid API key, but proves the method exists
    let result = helius.rpc().get_compressed_account(request).await;
    assert!(result.is_err(), "Expected error with dummy API key");
}

/// Test CompressedAccountData deserialization with all fields
#[test]
fn test_compressed_account_data_full() {
    let json_data = json!({
        "discriminator": [1, 2, 3],
        "data": [10, 20, 30, 40],
        "dataHash": "hash123"
    });

    let result: Result<CompressedAccountData, _> = serde_json::from_value(json_data);
    assert!(result.is_ok());

    let data = result.unwrap();
    assert_eq!(data.discriminator, Some(vec![1, 2, 3]));
    assert_eq!(data.data, Some(vec![10, 20, 30, 40]));
    assert_eq!(data.data_hash, Some("hash123".to_string()));
}

/// Test CompressedAccountData with missing fields
#[test]
fn test_compressed_account_data_minimal() {
    let json_data = json!({});

    let result: Result<CompressedAccountData, _> = serde_json::from_value(json_data);
    assert!(result.is_ok());

    let data = result.unwrap();
    assert!(data.discriminator.is_none());
    assert!(data.data.is_none());
    assert!(data.data_hash.is_none());
}
