//! Helius **Pre Confirmations** (`preconfSubscribe`) WebSocket client.
//!
//! Pre Confirmations are Helius's lowest-latency transaction stream: scheduled
//! transactions are delivered over WebSocket *before* they are shredded. A
//! pre-confirmation is an **early signal, not a guarantee** — a transaction that
//! is streamed here may still fail to land.
//!
//! Coverage is **not continuous**: the stream scales with the share of stake
//! forwarding scheduled transactions to Helius, so expect gaps — not every slot
//! or transaction will appear.
//!
//! # Pricing
//!
//! Pre Confirmations use the standard credit-based pricing model, the same as
//! other Helius WebSocket subscriptions. It is **not** tip-based. Each
//! notification message costs **10 credits**.
//!
//! # Transport & wire format
//!
//! The subscription is a JSON-RPC 2.0 WebSocket served from the Gatekeeper
//! endpoint (`wss://beta.helius-rpc.com/?api-key=<KEY>`):
//! - `preconfSubscribe` — subscribe (takes **no parameters**; streams *all*
//!   scheduled transactions). The ack is a JSON-RPC text frame carrying a
//!   numeric subscription id.
//! - `preconfUnsubscribe` — unsubscribe. Responds with a boolean.
//!
//! Notifications arrive as **binary** WebSocket frames (not JSON), little-endian,
//! with this exact layout:
//!
//! ```text
//! version:u8 (1) | slot:u64_le (8) | transaction_index:u64_le (8) | status:u8 (1) | bincode(VersionedTransaction)
//! ```
//!
//! The leading `version` byte is read and checked **first**: it is `1` today and
//! increments if the layout changes. Unknown versions are rejected (the frame is
//! dropped) rather than misparsed. `status` is `0 = failed`, `1 = success`,
//! `2 = unknown`.

use std::pin::Pin;
use std::task::{Context, Poll};

use bincode::deserialize;
use futures_util::{
    future::{BoxFuture, FutureExt},
    sink::SinkExt,
    stream::{BoxStream, StreamExt},
};
use solana_sdk::transaction::VersionedTransaction;
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        protocol::frame::{coding::CloseCode, CloseFrame},
        Message,
    },
};

use crate::error::{HeliusError, Result};

/// Base WebSocket URL for the Helius Pre Confirmations endpoint.
///
/// Pre Confirmations are served from the Gatekeeper endpoint
/// (`wss://beta.helius-rpc.com`). Despite the `beta` host name this is **not** a
/// beta product — it is where Pre Confirmations launch during the Gatekeeper
/// migration. The API key is appended as a query parameter.
pub const PRECONF_WEBSOCKET_URL: &str = "wss://beta.helius-rpc.com/?api-key=";

const JSONRPC_VERSION: &str = "2.0";
const SUBSCRIBE_METHOD: &str = "preconfSubscribe";
const UNSUBSCRIBE_METHOD: &str = "preconfUnsubscribe";

/// The wire schema version this client understands. Frames carrying any other
/// version in byte 0 are rejected (dropped) rather than misparsed.
pub const CURRENT_VERSION: u8 = 1;

/// Fixed-size header before the bincode payload:
/// `version(1) + slot(8) + transaction_index(8) + status(1)`.
const HEADER_LEN: usize = 18;

/// Landed status of a pre-confirmed transaction.
///
/// A pre-confirmation is an early signal; `status` reflects the scheduler's
/// current view and may still change before the transaction is finalized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreconfStatus {
    /// The transaction failed (`0`).
    Failed,
    /// The transaction succeeded (`1`).
    Success,
    /// Status is not yet known (`2`).
    Unknown,
}

impl PreconfStatus {
    /// Decode the on-the-wire `status` byte. Any value other than `0`/`1`/`2`
    /// maps to [`PreconfStatus::Unknown`].
    pub fn from_u8(byte: u8) -> Self {
        match byte {
            0 => PreconfStatus::Failed,
            1 => PreconfStatus::Success,
            _ => PreconfStatus::Unknown,
        }
    }
}

