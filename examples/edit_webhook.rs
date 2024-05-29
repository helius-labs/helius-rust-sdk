use helius::error::HeliusError;
use helius::types::{Cluster, EditWebhookRequest, TransactionType, Webhook, WebhookType};
use helius::Helius;

#[tokio::main]
async fn main() -> Result<(), HeliusError> {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();
    let request = EditWebhookRequest {
        webhook_id: "your_webhook_id".to_string(),
        webhook_url: "your_webhook".to_string(),
        transaction_types: vec![TransactionType::Any],
        account_addresses: vec!["your_addresses".to_string()],
        webhook_type: WebhookType::Enhanced,
        auth_header: None,
        txn_status: Default::default(),
        encoding: Default::default(),
    };

    let response: Result<Webhook, HeliusError> = helius.edit_webhook(request).await;
    println!("Webhook edited {:?}", response);
    Ok(())
}
