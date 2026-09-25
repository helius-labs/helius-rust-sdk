use helius::error::{HeliusError, Result};
use helius::request_handler::{decode_response, RequestHandler};

use bytes::Bytes;
use mockito::{self, Server};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Default)]
struct MockResponse {
    message: String,
}

#[tokio::test]
async fn test_successful_request() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "success"}"#)
        .create();

    let client: Arc<Client> = Arc::new(Client::new());
    let handler: RequestHandler = RequestHandler::new(client).unwrap();

    let response: Result<MockResponse> = handler
        .send::<(), MockResponse>(Method::GET, url.parse().unwrap(), None)
        .await;

    assert!(response.is_ok());
    assert_eq!(response.unwrap().message, "success");

    server.reset();
}

#[tokio::test]
async fn test_bad_request_error() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("GET", "/")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "bad request"}"#)
        .create();

    let client: Arc<Client> = Arc::new(Client::new());
    let handler: RequestHandler = RequestHandler::new(client).unwrap();

    let response: Result<MockResponse> = handler
        .send::<(), MockResponse>(Method::GET, url.parse().unwrap(), None)
        .await;

    assert!(response.is_err());
    match response {
        Err(HeliusError::BadRequest { text, .. }) => assert_eq!(text, "bad request"),
        _ => panic!("Expected BadRequest error"),
    }

    server.reset();
}

#[tokio::test]
async fn test_bad_request_with_json_rpc_error() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("GET", "/")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(
            r#"
        {
            "jsonrpc": "2.0",
            "error": {
                "code": -32603,
                "message": "internal error: please contact Helius support if this persists"
            }
        }"#,
        )
        .create();

    let client: Arc<Client> = Arc::new(Client::new());
    let handler: RequestHandler = RequestHandler::new(client).unwrap();

    let response: Result<MockResponse> = handler
        .send::<(), MockResponse>(Method::GET, url.parse().unwrap(), None)
        .await;

    assert!(response.is_err());
    match response {
        Err(HeliusError::BadRequest { text, .. }) => assert_eq!(
            text,
            "code: -32603, message: \"internal error: please contact Helius support if this persists\""
        ),
        _ => panic!("Expected BadRequest error"),
    }

    server.reset();
}

/// A payload `simd_json`'s serde bridge cannot handle but `serde_json` can. `u128` is one such
/// gap; the escaped string alongside it is what the in-place unescaping corrupts.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
struct WideNumberResponse {
    message: String,
    huge: u128,
}

/// The `serde_json` retry must read the response body, not the buffer `simd_json` already
/// rewrote. `simd_json` unescapes strings in place, so a failed parse leaves the buffer
/// corrupted; retrying on it fails spuriously with a "control character found while parsing a
/// string" error and, because that error is the one returned, it also masks `simd_json`'s
/// accurate diagnostic.
///
/// This body makes the difference observable end to end: `simd_json` rejects the `u128`, and
/// `serde_json` parses it fine, but only if it is handed the original bytes. Before the fix this
/// request failed; now it succeeds.
#[tokio::test]
async fn test_falls_back_to_serde_json_on_uncorrupted_body() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    // The escaped newline and quotes are what `simd_json` rewrites in place.
    server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"line\none \"quoted\"","huge":340282366920938463463374607431768211455}"#)
        .create();

    let client: Arc<Client> = Arc::new(Client::new());
    let handler: RequestHandler = RequestHandler::new(client).unwrap();

    let response: Result<WideNumberResponse> = handler
        .send::<(), WideNumberResponse>(Method::GET, url.parse().unwrap(), None)
        .await;

    let parsed = response.expect("serde_json should recover a payload simd_json cannot parse");
    assert_eq!(
        parsed,
        WideNumberResponse {
            message: "line\none \"quoted\"".to_string(),
            huge: u128::MAX,
        },
        "the fallback must see the original body, escapes intact"
    );

    server.reset();
}

/// A body that is genuinely unparseable must still surface an error rather than silently
/// deserializing as `T::default()`.
#[tokio::test]
async fn test_unparseable_body_still_errors() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message":"unterminated"#)
        .create();

    let client: Arc<Client> = Arc::new(Client::new());
    let handler: RequestHandler = RequestHandler::new(client).unwrap();

    let response: Result<MockResponse> = handler
        .send::<(), MockResponse>(Method::GET, url.parse().unwrap(), None)
        .await;

    assert!(
        matches!(response, Err(HeliusError::SerdeJson(_))),
        "a malformed body should surface a deserialization error, got {response:?}"
    );

    server.reset();
}

