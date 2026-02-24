use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::Server;
use reqwest::Client;

/// Creates a mockito server and a Helius client wired to it.
pub async fn setup_mock() -> (Server, Helius) {
    let server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url = server.url();

    let config = Arc::new(Config {
        api_key: Some(ApiKey::new("fake_api_key").unwrap()),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
        custom_url: None,
    });

    let client = Client::new();
    let rpc_client = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };

    (server, helius)
}
