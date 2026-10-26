//! Helius **Parsed Streams** WebSocket client.
//!
//! [Parsed Streams](https://www.helius.dev/docs/parsed-streams) pushes confirmed transactions
//! that match a server-side filter (program, instruction name, account, account role), already
//! decoded through Helius's IDL catalog. It is the real-time counterpart to
//! [Parsed Events](crate::parsed_events) and the successor to enhanced transaction webhooks.
//! Available on all plans at 1 credit per delivered event.
//!
//! # Delivery
//!
//! Delivery is **at most once**: the server does not replay what confirmed while a connection
//! was down, and it closes connections routinely (10 minutes without a match, deploys, network
//! edge recycling). [`ParsedStreamsClient`] reconnects with backoff and resubscribes every open
//! filter on its own, and reports the last slot it delivered through
//! [`ParsedStreamsOptions::with_on_reconnect`] so you can backfill the gap from RPC.
//!
//! A subscription fails, rather than silently dropping data, when it cannot be kept alive: the
//! server closed the connection with a policy violation (close code 1008, sent when the client
//! fell more than 2048 notifications behind, or when the API key or plan changed),
//! [`SUBSCRIPTION_BUFFER_LIMIT`] notifications sit unread, the server rejects its filter on
//! resubscribe, or the reconnect attempts run out.
//!
//! # Transport & wire format
//!
//! JSON-RPC 2.0 over one WebSocket to the Gatekeeper endpoint
//! (`wss://beta.helius-rpc.com/?api-key=<KEY>`):
//! - `parsedTransactionSubscribe` — subscribe with a filter and options. The ack carries a
//!   numeric subscription id, which changes on every reconnect.
//! - `parsedTransactionUnsubscribe` — unsubscribe. Responds with a boolean.
//! - `parsedTransactionNotification` — one per matching transaction per subscription.
//!
//! Client messages are paced to the server's limit of 10 per second, and each connection holds
//! up to 25 subscriptions. The server drops a connection the client has not written to for 5
//! minutes, even while it streams, so the client sends a WebSocket ping every 30 seconds; the
//! pongs also tell it the connection is alive.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;
use std::future::{pending, Future};
use std::marker::PhantomData;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::task::{Context, Poll};

use futures_util::{
    future::{BoxFuture, FutureExt},
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
};
use serde::de::DeserializeOwned;
use serde_json::{json, Map, Value};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep_until, timeout, Duration, Instant};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::frame::{coding::CloseCode, CloseFrame},
        Error as WsError, Message,
    },
};

use crate::error::{HeliusError, Result};
use crate::parsed_events::ensure_mainnet;
use crate::types::parsed_streams::{
    ParsedStreamsReconnectInfo, ParsedTransactionFilter, ParsedTransactionNotification,
    ParsedTransactionSubscribeOptions, ParsedTransactionValue, RawTransactionValue,
};
use crate::types::ApiKey;
use crate::Helius;

/// Gatekeeper endpoint that serves Parsed Streams subscriptions. The API key is appended
pub const PARSED_STREAMS_URL: &str = "wss://beta.helius-rpc.com/?api-key=";

/// Unread notifications a subscription buffers before it fails as too slow
pub const SUBSCRIPTION_BUFFER_LIMIT: usize = 10_000;

const SUBSCRIBE_METHOD: &str = "parsedTransactionSubscribe";
const UNSUBSCRIBE_METHOD: &str = "parsedTransactionUnsubscribe";
const NOTIFICATION_METHOD: &str = "parsedTransactionNotification";

/// Gap between client messages. The server allows 10 per second
const MIN_SEND_INTERVAL: Duration = Duration::from_millis(100);
const BACKOFF_BASE: Duration = Duration::from_secs(1);
const BACKOFF_MAX: Duration = Duration::from_secs(30);
/// How long a request, or a connection handshake, may go unanswered
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
/// How often the client pings. The server drops a connection the client has not written to for
/// 5 minutes, even while it streams
const PING_INTERVAL: Duration = Duration::from_secs(30);
/// Silence after which the connection is presumed dead. Every frame counts, including the pongs
/// that answer the client's pings, since the server does not ping on its own
const LIVENESS_TIMEOUT: Duration = Duration::from_secs(60);
/// Uptime after a reconnect that restores the full reconnect budget
const STABLE_CONNECTION: Duration = Duration::from_secs(30);
/// How long a goodbye close frame may take to send
const CLOSE_TIMEOUT: Duration = Duration::from_secs(1);
const DEFAULT_MAX_RECONNECT_ATTEMPTS: u32 = 10;

/// Policy violation. The server sends it when the client fell more than 2048 notifications behind,
/// and also when the API key is deleted or the plan changes, so reconnecting would not help
const POLICY_VIOLATION_CLOSE_CODE: u16 = 1008;
/// Reported for a connection that dropped without a close frame
const ABNORMAL_CLOSE_CODE: u16 = 1006;
/// Reported for a close frame without a code
const NO_STATUS_CLOSE_CODE: u16 = 1005;
/// Server not ready, rate limited
const TRANSIENT_ERROR_CODES: [i64; 2] = [-32001, -32002];

/// Stored in a subscription's id cell while it has no server-assigned id
const NO_SUBSCRIPTION_ID: u64 = u64::MAX;

type WsResult<T> = std::result::Result<T, WsError>;

/// A WebSocket connection, abstracted so tests can substitute an in-memory one
trait Duplex: Stream<Item = WsResult<Message>> + Sink<Message, Error = WsError> + Send {}

impl<T> Duplex for T where T: Stream<Item = WsResult<Message>> + Sink<Message, Error = WsError> + Send {}

type Transport = Pin<Box<dyn Duplex>>;
type Connector = Arc<dyn Fn(&str) -> BoxFuture<'static, Result<Transport>> + Send + Sync>;

/// Called with what was missed once a reconnect restores the subscriptions
pub type ReconnectCallback = Arc<dyn Fn(ParsedStreamsReconnectInfo) + Send + Sync>;

/// Options for [`ParsedStreamsClient`]
///
/// This struct is `#[non_exhaustive]`: construct it via [`ParsedStreamsOptions::default`] and
/// the `with_*` builder methods rather than a struct literal, so that adding future fields
/// remains non-breaking.
#[derive(Clone)]
#[non_exhaustive]
pub struct ParsedStreamsOptions {
    /// Override the subscription endpoint (full URL, including the API key)
    pub url: Option<String>,
    /// Reconnect and resubscribe when the connection closes. Defaults to `true`
    pub reconnect: bool,
    /// Consecutive failed reconnect attempts before giving up. Defaults to 10
    pub max_reconnect_attempts: u32,
    /// Called after a reconnect restores the subscriptions
    pub on_reconnect: Option<ReconnectCallback>,
}

impl Default for ParsedStreamsOptions {
    fn default() -> Self {
        Self {
            url: None,
            reconnect: true,
            max_reconnect_attempts: DEFAULT_MAX_RECONNECT_ATTEMPTS,
            on_reconnect: None,
        }
    }
}

impl ParsedStreamsOptions {
    /// Overrides the subscription endpoint (full URL, including the API key)
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets whether to reconnect and resubscribe when the connection closes. With `false`, a
    /// close ends every subscription with an error
    pub fn with_reconnect(mut self, reconnect: bool) -> Self {
        self.reconnect = reconnect;
        self
    }

    /// Sets how many consecutive reconnect attempts may fail before every subscription ends with
    /// an error. The budget refills once a reconnected connection stays up for 30 seconds or
    /// delivers a notification
    pub fn with_max_reconnect_attempts(mut self, max_reconnect_attempts: u32) -> Self {
        self.max_reconnect_attempts = max_reconnect_attempts;
        self
    }

    /// Sets a callback for each reconnect that restores the subscriptions, with the close code
    /// that started the outage and the last slot delivered before it. Transactions after that
    /// slot that confirmed while disconnected were not delivered; backfill them from RPC if you
    /// need them.
    ///
    /// The callback runs on the client's background task, so keep it short and hand anything
    /// slow to another task. A panic in it is caught and logged
    pub fn with_on_reconnect(
        mut self,
        on_reconnect: impl Fn(ParsedStreamsReconnectInfo) + Send + Sync + 'static,
    ) -> Self {
        self.on_reconnect = Some(Arc::new(on_reconnect));
        self
    }
}

impl fmt::Debug for ParsedStreamsOptions {
    // The URL carries the API key, so only report whether it is set
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParsedStreamsOptions")
            .field("url", &self.url.as_ref().map(|_| "<set>"))
            .field("reconnect", &self.reconnect)
            .field("max_reconnect_attempts", &self.max_reconnect_attempts)
            .field("on_reconnect", &self.on_reconnect.as_ref().map(|_| "<callback>"))
            .finish()
    }
}

impl Helius {
    /// Returns a [Parsed Streams](https://www.helius.dev/docs/parsed-streams) client using this
    /// client's API key
    ///
    /// Each call returns a new client with its own connection, and projects have a cap on
    /// concurrent connections (5 on the Free plan), so create one and clone it rather than
    /// calling this per subscription. The client connects on its first subscription.
    ///
    /// # Errors
    /// Returns `HeliusError::InvalidInput` if no API key is configured or the client is not on
    /// mainnet. The network check is the one [`Helius::parsed_events`] uses, so a mainnet custom
    /// URL needs `.with_cluster(Cluster::MainnetBeta)` on `HeliusBuilder`
    pub fn parsed_streams(&self) -> Result<ParsedStreamsClient> {
        self.parsed_streams_with_options(ParsedStreamsOptions::default())
    }

    /// Returns a [Parsed Streams](https://www.helius.dev/docs/parsed-streams) client using this
    /// client's API key and the given options. See [`Helius::parsed_streams`]
    pub fn parsed_streams_with_options(&self, options: ParsedStreamsOptions) -> Result<ParsedStreamsClient> {
        let api_key = self.config.require_api_key("Parsed Streams")?;
        ensure_mainnet(&self.config, "Parsed Streams")?;

        ParsedStreamsClient::with_options(api_key.as_str(), options)
    }
}

