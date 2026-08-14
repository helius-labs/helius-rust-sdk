use helius::types::{
    GetAssetSignatures, GetAssetsByAuthority, GetAssetsByCreator, GetAssetsByGroup, GetAssetsByOwner, GetNftEditions,
    GetTokenAccounts, ProgramName,
};

/// Asserts that serializing `value` produces a JSON object with no `null`-valued members —
/// i.e. every unset optional field was omitted via `skip_serializing_if`.
fn assert_no_null_fields<T: serde::Serialize>(label: &str, value: &T) {
    let json = serde_json::to_value(value).unwrap();
    let obj = json
        .as_object()
        .unwrap_or_else(|| panic!("{label} did not serialize to an object"));
    let nulls: Vec<&String> = obj.iter().filter(|(_, v)| v.is_null()).map(|(k, _)| k).collect();
    assert!(
        nulls.is_empty(),
        "{label} serialized null fields {nulls:?} instead of omitting them"
    );
}

/// The DAS API emits `"UNKNOWN"` for unrecognized swap programs. It must deserialize into the
/// typed `ProgramName::Unknown` variant (not fall through to `Other`), and round-trip back to
/// `"UNKNOWN"`. Guards the `Unkown` -> `Unknown` typo fix: previously the variant serialized to
/// `"UNKOWN"`, so `"UNKNOWN"` never matched it.
#[test]
fn test_program_name_unknown_roundtrips() {
    let parsed: ProgramName = serde_json::from_value(serde_json::json!("UNKNOWN")).unwrap();
    assert_eq!(
        parsed,
        ProgramName::Unknown,
        "\"UNKNOWN\" should map to the typed variant"
    );

    let serialized = serde_json::to_value(ProgramName::Unknown).unwrap();
    assert_eq!(
        serialized,
        serde_json::json!("UNKNOWN"),
        "Unknown should serialize to \"UNKNOWN\""
    );
}

/// A genuinely unrecognized program name still falls through to `Other`.
#[test]
fn test_program_name_unrecognized_is_other() {
    let parsed: ProgramName = serde_json::from_value(serde_json::json!("SOME_FUTURE_DEX")).unwrap();
    assert_eq!(parsed, ProgramName::Other("SOME_FUTURE_DEX".to_string()));
}

/// Unset optional fields must be omitted from the serialized `params` rather than sent as
/// explicit `null`s, and the retained fields keep their camelCase wire names.
#[test]
fn test_get_assets_by_owner_omits_none_fields() {
    let request = GetAssetsByOwner {
        owner_address: "GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz".to_string(),
        page: 1,
        limit: None,
        before: None,
        after: None,
        display_options: None,
        sort_by: None,
        cursor: None,
    };

    let value = serde_json::to_value(&request).unwrap();
    let obj = value.as_object().unwrap();

    assert!(obj.contains_key("ownerAddress"), "ownerAddress should be present");
    assert!(obj.contains_key("page"), "page should be present");
    for absent in ["limit", "before", "after", "displayOptions", "sortBy", "cursor"] {
        assert!(
            !obj.contains_key(absent),
            "{absent} should be omitted when None, got {value}"
        );
    }
}

/// `limit` is a `u32` (matching the sibling `GetAssetsBy*` request structs) and serializes as
/// the camelCase-neutral `limit`.
#[test]
fn test_get_assets_by_owner_limit_serializes() {
    let request = GetAssetsByOwner {
        owner_address: "GQUtvPx89ZNCwmvQqFmH59bJcU8fW8siETpaxod7Aydz".to_string(),
        page: 1,
        limit: Some(50u32),
        ..Default::default()
    };

    let value = serde_json::to_value(&request).unwrap();
    assert_eq!(value["limit"], serde_json::json!(50));
}

/// Every paginated DAS request struct omits its unset optional fields, so no `null`s are sent
/// in the JSON-RPC `params`. Keeps the whole `GetAssetsBy*`/token/edition family consistent.
#[test]
fn test_paginated_request_structs_omit_nulls() {
    assert_no_null_fields("GetAssetsByOwner", &GetAssetsByOwner::default());
    assert_no_null_fields("GetAssetsByAuthority", &GetAssetsByAuthority::default());
    assert_no_null_fields("GetAssetsByCreator", &GetAssetsByCreator::default());
    assert_no_null_fields("GetAssetsByGroup", &GetAssetsByGroup::default());
    assert_no_null_fields("GetAssetSignatures", &GetAssetSignatures::default());
    assert_no_null_fields("GetTokenAccounts", &GetTokenAccounts::default());
    assert_no_null_fields("GetNftEditions", &GetNftEditions::default());
}
