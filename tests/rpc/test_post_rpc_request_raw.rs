use super::helpers::mock_helius;

use helius::client::Helius;
use helius::error::{HeliusError, Result};
use helius::rpc_client::decode_rpc_response;

use bytes::Bytes;
use mockito::{self, Server};
use serde_json::{json, Value};

/// `post_rpc_request_raw` returns the whole JSON-RPC envelope untouched, and
/// `decode_rpc_response` unwraps `result` from it the way `post_rpc_request` does.
#[tokio::test]
async fn test_post_rpc_request_raw_returns_envelope() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let envelope: &str = r#"{"jsonrpc":"2.0","result":{"total":1,"items":[{"id":"abc"}]},"id":"1"}"#;
    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(envelope)
        .create();

    let helius: Helius = mock_helius(&url);

    let raw: Bytes = helius
        .rpc()
        .post_rpc_request_raw("getAssetsByOwner", json!({"ownerAddress": "owner"}))
        .await
        .expect("2xx response should yield its body");

    assert_eq!(
        raw.as_ref(),
        envelope.as_bytes(),
        "envelope must be returned unmodified"
    );

    let result: Value =
        decode_rpc_response("getAssetsByOwner", &raw).expect("envelope should decode to its result field");
    assert_eq!(result["total"], json!(1));
    assert_eq!(result["items"][0]["id"], json!("abc"));

    server.reset();
}

/// A JSON-RPC `error` object arrives inside a 2xx body, so the raw path hands it back as bytes;
/// `decode_rpc_response` must then surface it as `RpcError` exactly like the typed path.
#[tokio::test]
async fn test_decode_rpc_response_surfaces_rpc_error() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"jsonrpc":"2.0","error":{"code":-32602,"message":"Invalid params"},"id":"1"}"#)
        .create();

    let helius: Helius = mock_helius(&url);

    let raw: Bytes = helius
        .rpc()
        .post_rpc_request_raw("getAsset", json!({"id": "bad"}))
        .await
        .expect("a JSON-RPC error rides in a 2xx body and is not a transport failure");

    let decoded: Result<Value> = decode_rpc_response("getAsset", &raw);
    match decoded {
        Err(HeliusError::RpcError { code, message }) => {
            assert_eq!(code, -32602);
            assert!(message.contains("Invalid params"), "unexpected message: {message}");
        }
        other => panic!("Expected HeliusError::RpcError, got {other:?}"),
    }

    server.reset();
}

/// A `"result": null` envelope decodes to `None` for an `Option` target, matching the typed
/// path's handling of methods such as `getAsset`.
#[test]
fn test_decode_rpc_response_null_result_for_option() {
    let body: &[u8] = br#"{"jsonrpc":"2.0","result":null,"id":"1"}"#;

    let decoded: Option<Value> = decode_rpc_response("getAsset", body).expect("null result should decode to None");
    assert!(decoded.is_none());
}

#[test]
fn test_decode_rpc_response_rejects_missing_result() {
    for body in [
        br#"{"jsonrpc":"2.0","id":"1"}"#.as_slice(),
        br#"{"jsonrpc":"2.0","id":"1","error":null}"#,
    ] {
        let decoded: Result<Option<Value>> = decode_rpc_response("getAsset", body);
        assert_rpc_error(decoded, "RPC method 'getAsset' returned neither a result nor an error");
    }
}

fn assert_rpc_error<T: std::fmt::Debug>(result: Result<T>, expected_message: &str) {
    match result {
        Err(HeliusError::RpcError { code, message }) => {
            assert_eq!(code, 0);
            assert_eq!(message, expected_message);
        }
        other => panic!("Expected HeliusError::RpcError, got {other:?}"),
    }
}

#[test]
fn test_decode_rpc_response_rejects_empty_body() {
    let decoded: Result<Option<Value>> = decode_rpc_response("getAsset", b"");
    assert_rpc_error(decoded, "RPC method 'getAsset' returned an empty response body");
}

#[test]
fn test_decode_rpc_response_rejects_null_for_non_nullable_result() {
    let decoded: Result<u64> = decode_rpc_response("getAsset", br#"{"jsonrpc":"2.0","result":null}"#);
    assert_rpc_error(
        decoded,
        "RPC method 'getAsset' returned a null result that cannot be decoded",
    );
}

#[test]
fn test_decode_rpc_response_preserves_result_value() {
    let decoded: Option<Value> =
        decode_rpc_response("getAsset", br#"{"jsonrpc":"2.0","result":{"id":"asset"}}"#).unwrap();
    assert_eq!(decoded, Some(json!({"id": "asset"})));
}

#[tokio::test]
async fn test_get_asset_distinguishes_missing_result_from_null() {
    for (body, expected_error) in [
        ("", Some("RPC method 'getAsset' returned an empty response body")),
        (
            r#"{"jsonrpc":"2.0","id":"1"}"#,
            Some("RPC method 'getAsset' returned neither a result nor an error"),
        ),
        (
            r#"{"jsonrpc":"2.0","id":"1","error":null}"#,
            Some("RPC method 'getAsset' returned neither a result nor an error"),
        ),
        (r#"{"jsonrpc":"2.0","id":"1","result":null}"#, None),
    ] {
        let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
        let helius = mock_helius(&server.url());
        let mock = server
            .mock("POST", "/?api-key=fake_api_key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create_async()
            .await;

        let response = helius
            .rpc()
            .get_asset(helius::types::GetAsset {
                id: "asset".to_string(),
                ..Default::default()
            })
            .await;
        if let Some(message) = expected_error {
            assert_rpc_error(response, message);
        } else {
            assert!(matches!(response, Ok(None)), "expected no asset, got {response:?}");
        }
        mock.assert_async().await;
    }
}

/// A failure status is still mapped to an error on the raw path; the caller never sees an
/// HTTP error body as `Bytes`.
#[tokio::test]
async fn test_post_rpc_request_raw_maps_failure_status() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "rate limit exceeded"}"#)
        .create();

    let helius: Helius = mock_helius(&url);

    let response: Result<Bytes> = helius.rpc().post_rpc_request_raw("getAsset", json!({"id": "x"})).await;

    assert!(
        matches!(response, Err(HeliusError::RateLimitExceeded { .. })),
        "expected RateLimitExceeded, got {response:?}"
    );

    server.reset();
}