/// Client for Helius [Parsed Streams](https://www.helius.dev/docs/parsed-streams)
///
/// One client is one WebSocket connection, opened on the first subscription and shared by every
/// subscription made through it (up to 25). Cloning is cheap and shares the connection. See the
/// [module docs](self) for delivery guarantees.
///
/// # Example
/// ```no_run
/// # use futures_util::StreamExt;
/// # use helius::error::Result;
/// # use helius::parsed_streams::ParsedStreamsClient;
/// # use helius::types::parsed_streams::ParsedTransactionFilter;
/// # async fn run() -> Result<()> {
/// let streams = ParsedStreamsClient::new("your_api_key")?;
///
/// let mut subscription = streams
///     .parsed_transaction_subscribe(
///         ParsedTransactionFilter {
///             programs: Some(vec!["JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string()]),
///             ..Default::default()
///         },
///         None,
///     )
///     .await?;
///
/// while let Some(notification) = subscription.next().await {
///     let value = notification?.value;
///     for &i in value.matched_indexes.as_deref().unwrap_or_default() {
///         let instruction = &value.instructions[i];
///         println!("{} {:?}", value.transaction.signature, instruction.instruction_name);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct ParsedStreamsClient {
    inner: Arc<Inner>,
}

struct Inner {
    commands: mpsc::UnboundedSender<Command>,
    /// The background task, until the first call that needs it starts it
    idle_actor: Mutex<Option<Actor>>,
    next_key: AtomicU64,
    closed: AtomicBool,
}

impl fmt::Debug for ParsedStreamsClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParsedStreamsClient")
            .field("closed", &self.inner.closed.load(Ordering::Acquire))
            .finish_non_exhaustive()
    }
}

impl ParsedStreamsClient {
    /// Creates a client for an API key. It connects on the first subscription
    ///
    /// # Errors
    /// Returns `HeliusError::InvalidInput` if the API key is empty
    pub fn new(api_key: &str) -> Result<Self> {
        Self::with_options(api_key, ParsedStreamsOptions::default())
    }

    /// Creates a client for an API key with the given options. It connects on the first
    /// subscription
    ///
    /// # Errors
    /// Returns `HeliusError::InvalidInput` if the API key is empty
    pub fn with_options(api_key: &str, options: ParsedStreamsOptions) -> Result<Self> {
        let api_key = ApiKey::new(api_key)?;
        let url = options
            .url
            .clone()
            .unwrap_or_else(|| format!("{PARSED_STREAMS_URL}{}", api_key.as_str()));

        Ok(Self::with_connector(url, options, websocket_connector()))
    }

    fn with_connector(url: String, options: ParsedStreamsOptions, connector: Connector) -> Self {
        let (commands, receiver) = mpsc::unbounded_channel();
        let actor = Actor::new(url, options, connector, receiver);

        Self {
            inner: Arc::new(Inner {
                commands,
                idle_actor: Mutex::new(Some(actor)),
                next_key: AtomicU64::new(0),
                closed: AtomicBool::new(false),
            }),
        }
    }

    /// Subscribes to decoded transactions matching `filter`
    ///
    /// Resolves once the server accepts the filter. The returned subscription is a [`Stream`] of
    /// notifications that survives reconnects; dropping it unsubscribes. Pass `None` for the
    /// default options (`confirmed`, `full` details).
    ///
    /// # Errors
    /// Returns `HeliusError::RpcError` if the server rejects the subscription (`-32602` for an
    /// unknown field or bad pubkey, `-32000` for a filter over its limits, `-32006` for more than
    /// 25 subscriptions on one connection), `HeliusError::WebsocketClosed` if the connection
    /// fails or closes before the server answers, or the client was closed
    pub async fn parsed_transaction_subscribe(
        &self,
        filter: ParsedTransactionFilter,
        options: Option<ParsedTransactionSubscribeOptions>,
    ) -> Result<ParsedTransactionSubscription> {
        let params: Value = match options {
            Some(options) => json!([filter, options]),
            None => json!([filter]),
        };
        self.subscribe(params).await
    }

    /// Subscribes to matching transactions with `raw` details: each matching instruction is
    /// reduced to its position, program, and base58 data, with no decoded fields, account keys,
    /// or transfers
    ///
    /// Use this when you decode instruction data yourself. Otherwise identical to
    /// [`parsed_transaction_subscribe`](Self::parsed_transaction_subscribe)
    pub async fn raw_transaction_subscribe(
        &self,
        filter: ParsedTransactionFilter,
    ) -> Result<ParsedTransactionSubscription<RawTransactionValue>> {
        self.subscribe(json!([filter, { "details": "raw" }])).await
    }

    /// Unsubscribes by current subscription id, ending that subscription's stream. Returns
    /// `true` if the server confirmed it
    ///
    /// Subscription ids change on every reconnect; read the current one with
    /// [`ParsedTransactionSubscription::subscription_id`]
    pub async fn parsed_transaction_unsubscribe(&self, subscription_id: u64) -> bool {
        self.start();
        let (reply, response) = oneshot::channel();
        if self
            .inner
            .commands
            .send(Command::UnsubscribeById { subscription_id, reply })
            .is_err()
        {
            return false;
        }
        response.await.unwrap_or(false)
    }

    /// Closes the connection and ends every subscription's stream. Later subscribe calls fail
    ///
    /// Affects every clone of this client. Does not wait for the close frame to be sent
    pub fn close(&self) {
        self.inner.closed.store(true, Ordering::Release);

        // Never started: dropping the task closes the command channel
        let idle = self
            .inner
            .idle_actor
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if idle.is_none() {
            let _ = self.inner.commands.send(Command::Close);
        }
    }

    async fn subscribe<V>(&self, params: Value) -> Result<ParsedTransactionSubscription<V>> {
        if self.inner.closed.load(Ordering::Acquire) {
            return Err(client_closed());
        }
        self.start();

        let key: u64 = self.inner.next_key.fetch_add(1, Ordering::Relaxed);
        // One slot beyond the limit stays free for the error that ends the stream
        let (events_sender, events) = mpsc::channel(SUBSCRIPTION_BUFFER_LIMIT + 1);
        let subscription_id = Arc::new(AtomicU64::new(NO_SUBSCRIPTION_ID));
        let (reply, response) = oneshot::channel();

        self.inner
            .commands
            .send(Command::Subscribe {
                key,
                params,
                events: events_sender,
                subscription_id: subscription_id.clone(),
                reply,
            })
            .map_err(|_| client_closed())?;
        response.await.map_err(|_| client_closed())??;

        Ok(ParsedTransactionSubscription {
            key,
            events,
            subscription_id,
            commands: self.inner.commands.clone(),
            active: true,
            _value: PhantomData,
        })
    }

    /// Spawns the background task on first use, so a client can be created outside a runtime
    fn start(&self) {
        let idle = self
            .inner
            .idle_actor
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(actor) = idle {
            tokio::spawn(actor.run());
        }
    }
}

/// An active Parsed Streams subscription
///
/// A [`Stream`] of notifications that survives reconnects. It yields:
/// - `Ok(notification)` for each matching transaction
/// - `Err(HeliusError::SerdeJson)` for a notification that did not decode into `V`. The stream
///   continues after it
/// - one final `Err` when the subscription cannot be kept alive (slow consumer, a policy close,
///   filter rejected on resubscribe, reconnect attempts exhausted, or reconnects turned off),
///   followed by `None`
///
/// It ends with `None` and no error when you unsubscribe or close the client. Dropping it
/// unsubscribes.
pub struct ParsedTransactionSubscription<V = ParsedTransactionValue> {
    key: u64,
    events: mpsc::Receiver<Result<Value>>,
    subscription_id: Arc<AtomicU64>,
    commands: mpsc::UnboundedSender<Command>,
    /// Cleared by `unsubscribe`, so dropping afterwards does not unsubscribe again
    active: bool,
    _value: PhantomData<fn() -> V>,
}

impl<V> ParsedTransactionSubscription<V> {
    /// Current server-assigned id, or `None` while reconnecting. It changes after a reconnect
    pub fn subscription_id(&self) -> Option<u64> {
        match self.subscription_id.load(Ordering::Relaxed) {
            NO_SUBSCRIPTION_ID => None,
            id => Some(id),
        }
    }

    /// Stops the subscription. Returns `true` if the server confirmed it
    pub async fn unsubscribe(mut self) -> bool {
        self.active = false;
        let (reply, response) = oneshot::channel();
        if self
            .commands
            .send(Command::Unsubscribe {
                key: self.key,
                reply: Some(reply),
            })
            .is_err()
        {
            return false;
        }
        response.await.unwrap_or(false)
    }
}

impl<V: DeserializeOwned> Stream for ParsedTransactionSubscription<V> {
    type Item = Result<ParsedTransactionNotification<V>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Decoded here, on the consumer's task, rather than on the shared connection task
        self.events.poll_recv(cx).map(|event| {
            event.map(|event| event.and_then(|value| serde_json::from_value(value).map_err(HeliusError::from)))
        })
    }
}

impl<V> Drop for ParsedTransactionSubscription<V> {
    fn drop(&mut self) {
        if self.active {
            let _ = self.commands.send(Command::Unsubscribe {
                key: self.key,
                reply: None,
            });
        }
    }
}

impl<V> fmt::Debug for ParsedTransactionSubscription<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParsedTransactionSubscription")
            .field("subscription_id", &self.subscription_id())
            .finish_non_exhaustive()
    }
}

fn websocket_connector() -> Connector {
    Arc::new(|url: &str| {
        let url: String = url.to_string();
        async move {
            let (ws, _response) = connect_async(url).await?;
            Ok(Box::pin(ws) as Transport)
        }
        .boxed()
    })
}

fn client_closed() -> HeliusError {
    HeliusError::WebsocketClosed("Parsed Streams client closed".to_string())
}

fn closed_reason(code: u16) -> String {
    format!("Parsed Streams connection closed (code {code})")
}

/// Like [`closed_reason`], with the reason text from the server's close frame when it sent one
fn closed_reason_with(code: u16, server_reason: &str) -> String {
    match server_reason.trim() {
        "" => closed_reason(code),
        server_reason => format!("Parsed Streams connection closed (code {code}: {server_reason})"),
    }
}

