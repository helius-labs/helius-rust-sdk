use crate::error::{HeliusError, Result};
use crate::types::Cluster;
use crate::types::{RpcTransactionsConfig, TransactionNotification};
use futures_util::{
    future::{ready, BoxFuture, FutureExt},
    sink::SinkExt,
    stream::{BoxStream, StreamExt},
};
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use solana_account_decoder::UiAccount;
use solana_rpc_client_api::config::RpcAccountInfoConfig;
use solana_rpc_client_api::{error_object::RpcErrorObject, response::Response as RpcResponse};
use solana_sdk::pubkey::Pubkey;
use std::collections::BTreeMap;
use std::fmt::Debug;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot, RwLock},
    task::JoinHandle,
    time::{sleep, Duration},
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::frame::{coding::CloseCode, CloseFrame},
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};

/// Base WebSocket URL for the Helius enhanced (Geyser) endpoint on mainnet.
/// The API key is appended as a query parameter.
pub const ENHANCED_WEBSOCKET_URL_MAINNET: &str = "wss://atlas-mainnet.helius-rpc.com/?api-key=";

/// Base WebSocket URL for the Helius enhanced (Geyser) endpoint on devnet.
/// The API key is appended as a query parameter.
pub const ENHANCED_WEBSOCKET_URL_DEVNET: &str = "wss://atlas-devnet.helius-rpc.com/?api-key=";

/// Default interval in seconds between WebSocket ping frames sent to keep the connection alive.
pub const DEFAULT_PING_DURATION_SECONDS: u64 = 10;

/// Default maximum number of consecutive missed pong responses before the connection is
/// considered dead and closed.
pub const DEFAULT_MAX_FAILED_PINGS: usize = 3;

/// Maximum number of consecutive unusable frames tolerated before the connection is treated as
/// broken.
///
/// Individual bad frames are skipped so one cannot kill every live subscription; this bound keeps
/// that from becoming a silent spin against a peer that is persistently broken. The counter resets
/// on any frame the client can act on, so only an unbroken run trips it.
pub const MAX_CONSECUTIVE_UNUSABLE_FRAMES: usize = 10;

// pub type Result<T = ()> = Result<T, HeliusError>;

type UnsubscribeFn = Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send>;
type SubscribeResponseMsg = Result<(mpsc::UnboundedReceiver<Value>, UnsubscribeFn)>;
type SubscribeRequestMsg = (String, Value, oneshot::Sender<SubscribeResponseMsg>);
type SubscribeResult<'a, T> = Result<(BoxStream<'a, T>, UnsubscribeFn)>;
type RequestMsg = (String, Value, oneshot::Sender<Result<Value>>);
type UnsubscribeRequest = (String, u64, oneshot::Sender<()>);

/// Outcome of handling one incoming text frame: `None` if it was acted on, `Some(reason)` if it
/// could not be.
type FrameOutcome = Option<String>;

/// A client for subscribing to transaction or account updates from a Helius (Geyser) enhanced websocket server.
///
/// Forked from Solana's [`PubsubClient`].
pub struct EnhancedWebsocket {
    subscribe_sender: mpsc::UnboundedSender<SubscribeRequestMsg>,
    shutdown_sender: oneshot::Sender<()>,
    node_version: RwLock<Option<semver::Version>>,
    ws: JoinHandle<Result<()>>,
}