/// A single Pre Confirmations notification.
///
/// A pre-confirmation is an **early signal, not a guarantee**: the transaction
/// has been scheduled and is being streamed before it is shredded, but it may
/// still fail to land.
#[derive(Debug, Clone)]
pub struct PreconfNotification {
    /// The wire schema version (byte 0). Currently always [`CURRENT_VERSION`].
    pub version: u8,
    /// The slot the scheduled transaction targets.
    pub slot: u64,
    /// The transaction's index within the scheduled batch for that slot.
    pub transaction_index: u64,
    /// The reported landed status of the transaction.
    pub status: PreconfStatus,
    /// The decoded transaction (deserialized from bincode).
    pub transaction: VersionedTransaction,
    /// The raw `bincode(VersionedTransaction)` bytes, exposed alongside the
    /// decoded form for callers that want the original payload.
    pub transaction_bytes: Vec<u8>,
}

impl PreconfNotification {
    /// Parse a raw Pre Confirmations binary WebSocket frame.
    ///
    /// Expected layout:
    /// `version:u8 | slot:u64_le | transaction_index:u64_le | status:u8 | bincode(VersionedTransaction)`.
    ///
    /// The leading `version` byte is checked first; a frame with an unrecognized
    /// version is rejected so future format changes fail loudly instead of being
    /// silently misparsed.
    pub fn from_frame(buf: &[u8]) -> Result<Self> {
        if buf.len() < HEADER_LEN + 1 {
            return Err(HeliusError::InvalidInput(format!(
                "preconf frame too short: {} bytes (need > {})",
                buf.len(),
                HEADER_LEN
            )));
        }

        let version = buf[0];
        if version != CURRENT_VERSION {
            return Err(HeliusError::InvalidInput(format!(
                "unsupported preconf wire version {version} (this client understands {CURRENT_VERSION})"
            )));
        }

        let slot = u64::from_le_bytes(buf[1..9].try_into().unwrap());
        let transaction_index = u64::from_le_bytes(buf[9..17].try_into().unwrap());
        let status = PreconfStatus::from_u8(buf[17]);
        let transaction_bytes = buf[HEADER_LEN..].to_vec();

        let transaction: VersionedTransaction = deserialize(&transaction_bytes)
            .map_err(|e| HeliusError::InvalidInput(format!("failed to deserialize VersionedTransaction: {e}")))?;

        Ok(Self {
            version,
            slot,
            transaction_index,
            status,
            transaction,
            transaction_bytes,
        })
    }
}

type UnsubscribeFn = Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send>;

/// A client for subscribing to Helius Pre Confirmations (`preconfSubscribe`).
pub struct PreconfClient {
    /// Retained to keep the request channel open for the background task.
    #[allow(dead_code)]
    request_sender: mpsc::UnboundedSender<RequestMsg>,
    shutdown_sender: Option<oneshot::Sender<()>>,
    ws: JoinHandle<Result<()>>,
}

type RequestMsg = (String, oneshot::Sender<Result<()>>);

impl PreconfClient {
    /// Constructs the Pre Confirmations WebSocket URL with the API key appended.
    ///
    /// Uses the Gatekeeper endpoint [`PRECONF_WEBSOCKET_URL`].
    pub fn get_url(api_key: &str) -> String {
        format!("{PRECONF_WEBSOCKET_URL}{api_key}")
    }