fn invalid_ack(result: &Value) -> HeliusError {
    HeliusError::RpcError {
        code: 0,
        message: format!("expected a subscription id, got {result}"),
    }
}

/// A JSON-RPC error object
struct RpcFailure {
    code: Option<i64>,
    message: String,
}

impl RpcFailure {
    fn from_value(error: &Value) -> Self {
        Self {
            code: error.get("code").and_then(Value::as_i64),
            message: error
                .get("message")
                .and_then(Value::as_str)
                .map_or_else(|| error.to_string(), str::to_string),
        }
    }

    /// Server not ready, rate limited, or an error without a code: worth retrying the reconnect
    fn is_transient(&self) -> bool {
        self.code.is_none_or(|code| TRANSIENT_ERROR_CODES.contains(&code))
    }
}

impl From<RpcFailure> for HeliusError {
    fn from(failure: RpcFailure) -> Self {
        HeliusError::RpcError {
            code: failure.code.unwrap_or(0),
            message: failure.message,
        }
    }
}

enum Command {
    Subscribe {
        key: u64,
        params: Value,
        events: mpsc::Sender<Result<Value>>,
        subscription_id: Arc<AtomicU64>,
        reply: oneshot::Sender<Result<()>>,
    },
    Unsubscribe {
        key: u64,
        reply: Option<oneshot::Sender<bool>>,
    },
    UnsubscribeById {
        subscription_id: u64,
        reply: oneshot::Sender<bool>,
    },
    Close,
}

/// A subscription the server has accepted at least once
struct SubState {
    /// `parsedTransactionSubscribe` params, resent on every reconnect
    params: Value,
    /// Id on the current connection, if it has been restored there
    server_id: Option<u64>,
    events: mpsc::Sender<Result<Value>>,
    /// Shared with the handle, which reports it
    subscription_id: Arc<AtomicU64>,
}

impl SubState {
    fn set_server_id(&mut self, server_id: Option<u64>) {
        self.server_id = server_id;
        self.subscription_id
            .store(server_id.unwrap_or(NO_SUBSCRIPTION_ID), Ordering::Relaxed);
    }

    /// Queues the error that ends the stream. It always fits: notifications leave a slot free
    fn fail(&self, error: HeliusError) {
        let _ = self.events.try_send(Err(error));
    }
}

enum Request {
    /// A new subscription, which joins `subs` once the server accepts it
    Subscribe {
        key: u64,
        sub: SubState,
        reply: oneshot::Sender<Result<()>>,
    },
    /// An existing subscription, restored after a reconnect
    Resubscribe {
        key: u64,
    },
    Unsubscribe {
        reply: Option<oneshot::Sender<bool>>,
    },
    /// A request that timed out. It stays pending so that a late subscribe ack, whose
    /// server-side subscription nothing will read, is still released
    Abandoned,
}

impl Request {
    /// Releases whoever waits on this request
    fn fail(self, error: HeliusError) {
        match self {
            Request::Subscribe { reply, .. } => {
                let _ = reply.send(Err(error));
            }
            // The subscription stays open and is resubscribed on the next connection
            Request::Resubscribe { .. } | Request::Abandoned => {}
            Request::Unsubscribe { reply } => {
                if let Some(reply) = reply {
                    let _ = reply.send(false);
                }
            }
        }
    }
}

struct Pending {
    method: &'static str,
    request: Request,
    /// Set once the request is sent, and cleared once it times out
    deadline: Option<Instant>,
}

/// The background task: owns the connection and every subscription's state
struct Actor {
    url: String,
    reconnect: bool,
    max_reconnect_attempts: u32,
    on_reconnect: Option<ReconnectCallback>,
    connector: Connector,
    commands: mpsc::UnboundedReceiver<Command>,

    ws: Option<Transport>,
    connecting: Option<BoxFuture<'static, Result<Transport>>>,
    last_frame_at: Instant,
    next_ping_at: Instant,

    next_request_id: u64,
    /// Requests waiting for their send slot
    outbox: VecDeque<(u64, String)>,
    next_send_at: Instant,
    pending: HashMap<u64, Pending>,

    /// Keyed by the handle's local key, so resubscribes go out in creation order
    subs: BTreeMap<u64, SubState>,
    by_server_id: HashMap<u64, u64>,

    last_slot: Option<u64>,
    attempts: u32,
    /// The close that started the current outage, reported once the subscriptions are restored
    outage: Option<ParsedStreamsReconnectInfo>,
    reconnect_at: Option<Instant>,
    stable_at: Option<Instant>,
    /// Resubscribe acks still outstanding on this connection
    restoring: usize,
}

impl Actor {
    fn new(
        url: String,
        options: ParsedStreamsOptions,
        connector: Connector,
        commands: mpsc::UnboundedReceiver<Command>,
    ) -> Self {
        let now = Instant::now();

        Self {
            url,
            reconnect: options.reconnect,
            max_reconnect_attempts: options.max_reconnect_attempts,
            on_reconnect: options.on_reconnect,
            connector,
            commands,
            ws: None,
            connecting: None,
            last_frame_at: now,
            next_ping_at: now,
            next_request_id: 0,
            outbox: VecDeque::new(),
            next_send_at: now,
            pending: HashMap::new(),
            subs: BTreeMap::new(),
            by_server_id: HashMap::new(),
            last_slot: None,
            attempts: 0,
            outage: None,
            reconnect_at: None,
            stable_at: None,
            restoring: 0,
        }
    }

    async fn run(mut self) {
        loop {
            let request_deadline: Option<Instant> = self.pending.values().filter_map(|pending| pending.deadline).min();
            let connected: bool = self.ws.is_some();
            let can_send: bool = connected && !self.outbox.is_empty();

            tokio::select! {
                command = self.commands.recv() => match command {
                    Some(Command::Close) => {
                        self.close().await;
                        return;
                    }
                    Some(command) => self.handle_command(command),
                    // Every client handle and subscription is gone
                    None => {
                        self.close().await;
                        return;
                    }
                },
                result = poll_option(&mut self.connecting) => {
                    self.connecting = None;
                    match result {
                        Ok(ws) => self.on_connected(ws),
                        Err(err) => {
                            log::warn!("Parsed Streams connection failed: {err}");
                            self.connection_lost(ABNORMAL_CLOSE_CODE, format!("Parsed Streams connection failed: {err}"));
                        }
                    }
                },
                frame = next_frame(&mut self.ws) => self.handle_frame(frame),
                () = sleep_until(self.next_send_at), if can_send => self.send_next().await,
                () = sleep_until(self.next_ping_at), if connected => self.send_ping().await,
                () = sleep_until_some(request_deadline) => self.time_out_requests(),
                () = sleep_until_some(self.reconnect_at) => {
                    self.reconnect_at = None;
                    if !self.subs.is_empty() {
                        self.ensure_connection();
                    }
                },
                () = sleep_until_some(self.stable_at) => {
                    self.stable_at = None;
                    self.attempts = 0;
                },
                () = sleep_until(self.last_frame_at + LIVENESS_TIMEOUT), if connected => {
                    log::warn!("Parsed Streams connection silent for {}s; reconnecting", LIVENESS_TIMEOUT.as_secs());
                    self.connection_lost(ABNORMAL_CLOSE_CODE, closed_reason(ABNORMAL_CLOSE_CODE));
                },
            }
        }
    }

    fn handle_command(&mut self, command: Command) {
        match command {
            Command::Subscribe {
                key,
                params,
                events,
                subscription_id,
                reply,
            } => {
                let sub = SubState {
                    params: params.clone(),
                    server_id: None,
                    events,
                    subscription_id,
                };
                self.request(SUBSCRIBE_METHOD, params, Request::Subscribe { key, sub, reply });
                // Mid-backoff, this also restores the existing subscriptions right away
                self.ensure_connection();
            }
            Command::Unsubscribe { key, reply } => self.unsubscribe(key, reply),
            Command::UnsubscribeById { subscription_id, reply } => match self.by_server_id.get(&subscription_id) {
                Some(&key) => self.unsubscribe(key, Some(reply)),
                None => {
                    let _ = reply.send(false);
                }
            },
            Command::Close => unreachable!("handled by run"),
        }
    }

    fn ensure_connection(&mut self) {
        if self.ws.is_some() || self.connecting.is_some() {
            return;
        }
        self.reconnect_at = None;

        let connect = (self.connector)(&self.url);
        self.connecting = Some(
            async move {
                // An upgrade that never answers would stall everything behind it
                timeout(REQUEST_TIMEOUT, connect).await.unwrap_or_else(|_| {
                    Err(HeliusError::WebsocketClosed(
                        "Parsed Streams handshake timed out".to_string(),
                    ))
                })
            }
            .boxed(),
        );
    }

    fn on_connected(&mut self, ws: Transport) {
        let now = Instant::now();
        self.ws = Some(ws);
        self.last_frame_at = now;
        self.next_send_at = now;
        self.next_ping_at = now + PING_INTERVAL;

        // Subscription ids do not survive a connection, so every open subscription is restored
        let restore: Vec<(u64, Value)> = self.subs.iter().map(|(key, sub)| (*key, sub.params.clone())).collect();
        self.restoring = restore.len();
        for (key, params) in restore {
            self.request(SUBSCRIBE_METHOD, params, Request::Resubscribe { key });
        }
    }

    fn handle_frame(&mut self, frame: Option<WsResult<Message>>) {
        let (code, reason): (u16, String) = match frame {
            Some(Ok(message)) => {
                self.last_frame_at = Instant::now();
                match message {
                    Message::Text(text) => {
                        self.handle_text(text.as_str());
                        return;
                    }
                    Message::Close(Some(frame)) => {
                        let code = u16::from(frame.code);
                        (code, closed_reason_with(code, frame.reason.as_str()))
                    }
                    Message::Close(None) => (NO_STATUS_CLOSE_CODE, closed_reason(NO_STATUS_CLOSE_CODE)),
                    // Pongs to the client's pings count toward liveness; tungstenite answers pings
                    _ => return,
                }
            }
            Some(Err(err)) => {
                log::warn!("Parsed Streams connection error: {err}");
                (ABNORMAL_CLOSE_CODE, closed_reason(ABNORMAL_CLOSE_CODE))
            }
            None => (ABNORMAL_CLOSE_CODE, closed_reason(ABNORMAL_CLOSE_CODE)),
        };

        self.connection_lost(code, reason);
    }

