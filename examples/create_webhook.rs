use helius::types::{Cluster, WebhookType};
use helius::{
    types::{CreateWebhookRequest, TransactionType},
    Helius,
};

#[tokio::main]
async fn main() {
    let api_key: &str = "your_api_key";
    let cluster: Cluster = Cluster::MainnetBeta;

    let helius: Helius = Helius::new(api_key, cluster).unwrap();

    let request = CreateWebhookRequest {
        webhook_url: "your_webhook".to_string(),
        transaction_types: vec![TransactionType::Any],
        account_addresses: vec!["your_addresses".to_string()],
        webhook_type: WebhookType::Enhanced,
        auth_header: None,
        txn_status: Default::default(),
        encoding: Default::default(),
    };

    let response = helius.create_webhook(request).await;
    println!("Webhook created {:?}", response);
}