    /// Connect to a Pre Confirmations WebSocket endpoint and start streaming.
    ///
    /// The returned stream yields a [`PreconfNotification`] per scheduled
    /// transaction. Dropping the returned [`PreconfStream`] (or calling its
    /// unsubscribe future) tears the subscription down.
    ///
    /// `preconfSubscribe` takes no filter parameters: it streams **all**
    /// scheduled transactions.
    pub async fn connect(url: &str) -> Result<(Self, PreconfStream)> {
        let (ws_stream, _resp) = connect_async(url).await.map_err(HeliusError::Tungstenite)?;
        let (mut ws_sink, mut ws_read) = ws_stream.split();

        // Send the subscribe request immediately.
        let subscribe_req = serde_json::json!({
            "jsonrpc": JSONRPC_VERSION,
            "id": 1,
            "method": SUBSCRIBE_METHOD,
        })
        .to_string();
        ws_sink
            .send(Message::Text(subscribe_req.into()))
            .await
            .map_err(HeliusError::Tungstenite)?;

        let (notif_tx, notif_rx) = mpsc::unbounded_channel::<PreconfNotification>();
        let (request_sender, mut request_receiver) = mpsc::unbounded_channel::<RequestMsg>();
        let (shutdown_sender, mut shutdown_receiver) = oneshot::channel::<()>();

        let ws: JoinHandle<Result<()>> = tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;

                    _ = &mut shutdown_receiver => {
                        let _ = ws_sink
                            .send(Message::Close(Some(CloseFrame {
                                code: CloseCode::Normal,
                                reason: "client shutdown".into(),
                            })))
                            .await;
                        break;
                    }

                    Some((method, ack)) = request_receiver.recv() => {
                        let req = serde_json::json!({
                            "jsonrpc": JSONRPC_VERSION,
                            "id": 1,
                            "method": method,
                        })
                        .to_string();
                        let res = ws_sink
                            .send(Message::Text(req.into()))
                            .await
                            .map_err(HeliusError::Tungstenite);
                        let _ = ack.send(res.map(|_| ()));
                    }

                    msg = ws_read.next() => {
                        match msg {
                            Some(Ok(Message::Binary(bytes))) => {
                                match PreconfNotification::from_frame(&bytes) {
                                    Ok(notif) => {
                                        if notif_tx.send(notif).is_err() {
                                            // Receiver dropped; tear down.
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        // Malformed or unknown-version frame: skip
                                        // rather than kill the stream.
                                        eprintln!("helius: dropping malformed preconf frame: {e}");
                                    }
                                }
                            }
                            Some(Ok(Message::Ping(payload))) => {
                                let _ = ws_sink.send(Message::Pong(payload)).await;
                            }
                            // Text frames are JSON-RPC sub/unsub acks; ignore.
                            Some(Ok(Message::Text(_))) => {}
                            Some(Ok(Message::Close(_))) | None => break,
                            Some(Ok(_)) => {}
                            Some(Err(e)) => return Err(HeliusError::Tungstenite(e)),
                        }
                    }
                }
            }
            Ok(())
        });

        let request_sender_for_unsub = request_sender.clone();
        let unsubscribe: UnsubscribeFn = Box::new(move || {
            async move {
                let (ack_tx, ack_rx) = oneshot::channel();
                if request_sender_for_unsub
                    .send((UNSUBSCRIBE_METHOD.to_string(), ack_tx))
                    .is_ok()
                {
                    let _ = ack_rx.await;
                }
            }
            .boxed()
        });

        let client = Self {
            request_sender,
            shutdown_sender: Some(shutdown_sender),
            ws,
        };

        let stream = PreconfStream {
            inner: UnboundedReceiverStream::new(notif_rx).boxed(),
            unsubscribe: Some(unsubscribe),
        };

        Ok((client, stream))
    }

    /// Convenience constructor: connect with an API key using the Gatekeeper endpoint.
    pub async fn connect_with_api_key(api_key: &str) -> Result<(Self, PreconfStream)> {
        let url = Self::get_url(api_key);
        Self::connect(&url).await
    }

    /// Signal the background task to close the connection and stop streaming.
    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_sender.take() {
            let _ = tx.send(());
        }
        match self.ws.await {
            Ok(res) => res,
            Err(e) => Err(HeliusError::InvalidInput(format!("preconf ws task join error: {e}"))),
        }
    }
}

/// A stream of [`PreconfNotification`]s plus an unsubscribe handle.
pub struct PreconfStream {
    inner: BoxStream<'static, PreconfNotification>,
    unsubscribe: Option<UnsubscribeFn>,
}

impl PreconfStream {
    /// Send a `preconfUnsubscribe` for this subscription.
    pub async fn unsubscribe(mut self) {
        if let Some(unsub) = self.unsubscribe.take() {
            unsub().await;
        }
    }

    /// Borrow the underlying notification stream.
    pub fn stream(&mut self) -> &mut BoxStream<'static, PreconfNotification> {
        &mut self.inner
    }
}

