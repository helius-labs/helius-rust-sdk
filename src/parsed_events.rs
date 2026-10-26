use crate::config::Config;
use crate::error::{HeliusError, Result};
use crate::types::parsed_events::{
    GetTransactionHistoryRequest, GetTransactionHistoryResponse, ParseTransactionsRequest, ParsedTransactionResult,
    ScanRequest, ScanResponse,
};
use crate::types::Cluster;
use crate::Helius;

use bytes::Bytes;
use reqwest::{Method, Url};
use url::Host;

/// Base URL for Parsed Events endpoints. Parsed Events is mainnet-only.
const PARSED_EVENTS_BASE_URL: &str = "https://mainnet.helius-rpc.com/v1/parsed-events";

impl Helius {
    /// Returns the [Parsed Events API](https://www.helius.dev/docs/parsed-events) client
    ///
    /// Parsed Events decodes transactions and address history into instructions, transfers, and
    /// summaries using Helius's IDL catalog. It is the successor to the Enhanced Transactions API
    /// ([`Helius::parse_transactions`], [`Helius::parsed_transaction_history`]). Requires an API
    /// key, and is mainnet only. Each request costs 10 credits, plus 1 per history page scanned
    /// for [`ParsedEvents::scan`].
    ///
    /// # Example
    /// ```no_run
    /// # use helius::error::Result;
    /// # use helius::types::parsed_events::{ParseTransactionsRequest, ParserStatus};
    /// # use helius::types::Cluster;
    /// # use helius::Helius;
    /// # async fn run() -> Result<()> {
    /// let helius: Helius = Helius::new("your_api_key", Cluster::MainnetBeta)?;
    ///
    /// let results = helius
    ///     .parsed_events()
    ///     .parse_transactions(ParseTransactionsRequest {
    ///         transactions: vec!["5xSKzM8bvpudE521jikHqASzMr23Ms4X4ieY3K8oFPFrJWCSSgYocJmHznrR8b12voDxKDH8ykdCLXSRrx6duVLH".to_string()],
    ///         ..Default::default()
    ///     })
    ///     .await?;
    ///
    /// for result in results {
    ///     match (result.parser_status, result.parsed, result.parser_error) {
    ///         (ParserStatus::Ok, Some(parsed), _) => println!("{:?}", parsed.summary.map(|s| s.description)),
    ///         (_, _, error) => eprintln!("{}: {:?}", result.signature, error),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn parsed_events(&self) -> ParsedEvents<'_> {
        ParsedEvents { helius: self }
    }
}

/// Client for the [Parsed Events API](https://www.helius.dev/docs/parsed-events), returned by
/// [`Helius::parsed_events`]
pub struct ParsedEvents<'a> {
    helius: &'a Helius,
}

