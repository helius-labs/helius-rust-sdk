use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::{self, Matcher, Server};
use reqwest::Client;
use serde_json::json;

const PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// A single `getProgramAccountsV2` page carrying one account and the given `paginationKey`
/// (`None` serializes to JSON `null`, signalling the terminal page).
fn page_body(pagination_key: Option<&str>) -> String {
    let response = json!({
        "jsonrpc": "2.0",
        "id": "1",
        "result": {
            "accounts": [
                {
                    "pubkey": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
                    "account": {
                        "lamports": 23357760,
                        "owner": PROGRAM_ID,
                        "data": ["base64encodeddata", "base64"],
                        "executable": false,
                        "rentEpoch": 361,
                        "space": 165
                    }
                }
            ],
            "paginationKey": pagination_key,
            "totalResults": 1
        }
    });
    serde_json::to_string(&response).unwrap()
}

fn mock_helius(url: &str) -> Helius {
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
    Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    }
}

/// Happy path: a terminal `paginationKey: null` ends the loop cleanly on the first page.
#[tokio::test]
async fn test_get_all_program_accounts_terminates_on_null_cursor() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page_body(None))
        .create();

    let helius = mock_helius(&url);
    let accounts = helius
        .rpc()
        .get_all_program_accounts(PROGRAM_ID.to_string(), GetProgramAccountsV2Config::default())
        .await
        .expect("auto-pagination should terminate and return Ok");

    assert_eq!(accounts.len(), 1, "a single terminal page should yield one account");
}

/// A server that keeps returning the *same* `paginationKey` (never a terminal `None`) must not
/// loop forever. The non-advancing-cursor guard stops it after the repeat.
#[tokio::test]
async fn test_get_all_program_accounts_stops_on_non_advancing_cursor() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    // No `.expect(n)`, so this mock answers every matching request with the same key.
    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page_body(Some("stuck")))
        .create();

    let helius = mock_helius(&url);
    let accounts = helius
        .rpc()
        .get_all_program_accounts(PROGRAM_ID.to_string(), GetProgramAccountsV2Config::default())
        .await
        .expect("auto-pagination should terminate and return Ok");

    // Page 1 sets the key; page 2 sees it unchanged and stops. One account per page.
    assert_eq!(
        accounts.len(),
        2,
        "expected the loop to stop after the cursor stopped advancing"
    );
}

/// A caller-supplied `max_pages` caps the fetch. With `max_pages = 1` and a server that always
/// returns a (same) cursor, the cap trips on page 1 — before the non-advancing guard would on
/// page 2 — proving the config override is honored.
#[tokio::test]
async fn test_get_all_program_accounts_honors_custom_max_pages() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page_body(Some("stuck")))
        .create();

    let helius = mock_helius(&url);
    let config = GetProgramAccountsV2Config {
        max_pages: Some(1),
        ..Default::default()
    };
    let accounts = helius
        .rpc()
        .get_all_program_accounts(PROGRAM_ID.to_string(), config)
        .await
        .expect("auto-pagination should terminate and return Ok");

    assert_eq!(accounts.len(), 1, "max_pages = 1 should cap the fetch at a single page");
}
