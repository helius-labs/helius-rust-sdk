use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::{self, Server};
use reqwest::Client;
use serde_json::json;

/// A server that keeps returning the *same* `paginationKey` (never a terminal `None`) must not
/// loop forever. `get_all_program_accounts` detects the non-advancing cursor and stops after the
/// repeat, returning what it has collected so far.
#[tokio::test]
async fn test_get_all_program_accounts_stops_on_non_advancing_cursor() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    // Every request gets the identical pagination key, so the cursor never advances.
    let mock_response = json!({
        "jsonrpc": "2.0",
        "id": "1",
        "result": {
            "accounts": [
                {
                    "pubkey": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
                    "account": {
                        "lamports": 23357760,
                        "owner": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
                        "data": ["base64encodeddata", "base64"],
                        "executable": false,
                        "rentEpoch": 361,
                        "space": 165
                    }
                }
            ],
            "paginationKey": "stuck",
            "totalResults": 1
        }
    });

    // No `.expect(n)`, so this mock answers every matching request.
    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: Some(ApiKey::new("fake_api_key").unwrap()),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
        custom_url: None,
    });

    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };

    let accounts = helius
        .rpc()
        .get_all_program_accounts(
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            GetProgramAccountsV2Config::default(),
        )
        .await
        .expect("auto-pagination should terminate and return Ok");

    // Two pages fetched before the non-advancing cursor is detected (first sets the key, second
    // sees it unchanged and stops), one account each.
    assert_eq!(
        accounts.len(),
        2,
        "expected the loop to stop after the cursor stopped advancing"
    );
}