impl ParsedEvents<'_> {
    /// Parses transaction signatures (max 100) into decoded instructions, transfers, and summaries
    ///
    /// Results come back in the same order as `transactions`, including duplicates. A missing or
    /// unparseable transaction does not fail the request: its item has
    /// `parser_status: ParserStatus::Error` and a `parser_error`, so check `parser_status` before
    /// reading `parsed`. Failed transactions still parse; check `parsed.transaction_status`.
    ///
    /// # Errors
    /// Returns `HeliusError::InvalidInput` if no API key is configured or the client is not on
    /// mainnet, and the status-mapped variant on a request-level failure (invalid body, auth,
    /// rate limit)
    pub async fn parse_transactions(&self, request: ParseTransactionsRequest) -> Result<Vec<ParsedTransactionResult>> {
        let url: Url = self.url("transactions")?;

        self.helius
            .rpc_client
            .handler
            .send(Method::POST, url, Some(&request))
            .await
    }

    /// Parses transaction signatures, returning the response body undecoded
    ///
    /// Identical to [`parse_transactions`](Self::parse_transactions) on the wire and in error
    /// handling, but the JSON is handed back as `Bytes`. Decode it with
    /// [`decode_response`](crate::request_handler::decode_response), typically inside
    /// `tokio::task::spawn_blocking`, so a large batch does not park an async worker.
    pub async fn parse_transactions_raw(&self, request: ParseTransactionsRequest) -> Result<Bytes> {
        let url: Url = self.url("transactions")?;

        self.helius
            .rpc_client
            .handler
            .send_raw(Method::POST, url, Some(&request))
            .await
    }

    /// Gets a page of parsed transaction history for an address
    ///
    /// Returns up to 100 parsed transactions per page, newest first by default. Pagination is
    /// manual: pass the returned `pagination_token` back to fetch the next page; it is `None`
    /// once the end of the range is reached. Filter by slot or block time with the `slot` and
    /// `time` bounds. Each item has the same shape as a [`parse_transactions`](Self::parse_transactions)
    /// result, so check `parser_status` before reading `parsed`.
    ///
    /// # Example
    /// ```no_run
    /// # use helius::error::Result;
    /// # use helius::types::parsed_events::{ComparisonBounds, GetTransactionHistoryRequest};
    /// # use helius::Helius;
    /// # async fn run(helius: Helius) -> Result<()> {
    /// let mut pagination_token: Option<String> = None;
    /// loop {
    ///     let page = helius
    ///         .parsed_events()
    ///         .get_transaction_history(GetTransactionHistoryRequest {
    ///             address: "86xCnPeV69n6t3DnyGvkKobf9FdN2H9oiVDdaMpo2MMY".to_string(),
    ///             time: Some(ComparisonBounds { gte: Some(1_767_225_600), ..Default::default() }),
    ///             pagination_token,
    ///             ..Default::default()
    ///         })
    ///         .await?;
    ///
    ///     for result in &page.data {
    ///         println!("{}", result.signature);
    ///     }
    ///
    ///     pagination_token = page.pagination_token;
    ///     if pagination_token.is_none() {
    ///         break;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `HeliusError::InvalidInput` if no API key is configured or the client is not on
    /// mainnet, and the status-mapped variant on a request-level failure (invalid body, auth,
    /// rate limit)
    pub async fn get_transaction_history(
        &self,
        request: GetTransactionHistoryRequest,
    ) -> Result<GetTransactionHistoryResponse> {
        let url: Url = self.url("transaction-history")?;

        self.helius
            .rpc_client
            .handler
            .send(Method::POST, url, Some(&request))
            .await
    }

    /// Gets a page of parsed transaction history, returning the response body undecoded
    ///
    /// Identical to [`get_transaction_history`](Self::get_transaction_history) on the wire and
    /// in error handling, but the JSON is handed back as `Bytes`. A full page of history is a
    /// large payload, so this is the method to reach for when decoding must not block the
    /// runtime. Decode with [`decode_response`](crate::request_handler::decode_response) on a
    /// blocking thread.
    pub async fn get_transaction_history_raw(&self, request: GetTransactionHistoryRequest) -> Result<Bytes> {
        let url: Url = self.url("transaction-history")?;

        self.helius
            .rpc_client
            .handler
            .send_raw(Method::POST, url, Some(&request))
            .await
    }

    /// Scans an address's history server-side and returns only the transactions that match
    /// `criteria`, parsed
    ///
    /// Match on a program's instructions (by IDL name or data discriminator), on token mints,
    /// or both. Use this instead of paging through
    /// [`get_transaction_history`](Self::get_transaction_history) and filtering client-side.
    /// Each item in `data` has the same shape as a
    /// [`parse_transactions`](Self::parse_transactions) result.
    ///
    /// One request stops at the first of: `limit` matches, `budget.max_scanned_transactions`
    /// scanned (10,000 by default), the end of the history or window, or the server's time
    /// limit. `scan.stopped_on` says which. A stopped scan is still a success: call again with
    /// the returned `pagination_token` while one is present, **even when `data` is empty**.
    ///
    /// Costs 10 credits plus 1 per history page scanned (up to 1,000 transactions each), so at
    /// most 20 credits with the default budget. `scan.credits_charged` reports the charge.
    ///
    /// # Example
    /// ```no_run
    /// # use helius::error::Result;
    /// # use helius::types::parsed_events::{ScanMatch, ScanProgramMatch, ScanRequest};
    /// # use helius::Helius;
    /// # async fn run(helius: Helius) -> Result<()> {
    /// const PUMP_FUN: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
    ///
    /// let mut pagination_token: Option<String> = None;
    /// loop {
    ///     let page = helius
    ///         .parsed_events()
    ///         .scan(ScanRequest {
    ///             address: PUMP_FUN.to_string(),
    ///             criteria: ScanMatch {
    ///                 program: Some(ScanProgramMatch {
    ///                     program_id: PUMP_FUN.to_string(),
    ///                     instructions: Some(vec!["buy".to_string(), "sell".to_string()]),
    ///                     ..Default::default()
    ///                 }),
    ///                 ..Default::default()
    ///             },
    ///             pagination_token,
    ///             ..Default::default()
    ///         })
    ///         .await?;
    ///
    ///     for result in &page.data {
    ///         println!("{}", result.signature);
    ///     }
    ///
    ///     // Keep going while a token is returned, even when this page had no matches
    ///     pagination_token = page.pagination_token;
    ///     if pagination_token.is_none() {
    ///         break;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns `HeliusError::InvalidInput` if no API key is configured or the client is not on
    /// mainnet. The server's `400` (e.g. `criteria` sets neither `program` nor `mints`, an
    /// instruction name is not in the program's IDLs, or a limit is out of range) surfaces as
    /// `HeliusError::BadRequest` with its message. A `503` (at capacity, or the first history
    /// page missed the time limit) surfaces as `HeliusError::Unknown`; retry it
    pub async fn scan(&self, request: ScanRequest) -> Result<ScanResponse> {
        let url: Url = self.url("scan")?;

        self.helius
            .rpc_client
            .handler
            .send(Method::POST, url, Some(&request))
            .await
    }

    /// Scans an address's history, returning the response body undecoded
    ///
    /// Identical to [`scan`](Self::scan) on the wire and in error handling, but the JSON is
    /// handed back as `Bytes`. Decode it with
    /// [`decode_response`](crate::request_handler::decode_response) on a blocking thread when
    /// pages carry many parsed transactions.
    pub async fn scan_raw(&self, request: ScanRequest) -> Result<Bytes> {
        let url: Url = self.url("scan")?;

        self.helius
            .rpc_client
            .handler
            .send_raw(Method::POST, url, Some(&request))
            .await
    }

    /// Builds the URL for a Parsed Events endpoint, after checking the API key and network
    ///
    /// Requests always go to Helius mainnet, as in the TS SDK, whatever the configured RPC URL.
    /// The one exception is a hand-built [`Config`](crate::config::Config) (no `custom_url`)
    /// whose RPC endpoint is a loopback address, which is used as-is so the endpoints can be
    /// mocked in tests. A loopback custom URL, such as a local validator, still goes to Helius.
    fn url(&self, path: &str) -> Result<Url> {
        let config = &self.helius.config;
        let api_key = config.require_api_key("Parsed Events")?;
        ensure_mainnet(config, "Parsed Events")?;
        let rpc_url: Url = Url::parse(&config.endpoints.rpc)?;

        let mut url: Url = if config.custom_url.is_none() && is_loopback_host(&rpc_url) {
            let mut url = rpc_url;
            let base_path = url.path().trim_end_matches('/').to_string();
            url.set_query(None);
            url.set_path(&format!("{base_path}/v1/parsed-events/{path}"));
            url
        } else {
            Url::parse(&format!("{PARSED_EVENTS_BASE_URL}/{path}"))?
        };
        url.query_pairs_mut().append_pair("api-key", api_key.as_str());

        Ok(url)
    }
}

