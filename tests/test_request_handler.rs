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
        .with_body(r#"
        {
            "jsonrpc": "2.0",
            "error": {
                "code": -32603,
                "message": "internal error: please contact Helius support if this persists"
            }
        }"#)
        .create();

    let client: Arc<Client> = Arc::new(Client::new());
    let handler: RequestHandler = RequestHandler::new(client).unwrap();

    let response: Result<MockResponse> = handler
        .send::<(), MockResponse>(Method::GET, url.parse().unwrap(), None)
        .await;

    assert!(response.is_err());
    match response {
        Err(HeliusError::BadRequest { text, .. }) =>
            assert_eq!(
                text,
                "code: -32603, message: \"internal error: please contact Helius support if this persists\""
            ),
        _ => panic!("Expected BadRequest error"),
    }

    server.reset();
}