impl futures_util::Stream for PreconfStream {
    type Item = PreconfNotification;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::message::{Message as LegacyMessage, VersionedMessage};
    use solana_sdk::signature::{Keypair, Signer};
    use solana_system_interface::instruction as system_instruction;

    fn sample_versioned_tx() -> VersionedTransaction {
        let payer = Keypair::new();
        let to = Keypair::new();
        let ix = system_instruction::transfer(&payer.pubkey(), &to.pubkey(), 1);
        let msg = LegacyMessage::new(&[ix], Some(&payer.pubkey()));
        VersionedTransaction {
            signatures: vec![solana_sdk::signature::Signature::default()],
            message: VersionedMessage::Legacy(msg),
        }
    }

    fn build_frame(slot: u64, idx: u64, status: u8, tx: &VersionedTransaction) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(CURRENT_VERSION);
        buf.extend_from_slice(&slot.to_le_bytes());
        buf.extend_from_slice(&idx.to_le_bytes());
        buf.push(status);
        buf.extend_from_slice(&bincode::serialize(tx).unwrap());
        buf
    }

    #[test]
    fn parses_valid_frame() {
        let tx = sample_versioned_tx();
        let frame = build_frame(123, 7, 1, &tx);
        let notif = PreconfNotification::from_frame(&frame).unwrap();
        assert_eq!(notif.version, 1);
        assert_eq!(notif.slot, 123);
        assert_eq!(notif.transaction_index, 7);
        assert_eq!(notif.status, PreconfStatus::Success);
        assert_eq!(notif.transaction.signatures.len(), tx.signatures.len());
        assert_eq!(notif.transaction_bytes, bincode::serialize(&tx).unwrap());
    }

    #[test]
    fn decodes_each_status_variant() {
        let tx = sample_versioned_tx();
        assert_eq!(
            PreconfNotification::from_frame(&build_frame(1, 0, 0, &tx))
                .unwrap()
                .status,
            PreconfStatus::Failed
        );
        assert_eq!(
            PreconfNotification::from_frame(&build_frame(1, 0, 1, &tx))
                .unwrap()
                .status,
            PreconfStatus::Success
        );
        assert_eq!(
            PreconfNotification::from_frame(&build_frame(1, 0, 2, &tx))
                .unwrap()
                .status,
            PreconfStatus::Unknown
        );
        // Out-of-range status byte falls back to Unknown.
        assert_eq!(
            PreconfNotification::from_frame(&build_frame(1, 0, 9, &tx))
                .unwrap()
                .status,
            PreconfStatus::Unknown
        );
    }

    #[test]
    fn rejects_unknown_version() {
        let tx = sample_versioned_tx();
        let mut frame = build_frame(1, 0, 1, &tx);
        frame[0] = 2; // bump the version byte to an unsupported value
        let err = PreconfNotification::from_frame(&frame).unwrap_err();
        assert!(format!("{err}").contains("unsupported preconf wire version"));
    }

    #[test]
    fn rejects_short_frame() {
        assert!(PreconfNotification::from_frame(&[0u8; 18]).is_err());
        assert!(PreconfNotification::from_frame(&[]).is_err());
    }

    #[test]
    fn rejects_garbage_transaction_bytes() {
        let mut frame = Vec::new();
        frame.push(CURRENT_VERSION);
        frame.extend_from_slice(&1u64.to_le_bytes());
        frame.extend_from_slice(&0u64.to_le_bytes());
        frame.push(1); // status = success
        frame.extend_from_slice(&[0xFF; 8]); // not a valid bincode VersionedTransaction
        assert!(PreconfNotification::from_frame(&frame).is_err());
    }

    #[test]
    fn large_slot_and_index() {
        let tx = sample_versioned_tx();
        let frame = build_frame(u64::MAX, u64::MAX - 1, 1, &tx);
        let notif = PreconfNotification::from_frame(&frame).unwrap();
        assert_eq!(notif.slot, u64::MAX);
        assert_eq!(notif.transaction_index, u64::MAX - 1);
    }

    #[test]
    fn url_uses_gatekeeper_endpoint() {
        assert_eq!(
            PreconfClient::get_url("key123"),
            "wss://beta.helius-rpc.com/?api-key=key123"
        );
    }
}
