use std::sync::Arc;
use mockito::Server;
use reqwest::{Client};
use helius::config::Config;
use helius::Helius;
use helius::rpc_client::RpcClient;
use helius::types::{ Cluster, HeliusEndpoints};



#[tokio::test]
async fn test_delete_webhook_success(){
    let mut server:Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url:String = format!("{}/",server.url());


    server.mock("DELETE","/v0/webhooks/0e8250a1-ceec-4757-ad69/?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });

    let client: Client = Client::new();
    let rpc_client:Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()),Arc::clone(&config)).unwrap());
    let helius = Helius{
        config,
        client,
        rpc_client,
    };


    let response = helius.delete_webhook("0e8250a1-ceec-4757-ad69").await;

    assert!(response.is_ok(),"The API call failed: {:?}",response.err());

}

#[tokio::test]
async fn test_delete_webhook_failure(){
    let mut server:Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url:String = format!("{}/",server.url());


    server.mock("DELETE","/v0/webhooks/0e8250a1-ceec-4757-ad69/?api-key=fake_api_key")
        .with_status(500)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error":"Internal Server Error"}"#)
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });


    let client: Client = Client::new();
    let rpc_client:Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()),Arc::clone(&config)).unwrap());
    let helius = Helius{
        config,
        client,
        rpc_client,
    };
    let response= helius.delete_webhook("0e8250a1-ceec-4757-ad69").await;
    assert!(response.is_err(),"Expected an error due to server failure");
}