/// Handles one incoming JSON-RPC text frame, returning `Some(reason)` if it could not be acted on.
///
/// Every failure in here belongs to the frame, or at most to the single request waiting on it —
/// never to the connection. `run_ws` owns the only stream and the whole subscription map, so a
/// failure that propagates out of it drops every subscriber's channel at once.
fn handle_frame(
    text: &str,
    other_requests: &mut BTreeMap<u64, oneshot::Sender<Result<Value>>>,
    requests_unsubscribe: &mut BTreeMap<u64, oneshot::Sender<()>>,
    requests_subscribe: &mut BTreeMap<u64, (String, oneshot::Sender<SubscribeResponseMsg>)>,
    subscriptions: &mut BTreeMap<u64, mpsc::UnboundedSender<Value>>,
    unsubscribe_sender: &mpsc::UnboundedSender<UnsubscribeRequest>,
) -> FrameOutcome {
    let mut json: Map<String, Value> = match serde_json::from_str(text) {
        Ok(json) => json,
        Err(err) => return Some(format!("not valid JSON-RPC: {err}")),
    };

    // Response to one of our requests, e.g. `{"jsonrpc":"2.0","result":5308752,"id":1}`
    if let Some(id_value) = json.get("id") {
        // Servers answer a parse or invalid-request error with `"id": null`, so a non-numeric id
        // is an expected frame rather than grounds for a teardown.
        let Some(id) = id_value.as_u64() else {
            return Some(format!("unusable `id` field: {id_value}"));
        };

        let err = json.get("error").map(|error_object| {
            match serde_json::from_value::<RpcErrorObject>(error_object.clone()) {
                Ok(rpc_error_object) => format!("{} ({})", rpc_error_object.message, rpc_error_object.code),
                Err(err) => format!(
                    "Failed to deserialize RPC error response: {} [{}]",
                    serde_json::to_string(error_object).unwrap_or_default(),
                    err
                ),
            }
        });

        if let Some(response_sender) = other_requests.remove(&id) {
            match err {
                Some(reason) => {
                    let _ = response_sender.send(Err(HeliusError::EnhancedWebsocket {
                        reason,
                        message: text.to_string(),
                    }));
                }
                None => match json.get("result") {
                    // A send error here only means the caller stopped waiting.
                    Some(json_result) => {
                        let _ = response_sender.send(Ok(json_result.clone()));
                    }
                    None => {
                        // Fail the one request that is waiting, not every subscriber.
                        let _ = response_sender.send(Err(HeliusError::EnhancedWebsocket {
                            reason: "missing `result` field".into(),
                            message: text.to_string(),
                        }));
                        return Some(format!("response to request {id} had neither `result` nor `error`"));
                    }
                },
            }
        } else if let Some(response_sender) = requests_unsubscribe.remove(&id) {
            let _ = response_sender.send(()); // do not care if receiver is closed
        } else if let Some((operation, response_sender)) = requests_subscribe.remove(&id) {
            match err {
                Some(reason) => {
                    let _ = response_sender.send(Err(HeliusError::EnhancedWebsocket {
                        reason,
                        message: text.to_string(),
                    }));
                }
                None => {
                    let Some(sid) = json.get("result").and_then(Value::as_u64) else {
                        let _ = response_sender.send(Err(HeliusError::EnhancedWebsocket {
                            reason: "invalid `result` field".into(),
                            message: text.to_string(),
                        }));
                        return Some(format!("subscribe ack {id} carried no subscription id"));
                    };

                    // Create notifications channel and unsubscribe function
                    let (notifications_sender, notifications_receiver) = mpsc::unbounded_channel();
                    let unsubscribe_operation = operation.clone();
                    let unsubscribe_sender_for_fn = unsubscribe_sender.clone();
                    let unsubscribe = Box::new(move || {
                        async move {
                            let (response_sender, response_receiver) = oneshot::channel();
                            // do nothing if ws already closed
                            if unsubscribe_sender_for_fn
                                .send((unsubscribe_operation, sid, response_sender))
                                .is_ok()
                            {
                                let _ = response_receiver.await; // channel can be closed only if ws is closed
                            }
                        }
                        .boxed()
                    });

                    if response_sender.send(Ok((notifications_receiver, unsubscribe))).is_err() {
                        // The subscriber gave up before its ack arrived. The server-side
                        // subscription already exists, so release it rather than leak it.
                        let (drop_sender, _drop_receiver) = oneshot::channel();
                        let _ = unsubscribe_sender.send((operation, sid, drop_sender));
                    } else {
                        subscriptions.insert(sid, notifications_sender);
                    }
                }
            }
        } else {
            // A duplicate or late ack for a request that is no longer pending.
            return Some(format!("no pending request with id {id}"));
        }

        return None;
    }

    // Notification, example:
    // `{"jsonrpc":"2.0","method":"logsNotification","params":{"result":{...},"subscription":3114862}}`
    if let Some(Value::Object(params)) = json.get_mut("params") {
        if let Some(sid) = params.get("subscription").and_then(Value::as_u64) {
            let mut unsubscribe_required = false;

            if let Some(notifications_sender) = subscriptions.get(&sid) {
                if let Some(result) = params.remove("result") {
                    if notifications_sender.send(result).is_err() {
                        unsubscribe_required = true;
                    }
                }
            } else {
                unsubscribe_required = true;
            }

            if unsubscribe_required {
                if let Some(Value::String(method)) = json.remove("method") {
                    if let Some(operation) = method.strip_suffix("Notification") {
                        let (response_sender, _response_receiver) = oneshot::channel();
                        let _ = unsubscribe_sender.send((operation.to_string(), sid, response_sender));
                    }
                }
            }
        }

        return None;
    }

    // Well-formed JSON that is neither a response nor a notification. Left uncounted: it may be
    // benign server traffic, and ignoring it costs nothing.
    log::debug!("Ignoring unrecognized websocket frame: {text}");

    None
}

