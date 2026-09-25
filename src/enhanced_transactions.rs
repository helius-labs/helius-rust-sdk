use crate::error::Result;
use crate::types::{EnhancedTransaction, ParseTransactionsRequest, ParsedTransactionHistoryRequest};
use crate::Helius;

use bytes::Bytes;
use reqwest::{Method, Url};

impl Helius {
    /// Parses transactions given an array of transaction IDs
    ///
    /// # Arguments
    /// * `ParseTransactionsRequest` - A parse transaction request, which includes:
    /// - A vector of transaction IDs to be parsed
    ///
    /// # Returns
    /// A `Result` wrapping a vector of `EnhancedTransaction`s
    pub async fn parse_transactions(&self, request: ParseTransactionsRequest) -> Result<Vec<EnhancedTransaction>> {
        let url: Url = self.parse_transactions_url()?;

        self.rpc_client.handler.send(Method::POST, url, Some(&request)).await
    }

    /// Parses transactions given an array of transaction IDs, returning the response body undecoded
    ///
    /// Identical to [`parse_transactions`](Self::parse_transactions) on the wire and in error
    /// handling, but the JSON is handed back as `Bytes` instead of being deserialized on the
    /// calling task. Decode it with
    /// [`decode_response`](crate::request_handler::decode_response), typically inside
    /// `tokio::task::spawn_blocking` so a large batch does not park an async worker.
    /// `HeliusError` converts from `tokio::task::JoinError`, so the join and the decode can
    /// both be handled with `?`:
    ///
    /// ```no_run
    /// # use helius::error::Result;
    /// # use helius::request_handler::decode_response;
    /// # use helius::types::{Cluster, EnhancedTransaction, ParseTransactionsRequest};
    /// # use helius::Helius;
    /// # async fn run(signatures: Vec<String>) -> Result<()> {
    /// let helius: Helius = Helius::new("your_api_key", Cluster::MainnetBeta)?;
    /// let request: ParseTransactionsRequest = ParseTransactionsRequest { transactions: signatures };
    ///
    /// let body = helius.parse_transactions_raw(request).await?;
    /// let txs: Vec<EnhancedTransaction> =
    ///     tokio::task::spawn_blocking(move || decode_response(&body)).await??;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Arguments
    /// * `ParseTransactionsRequest` - A parse transaction request, which includes:
    /// - A vector of transaction IDs to be parsed
    ///
    /// # Returns
    /// A `Result` wrapping the raw JSON body of a successful response
    pub async fn parse_transactions_raw(&self, request: ParseTransactionsRequest) -> Result<Bytes> {
        let url: Url = self.parse_transactions_url()?;

        self.rpc_client
            .handler
            .send_raw(Method::POST, url, Some(&request))
            .await
    }

    /// Retrieves a parsed transaction history for a specific address
    ///
    /// # Arguments
    /// * `ParsedTransactionHistoryRequest` - A parsed transaction history request, which includes:
    /// - An address for which a given parsed transaction history will be retrieved
    /// - An optional `before` parameter that, when provided, fetches the parsed transaction history before the given signature. This is useful for pagination
    ///
    /// # Returns
    /// A `Result` wrapping a vector of `EnhancedTransaction`s
    pub async fn parsed_transaction_history(
        &self,
        request: ParsedTransactionHistoryRequest,
    ) -> Result<Vec<EnhancedTransaction>> {
        let url: Url = self.parsed_transaction_history_url(&request)?;

        self.rpc_client.handler.send(Method::GET, url, None::<&()>).await
    }

    /// Retrieves a parsed transaction history for a specific address, returning the response body undecoded
    ///
    /// Identical to [`parsed_transaction_history`](Self::parsed_transaction_history) on the
    /// wire and in error handling, but the JSON is handed back as `Bytes` instead of being
    /// deserialized on the calling task. A full page of history is the largest payload the
    /// enhanced transactions API returns, so this is the method to reach for when decoding
    /// must not block the runtime. Decode with
    /// [`decode_response`](crate::request_handler::decode_response) on a blocking thread.
    /// `HeliusError` converts from `tokio::task::JoinError`, so the join and the decode can
    /// both be handled with `?`:
    ///
    /// ```no_run
    /// # use helius::error::Result;
    /// # use helius::request_handler::decode_response;
    /// # use helius::types::{Cluster, EnhancedTransaction, ParsedTransactionHistoryRequest};
    /// # use helius::Helius;
    /// # async fn run() -> Result<()> {
    /// let helius: Helius = Helius::new("your_api_key", Cluster::MainnetBeta)?;
    /// let request: ParsedTransactionHistoryRequest = ParsedTransactionHistoryRequest {
    ///     address: "2k5AXX4guW9XwRQ1AKCpAuUqgWDpQpwFfpVFh3hnm2Ha".to_string(),
    ///     before: None,
    ///     until: None,
    ///     commitment: None,
    ///     source: None,
    ///     transaction_type: None,
    ///     limit: Some(100),
    /// };
    ///
    /// let body = helius.parsed_transaction_history_raw(request).await?;
    /// let txs: Vec<EnhancedTransaction> =
    ///     tokio::task::spawn_blocking(move || decode_response(&body)).await??;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Arguments
    /// * `ParsedTransactionHistoryRequest` - A parsed transaction history request, which includes:
    /// - An address for which a given parsed transaction history will be retrieved
    /// - An optional `before` parameter that, when provided, fetches the parsed transaction history before the given signature. This is useful for pagination
    ///
    /// # Returns
    /// A `Result` wrapping the raw JSON body of a successful response
    pub async fn parsed_transaction_history_raw(&self, request: ParsedTransactionHistoryRequest) -> Result<Bytes> {
        let url: Url = self.parsed_transaction_history_url(&request)?;

        self.rpc_client.handler.send_raw(Method::GET, url, None::<&()>).await
    }

    /// Builds the `POST /v0/transactions` URL shared by the typed and raw parse methods
    fn parse_transactions_url(&self) -> Result<Url> {
        let api_key = self.config.require_api_key("enhanced transaction parsing")?;
        let url: String = format!(
            "{}v0/transactions?api-key={}",
            self.config.endpoints.api,
            api_key.as_str()
        );

        Ok(Url::parse(&url)?)
    }

    /// Builds the `GET /v0/addresses/{address}/transactions` URL shared by the typed and raw history methods
    fn parsed_transaction_history_url(&self, request: &ParsedTransactionHistoryRequest) -> Result<Url> {
        let api_key = self.config.require_api_key("enhanced transaction history")?;
        let mut url: String = format!(
            "{}v0/addresses/{}/transactions?api-key={}",
            self.config.endpoints.api,
            request.address,
            api_key.as_str()
        );

        if let Some(before) = &request.before {
            url = format!("{}&before={}", url, before);
        }

        if let Some(until) = &request.until {
            url = format!("{}&until={}", url, until);
        }

        if let Some(commitment) = &request.commitment {
            url = format!("{}&commitment={}", url, commitment);
        }

        if let Some(source) = &request.source {
            url = format!("{}&source={}", url, source);
        }

        if let Some(transaction_type) = &request.transaction_type {
            url = format!("{}&type={}", url, transaction_type);
        }

        if let Some(limit) = request.limit {
            url = format!("{}&limit={}", url, limit);
        }

        Ok(Url::parse(&url)?)
    }
}
