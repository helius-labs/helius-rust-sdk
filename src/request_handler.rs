use crate::error::{HeliusError, Result};
use bytes::Bytes;
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::fmt::Debug;
use std::sync::Arc;

/// The SDK version, sourced from Cargo.toml at compile time
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The User-Agent header value for SDK requests
/// Format: "helius-rust-sdk/{version} (server)"
pub const SDK_USER_AGENT: &str = concat!("helius-rust-sdk/", env!("CARGO_PKG_VERSION"), " (server)");

/// Manages HTTP requests for the `Helius` client
///
/// This struct is responsible for sending HTTP requests and handling responses. It encapsulates details
/// of the `reqwest::Client` to provide a simplified interface for making requests and processing responses
#[derive(Clone)]
pub struct RequestHandler {
    pub http_client: Arc<Client>,
}

impl RequestHandler {
    /// Creates a new instance of `RequestHandler`
    ///
    /// # Arguments
    /// * `client` - A shared instance of a `reqwest::Client`
    pub fn new(client: Arc<Client>) -> Result<Self> {
        Ok(Self { http_client: client })
    }

    /// Asynchronously sends a specified HTTP request using the `RequestBuilder`
    ///
    /// # Arguments
    /// * `request_builder` - Configured `RequestBuilder` for a specific HTTP request
    ///
    /// # Returns
    /// A `Result` wrapping a `reqwest::Response` if the request is send and received successfully
    ///
    /// # Errors
    /// Returns `HeliusError::Network` if there is an issue sending the request
    async fn send_request(&self, request_builder: RequestBuilder) -> Result<Response> {
        request_builder.send().await.map_err(HeliusError::Network)
    }

    /// Sends an HTTP request and processes the response to deserialize into a specified generic type
    ///
    /// This is [`send_raw`](Self::send_raw) followed by [`decode_response`], so the two paths
    /// share status handling, error mapping, and JSON parsing exactly. The decode runs inline on
    /// the calling task; callers who want it off the async runtime use `send_raw` and decode on
    /// a blocking thread themselves.
    ///
    /// # Type Parameters
    /// * `R` - The type of the request body, which must implement `Serialize`
    /// * `T` - The expected type of the response, which must implement `Deserialize`
    ///
    /// # Arguments
    /// * `method` - The HTTP method to be used for the request
    /// * `url` - The URL to which the request is sent
    /// * `body` - An optional request body, serialized as JSON if provided
    ///
    /// # Returns
    /// A `Result` wrapping the deserialized response data if the response is successful
    ///
    /// # Errors
    /// Returns an error if the request fails at any stage, including network errors, serialization errors
    /// or if the response status is not successful
    pub async fn send<R, T>(&self, method: Method, url: Url, body: Option<&R>) -> Result<T>
    where
        R: Serialize + ?Sized + Send + Sync + Debug,
        T: DeserializeOwned + Default,
    {
        let body_bytes: Bytes = self.send_raw(method, url, body).await?;
        decode_response(&body_bytes)
    }

    /// Sends an HTTP request and returns the successful response body undecoded
    ///
    /// Status handling and error mapping are identical to [`send`](Self::send): a non-2xx
    /// response is turned into the matching [`HeliusError`] variant and never reaches the
    /// caller as bytes. What the caller gets back is the raw body of a successful response,
    /// still to be parsed.
    ///
    /// Use this when JSON decoding should not run on the async runtime. Deserializing a large
    /// payload (a page of parsed transaction history, a `getProgramAccountsV2` page) is CPU
    /// work that parks a tokio worker for its duration; on a busy runtime that shows up as
    /// latency on every other task scheduled there. `Bytes` is `Send + 'static` and cheap to
    /// move, so it can be handed straight to `tokio::task::spawn_blocking` and decoded there
    /// with [`decode_response`], which applies the same simd-json/serde_json parsing the typed
    /// path uses. `HeliusError` converts from `tokio::task::JoinError`, so the join and the
    /// decode can both be handled with `?`:
    ///
    /// ```no_run
    /// # use helius::error::Result;
    /// # use helius::request_handler::{decode_response, RequestHandler};
    /// # use reqwest::{Client, Method, Url};
    /// # use std::sync::Arc;
    /// # async fn run() -> Result<()> {
    /// let handler: RequestHandler = RequestHandler::new(Arc::new(Client::new()))?;
    /// let url: Url = Url::parse("https://api.helius.xyz/v0/addresses/<address>/transactions?api-key=<key>")?;
    ///
    /// let body = handler.send_raw(Method::GET, url, None::<&()>).await?;
    /// let page: serde_json::Value = tokio::task::spawn_blocking(move || decode_response(&body)).await??;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Type Parameters
    /// * `R` - The type of the request body, which must implement `Serialize`
    ///
    /// # Arguments
    /// * `method` - The HTTP method to be used for the request
    /// * `url` - The URL to which the request is sent
    /// * `body` - An optional request body, serialized as JSON if provided
    ///
    /// # Returns
    /// A `Result` wrapping the raw response body. An empty body is returned as empty `Bytes`.
    ///
    /// # Errors
    /// Returns `HeliusError::Network` if the request cannot be sent or the body cannot be read,
    /// or the status-mapped variant (`BadRequest`, `Unauthorized`, `RateLimitExceeded`, ...) if
    /// the response status is not successful
    pub async fn send_raw<R>(&self, method: Method, url: Url, body: Option<&R>) -> Result<Bytes>
    where
        R: Serialize + ?Sized + Send + Sync + Debug,
    {
        let mut request_builder: RequestBuilder = self.http_client.request(method, url);

        request_builder = request_builder.header("User-Agent", SDK_USER_AGENT);

        if let Some(body) = body {
            request_builder = request_builder.json(body);
        }

        let response: Response = self.send_request(request_builder).await?;
        self.handle_response(response).await
    }