    fn handle_text(&mut self, text: &str) {
        let mut message: Map<String, Value> = match serde_json::from_str(text) {
            Ok(message) => message,
            Err(err) => {
                log::debug!("Ignoring unparseable Parsed Streams frame: {err}");
                return;
            }
        };

        if message.get("method").and_then(Value::as_str) == Some(NOTIFICATION_METHOD) {
            if let Some(Value::Object(params)) = message.remove("params") {
                self.handle_notification(params);
            }
            return;
        }

        let Some(id) = message.get("id").and_then(Value::as_u64) else {
            log::debug!("Ignoring unrecognized Parsed Streams frame: {text}");
            return;
        };
        let Some(pending) = self.pending.remove(&id) else {
            return;
        };

        let outcome = match message.get("error").filter(|error| !error.is_null()) {
            Some(error) => Err(RpcFailure::from_value(error)),
            None => Ok(message.remove("result").unwrap_or(Value::Null)),
        };
        self.handle_response(pending.request, outcome);
    }

    fn handle_notification(&mut self, mut params: Map<String, Value>) {
        let Some(server_id) = params.get("subscription").and_then(Value::as_u64) else {
            return;
        };
        let Some(result) = params.remove("result") else {
            return;
        };
        // Unsubscribed, or not yet restored on this connection
        let Some(&key) = self.by_server_id.get(&server_id) else {
            return;
        };
        let Some(sub) = self.subs.get(&key) else {
            return;
        };

        if let Some(slot) = result.pointer("/context/slot").and_then(Value::as_u64) {
            self.last_slot = Some(self.last_slot.map_or(slot, |last| last.max(slot)));
        }
        // Delivering: the connection works
        self.attempts = 0;

        if sub.events.capacity() > 1 {
            if sub.events.try_send(Ok(result)).is_ok() {
                return;
            }
            // Otherwise the handle was dropped
        } else {
            // The consumer stopped reading: fail loudly rather than drop data
            sub.fail(HeliusError::WebsocketClosed(format!(
                "Parsed Streams subscription buffered {SUBSCRIPTION_BUFFER_LIMIT} unread notifications; the consumer is too slow"
            )));
        }
        self.unsubscribe(key, None);
    }

    fn handle_response(&mut self, request: Request, outcome: std::result::Result<Value, RpcFailure>) {
        match request {
            Request::Subscribe { key, mut sub, reply } => {
                let id: u64 = match outcome {
                    Ok(result) => match result.as_u64() {
                        Some(id) => id,
                        None => {
                            let _ = reply.send(Err(invalid_ack(&result)));
                            return;
                        }
                    },
                    Err(failure) => {
                        let _ = reply.send(Err(failure.into()));
                        return;
                    }
                };

                if reply.is_closed() {
                    // The caller stopped waiting, so nothing will read this subscription
                    self.cancel(id);
                    return;
                }
                sub.set_server_id(Some(id));
                self.by_server_id.insert(id, key);
                self.subs.insert(key, sub);
                let _ = reply.send(Ok(()));
            }
            Request::Resubscribe { key } => {
                self.restoring = self.restoring.saturating_sub(1);

                match outcome {
                    Ok(result) => match (result.as_u64(), self.subs.get_mut(&key)) {
                        (Some(id), Some(sub)) => {
                            sub.set_server_id(Some(id));
                            self.by_server_id.insert(id, key);
                        }
                        // Unsubscribed while the ack was in flight
                        (Some(id), None) => self.cancel(id),
                        (None, _) => self.fail_subscription(key, invalid_ack(&result)),
                    },
                    Err(failure) if failure.is_transient() => {
                        // Retry the whole reconnect with backoff
                        self.connection_lost(ABNORMAL_CLOSE_CODE, closed_reason(ABNORMAL_CLOSE_CODE));
                        return;
                    }
                    // The filter is now rejected: fail just this subscription
                    Err(failure) => self.fail_subscription(key, failure.into()),
                }

                if self.restoring == 0 {
                    self.finish_restore();
                }
            }
            Request::Unsubscribe { reply } => {
                if let Some(reply) = reply {
                    let _ = reply.send(matches!(outcome, Ok(Value::Bool(true))));
                }
            }
            // A subscribe that timed out was accepted after all, and nothing will read it
            Request::Abandoned => {
                if let Some(id) = outcome.ok().and_then(|result| result.as_u64()) {
                    self.cancel(id);
                }
            }
        }
    }

    fn finish_restore(&mut self) {
        // Reset the budget only once stable, so a server that accepts and then closes still
        // exhausts it
        self.stable_at = Some(Instant::now() + STABLE_CONNECTION);

        let (Some(info), Some(on_reconnect)) = (self.outage.take(), &self.on_reconnect) else {
            return;
        };
        if catch_unwind(AssertUnwindSafe(|| on_reconnect(info))).is_err() {
            log::error!("Parsed Streams on_reconnect callback panicked; streaming continues");
        }
    }

    fn request(&mut self, method: &'static str, params: Value, request: Request) {
        self.next_request_id += 1;
        let id: u64 = self.next_request_id;
        let text: String = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string();

        self.outbox.push_back((id, text));
        self.pending.insert(
            id,
            Pending {
                method,
                request,
                deadline: None,
            },
        );
    }

    async fn send_next(&mut self) {
        let Some(ws) = self.ws.as_mut() else {
            return;
        };
        let Some((id, text)) = self.outbox.pop_front() else {
            return;
        };

        match timeout(REQUEST_TIMEOUT, ws.send(Message::Text(text.into()))).await {
            Ok(Ok(())) => {
                let now = Instant::now();
                self.next_send_at = now + MIN_SEND_INTERVAL;
                if let Some(pending) = self.pending.get_mut(&id) {
                    pending.deadline = Some(now + REQUEST_TIMEOUT);
                }
            }
            Ok(Err(err)) => {
                log::warn!("Parsed Streams send failed: {err}");
                self.connection_lost(ABNORMAL_CLOSE_CODE, closed_reason(ABNORMAL_CLOSE_CODE));
            }
            Err(_) => {
                log::warn!("Parsed Streams send timed out");
                self.connection_lost(ABNORMAL_CLOSE_CODE, closed_reason(ABNORMAL_CLOSE_CODE));
            }
        }
    }

    async fn send_ping(&mut self) {
        let Some(ws) = self.ws.as_mut() else {
            return;
        };

        match timeout(REQUEST_TIMEOUT, ws.send(Message::Ping(Default::default()))).await {
            Ok(Ok(())) => {
                let now = Instant::now();
                self.next_ping_at = now + PING_INTERVAL;
                // Count the ping against the message rate, in case the server does
                self.next_send_at = self.next_send_at.max(now + MIN_SEND_INTERVAL);
            }
            Ok(Err(err)) => {
                log::warn!("Parsed Streams ping failed: {err}");
                self.connection_lost(ABNORMAL_CLOSE_CODE, closed_reason(ABNORMAL_CLOSE_CODE));
            }
            Err(_) => {
                log::warn!("Parsed Streams ping timed out");
                self.connection_lost(ABNORMAL_CLOSE_CODE, closed_reason(ABNORMAL_CLOSE_CODE));
            }
        }
    }

    /// Fails each request that went unanswered, leaving the connection and every other
    /// subscription alone. A dead connection is caught by the liveness check instead
    fn time_out_requests(&mut self) {
        let now = Instant::now();
        let expired: Vec<u64> = self
            .pending
            .iter()
            .filter(|(_, pending)| pending.deadline.is_some_and(|deadline| deadline <= now))
            .map(|(id, _)| *id)
            .collect();

        for id in expired {
            let Some(pending) = self.pending.get_mut(&id) else {
                continue;
            };
            let method: &'static str = pending.method;
            pending.deadline = None;
            let request = std::mem::replace(&mut pending.request, Request::Abandoned);
            log::warn!("Parsed Streams {method} timed out");

            match request {
                // Retry on this connection. A late ack for the abandoned attempt is released
                Request::Resubscribe { key } => match self.subs.get(&key).map(|sub| sub.params.clone()) {
                    Some(params) => self.request(SUBSCRIBE_METHOD, params, Request::Resubscribe { key }),
                    // Unsubscribed meanwhile: nothing left to restore
                    None => {
                        self.restoring = self.restoring.saturating_sub(1);
                        if self.restoring == 0 {
                            self.finish_restore();
                        }
                    }
                },
                request => request.fail(HeliusError::WebsocketClosed(format!(
                    "Parsed Streams {method} timed out; the connection stays open"
                ))),
            }
        }
    }

    fn unsubscribe(&mut self, key: u64, reply: Option<oneshot::Sender<bool>>) {
        // Dropping the state ends the subscription's stream
        let server_id: Option<u64> = self.remove_sub(key).and_then(|sub| sub.server_id);

        match server_id {
            Some(id) if self.ws.is_some() => {
                self.request(UNSUBSCRIBE_METHOD, json!([id]), Request::Unsubscribe { reply })
            }
            _ => {
                if let Some(reply) = reply {
                    let _ = reply.send(false);
                }
            }
        }
    }

    /// Releases a server-side subscription that no handle will read
    fn cancel(&mut self, server_id: u64) {
        self.request(
            UNSUBSCRIBE_METHOD,
            json!([server_id]),
            Request::Unsubscribe { reply: None },
        );
    }

    fn fail_subscription(&mut self, key: u64, error: HeliusError) {
        if let Some(sub) = self.remove_sub(key) {
            sub.fail(error);
        }
    }

    fn remove_sub(&mut self, key: u64) -> Option<SubState> {
        let sub = self.subs.remove(&key)?;
        if let Some(id) = sub.server_id {
            self.by_server_id.remove(&id);
        }
        sub.subscription_id.store(NO_SUBSCRIPTION_ID, Ordering::Relaxed);

        // The next subscription starts with a full budget and no stale outage
        if self.subs.is_empty() {
            self.reset_reconnect_state();
        }
        Some(sub)
    }

