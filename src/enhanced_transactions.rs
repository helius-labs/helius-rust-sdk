use std::sync::Arc;

use crate::config::Config;
use crate::error::{HeliusError, Result};
use crate::request_handler::RequestHandler;
use crate::types::{
    EnhancedTransaction, ParseTransactionsRequest, ParsedTransactionHistoryRequest, TransactionHistoryV2Request,
    TransactionPageV2, TransactionResultV2, TransactionsV2Request,
};
use crate::Helius;

use reqwest::{Method, Url};

/// Client namespace for Helius Enhanced Transaction APIs.
///
/// Use [`EnhancedTransactionsClient::v1`] for the existing Enhanced Transactions API
/// and [`EnhancedTransactionsClient::v2`] for the newer POST-based API.
#[derive(Clone)]
pub struct EnhancedTransactionsClient {
    config: Arc<Config>,
    handler: RequestHandler,
}

impl EnhancedTransactionsClient {
    pub(crate) fn new(config: Arc<Config>, handler: RequestHandler) -> Self {
        Self { config, handler }
    }

    /// Access v1 Enhanced Transactions methods.
    pub fn v1(&self) -> EnhancedTransactionsV1 {
        EnhancedTransactionsV1::new(self.clone())
    }

    /// Access v2 Enhanced Transactions methods.
    pub fn v2(&self) -> EnhancedTransactionsV2 {
        EnhancedTransactionsV2::new(self.clone())
    }

    fn api_url(&self, feature: &str, segments: &[&str]) -> Result<Url> {
        let api_key = self.config.require_api_key(feature)?;
        let mut url = Url::parse(&self.config.endpoints.api)?;

        {
            let mut path_segments = url
                .path_segments_mut()
                .map_err(|_| HeliusError::InvalidInput("API endpoint cannot be used as a base URL".to_string()))?;
            path_segments.pop_if_empty();
            path_segments.extend(segments.iter().copied());
        }

        url.query_pairs_mut().append_pair("api-key", api_key.as_str());
        Ok(url)
    }
}

/// v1 Enhanced Transactions API.
#[derive(Clone)]
pub struct EnhancedTransactionsV1 {
    client: EnhancedTransactionsClient,
}

impl EnhancedTransactionsV1 {
    fn new(client: EnhancedTransactionsClient) -> Self {
        Self { client }
    }

    /// Parses transactions given an array of transaction IDs.
    pub async fn parse_transactions(&self, request: ParseTransactionsRequest) -> Result<Vec<EnhancedTransaction>> {
        let url = self
            .client
            .api_url("enhanced transaction parsing", &["v0", "transactions"])?;

        self.client.handler.send(Method::POST, url, Some(&request)).await
    }

    /// Retrieves parsed transaction history for a specific address.
    pub async fn parsed_transaction_history(
        &self,
        request: ParsedTransactionHistoryRequest,
    ) -> Result<Vec<EnhancedTransaction>> {
        let mut url = self.client.api_url(
            "enhanced transaction history",
            &["v0", "addresses", &request.address, "transactions"],
        )?;

        {
            let mut query = url.query_pairs_mut();
            if let Some(before) = request.before {
                query.append_pair("before", &before);
            }

            if let Some(until) = request.until {
                query.append_pair("until", &until);
            }

            if let Some(commitment) = request.commitment {
                query.append_pair("commitment", &commitment.to_string());
            }

            if let Some(source) = request.source {
                query.append_pair("source", &source.to_string());
            }

            if let Some(transaction_type) = request.transaction_type {
                query.append_pair("type", &transaction_type.to_string());
            }

            if let Some(limit) = request.limit {
                query.append_pair("limit", &limit.to_string());
            }
        }

        self.client.handler.send(Method::GET, url, None::<&()>).await
    }
}

/// v2 Enhanced Transactions API.
#[derive(Clone)]
pub struct EnhancedTransactionsV2 {
    client: EnhancedTransactionsClient,
}

impl EnhancedTransactionsV2 {
    fn new(client: EnhancedTransactionsClient) -> Self {
        Self { client }
    }

    /// Parses a batch of transactions by signature via `POST /transactions`.
    pub async fn transactions(&self, request: TransactionsV2Request) -> Result<Vec<TransactionResultV2>> {
        let url = self.client.api_url("enhanced transactions v2", &["transactions"])?;

        self.client.handler.send(Method::POST, url, Some(&request)).await
    }

    /// Retrieves parsed transaction history via `POST /transaction-history`.
    pub async fn transaction_history(&self, request: TransactionHistoryV2Request) -> Result<TransactionPageV2> {
        let url = self
            .client
            .api_url("enhanced transaction history v2", &["transaction-history"])?;

        self.client.handler.send(Method::POST, url, Some(&request)).await
    }
}

impl Helius {
    /// Returns the Enhanced Transactions API namespace.
    pub fn enhanced(&self) -> EnhancedTransactionsClient {
        EnhancedTransactionsClient::new(self.config.clone(), self.rpc_client.handler.clone())
    }

    /// Parses transactions given an array of transaction IDs
    ///
    /// # Arguments
    /// * `ParseTransactionsRequest` - A parse transaction request, which includes:
    /// - A vector of transaction IDs to be parsed
    ///
    /// # Returns
    /// A `Result` wrapping a vector of `EnhancedTransaction`s
    pub async fn parse_transactions(&self, request: ParseTransactionsRequest) -> Result<Vec<EnhancedTransaction>> {
        self.enhanced().v1().parse_transactions(request).await
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
        self.enhanced().v1().parsed_transaction_history(request).await
    }
}
