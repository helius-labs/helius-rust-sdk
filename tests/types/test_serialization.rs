use helius::types::{GetAssetsByOwner, ProgramName};

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