    /// Drops the connection and, while subscriptions remain, schedules a reconnect or gives up
    fn connection_lost(&mut self, code: u16, reason: String) {
        self.ws = None;
        self.connecting = None;
        self.stable_at = None;
        self.restoring = 0;
        self.outbox.clear();
        for (_, pending) in self.pending.drain() {
            pending.request.fail(HeliusError::WebsocketClosed(reason.clone()));
        }
        self.by_server_id.clear();
        for sub in self.subs.values_mut() {
            sub.set_server_id(None);
        }

        if self.subs.is_empty() {
            return;
        }
        if code == POLICY_VIOLATION_CLOSE_CODE {
            self.end_all(Some(format!(
                "{reason}. The server sends 1008 when the client falls more than 2048 notifications \
                 behind, or when the API key or plan changes, so the client does not reconnect"
            )));
            return;
        }
        if !self.reconnect {
            self.end_all(Some(reason));
            return;
        }

        // The first close of an outage marks where the gap starts
        let last_slot: Option<u64> = self.last_slot;
        self.outage.get_or_insert(ParsedStreamsReconnectInfo {
            close_code: code,
            last_slot,
        });
        self.attempts += 1;
        if self.attempts > self.max_reconnect_attempts {
            self.end_all(Some(format!(
                "Parsed Streams gave up after {} reconnect attempts (last close code {code})",
                self.max_reconnect_attempts
            )));
            return;
        }

        let delay: Duration = BACKOFF_BASE
            .saturating_mul(2u32.saturating_pow(self.attempts - 1))
            .min(BACKOFF_MAX);
        self.reconnect_at = Some(Instant::now() + delay);
    }

    /// Ends every subscription, with `error` as its last item if given
    fn end_all(&mut self, error: Option<String>) {
        for (_, sub) in std::mem::take(&mut self.subs) {
            sub.subscription_id.store(NO_SUBSCRIPTION_ID, Ordering::Relaxed);
            if let Some(error) = &error {
                sub.fail(HeliusError::WebsocketClosed(error.clone()));
            }
        }
        self.by_server_id.clear();
        self.reset_reconnect_state();
    }

    fn reset_reconnect_state(&mut self) {
        self.attempts = 0;
        self.outage = None;
        self.reconnect_at = None;
    }

    async fn close(&mut self) {
        self.end_all(None);
        for (_, pending) in self.pending.drain() {
            pending.request.fail(client_closed());
        }
        self.outbox.clear();
        self.connecting = None;

        if let Some(mut ws) = self.ws.take() {
            let frame = CloseFrame {
                code: CloseCode::Normal,
                reason: "".into(),
            };
            let _ = timeout(CLOSE_TIMEOUT, ws.send(Message::Close(Some(frame)))).await;
        }
    }
}

async fn poll_option<F: Future + Unpin>(future: &mut Option<F>) -> F::Output {
    match future {
        Some(future) => future.await,
        None => pending().await,
    }
}

async fn next_frame(ws: &mut Option<Transport>) -> Option<WsResult<Message>> {
    match ws {
        Some(ws) => ws.next().await,
        None => pending().await,
    }
}

async fn sleep_until_some(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => sleep_until(deadline).await,
        None => pending().await,
    }
}

#[cfg(test)]
mod tests {
    //! Driven over an in-memory transport with tokio's paused clock, so reconnect backoff,
    //! pacing, and timeouts run deterministically and instantly.

