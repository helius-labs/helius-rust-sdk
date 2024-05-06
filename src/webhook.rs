use crate::error::Result;
use crate::types::{CreateWebhookRequest, EditWebhookRequest, Webhook};
use crate::Helius;

use reqwest::{Method, Url};

impl Helius {
    /// Creates a webhook given account addresses,
    pub async fn create_webhook(&self, request: CreateWebhookRequest) -> Result<Webhook> {
        let url: String = format!(
            "{}v0/webhooks/?api-key={}",
            self.config.endpoints.api, self.config.api_key
        );

        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");
        println!("PARSED URL: {}", parsed_url);

        self.rpc_client
            .handler
            .send(Method::POST, parsed_url, Some(&request))
            .await
    }

    /// Edits a Helius webhook programmatically
    pub async fn edit_webhook(&self, request: EditWebhookRequest) -> Result<Webhook> {
        let url: String = format!(
            "{}v0/webhooks/{}/?api-key={}",
            self.config.endpoints.api, request.webhook_id, self.config.api_key
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client
            .handler
            .send(Method::PUT, parsed_url, Some(&request))
            .await
    }

    /// Appends a set of addresses to a given webhook
    pub async fn append_addresses_to_webhook(&self, webhook_id: &str, new_addresses: &[String]) -> Result<Webhook> {
        let mut webhook: Webhook = self.get_webhook_by_id(webhook_id).await?;
        webhook.account_addresses.extend(new_addresses.to_vec());

        let edit_request: EditWebhookRequest = EditWebhookRequest {
            webhook_id: webhook_id.to_string(),
            webhook_url: webhook.webhook_url,
            transaction_types: webhook.transaction_types,
            auth_header: webhook.auth_header,
            txn_status: webhook.txn_status,
            encoding: webhook.encoding,
            account_addresses: webhook.account_addresses,
            webhook_type: webhook.webhook_type,
        };
        self.edit_webhook(edit_request).await
    }

    /// Gets a webhook config given a webhook ID
    pub async fn get_webhook_by_id(&self, webhook_id: &str) -> Result<Webhook> {
        let url: String = format!(
            "{}v0/webhooks/{}/?api-key={}",
            self.config.endpoints.api, webhook_id, self.config.api_key
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client.handler.send(Method::GET, parsed_url, None::<&()>).await
    }

    /// Retrieves all Helius webhooks programmatically
    ///
    /// Due to response size limitations, we will truncate addresses returned to 100 per config
    pub async fn get_all_webhooks(&self) -> Result<Vec<Webhook>> {
        let url: String = format!(
            "{}v0/webhooks/?api-key={}",
            self.config.endpoints.api, self.config.api_key
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client.handler.send(Method::GET, parsed_url, None::<&()>).await
    }
}
