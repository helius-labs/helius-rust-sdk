use helius::optimized_transaction::{
    sender_fast_url, sender_ping_url, SENDER_ENDPOINTS, SENDER_REGION_ALIASES,
};

#[test]
fn test_sender_fast_url_default() {
    let url = sender_fast_url("Default");
    assert_eq!(url, "http://sender.helius-rpc.com/fast");
}

#[test]
fn test_sender_fast_url_us_slc() {
    let url = sender_fast_url("US_SLC");
    assert_eq!(url, "http://slc-sender.helius-rpc.com/fast");
}

#[test]
fn test_sender_fast_url_eu_central() {
    let url = sender_fast_url("EU_CENTRAL");
    assert_eq!(url, "http://fra-sender.helius-rpc.com/fast");
}

#[test]
fn test_sender_fast_url_ap_tokyo() {
    let url = sender_fast_url("AP_TOKYO");
    assert_eq!(url, "http://tyo-sender.helius-rpc.com/fast");
}

#[test]
fn test_sender_fast_url_unknown_region_falls_back() {
    let url = sender_fast_url("UNKNOWN_REGION");
    assert_eq!(url, "http://slc-sender.helius-rpc.com/fast");
}

#[test]
fn test_sender_ping_url_default() {
    let url = sender_ping_url("Default");
    assert_eq!(url, "http://sender.helius-rpc.com/ping");
}

#[test]
fn test_sender_ping_url_us_east() {
    let url = sender_ping_url("US_EAST");
    assert_eq!(url, "http://ewr-sender.helius-rpc.com/ping");
}

#[test]
fn test_sender_region_alias_resolution() {
    // Hyphenated aliases should resolve to the same URL as underscored keys
    assert_eq!(sender_fast_url("US-EAST"), sender_fast_url("US_EAST"));
    assert_eq!(sender_fast_url("EU-CENTRAL"), sender_fast_url("EU_CENTRAL"));
    assert_eq!(sender_fast_url("AP-TOKYO"), sender_fast_url("AP_TOKYO"));
    assert_eq!(sender_fast_url("EU-WEST"), sender_fast_url("EU_WEST"));
    assert_eq!(sender_fast_url("EU-NORTH"), sender_fast_url("EU_NORTH"));
    assert_eq!(sender_fast_url("AP-SINGAPORE"), sender_fast_url("AP_SINGAPORE"));
    assert_eq!(sender_fast_url("US-SLC"), sender_fast_url("US_SLC"));
}

#[test]
fn test_sender_endpoints_has_all_regions() {
    let expected_regions = [
        "Default",
        "US_SLC",
        "US_EAST",
        "EU_WEST",
        "EU_CENTRAL",
        "EU_NORTH",
        "AP_SINGAPORE",
        "AP_TOKYO",
    ];
    for region in &expected_regions {
        assert!(
            SENDER_ENDPOINTS.contains_key(region),
            "Missing region: {}",
            region
        );
    }
}

#[test]
fn test_sender_region_aliases_has_all_aliases() {
    let expected_aliases = [
        "US-EAST",
        "US-SLC",
        "EU-WEST",
        "EU-CENTRAL",
        "EU-NORTH",
        "AP-SINGAPORE",
        "AP-TOKYO",
    ];
    for alias in &expected_aliases {
        assert!(
            SENDER_REGION_ALIASES.contains_key(alias),
            "Missing alias: {}",
            alias
        );
    }
}
