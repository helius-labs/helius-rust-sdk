use helius::websocket::EnhancedWebsocket;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

/// Starts a local WebSocket server that accepts one connection and echoes messages.
/// Returns the ws:// URL to connect to.
async fn start_mock_ws_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{}", addr);

    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let mut ws = accept_async(stream).await.unwrap();
            // Keep the connection alive and respond to pings/close
            while let Some(Ok(msg)) = futures_util::StreamExt::next(&mut ws).await {
                match msg {
                    Message::Close(_) => {
                        let _ = ws.close(None).await;
                        break;
                    }
                    Message::Ping(data) => {
                        let _ = ws.send(Message::Pong(data)).await;
                    }
                    _ => {}
                }
            }
        }
    });

    // Give the server a moment to start listening
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    url
}

#[tokio::test]
async fn test_websocket_connect_and_shutdown() {
    let url = start_mock_ws_server().await;

    let ws = EnhancedWebsocket::new(&url, Some(30), None).await;
    assert!(ws.is_ok(), "Failed to connect to mock WebSocket: {:?}", ws.err());

    let ws = ws.unwrap();
    let result = ws.shutdown().await;
    assert!(result.is_ok(), "Shutdown failed: {:?}", result.err());
}

#[tokio::test]
async fn test_websocket_connect_invalid_url() {
    let result = EnhancedWebsocket::new("ws://127.0.0.1:1", None, None).await;
    assert!(result.is_err(), "Expected connection error for invalid URL");
}

#[tokio::test]
async fn test_websocket_set_node_version() {
    let url = start_mock_ws_server().await;

    let ws = EnhancedWebsocket::new(&url, Some(30), None).await.unwrap();

    let version = semver::Version::new(2, 1, 0);
    let result = ws.set_node_version(version).await;
    assert!(result.is_ok(), "set_node_version failed: {:?}", result.err());

    let _ = ws.shutdown().await;
}