/// Fails unless `config` is on mainnet, for a mainnet-only `product`
///
/// The check fails closed, so a non-mainnet client never silently gets mainnet data: the cluster
/// must be a mainnet one, and the RPC host must not name another network. A custom URL defaults
/// the cluster to `Devnet`, so a mainnet custom URL needs an explicit
/// `with_cluster(Cluster::MainnetBeta)`.
pub(crate) fn ensure_mainnet(config: &Config, product: &str) -> Result<()> {
    let rpc_url: Url = Url::parse(&config.endpoints.rpc)?;

    if config.cluster == Cluster::Devnet {
        let hint = if config.custom_url.is_some() {
            ". For a mainnet custom URL, set .with_cluster(Cluster::MainnetBeta) on HeliusBuilder"
        } else {
            ""
        };
        return Err(HeliusError::InvalidInput(format!(
            "{product} is only available on mainnet{hint}"
        )));
    }
    if is_non_mainnet_host(&rpc_url) {
        return Err(HeliusError::InvalidInput(format!(
            "{product} is only available on mainnet, but the RPC URL host {} names another network",
            rpc_url.host_str().unwrap_or_default()
        )));
    }

    Ok(())
}

/// Whether a hostname label is `devnet` or `testnet` (e.g. `devnet.helius-rpc.com`,
/// `abc-fast-devnet.helius-rpc.com`, `api.testnet.solana.com`)
fn is_non_mainnet_host(url: &Url) -> bool {
    url.host_str().is_some_and(|host| {
        host.split(['.', '-'])
            .any(|label| label.eq_ignore_ascii_case("devnet") || label.eq_ignore_ascii_case("testnet"))
    })
}