impl EnhancedWebsocket {
    /// Constructs the complete websocket URL for connecting to Helius's enhanced websocket endpoints.
    ///
    /// # Arguments
    ///
    /// * `cluster` - The Solana cluster to connect to (MainnetBeta or Devnet)
    /// * `api_key` - Your Helius API key
    ///
    /// # Returns
    ///
    /// Returns a Result containing the formatted websocket URL or an error if an unsupported cluster is specified.
    ///
    /// # Errors
    ///
    /// Returns `HeliusError::EnhancedWebsocket` if the specified cluster is not MainnetBeta or Devnet.
    /// Note: StakedMainnetBeta is not supported for websocket connections.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use helius::websocket::EnhancedWebsocket;
    /// use helius::types::Cluster;
    ///
    /// let api_key = "your_api_key";
    ///
    /// // For Mainnet
    /// let mainnet_url = EnhancedWebsocket::get_url(&Cluster::MainnetBeta, api_key).expect("Failed to get URL");
    /// println!("Mainnet URL: {}", mainnet_url);
    /// assert!(mainnet_url.eq("wss://atlas-mainnet.helius-rpc.com/?api-key=your_api_key"));
    ///
    /// // For Devnet
    /// let devnet_url = EnhancedWebsocket::get_url(&Cluster::Devnet, api_key).expect("Failed to get URL");
    /// println!("Devnet URL: {}", devnet_url);
    /// assert!(devnet_url.eq("wss://atlas-devnet.helius-rpc.com/?api-key=your_api_key"));
    ///
    /// // For Staked Mainnet (will error)
    /// let staked_result = EnhancedWebsocket::get_url(&Cluster::StakedMainnetBeta, api_key);
    /// assert!(staked_result.is_err());
    /// ```
    pub fn get_url(cluster: &Cluster, api_key: &str) -> Result<String> {
        match cluster {
            Cluster::MainnetBeta => Ok(format!("{}{}", ENHANCED_WEBSOCKET_URL_MAINNET, api_key)),
            Cluster::Devnet => Ok(format!("{}{}", ENHANCED_WEBSOCKET_URL_DEVNET, api_key)),
            Cluster::StakedMainnetBeta => Err(HeliusError::EnhancedWebsocket {
                reason: "Unsupported cluster".into(),
                message: "only mainnet and devnet are supported".into(),
            }),
        }
    }

    /// Expects enhanced websocket endpoint: wss://atlas-mainnet.helius-rpc.com?api-key=<API_KEY>
    pub async fn new(url: &str, ping_interval_secs: Option<u64>, pong_timeout_secs: Option<u64>) -> Result<Self> {
        let (ws, _response) = connect_async(url).await.map_err(HeliusError::Tungstenite)?;

        let (subscribe_sender, subscribe_receiver) = mpsc::unbounded_channel();
        let (_request_sender, request_receiver) = mpsc::unbounded_channel();
        let (shutdown_sender, shutdown_receiver) = oneshot::channel();

        let ping_interval = ping_interval_secs
            .filter(|interval: &u64| *interval != 0)
            .unwrap_or(DEFAULT_PING_DURATION_SECONDS);
        let max_failed_pings = pong_timeout_secs
            .map(|timeout| (timeout as f64 / ping_interval as f64).ceil() as usize)
            .map_or(DEFAULT_MAX_FAILED_PINGS, |max_failed_pings| {
                if max_failed_pings != 0 {
                    max_failed_pings
                } else {
                    usize::MAX
                }
            });

        Ok(Self {
            subscribe_sender,
            shutdown_sender,
            node_version: RwLock::new(None),
            ws: tokio::spawn(EnhancedWebsocket::run_ws(
                ws,
                subscribe_receiver,
                request_receiver,
                shutdown_receiver,
                ping_interval,
                max_failed_pings,
            )),
        })
    }

    /// Gracefully shuts down the WebSocket connection.
    ///
    /// Sends a shutdown signal and waits for the background WebSocket task to complete.
    /// This consumes `self`, preventing further use of the connection.
    pub async fn shutdown(self) -> Result<()> {
        let _ = self.shutdown_sender.send(());
        self.ws
            .await
            .map_err(|e| HeliusError::WebsocketClosed(format!("WebSocket task failed: {}", e)))?
    }

