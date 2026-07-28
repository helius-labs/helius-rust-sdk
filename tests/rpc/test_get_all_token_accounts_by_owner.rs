use std::sync::Arc;

use helius::client::Helius;
use helius::config::Config;
use helius::rpc_client::RpcClient;
use helius::types::*;

use mockito::{self, Matcher, Server};
use reqwest::Client;
use serde_json::json;

const OWNER: &str = "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY";
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// A single `getTokenAccountsByOwnerV2` page (one account) with the given `paginationKey`.
fn page_body(pagination_key: Option<&str>) -> String {
    let response = json!({
        "jsonrpc": "2.0",
        "id": "1",
        "result": {
            "context": { "slot": 1 },
            "value": {
                "accounts": [
                    {
                        "pubkey": "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin",
                        "account": {
                            "lamports": 2039280,
                            "owner": TOKEN_PROGRAM,
                            "data": ["base64encodeddata", "base64"],
                            "executable": false,
                            "rentEpoch": 361,
                            "space": 165
                        }
                    }
                ],
                "paginationKey": pagination_key,
                "count": 1
            },
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

fn program_filter() -> TokenAccountsOwnerFilter {
    TokenAccountsOwnerFilter::Program {
        program_id: TOKEN_PROGRAM.to_string(),
    }
}

/// Happy path: a terminal `paginationKey: null` ends the loop on the first page.
#[tokio::test]
async fn test_get_all_token_accounts_terminates_on_null_cursor() {
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
        .get_all_token_accounts_by_owner(
            OWNER.to_string(),
            program_filter(),
            GetTokenAccountsByOwnerV2Config::default(),
        )
        .await
        .expect("auto-pagination should terminate and return Ok");

    assert_eq!(accounts.len(), 1, "a single terminal page should yield one account");
}

/// A non-advancing cursor (same key every page) must not loop forever; the guard stops it after
/// the repeat. Guards the token-account copy of the pagination logic independently.
#[tokio::test]
async fn test_get_all_token_accounts_stops_on_non_advancing_cursor() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", "/?api-key=fake_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page_body(Some("stuck")))
        .create();

    let helius = mock_helius(&url);
    let accounts = helius
        .rpc()
        .get_all_token_accounts_by_owner(
            OWNER.to_string(),
            program_filter(),
            GetTokenAccountsByOwnerV2Config::default(),
        )
        .await
        .expect("auto-pagination should terminate and return Ok");

    assert_eq!(
        accounts.len(),
        2,
        "expected the loop to stop after the cursor stopped advancing"
    );
}

/// A caller-supplied `max_pages` caps the fetch (proves the config field is wired on this path).
#[tokio::test]
async fn test_get_all_token_accounts_honors_custom_max_pages() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = server.url();

    server
        .mock("POST", Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(page_body(Some("stuck")))
        .create();

    let helius = mock_helius(&url);
    let config = GetTokenAccountsByOwnerV2Config {
        max_pages: Some(1),
        ..Default::default()
    };
    let accounts = helius
        .rpc()
        .get_all_token_accounts_by_owner(OWNER.to_string(), program_filter(), config)
        .await
        .expect("auto-pagination should terminate and return Ok");

    assert_eq!(accounts.len(), 1, "max_pages = 1 should cap the fetch at a single page");
}