    /// Handles the Response for a given HTTP request, returning the body of a successful response
    /// and mapping a failure status to the matching error
    ///
    /// # Arguments
    /// * `response` - The `reqwest::Response` received from the HTTP request
    ///
    /// # Returns
    /// A `Result` wrapping the raw body if the response status is successful
    ///
    /// # Errors
    /// Returns an error if the body cannot be read or if the response status indicates a failure (e.g., 404 Not Found)
    async fn handle_response(&self, response: Response) -> Result<Bytes> {
        let status: StatusCode = response.status();
        let path: String = response.url().path().to_string();
        // Propagate a body-read failure instead of silently substituting an empty body: an
        // empty body would otherwise be decoded as `T::default()` on a 2xx (a bogus "success")
        // or produce an empty error message on a failure status. A genuinely empty body still
        // returns `Ok` here with zero bytes and is handled by the decoder.
        //
        // The body is read as bytes rather than text so that no UTF-8 validation pass runs over
        // it; the JSON parsers validate what they consume.
        let body: Bytes = response.bytes().await.map_err(HeliusError::Network)?;

        if status.is_success() {
            return Ok(body);
        }

        // Parse the error body in place; the lossy `String` copy is only needed when the body
        // is not JSON and is itself the error message.
        match serde_json::from_slice::<Value>(&body) {
            Ok(body) => {
                let error_message = match body["error"].clone() {
                    Value::Object(error_value) => error_value
                        .into_iter()
                        .map(|(k, v)| format!("{}: {}", k, v))
                        .collect::<Vec<String>>()
                        .join(", ")
                        .to_string(),
                    Value::String(error_value) => error_value,
                    _ => "Unknown error".to_string(),
                };
                Err(HeliusError::from_response_status(status, path, error_message))
            }
            Err(_) => Err(HeliusError::from_response_status(
                status,
                path,
                String::from_utf8_lossy(&body).into_owned(),
            )),
        }
    }
}

/// Decodes a successful response body the way the SDK's typed methods do
///
/// This is the parsing half of [`RequestHandler::send`], exposed so a body obtained from
/// [`RequestHandler::send_raw`] (or one of the `_raw` methods built on it) can be decoded
/// somewhere other than the async task that fetched it, typically inside
/// `tokio::task::spawn_blocking`. The semantics match the typed path exactly:
///
/// - An empty body decodes to `T::default()`
/// - The body is parsed with simd-json first; if that fails, serde_json is tried on the
///   original bytes, and its error is the one returned if both fail
///
/// ```
/// use helius::request_handler::decode_response;
///
/// let body = br#"{"total": 2, "items": ["a", "b"]}"#;
/// let page: serde_json::Value = decode_response(body).unwrap();
///
/// assert_eq!(page["total"], 2);
/// assert_eq!(page["items"].as_array().map(Vec::len), Some(2));
/// ```
///
/// # Type Parameters
/// * `T` - The type to decode into
///
/// # Arguments
/// * `body` - The raw response body
///
/// # Returns
/// A `Result` wrapping the decoded value
///
/// # Errors
/// Returns `HeliusError::SerdeJson` if the body is not valid JSON for `T`
pub fn decode_response<T>(body: &[u8]) -> Result<T>
where
    T: DeserializeOwned + Default,
{
    if body.is_empty() {
        return Ok(T::default());
    }

    // simd-json parses faster than serde_json on large payloads (DAS, getProgramAccountsV2,
    // parsed transaction history), but it unescapes strings *in place*, so the buffer it was
    // handed is already rewritten by the time it reports an error. Parsing that buffer again —
    // as the `serde_json` retry and the raw-JSON log below do — reads corrupted bytes and
    // produces a spurious error that then masks simd-json's accurate one.
    //
    // simd-json therefore gets a scratch copy and `body` stays pristine for both. The copy
    // costs one allocation and memcpy per response, which is small next to the parse it feeds
    // and is the price of a fallback that can actually recover (and of a debug log that shows
    // the real payload).
    let mut scratch: Vec<u8> = body.to_vec();
    match simd_json::serde::from_slice::<T>(&mut scratch) {
        Ok(data) => Ok(data),
        Err(simd_err) => match serde_json::from_slice::<T>(body) {
            Ok(data) => Ok(data),
            Err(serde_err) => {
                log::error!("Deserialization error (simd-json): {}", simd_err);
                log::error!("Deserialization error (serde_json): {}", serde_err);
                log::debug!("Raw JSON: {}", String::from_utf8_lossy(body));
                Err(HeliusError::from(serde_err))
            }
        },
    }
}