fn is_loopback_host(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(ip)) => ip.is_loopback(),
        Some(Host::Ipv6(ip)) => ip.is_loopback(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::is_non_mainnet_host;
    use crate::config::Config;
    use crate::error::HeliusError;
    use crate::types::{ApiKey, Cluster, HeliusEndpoints};
    use crate::Helius;
    use reqwest::Url;

    fn transactions_url(config: Config) -> Result<String, HeliusError> {
        let helius: Helius = config.create_client().unwrap();
        helius.parsed_events().url("transactions").map(|url| url.to_string())
    }

    fn config(rpc: &str, cluster: Cluster, custom: bool) -> Config {
        Config {
            api_key: Some(ApiKey::new("k").unwrap()),
            cluster,
            endpoints: HeliusEndpoints {
                api: rpc.to_string(),
                rpc: rpc.to_string(),
            },
            custom_url: custom.then(|| rpc.to_string()),
        }
    }

    const MAINNET_URL: &str = "https://mainnet.helius-rpc.com/v1/parsed-events/transactions?api-key=k";

    #[test]
    fn url_targets_helius_mainnet() {
        let mainnet = Config::new("k", Cluster::MainnetBeta).unwrap();
        assert_eq!(transactions_url(mainnet.clone()).unwrap(), MAINNET_URL);

        // RPC-only query parameters stay off the Parsed Events URL
        assert_eq!(transactions_url(mainnet.with_mev_protect(true)).unwrap(), MAINNET_URL);

        let staked = Config::new("k", Cluster::StakedMainnetBeta).unwrap();
        assert_eq!(transactions_url(staked).unwrap(), MAINNET_URL);

        // As in the TS SDK, a mainnet custom URL still queries Helius mainnet
        let proxy = config("https://proxy.example.com/rpc?x=1", Cluster::MainnetBeta, true);
        assert_eq!(transactions_url(proxy).unwrap(), MAINNET_URL);

        // So does a loopback custom URL, such as a local validator
        let local_validator = config("http://localhost:8899", Cluster::MainnetBeta, true);
        assert_eq!(transactions_url(local_validator).unwrap(), MAINNET_URL);
    }

    #[test]
    fn url_uses_a_hand_built_loopback_endpoint_as_is() {
        let mocked = config(
            "http://127.0.0.1:1234/base/?mev-protect=true",
            Cluster::MainnetBeta,
            false,
        );
        assert_eq!(
            transactions_url(mocked).unwrap(),
            "http://127.0.0.1:1234/base/v1/parsed-events/transactions?api-key=k"
        );
    }

    /// A custom URL defaults the cluster to Devnet, and only an explicit mainnet cluster opts in,
    /// so devnet nodes the host check cannot recognize (an IP, a path-based URL) are still refused
    #[test]
    fn url_rejects_a_custom_url_without_a_mainnet_cluster() {
        for rpc in ["https://rpc.example.com/solana_devnet", "http://10.0.0.5:8899"] {
            match transactions_url(config(rpc, Cluster::Devnet, true)) {
                Err(HeliusError::InvalidInput(msg)) => assert!(msg.contains("with_cluster"), "{msg}"),
                other => panic!("{rpc}: expected InvalidInput, got {other:?}"),
            }
        }
    }

    #[test]
    fn url_rejects_a_non_mainnet_host_even_with_a_mainnet_cluster() {
        let testnet = config("https://api.testnet.solana.com/", Cluster::MainnetBeta, true);
        assert!(matches!(transactions_url(testnet), Err(HeliusError::InvalidInput(_))));
    }

    #[test]
    fn non_mainnet_host_matches_a_devnet_or_testnet_label_only() {
        let non_mainnet = |url: &str| is_non_mainnet_host(&Url::parse(url).unwrap());

        assert!(non_mainnet("https://devnet.helius-rpc.com/"));
        assert!(non_mainnet("https://devnet.helius-rpc.com./"));
        assert!(non_mainnet("https://abc-fast-devnet.helius-rpc.com/"));
        assert!(non_mainnet("https://api.devnet.solana.com/"));
        assert!(non_mainnet("https://my-devnet-proxy.example.com/"));
        assert!(non_mainnet("https://api.testnet.solana.com/"));

        assert!(!non_mainnet("https://mainnet.helius-rpc.com/"));
        assert!(!non_mainnet("https://devnetwork.example.com/"));
        assert!(!non_mainnet("https://rpc.example.com/devnet"));
    }
}