/// `send_raw` hands back the body of a 2xx response byte for byte, leaving decoding to the
/// caller; `decode_response` on those bytes must produce what `send` would have.
#[tokio::test]
async fn test_send_raw_returns_undecoded_body() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    let body: &str = r#"{"message": "success"}"#;
    server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(body)
        .create();

    let client: Arc<Client> = Arc::new(Client::new());
    let handler: RequestHandler = RequestHandler::new(client).unwrap();

    let raw: Bytes = handler
        .send_raw::<()>(Method::GET, url.parse().unwrap(), None)
        .await
        .expect("2xx response should yield its body");

    assert_eq!(raw.as_ref(), body.as_bytes(), "body must be returned unmodified");

    let decoded: MockResponse = decode_response(&raw).expect("body should decode like the typed path");
    assert_eq!(decoded.message, "success");

    server.reset();
}

/// A failure status is mapped to the matching error on the raw path too — the caller never
/// receives an error body as `Bytes` and has to inspect the status themselves.
#[tokio::test]
async fn test_send_raw_maps_failure_status_to_error() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("GET", "/")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "invalid api key"}"#)
        .create();

    let client: Arc<Client> = Arc::new(Client::new());
    let handler: RequestHandler = RequestHandler::new(client).unwrap();

    let response: Result<Bytes> = handler.send_raw::<()>(Method::GET, url.parse().unwrap(), None).await;

    match response {
        Err(HeliusError::Unauthorized { text, .. }) => assert_eq!(text, "invalid api key"),
        other => panic!("Expected Unauthorized error, got {other:?}"),
    }

    server.reset();
}

/// The intended use: fetch on the runtime, decode on the blocking pool. `Bytes` is
/// `Send + 'static`, so it moves into the closure without a copy.
#[tokio::test]
async fn test_send_raw_body_decodes_on_blocking_thread() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"message": "decoded off-runtime"}"#)
        .create();

    let client: Arc<Client> = Arc::new(Client::new());
    let handler: RequestHandler = RequestHandler::new(client).unwrap();

    let raw: Bytes = handler
        .send_raw::<()>(Method::GET, url.parse().unwrap(), None)
        .await
        .unwrap();

    let decoded: MockResponse = tokio::task::spawn_blocking(move || decode_response::<MockResponse>(&raw))
        .await
        .expect("blocking task should complete")
        .expect("body should decode");

    assert_eq!(decoded.message, "decoded off-runtime");

    server.reset();
}

/// A `spawn_blocking` task that fails to complete converts into `HeliusError::Unknown` with a
/// 500 status, which is what makes the documented `.await??` decode pattern compile against
/// the SDK's `Result`.
#[tokio::test]
async fn test_join_error_converts_to_unknown() {
    async fn decode_on_blocking_pool() -> Result<MockResponse> {
        let decoded: MockResponse =
            tokio::task::spawn_blocking(|| -> Result<MockResponse> { panic!("decoder panicked") }).await??;
        Ok(decoded)
    }

    let result: Result<MockResponse> = decode_on_blocking_pool().await;

    match result {
        Err(HeliusError::Unknown { code, text }) => {
            assert_eq!(code, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
            assert!(
                text.contains("Blocking task failed to complete"),
                "unexpected error text: {text}"
            );
        }
        other => panic!("Expected HeliusError::Unknown, got {other:?}"),
    }
}

/// `decode_response` keeps the typed path's empty-body rule: no bytes decode to `T::default()`.
#[test]
fn test_decode_response_empty_body_is_default() {
    let decoded: MockResponse = decode_response(b"").expect("empty body should decode to the default");
    assert_eq!(decoded.message, String::default());
}

/// `decode_response` keeps the typed path's fallback: a payload simd-json rejects but
/// serde_json accepts still decodes, from the original (uncorrupted) bytes.
#[test]
fn test_decode_response_falls_back_to_serde_json() {
    let body: &[u8] = br#"{"message":"line\none \"quoted\"","huge":340282366920938463463374607431768211455}"#;

    let decoded: WideNumberResponse = decode_response(body).expect("serde_json should recover the payload");
    assert_eq!(
        decoded,
        WideNumberResponse {
            message: "line\none \"quoted\"".to_string(),
            huge: u128::MAX,
        }
    );
}

/// `decode_response` surfaces a malformed body as a deserialization error rather than a default.
#[test]
fn test_decode_response_rejects_malformed_body() {
    let result: Result<MockResponse> = decode_response(br#"{"message":"unterminated"#);
    assert!(
        matches!(result, Err(HeliusError::SerdeJson(_))),
        "malformed body should surface a deserialization error, got {result:?}"
    );
}