    /// Sets the node version for compatibility-aware message handling.
    ///
    /// # Arguments
    /// * `version` - The semver version of the connected Solana node
    pub async fn set_node_version(&self, version: semver::Version) -> Result<()> {
        let mut w_node_version = self.node_version.write().await;
        *w_node_version = Some(version);
        Ok(())
    }

    async fn subscribe<'a, T: DeserializeOwned + Send + Debug + 'a>(
        &self,
        operation: &str,
        params: Value,
    ) -> SubscribeResult<'a, T> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.subscribe_sender
            .send((operation.to_string(), params, response_sender))
            .map_err(|err| HeliusError::WebsocketClosed(err.to_string()))?;

        let (notifications, unsubscribe) = response_receiver
            .await
            .map_err(|err| HeliusError::WebsocketClosed(err.to_string()))??;
        Ok((
            UnboundedReceiverStream::new(notifications)
                .filter_map(|value| match serde_json::from_value::<T>(value.clone()) {
                    Err(e) => {
                        log::warn!(
                            "Failed to parse websocket notification: {:#?} for value: {:#?}",
                            e,
                            value
                        );
                        ready(None)
                    }
                    Ok(res) => ready(Some(res)),
                })
                .boxed(),
            unsubscribe,
        ))
    }

    /// Stream transactions with numerous configurations and filters to choose from.
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::error::Result;
    /// use helius::types::{Cluster, RpcTransactionsConfig, TransactionSubscribeFilter, TransactionSubscribeOptions};
    /// use solana_sdk::pubkey;
    /// use tokio_stream::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///   let helius = Helius::new_async("your_api_key", Cluster::MainnetBeta).await.expect("Failed to create a Helius client");
    ///   // you may monitor transactions for any pubkey, this is just an example.
    ///   let key = pubkey!("BtsmiEEvnSuUnKxqXj2PZRYpPJAc7C34mGz8gtJ1DAaH");
    ///   let config = RpcTransactionsConfig {
    ///     filter: TransactionSubscribeFilter::standard(&key),
    ///     options: TransactionSubscribeOptions::default(),
    ///   };
    ///   if let Some(ws) = helius.ws() {
    ///     let (mut stream, _unsub) = ws.transaction_subscribe(config).await?;
    ///     while let Some(event) = stream.next().await {
    ///       println!("{:#?}", event);
    ///     }
    ///   }
    ///   Ok(())
    /// }
    /// ```
    pub async fn transaction_subscribe(
        &self,
        config: RpcTransactionsConfig,
    ) -> SubscribeResult<'_, TransactionNotification> {
        let params = json!([config.filter, config.options]);
        self.subscribe("transaction", params).await
    }

    /// Stream accounts with numerous configurations and filters to choose from.
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::error::Result;
    /// use helius::types::{Cluster, RpcTransactionsConfig, TransactionSubscribeFilter, TransactionSubscribeOptions};
    /// use solana_sdk::pubkey;
    /// use tokio_stream::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///   let helius = Helius::new_async("your_api_key", Cluster::MainnetBeta).await.expect("Failed to create a Helius client");
    ///   // you may monitor updates for any account pubkey, this is just an example.
    ///   let key = pubkey!("BtsmiEEvnSuUnKxqXj2PZRYpPJAc7C34mGz8gtJ1DAaH");
    ///   if let Some(ws) = helius.ws() {
    ///     let (mut stream, _unsub) = ws.account_subscribe(&key, None).await?;
    ///     while let Some(event) = stream.next().await {
    ///       println!("{:#?}", event);
    ///     }
    ///   }
    ///   Ok(())
    /// }
    /// ```
    pub async fn account_subscribe(
        &self,
        pubkey: &Pubkey,
        config: Option<RpcAccountInfoConfig>,
    ) -> SubscribeResult<'_, RpcResponse<UiAccount>> {
        let params = json!([pubkey.to_string(), config]);
        self.subscribe("account", params).await
    }

    async fn run_ws(
        mut ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
        mut subscribe_receiver: mpsc::UnboundedReceiver<SubscribeRequestMsg>,
        mut request_receiver: mpsc::UnboundedReceiver<RequestMsg>,
        mut shutdown_receiver: oneshot::Receiver<()>,
        ping_duration_seconds: u64,
        max_failed_pings: usize,
    ) -> Result<()> {
        let mut request_id: u64 = 0;
        let mut unmatched_pings: usize = 0;
        let mut consecutive_unusable_frames: usize = 0;

        let mut requests_subscribe = BTreeMap::new();
        let mut requests_unsubscribe = BTreeMap::<u64, oneshot::Sender<()>>::new();
        let mut other_requests = BTreeMap::new();
        let mut subscriptions = BTreeMap::new();
        let (unsubscribe_sender, mut unsubscribe_receiver) = mpsc::unbounded_channel();

        loop {
            tokio::select! {
              // Send close on shutdown signal
              _ = &mut shutdown_receiver => {
                let frame = CloseFrame { code: CloseCode::Normal, reason: "".into() };
                ws.send(Message::Close(Some(frame))).await?;
                ws.flush().await?;
                break;
              },
              // Send `Message::Ping` each 10s if no any other communication
              () = sleep(Duration::from_secs(ping_duration_seconds)) => {
                // Check if we've exceeded our failed ping threshold
                if unmatched_pings >= max_failed_pings {
                  let frame = CloseFrame {
                    code: CloseCode::Abnormal,
                    reason: format!("No pong received after {} pings", max_failed_pings).into()
                  };

                  ws.send(Message::Close(Some(frame))).await?;
                  ws.flush().await?;

                  return Err(HeliusError::WebsocketClosed(
                    format!("Connection timeout: no pong received after {} pings", max_failed_pings)
                  ));
                }

                ws.send(Message::Ping(Default::default())).await?;
                unmatched_pings += 1;
              },
              // Read message for subscribe
              Some((operation, params, response_sender)) = subscribe_receiver.recv() => {
                request_id += 1;
                let method = format!("{operation}Subscribe");
                let body = json!({"jsonrpc":"2.0","id":request_id,"method":method,"params":params});
                ws.send(body.to_string().into()).await?;
                requests_subscribe.insert(request_id, (operation, response_sender));
              },
              // Read message for unsubscribe
              Some((operation, sid, response_sender)) = unsubscribe_receiver.recv() => {
                subscriptions.remove(&sid);
                request_id += 1;
                let method = format!("{operation}Unsubscribe");
                let text = json!({"jsonrpc":"2.0","id":request_id,"method":method,"params":[sid]}).to_string();
                ws.send(text.into()).await?;
                requests_unsubscribe.insert(request_id, response_sender);
              },
              // Read message for other requests
              Some((method, params, response_sender)) = request_receiver.recv() => {
                request_id += 1;
                let text = json!({"jsonrpc":"2.0","id":request_id,"method":method,"params":params}).to_string();
                ws.send(text.into()).await?;
                other_requests.insert(request_id, response_sender);
              }
              // Read incoming WebSocket message
              next_msg = ws.next() => {
                let msg = match next_msg {
                  Some(msg) => msg?,
                  None => break,
                };

                // Reset unmatched_pings on any received frame
                unmatched_pings = 0;

                // Get text from the message
                let text = match msg {
                  Message::Text(text) => text,
                  Message::Binary(_data) => continue, // Ignore
                  Message::Ping(data) => {
                      ws.send(Message::Pong(data)).await?;
                      continue
                  },
                  Message::Pong(_data) => {
                    continue;
                  },
                  Message::Close(_frame) => break,
                  Message::Frame(_frame) => continue,
                };

                let unusable: FrameOutcome = handle_frame(
                    text.as_str(),
                    &mut other_requests,
                    &mut requests_unsubscribe,
                    &mut requests_subscribe,
                    &mut subscriptions,
                    &unsubscribe_sender,
                );

                match unusable {
                    Some(reason) => {
                        consecutive_unusable_frames += 1;
                        log::warn!(
                            "Ignoring websocket frame ({reason}); {consecutive_unusable_frames} consecutive unusable frame(s)"
                        );

                        // A run this long means the peer is broken rather than glitching. Close and
                        // fail loudly so the caller can reconnect, instead of spinning silently.
                        if consecutive_unusable_frames >= MAX_CONSECUTIVE_UNUSABLE_FRAMES {
                            let frame = CloseFrame {
                                code: CloseCode::Protocol,
                                reason: format!("{MAX_CONSECUTIVE_UNUSABLE_FRAMES} consecutive unusable frames").into(),
                            };
                            let _ = ws.send(Message::Close(Some(frame))).await;
                            let _ = ws.flush().await;

                            return Err(HeliusError::EnhancedWebsocket {
                                reason: format!(
                                    "{MAX_CONSECUTIVE_UNUSABLE_FRAMES} consecutive unusable frames; last: {reason}"
                                ),
                                message: text.as_str().to_string(),
                            });
                        }
                    }
                    None => consecutive_unusable_frames = 0,
                }
              }
            }
        }

        Ok(())
    }
}