    use super::*;
    use crate::types::parsed_streams::{ParsedStreamDetails, ParsedStreamTransactionStatus};
    use std::sync::atomic::AtomicUsize;
    use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender, WeakUnboundedSender};
    use tokio::task::JoinHandle;
    use tokio::time::sleep;

    /// Upper bound on any single wait, in virtual time. Reaching it means the test stalled
    const STALL: Duration = Duration::from_secs(600);

    struct MockTransport {
        incoming: UnboundedReceiver<WsResult<Message>>,
        outgoing: UnboundedSender<Message>,
        /// Answers the client's pings, as a real server does. Weak, so that dropping the
        /// server's end still ends the stream
        pong: Option<WeakUnboundedSender<WsResult<Message>>>,
    }

    impl Stream for MockTransport {
        type Item = WsResult<Message>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.incoming.poll_recv(cx)
        }
    }

    impl Sink<Message> for MockTransport {
        type Error = WsError;

        fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<WsResult<()>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, message: Message) -> WsResult<()> {
            if let (Message::Ping(payload), Some(pong)) = (&message, self.pong.as_ref().and_then(|pong| pong.upgrade()))
            {
                let _ = pong.send(Ok(Message::Pong(payload.clone())));
            }
            self.outgoing.send(message).map_err(|_| WsError::ConnectionClosed)
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<WsResult<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<WsResult<()>> {
            Poll::Ready(Ok(()))
        }
    }

    /// The server's end of one in-memory connection
    struct ServerConn {
        url: String,
        from_client: UnboundedReceiver<Message>,
        to_client: Option<UnboundedSender<WsResult<Message>>>,
    }

    impl ServerConn {
        /// The next JSON-RPC request the client sends
        async fn request(&mut self) -> Value {
            loop {
                let message = timeout(STALL, self.from_client.recv())
                    .await
                    .expect("client sent no request")
                    .expect("client hung up");
                if let Message::Text(text) = message {
                    return serde_json::from_str(text.as_str()).unwrap();
                }
            }
        }

        /// Whether the client has dropped this connection, after reading what it sent
        async fn hung_up(&mut self) -> bool {
            timeout(STALL, async { while self.from_client.recv().await.is_some() {} })
                .await
                .is_ok()
        }

        fn send(&self, value: Value) {
            if let Some(to_client) = &self.to_client {
                let _ = to_client.send(Ok(Message::Text(value.to_string().into())));
            }
        }

        fn reply(&self, request: &Value, result: Value) {
            self.send(json!({"jsonrpc": "2.0", "id": request["id"], "result": result}));
        }

        fn reply_error(&self, request: &Value, code: i64, message: &str) {
            self.send(json!({"jsonrpc": "2.0", "id": request["id"], "error": {"code": code, "message": message}}));
        }

        fn notify(&self, subscription: u64, slot: u64) {
            self.send(notification(subscription, slot));
        }

        /// A server-initiated close with `code`
        fn close(&mut self, code: u16) {
            self.close_with_reason(code, "");
        }

        fn close_with_reason(&mut self, code: u16, reason: &str) {
            if let Some(to_client) = self.to_client.take() {
                let frame = CloseFrame {
                    code: CloseCode::from(code),
                    reason: reason.into(),
                };
                let _ = to_client.send(Ok(Message::Close(Some(frame))));
            }
        }

        /// How many pings the client has sent, reading everything it sent so far
        fn pings_received(&mut self) -> usize {
            std::iter::from_fn(|| self.from_client.try_recv().ok())
                .filter(|message| matches!(message, Message::Ping(_)))
                .count()
        }
    }

    #[derive(Clone, Copy)]
    enum Plan {
        Open,
        /// Opens, but never answers the client's pings
        Mute,
        Fail,
        /// The upgrade never answers
        Hang,
    }

    struct Mock {
        connections: UnboundedReceiver<ServerConn>,
        plans: Arc<Mutex<VecDeque<Plan>>>,
        attempts: Arc<AtomicUsize>,
    }

    impl Mock {
        fn new() -> (Self, Connector) {
            let (connections_sender, connections) = unbounded_channel();
            let plans: Arc<Mutex<VecDeque<Plan>>> = Arc::default();
            let attempts: Arc<AtomicUsize> = Arc::default();

            let connector: Connector = {
                let plans = plans.clone();
                let attempts = attempts.clone();
                Arc::new(move |url: &str| {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    let plan = plans.lock().unwrap().pop_front().unwrap_or(Plan::Open);
                    let url = url.to_string();
                    let connections = connections_sender.clone();
                    async move {
                        match plan {
                            Plan::Open | Plan::Mute => {
                                let (to_client, incoming) = unbounded_channel();
                                let (outgoing, from_client) = unbounded_channel();
                                let pong = matches!(plan, Plan::Open).then(|| to_client.downgrade());
                                let _ = connections.send(ServerConn {
                                    url,
                                    from_client,
                                    to_client: Some(to_client),
                                });
                                Ok(Box::pin(MockTransport {
                                    incoming,
                                    outgoing,
                                    pong,
                                }) as Transport)
                            }
                            Plan::Fail => Err(HeliusError::WebsocketClosed("refused".to_string())),
                            Plan::Hang => pending().await,
                        }
                    }
                    .boxed()
                })
            };

            (
                Self {
                    connections,
                    plans,
                    attempts,
                },
                connector,
            )
        }

        fn plan(&self, plan: Plan, times: usize) {
            self.plans.lock().unwrap().extend(std::iter::repeat_n(plan, times));
        }

        fn attempts(&self) -> usize {
            self.attempts.load(Ordering::SeqCst)
        }

        async fn connection(&mut self) -> ServerConn {
            timeout(STALL, self.connections.recv())
                .await
                .expect("client never connected")
                .unwrap()
        }

        fn no_new_connection(&mut self) -> bool {
            self.connections.try_recv().is_err()
        }
    }

    fn client(options: ParsedStreamsOptions) -> (ParsedStreamsClient, Mock) {
        let (mock, connector) = Mock::new();
        let client = ParsedStreamsClient::with_connector("wss://stream/?api-key=KEY".to_string(), options, connector);
        (client, mock)
    }

    fn programs(program: &str) -> ParsedTransactionFilter {
        ParsedTransactionFilter {
            programs: Some(vec![program.to_string()]),
            ..Default::default()
        }
    }

    fn notification(subscription: u64, slot: u64) -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "parsedTransactionNotification",
            "params": {
                "subscription": subscription,
                "result": {
                    "context": {"slot": slot},
                    "value": {
                        "transaction": {
                            "signature": format!("sig-{slot}"),
                            "slot": slot,
                            "blockTime": null,
                            "feePayer": "payer",
                            "fee": 5000,
                            "accountKeys": ["payer"],
                            "status": "ok",
                            "error": null,
                            "summary": null,
                            "nativeTransfers": [],
                            "tokenTransfers": []
                        },
                        "instructions": [],
                        "matchedIndexes": []
                    }
                }
            }
        })
    }

    fn spawn_subscribe(
        client: &ParsedStreamsClient,
        program: &str,
    ) -> JoinHandle<Result<ParsedTransactionSubscription>> {
        let client = client.clone();
        let filter = programs(program);
        tokio::spawn(async move { client.parsed_transaction_subscribe(filter, None).await })
    }

    /// Subscribes on an open connection and acks with `id`
    async fn subscribed(
        client: &ParsedStreamsClient,
        conn: &mut ServerConn,
        program: &str,
        id: u64,
    ) -> ParsedTransactionSubscription {
        let pending = spawn_subscribe(client, program);
        let request = conn.request().await;
        conn.reply(&request, json!(id));
        pending.await.unwrap().unwrap()
    }

    /// Makes the first subscription, which opens the connection
    async fn first_subscription(
        client: &ParsedStreamsClient,
        mock: &mut Mock,
        id: u64,
    ) -> (ServerConn, ParsedTransactionSubscription) {
        let pending = spawn_subscribe(client, "P1");
        let mut conn = mock.connection().await;
        let request = conn.request().await;
        conn.reply(&request, json!(id));
        (conn, pending.await.unwrap().unwrap())
    }

    async fn next<V: DeserializeOwned>(
        sub: &mut ParsedTransactionSubscription<V>,
    ) -> Option<Result<ParsedTransactionNotification<V>>> {
        timeout(STALL, sub.next()).await.expect("stream stalled")
    }

    async fn next_slot(sub: &mut ParsedTransactionSubscription) -> u64 {
        next(sub)
            .await
            .expect("stream ended")
            .expect("stream failed")
            .context
            .slot
    }

    async fn next_error(sub: &mut ParsedTransactionSubscription) -> String {
        match next(sub).await {
            Some(Err(err)) => err.to_string(),
            other => panic!("expected an error, got {other:?}"),
        }
    }

    /// Advances virtual time in small steps until `condition` holds
    async fn until(mut condition: impl FnMut() -> bool) {
        for _ in 0..60_000 {
            if condition() {
                return;
            }
            sleep(Duration::from_millis(10)).await;
        }
        panic!("condition not met");
    }

    fn recorder() -> (ParsedStreamsOptions, Arc<Mutex<Vec<ParsedStreamsReconnectInfo>>>) {
        let calls: Arc<Mutex<Vec<ParsedStreamsReconnectInfo>>> = Arc::default();
        let sink = calls.clone();
        let options = ParsedStreamsOptions::default().with_on_reconnect(move |info| sink.lock().unwrap().push(info));
        (options, calls)
    }

    // ── Subscribe / notifications / unsubscribe ───────────────────────

    #[tokio::test(start_paused = true)]
    async fn sends_the_filter_and_options_to_the_subscription_url() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let pending = {
            let client = client.clone();
            tokio::spawn(async move {
                client
                    .parsed_transaction_subscribe(
                        ParsedTransactionFilter {
                            programs: Some(vec!["JUP".to_string()]),
                            include_cpi: Some(false),
                            ..Default::default()
                        },
                        Some(ParsedTransactionSubscribeOptions {
                            details: Some(ParsedStreamDetails::Matched),
                            ..Default::default()
                        }),
                    )
                    .await
            })
        };

        let mut conn = mock.connection().await;
        assert_eq!(conn.url, "wss://stream/?api-key=KEY");
        let request = conn.request().await;
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "parsedTransactionSubscribe");
        assert_eq!(
            request["params"],
            json!([{"programs": ["JUP"], "includeCpi": false}, {"details": "matched"}])
        );

        conn.reply(&request, json!(23));
        let sub = pending.await.unwrap().unwrap();
        assert_eq!(sub.subscription_id(), Some(23));
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn omits_the_options_param_when_none_are_given() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let pending = spawn_subscribe(&client, "P1");
        let mut conn = mock.connection().await;
        let request = conn.request().await;
        assert_eq!(request["params"], json!([{"programs": ["P1"]}]));
        conn.reply(&request, json!(1));
        pending.await.unwrap().unwrap();
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn delivers_its_own_notifications_in_order() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (conn, mut sub) = first_subscription(&client, &mut mock, 7).await;

        conn.notify(99, 1); // another subscription's id: ignored
        conn.notify(7, 10);
        conn.notify(7, 11);

        let first = next(&mut sub).await.unwrap().unwrap();
        assert_eq!(first.context.slot, 10);
        assert_eq!(first.value.transaction.signature, "sig-10");
        assert_eq!(first.value.transaction.status, ParsedStreamTransactionStatus::Ok);
        assert_eq!(next_slot(&mut sub).await, 11);
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn yields_a_decode_error_and_keeps_streaming() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (conn, mut sub) = first_subscription(&client, &mut mock, 7).await;

        conn.send(json!({
            "jsonrpc": "2.0",
            "method": "parsedTransactionNotification",
            "params": {"subscription": 7, "result": {"context": {"slot": 1}, "value": {"unexpected": true}}}
        }));
        conn.notify(7, 2);

        assert!(matches!(next(&mut sub).await, Some(Err(HeliusError::SerdeJson(_)))));
        assert_eq!(next_slot(&mut sub).await, 2);
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn raw_subscribe_requests_raw_details_and_decodes_them() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let pending = {
            let client = client.clone();
            tokio::spawn(async move { client.raw_transaction_subscribe(programs("JUP")).await })
        };
        let mut conn = mock.connection().await;
        let request = conn.request().await;
        assert_eq!(request["params"], json!([{"programs": ["JUP"]}, {"details": "raw"}]));
        conn.reply(&request, json!(4));
        let mut sub = pending.await.unwrap().unwrap();

        conn.send(json!({
            "jsonrpc": "2.0",
            "method": "parsedTransactionNotification",
            "params": {
                "subscription": 4,
                "result": {
                    "context": {"slot": 430172053},
                    "value": {
                        "transaction": {
                            "signature": "sig", "slot": 430172053, "blockTime": null, "feePayer": "payer",
                            "fee": 5000, "status": "ok", "error": null, "summary": null
                        },
                        "instructions": [{
                            "instructionIndex": 4, "innerInstructionIndex": null, "stackHeight": 1,
                            "programId": "JUP", "data": "3Bxs4h24hBtQy9rw"
                        }]
                    }
                }
            }
        }));

        let value = next(&mut sub).await.unwrap().unwrap().value;
        assert_eq!(value.instructions[0].data, "3Bxs4h24hBtQy9rw");
        assert_eq!(value.instructions[0].instruction_index, 4);
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn returns_the_json_rpc_error_when_the_filter_is_rejected() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let pending = spawn_subscribe(&client, "bad");
        let mut conn = mock.connection().await;
        let request = conn.request().await;
        conn.reply_error(&request, -32602, "invalid pubkey at programs[0]");

        match pending.await.unwrap() {
            Err(HeliusError::RpcError { code, message }) => {
                assert_eq!(code, -32602);
                assert_eq!(message, "invalid pubkey at programs[0]");
            }
            other => panic!("expected an RPC error, got {other:?}"),
        }
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn unsubscribe_sends_the_id_and_ends_the_stream() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, sub) = first_subscription(&client, &mut mock, 5).await;
        assert_eq!(sub.subscription_id(), Some(5));
        let ids = sub.subscription_id.clone();

        let pending = tokio::spawn(sub.unsubscribe());
        let request = conn.request().await;
        assert_eq!(request["method"], "parsedTransactionUnsubscribe");
        assert_eq!(request["params"], json!([5]));
        conn.reply(&request, json!(true));

        assert!(pending.await.unwrap());
        assert_eq!(ids.load(Ordering::Relaxed), NO_SUBSCRIPTION_ID);
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn unsubscribe_by_id_ends_that_stream() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, mut sub) = first_subscription(&client, &mut mock, 5).await;

        let pending = {
            let client = client.clone();
            tokio::spawn(async move { client.parsed_transaction_unsubscribe(5).await })
        };
        let request = conn.request().await;
        assert_eq!(request["params"], json!([5]));
        conn.reply(&request, json!(true));

        assert!(pending.await.unwrap());
        assert!(next(&mut sub).await.is_none());
        assert!(!client.parsed_transaction_unsubscribe(404).await);
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn dropping_the_subscription_unsubscribes() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, sub) = first_subscription(&client, &mut mock, 3).await;
        drop(sub);

        let request = conn.request().await;
        assert_eq!(request["method"], "parsedTransactionUnsubscribe");
        assert_eq!(request["params"], json!([3]));
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn paces_client_messages_under_ten_per_second() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let _a = spawn_subscribe(&client, "A");
        let _b = spawn_subscribe(&client, "B");
        let mut conn = mock.connection().await;
        conn.request().await;

        sleep(Duration::from_millis(50)).await;
        assert!(conn.from_client.try_recv().is_err(), "second message sent too soon");
        sleep(Duration::from_millis(60)).await;
        assert!(
            conn.from_client.try_recv().is_ok(),
            "second message not sent after 100ms"
        );
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn fails_a_request_whose_connection_closes_before_it_is_sent() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let a = spawn_subscribe(&client, "A");
        let b = spawn_subscribe(&client, "B");
        let mut conn = mock.connection().await;
        let request = conn.request().await;
        conn.reply(&request, json!(1));
        let _a = a.await.unwrap().unwrap();

        conn.close(1001); // b is still waiting for its send slot
        let err = b.await.unwrap().unwrap_err().to_string();
        assert!(err.contains("connection closed (code 1001)"), "{err}");
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn delivers_what_it_buffered_then_fails_a_consumer_that_falls_behind() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, mut sub) = first_subscription(&client, &mut mock, 2).await;
        for slot in 0..=SUBSCRIPTION_BUFFER_LIMIT as u64 {
            conn.notify(2, slot);
        }

        // The overflowing subscription is unsubscribed server-side. Waiting for that first keeps
        // the consumer from reading before the buffer fills
        let request = conn.request().await;
        assert_eq!(request["method"], "parsedTransactionUnsubscribe");
        assert_eq!(request["params"], json!([2]));

        for slot in 0..SUBSCRIPTION_BUFFER_LIMIT as u64 {
            assert_eq!(next_slot(&mut sub).await, slot);
        }
        let err = next_error(&mut sub).await;
        assert!(err.contains("consumer is too slow"), "{err}");
        assert!(next(&mut sub).await.is_none());
        client.close();
    }

    // ── Reconnects ────────────────────────────────────────────────────

    #[tokio::test(start_paused = true)]
    async fn reconnects_resubscribes_and_keeps_the_stream_alive() {
        for code in [1000, 1001, 1006] {
            let (options, calls) = recorder();
            let (client, mut mock) = client(options);
            let (mut conn, mut sub) = first_subscription(&client, &mut mock, 1).await;
            conn.notify(1, 100);
            assert_eq!(next_slot(&mut sub).await, 100);

            conn.close(code);
            let mut conn = mock.connection().await;
            let request = conn.request().await;
            assert_eq!(request["method"], "parsedTransactionSubscribe");
            assert_eq!(request["params"], json!([{"programs": ["P1"]}]));
            conn.reply(&request, json!(42));

            until(|| calls.lock().unwrap().len() == 1).await;
            assert_eq!(
                calls.lock().unwrap()[0],
                ParsedStreamsReconnectInfo {
                    close_code: code,
                    last_slot: Some(100)
                }
            );
            assert_eq!(sub.subscription_id(), Some(42));

            conn.notify(42, 105);
            assert_eq!(next_slot(&mut sub).await, 105);
            client.close();
        }
    }

    #[tokio::test(start_paused = true)]
    async fn fails_the_stream_on_a_policy_close_without_reconnecting() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, mut sub) = first_subscription(&client, &mut mock, 1).await;
        conn.close_with_reason(1008, "api key deleted");

        // Not only slow consumers get 1008, so the server's reason is passed along
        let err = next_error(&mut sub).await;
        assert!(err.contains("code 1008: api key deleted"), "{err}");
        assert!(err.contains("or when the API key or plan changes"), "{err}");
        assert!(next(&mut sub).await.is_none());
        sleep(Duration::from_secs(60)).await;
        assert!(mock.no_new_connection());
    }

    #[tokio::test(start_paused = true)]
    async fn reports_a_policy_close_without_a_reason() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, mut sub) = first_subscription(&client, &mut mock, 1).await;
        conn.close(1008);

        let err = next_error(&mut sub).await;
        assert!(err.contains("(code 1008)."), "{err}");
    }

    #[tokio::test(start_paused = true)]
    async fn fails_the_stream_instead_of_reconnecting_when_reconnect_is_off() {
        let (client, mut mock) = client(ParsedStreamsOptions::default().with_reconnect(false));
        let (mut conn, mut sub) = first_subscription(&client, &mut mock, 1).await;
        conn.close(1001);

        let err = next_error(&mut sub).await;
        assert!(err.contains("closed (code 1001)"), "{err}");
        sleep(Duration::from_secs(60)).await;
        assert!(mock.no_new_connection());
    }

    #[tokio::test(start_paused = true)]
    async fn gives_up_after_max_reconnect_attempts() {
        let (client, mut mock) = client(ParsedStreamsOptions::default().with_max_reconnect_attempts(3));
        let (mut conn, mut sub) = first_subscription(&client, &mut mock, 1).await;
        mock.plan(Plan::Fail, 3);
        conn.close(1006);

        let err = next_error(&mut sub).await;
        assert!(err.contains("gave up after 3 reconnect attempts"), "{err}");
        assert_eq!(mock.attempts(), 4); // original + 3 failed attempts
        sleep(Duration::from_secs(60)).await;
        assert_eq!(mock.attempts(), 4);
    }

    #[tokio::test(start_paused = true)]
    async fn backs_off_exponentially_between_failed_attempts() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, _sub) = first_subscription(&client, &mut mock, 1).await;
        mock.plan(Plan::Fail, 2);
        conn.close(1006);

        sleep(Duration::from_millis(990)).await;
        assert_eq!(mock.attempts(), 1);
        sleep(Duration::from_millis(20)).await; // 1s: attempt 1 (fails)
        assert_eq!(mock.attempts(), 2);
        sleep(Duration::from_millis(1_980)).await;
        assert_eq!(mock.attempts(), 2);
        sleep(Duration::from_millis(40)).await; // +2s: attempt 2 (fails)
        assert_eq!(mock.attempts(), 3);
        sleep(Duration::from_millis(3_950)).await;
        assert_eq!(mock.attempts(), 3);
        sleep(Duration::from_millis(60)).await; // +4s: attempt 3 (opens)
        assert_eq!(mock.attempts(), 4);
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn fails_only_the_subscription_the_server_now_rejects() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, mut a) = first_subscription(&client, &mut mock, 1).await;
        let mut b = subscribed(&client, &mut conn, "B", 2).await;
        conn.close(1001);

        let mut conn = mock.connection().await;
        let request = conn.request().await;
        assert_eq!(request["params"][0]["programs"], json!(["P1"]));
        conn.reply_error(&request, -32602, "unknown field");
        let request = conn.request().await;
        assert_eq!(request["params"][0]["programs"], json!(["B"]));
        conn.reply(&request, json!(20));

        match next(&mut a).await {
            Some(Err(HeliusError::RpcError { code: -32602, message })) => assert_eq!(message, "unknown field"),
            other => panic!("expected the rejection, got {other:?}"),
        }
        assert!(next(&mut a).await.is_none());

        until(|| b.subscription_id() == Some(20)).await;
        conn.notify(20, 7);
        assert_eq!(next_slot(&mut b).await, 7);
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn retries_the_reconnect_when_resubscribing_hits_a_transient_error() {
        for error in [
            json!({"code": -32001, "message": "server not ready"}),
            json!({"code": -32002, "message": "rate limited"}),
            json!({"message": "busy"}), // no code: treated as transient
        ] {
            let (client, mut mock) = client(ParsedStreamsOptions::default());
            let (mut conn, sub) = first_subscription(&client, &mut mock, 1).await;
            conn.close(1001);

            let mut conn = mock.connection().await;
            let request = conn.request().await;
            conn.send(json!({"jsonrpc": "2.0", "id": request["id"], "error": error}));
            assert!(conn.hung_up().await);

            let mut conn = mock.connection().await;
            let request = conn.request().await;
            conn.reply(&request, json!(9));
            until(|| sub.subscription_id() == Some(9)).await;
            client.close();
        }
    }

    #[tokio::test(start_paused = true)]
    async fn does_not_reconnect_when_no_subscriptions_are_open() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, sub) = first_subscription(&client, &mut mock, 1).await;
        let pending = tokio::spawn(sub.unsubscribe());
        let request = conn.request().await;
        conn.reply(&request, json!(true));
        assert!(pending.await.unwrap());

        conn.close(1000);
        sleep(Duration::from_secs(60)).await;
        assert!(mock.no_new_connection());
        assert_eq!(mock.attempts(), 1);
    }

    // ── Lifecycle edge cases ──────────────────────────────────────────

    #[tokio::test(start_paused = true)]
    async fn cancels_a_subscription_unsubscribed_while_its_resubscribe_ack_is_in_flight() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, sub) = first_subscription(&client, &mut mock, 1).await;
        conn.close(1001);
        let mut conn = mock.connection().await;
        let resubscribe = conn.request().await;

        // Before the server acks the resubscribe, so there is no id to unsubscribe yet
        assert!(!sub.unsubscribe().await);
        conn.reply(&resubscribe, json!(77));

        let request = conn.request().await;
        assert_eq!(request["method"], "parsedTransactionUnsubscribe");
        assert_eq!(request["params"], json!([77]));
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn keeps_streaming_when_on_reconnect_panics() {
        let options = ParsedStreamsOptions::default().with_on_reconnect(|_| panic!("user callback bug"));
        let (client, mut mock) = client(options);
        let (mut conn, mut sub) = first_subscription(&client, &mut mock, 1).await;
        conn.close(1000);
        let mut conn = mock.connection().await;
        let request = conn.request().await;
        conn.reply(&request, json!(2));
        until(|| sub.subscription_id() == Some(2)).await;

        conn.notify(2, 50);
        assert_eq!(next_slot(&mut sub).await, 50);
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn times_out_only_the_unanswered_request() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, mut sub) = first_subscription(&client, &mut mock, 1).await;

        let hung = spawn_subscribe(&client, "B");
        let late = conn.request().await;
        let err = hung.await.unwrap().unwrap_err().to_string();
        assert!(err.contains("parsedTransactionSubscribe timed out"), "{err}");

        // The connection and the existing subscription carry on
        conn.notify(1, 5);
        assert_eq!(next_slot(&mut sub).await, 5);
        assert!(mock.no_new_connection());
        assert_eq!(mock.attempts(), 1);

        // An ack that arrives after all creates a subscription nothing reads, so it is released
        conn.reply(&late, json!(9));
        let request = conn.request().await;
        assert_eq!(request["method"], "parsedTransactionUnsubscribe");
        assert_eq!(request["params"], json!([9]));
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn resends_a_resubscribe_that_times_out_on_the_same_connection() {
        let (options, calls) = recorder();
        let (client, mut mock) = client(options);
        let (mut conn, sub) = first_subscription(&client, &mut mock, 1).await;
        conn.close(1001);

        let mut conn = mock.connection().await;
        let first = conn.request().await; // never answered in time
        let second = conn.request().await;
        assert_eq!(second["method"], "parsedTransactionSubscribe");
        assert_eq!(second["params"], first["params"]);
        assert_ne!(second["id"], first["id"]);

        // The first attempt's late ack is released, and the retry restores the subscription
        conn.reply(&first, json!(70));
        let request = conn.request().await;
        assert_eq!(request["method"], "parsedTransactionUnsubscribe");
        assert_eq!(request["params"], json!([70]));
        conn.reply(&second, json!(71));
        until(|| sub.subscription_id() == Some(71)).await;
        until(|| calls.lock().unwrap().len() == 1).await;
        assert_eq!(mock.attempts(), 2);
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn restores_existing_subscriptions_right_away_when_a_new_subscribe_lands_mid_backoff() {
        let (options, calls) = recorder();
        let (client, mut mock) = client(options);
        let (mut conn, mut old) = first_subscription(&client, &mut mock, 1).await;
        conn.notify(1, 100);
        assert_eq!(next_slot(&mut old).await, 100);

        mock.plan(Plan::Fail, 3); // Pushes the next backoff out to 8s
        conn.close(1006);
        until(|| mock.attempts() == 4).await;

        let start = Instant::now();
        let fresh = spawn_subscribe(&client, "NEW");
        let mut conn = mock.connection().await;
        let first = conn.request().await;
        let second = conn.request().await;
        assert!(start.elapsed() < Duration::from_secs(1), "waited for the backoff timer");

        for request in [&first, &second] {
            let id = if request["params"][0]["programs"] == json!(["P1"]) {
                10
            } else {
                11
            };
            conn.reply(request, json!(id));
        }
        let fresh = fresh.await.unwrap().unwrap();
        assert_eq!(fresh.subscription_id(), Some(11));
        until(|| calls.lock().unwrap().len() == 1).await;
        // The gap starts at the slot before the outage, not a later one
        assert_eq!(
            calls.lock().unwrap()[0],
            ParsedStreamsReconnectInfo {
                close_code: 1006,
                last_slot: Some(100)
            }
        );
        assert_eq!(old.subscription_id(), Some(10));
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn treats_unanswered_pings_as_a_dead_connection() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        mock.plan(Plan::Mute, 1);
        let (mut conn, sub) = first_subscription(&client, &mut mock, 1).await;

        let start = Instant::now();
        assert!(conn.hung_up().await);
        assert!(start.elapsed() >= LIVENESS_TIMEOUT);

        let mut conn = mock.connection().await;
        let request = conn.request().await;
        conn.reply(&request, json!(2));
        until(|| sub.subscription_id() == Some(2)).await;
        client.close();
    }

    /// The server sends no pings and drops a connection the client has not written to for 5
    /// minutes, so a quiet filter stays up only on the client's own pings and their pongs
    #[tokio::test(start_paused = true)]
    async fn keeps_a_quiet_connection_alive_with_pings() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, _sub) = first_subscription(&client, &mut mock, 1).await;

        sleep(PING_INTERVAL - Duration::from_millis(10)).await;
        assert_eq!(conn.pings_received(), 0);
        sleep(Duration::from_millis(20)).await;
        assert_eq!(conn.pings_received(), 1);

        sleep(Duration::from_secs(10 * 60)).await;
        assert_eq!(conn.pings_received(), 20);
        assert!(mock.no_new_connection());
        assert_eq!(mock.attempts(), 1);
        client.close();
    }

    // ── Reconnect budget ──────────────────────────────────────────────

    #[tokio::test(start_paused = true)]
    async fn starts_a_fresh_budget_for_subscriptions_made_after_a_give_up() {
        let (options, calls) = recorder();
        let (client, mut mock) = client(options.with_max_reconnect_attempts(2));
        let (mut conn, mut first) = first_subscription(&client, &mut mock, 1).await;
        conn.notify(1, 100);
        assert_eq!(next_slot(&mut first).await, 100);

        mock.plan(Plan::Fail, 2);
        conn.close(1006);
        let err = next_error(&mut first).await;
        assert!(err.contains("gave up after 2 reconnect attempts"), "{err}");

        // Same client, new subscription, routine idle close: it must reconnect
        let (mut conn, mut second) = first_subscription(&client, &mut mock, 5).await;
        conn.notify(5, 200);
        assert_eq!(next_slot(&mut second).await, 200);
        conn.close(1000);

        let mut conn = mock.connection().await;
        let request = conn.request().await;
        conn.reply(&request, json!(6));
        until(|| calls.lock().unwrap().len() == 1).await;
        // No stale outage from the first subscription
        assert_eq!(
            calls.lock().unwrap()[0],
            ParsedStreamsReconnectInfo {
                close_code: 1000,
                last_slot: Some(200)
            }
        );
        assert_eq!(second.subscription_id(), Some(6));
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn gives_up_on_a_server_that_accepts_the_resubscribe_and_immediately_closes() {
        let (client, mut mock) = client(ParsedStreamsOptions::default().with_max_reconnect_attempts(3));
        let (mut conn, mut sub) = first_subscription(&client, &mut mock, 1).await;

        conn.close(1006);
        for i in 0..3 {
            let mut conn = mock.connection().await;
            let request = conn.request().await;
            conn.reply(&request, json!(10 + i));
            until(|| sub.subscription_id() == Some(10 + i)).await;
            conn.close(1006); // Flap before the connection proves stable
        }

        let err = next_error(&mut sub).await;
        assert!(err.contains("gave up after 3 reconnect attempts"), "{err}");
        assert_eq!(mock.attempts(), 4); // original + 3 attempts, then no more
    }

    #[tokio::test(start_paused = true)]
    async fn resets_the_budget_after_a_stable_or_delivering_connection() {
        for deliver in [false, true] {
            let (client, mut mock) = client(ParsedStreamsOptions::default().with_max_reconnect_attempts(1));
            let (mut conn, sub) = first_subscription(&client, &mut mock, 1).await;

            // Each reconnect uses the single allowed attempt, then proves stable
            for i in 0..3 {
                conn.close(1006);
                conn = mock.connection().await;
                let request = conn.request().await;
                conn.reply(&request, json!(100 + i));
                until(|| sub.subscription_id() == Some(100 + i)).await;
                if deliver {
                    conn.notify(100 + i, 1);
                } else {
                    sleep(STABLE_CONNECTION + Duration::from_millis(10)).await;
                }
            }
            assert_eq!(sub.subscription_id(), Some(102));
            client.close();
        }
    }

    // ── Handshake timeout ─────────────────────────────────────────────

    #[tokio::test(start_paused = true)]
    async fn fails_a_subscribe_whose_upgrade_never_answers_without_blocking_later_ones() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        mock.plan(Plan::Hang, 1);
        let err = spawn_subscribe(&client, "A").await.unwrap().unwrap_err().to_string();
        assert!(err.contains("handshake timed out"), "{err}");

        first_subscription(&client, &mut mock, 1).await;
        assert_eq!(mock.attempts(), 2);
        client.close();
    }

    #[tokio::test(start_paused = true)]
    async fn counts_a_hung_reconnect_as_a_failed_attempt_and_retries() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, sub) = first_subscription(&client, &mut mock, 1).await;
        mock.plan(Plan::Hang, 1);
        conn.close(1001);

        let mut conn = mock.connection().await;
        assert_eq!(mock.attempts(), 3); // original, the hung one, this one
        let request = conn.request().await;
        conn.reply(&request, json!(2));
        until(|| sub.subscription_id() == Some(2)).await;
        client.close();
    }

    // ── close() ───────────────────────────────────────────────────────

    #[tokio::test(start_paused = true)]
    async fn close_ends_every_stream_cleanly_and_never_reconnects() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let (mut conn, mut sub) = first_subscription(&client, &mut mock, 1).await;

        client.close();
        assert!(next(&mut sub).await.is_none());
        let goodbye = timeout(STALL, conn.from_client.recv()).await.unwrap();
        assert!(matches!(goodbye, Some(Message::Close(_))), "{goodbye:?}");

        sleep(Duration::from_secs(60)).await;
        assert!(mock.no_new_connection());
        let err = client
            .parsed_transaction_subscribe(programs("A"), None)
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("client closed"), "{err}");
    }

    #[tokio::test(start_paused = true)]
    async fn close_fails_a_subscribe_still_waiting_for_its_response() {
        let (client, mut mock) = client(ParsedStreamsOptions::default());
        let pending = spawn_subscribe(&client, "A");
        let mut conn = mock.connection().await;
        conn.request().await;

        client.close();
        let err = pending.await.unwrap().unwrap_err().to_string();
        assert!(err.contains("client closed"), "{err}");
    }

    #[tokio::test(start_paused = true)]
    async fn close_fails_a_subscribe_whose_connection_is_still_opening() {
        let (client, mock) = client(ParsedStreamsOptions::default());
        mock.plan(Plan::Hang, 1);
        let pending = spawn_subscribe(&client, "A");
        until(|| mock.attempts() == 1).await;

        client.close();
        let err = pending.await.unwrap().unwrap_err().to_string();
        assert!(err.contains("client closed"), "{err}");
    }

    #[tokio::test(start_paused = true)]
    async fn close_before_any_subscription_fails_later_ones() {
        let (client, mock) = client(ParsedStreamsOptions::default());
        client.close();
        assert!(client.parsed_transaction_subscribe(programs("A"), None).await.is_err());
        assert!(!client.parsed_transaction_unsubscribe(1).await);
        assert_eq!(mock.attempts(), 0);
    }

    // ── URLs ──────────────────────────────────────────────────────────

    fn url(client: &ParsedStreamsClient) -> String {
        let actor = client.inner.idle_actor.lock().unwrap();
        actor.as_ref().unwrap().url.clone()
    }

    #[test]
    fn url_defaults_to_helius_with_the_api_key() {
        let client = ParsedStreamsClient::new("KEY").unwrap();
        assert_eq!(url(&client), "wss://beta.helius-rpc.com/?api-key=KEY");
    }

    #[test]
    fn url_can_be_overridden() {
        let options = ParsedStreamsOptions::default().with_url("wss://custom/stream");
        let client = ParsedStreamsClient::with_options("KEY", options).unwrap();
        assert_eq!(url(&client), "wss://custom/stream");
    }

    #[test]
    fn rejects_an_empty_api_key() {
        assert!(matches!(
            ParsedStreamsClient::new("  "),
            Err(HeliusError::InvalidInput(_))
        ));
    }

    #[test]
    fn debug_output_does_not_leak_the_api_key() {
        let options = ParsedStreamsOptions::default().with_url("wss://x/?api-key=SECRET");
        assert!(!format!("{options:?}").contains("SECRET"));
        let client = ParsedStreamsClient::with_options("SECRET", options).unwrap();
        assert!(!format!("{client:?}").contains("SECRET"));
    }
}
