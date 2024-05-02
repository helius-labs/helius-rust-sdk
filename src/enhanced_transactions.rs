use crate::error::Result;
use crate::types::{EnhancedTransaction, ParseTransactionsRequest};
use crate::Helius;

use reqwest::{Method, Url};

impl Helius {
    /// Parses transactions given an array of transaction IDs
    ///
    /// # Arguments
    /// * `transactions` - A vector of transaction IDs to be parsed
    ///
    /// # Returns
    /// A `Result` wrapping a vector of `EnhancedTransaction`s
    pub async fn parse_transactions(&self, transactions: Vec<String>) -> Result<Vec<EnhancedTransaction>> {
        let request = ParseTransactionsRequest { transactions };
        let url: String = format!(
            "{}/transactions?api-key={}",
            self.config.endpoints.rpc, self.config.api_key
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client
            .handler
            .send(Method::POST, parsed_url, Some(&request))
            .await
    }

    /// Retrieves parsed transaction history for a specific address
    pub async fn parsed_transaction_history(&self, address: &str) -> Result<Vec<EnhancedTransaction>> {
        let url: String = format!(
            "{}/addresses/{}/transactions?api-key={}",
            self.config.endpoints.rpc, address, self.config.api_key
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client.handler.send(Method::GET, parsed_url, None::<&()>).await
    }
}
