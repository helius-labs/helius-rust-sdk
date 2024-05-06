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
}
