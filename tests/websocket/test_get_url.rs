use helius::types::Cluster;
use helius::websocket::{
    EnhancedWebsocket, DEFAULT_MAX_FAILED_PINGS, DEFAULT_PING_DURATION_SECONDS, ENHANCED_WEBSOCKET_URL_DEVNET,
    ENHANCED_WEBSOCKET_URL_MAINNET,
};

#[test]
fn test_get_url_mainnet() {
    let url = EnhancedWebsocket::get_url(&Cluster::MainnetBeta, "test_key").unwrap();
    assert_eq!(url, "wss://atlas-mainnet.helius-rpc.com/?api-key=test_key");
}

#[test]
fn test_get_url_devnet() {
    let url = EnhancedWebsocket::get_url(&Cluster::Devnet, "test_key").unwrap();
    assert_eq!(url, "wss://atlas-devnet.helius-rpc.com/?api-key=test_key");
}

#[test]
fn test_get_url_staked_mainnet_errors() {
    let result = EnhancedWebsocket::get_url(&Cluster::StakedMainnetBeta, "test_key");
    assert!(
        result.is_err(),
        "StakedMainnetBeta should not be supported for WebSocket"
    );
}

#[test]
fn test_get_url_empty_api_key() {
    let url = EnhancedWebsocket::get_url(&Cluster::MainnetBeta, "").unwrap();
    assert_eq!(url, "wss://atlas-mainnet.helius-rpc.com/?api-key=");
}

#[test]
fn test_get_url_api_key_with_special_characters() {
    let url = EnhancedWebsocket::get_url(&Cluster::MainnetBeta, "key-with_special.chars123").unwrap();
    assert_eq!(
        url,
        "wss://atlas-mainnet.helius-rpc.com/?api-key=key-with_special.chars123"
    );
}

#[test]
fn test_websocket_url_constants() {
    assert_eq!(
        ENHANCED_WEBSOCKET_URL_MAINNET,
        "wss://atlas-mainnet.helius-rpc.com/?api-key="
    );
    assert_eq!(
        ENHANCED_WEBSOCKET_URL_DEVNET,
        "wss://atlas-devnet.helius-rpc.com/?api-key="
    );
}

#[test]
fn test_default_ping_constants() {
    assert_eq!(DEFAULT_PING_DURATION_SECONDS, 10);
    assert_eq!(DEFAULT_MAX_FAILED_PINGS, 3);
}
