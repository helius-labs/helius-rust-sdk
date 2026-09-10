use helius::error::{HeliusError, Result};
use helius::request_handler::RequestHandler;

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
