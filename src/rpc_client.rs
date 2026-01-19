/// # RPC Client for Helius
///
/// This module provides access to the Helius API using an RPC client with an embedded Solana client
///
/// ## Errors
///
/// Most methods in this client will return a `Result<T, HeliusError>`, where `HeliusError` can be:
/// - `BadRequest`: Incorrect request format or parameters. Check the path and the text for details
/// - `Unauthorized`: Incorrect or missing API key. Ensure you've provided the correct API key
/// - `NotFound`: The requested resource was not found. This could mean an invalid ID or a non-existent endpoint
/// - `RateLimitExceeded`: Too many requests have been sent in a short period. Consider implementing retries with an exponential backoff
/// - `InternalError`: Server-side errors. These are rare and typically indicate issues on the server side. If these issues persist, please contact Helius support
/// - `Network`: Errors during HTTP communication, typically from underlying network issues
/// - `SerdeJson`: Errors during the serialization or deserialization process
/// - `Unknown`: Catch-all for unclassified errors, with a status code and message provided for further investigation
///
/// Ensure to handle these errors gracefully in your application to maintain robustness and stellar UX
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;
use crate::request_handler::RequestHandler;
use crate::types::inner::{RpcRequest, RpcResponse};
use crate::types::{
    Asset, AssetList, AssetProof, EditionsList, GetAsset, GetAssetBatch, GetAssetProof, GetAssetProofBatch,
    GetAssetSignatures, GetAssetsByAuthority, GetAssetsByCreator, GetAssetsByGroup, GetAssetsByOwner, GetNftEditions,
    GetPriorityFeeEstimateRequest, GetPriorityFeeEstimateResponse, GetProgramAccountsV2Config,
    GetProgramAccountsV2Request, GetProgramAccountsV2Response, GetTokenAccounts, GetTokenAccountsByOwnerV2Config,
    GetTokenAccountsByOwnerV2Request, GetTokenAccountsByOwnerV2Response, GetTransactionsForAddressOptions,
    GetTransactionsForAddressRequest, GetTransactionsForAddressResponse, GpaAccount, SearchAssets, TokenAccountRecord,
    TokenAccountsList, TokenAccountsOwnerFilter, TransactionSignatureList,
};

use reqwest::{Client, Method, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;
use solana_client::rpc_client::RpcClient as SolanaRpcClient;
use solana_commitment_config::CommitmentConfig;

pub struct RpcClient {
    pub handler: RequestHandler,
    pub config: Arc<Config>,
    pub solana_client: Arc<SolanaRpcClient>,
}

impl RpcClient {
    /// Initializes a new RpcClient instance with an embedded Solana client
    ///
    /// # Arguments
    /// * `client` - Shared HTTP client for making requests
    /// * `config` - Configuration holding a given API key and endpoint URLs
    ///
    /// # Returns
    /// A result that, if successful, contains the initialized RpcClient
    ///
    /// # Errors
    /// Returns `HeliusError` if the URL isn't formatted correctly or the `RequestHandler` fails to initialize
    pub fn new(client: Arc<Client>, config: Arc<Config>) -> Result<Self> {
        let handler: RequestHandler = RequestHandler::new(client)?;
        let url: String = format!("{}/?api-key={}", config.endpoints.rpc, config.api_key);
        let solana_client: Arc<SolanaRpcClient> = Arc::new(SolanaRpcClient::new(url));

        Ok(RpcClient {
            handler,
            config,
            solana_client,
        })
    }

    /// Initializes a new RpcClient instance with an embedded Solana client and a commitment config
    ///
    /// # Arguments
    /// * `client` - Shared HTTP client for making requests
    /// * `config` - Configuration holding a given API key and endpoint URLs
    /// * `commitment` - Commitment level to use for the Solana client
    ///
    /// # Returns
    /// A result that, if successful, contains the initialized RpcClient
    ///
    /// # Errors
    /// Returns `HeliusError` if the URL isn't formatted correctly or the `RequestHandler` fails to initialize
    pub fn new_with_commitment(client: Arc<Client>, config: Arc<Config>, commitment: CommitmentConfig) -> Result<Self> {
        let handler: RequestHandler = RequestHandler::new(client)?;
        let url: String = format!("{}/?api-key={}", config.endpoints.rpc, config.api_key);
        let solana_client: Arc<SolanaRpcClient> = Arc::new(SolanaRpcClient::new_with_commitment(url, commitment));

        Ok(RpcClient {
            handler,
            config,
            solana_client,
        })
    }

    /// Streamlines an RPC POST request
    ///
    /// # Arguments
    /// * `method` - RPC method name as a string reference (e.g., "getAsset")
    /// * `request` - Request data for a given method that conforms to the Debug, Serialize, Send, and Sync traits
    ///
    /// # Returns
    /// A result that, if successful, contains the deserialized response data
    ///
    /// # Errors
    /// Returns `HeliusError` if the URL cannot be parsed or the HTTP request fails
    pub async fn post_rpc_request<R, T>(&self, method: &str, request: R) -> Result<T>
    where
        R: Debug + Serialize + Send + Sync,
        T: Debug + DeserializeOwned + Default,
    {
        let base_url: String = format!("{}/?api-key={}", self.config.endpoints.rpc, self.config.api_key);
        let url: Url = Url::parse(&base_url).expect("Failed to parse URL");

        let rpc_request: RpcRequest<R> = RpcRequest::new(method.to_string(), request);
        let rpc_response: RpcResponse<T> = self.handler.send(Method::POST, url, Some(&rpc_request)).await?;

        Ok(rpc_response.result)
    }

    /// Gets an asset by its ID
    ///
    /// # Arguments
    /// * `request` - A struct containing the ID of the asset to fetch, along with other optional sorting and display options
    ///
    /// # Returns
    /// A `Result` with an optional `Asset` object if found. It can also return `None` if no asset matches the ID provided
    pub async fn get_asset(&self, request: GetAsset) -> Result<Option<Asset>> {
        self.post_rpc_request("getAsset", request).await
    }

    /// Gets multiple assets by their ID
    ///
    /// # Arguments
    /// * `request` - A struct containing the IDs of the assets to fetch in a batch
    ///
    /// # Returns
    /// A `Result` containing a vector of optional `Asset` objects, each corresponding to the IDs provided. It can also return `None` if no assets match the IDs provided
    pub async fn get_asset_batch(&self, request: GetAssetBatch) -> Result<Vec<Option<Asset>>> {
        self.post_rpc_request("getAssetBatch", request).await
    }

    /// Gets a merkle proof for a compressed asset by its ID
    ///
    /// # Arguments
    /// * `request` - A struct containing the ID of the asset for which the proof is requested
    ///
    /// # Returns
    /// A `Result` with an optional `AssetProof` object if the asset exists and the proof is retrievable. It can also return `None` if the proof doesn't exist or isn't retrievable
    pub async fn get_asset_proof(&self, request: GetAssetProof) -> Result<Option<AssetProof>> {
        self.post_rpc_request("getAssetProof", request).await
    }

    /// Gets multiple asset proofs by their IDs
    ///
    /// # Arguments
    /// * `request` - A struct containing the IDs of the assets for which proofs are requested
    ///
    /// # Returns
    /// A `Result` with a hashmap where each key is an asset ID and each value is an optional `AssetProof`. It can also return `None` if no proofs correspond to the IDs provided
    pub async fn get_asset_proof_batch(
        &self,
        request: GetAssetProofBatch,
    ) -> Result<HashMap<String, Option<AssetProof>>> {
        self.post_rpc_request("getAssetProofBatch", request).await
    }

    /// Gets a list of assets of a given authority
    ///
    /// # Arguments
    /// * `request` - A struct containing the authority's address, along with other optional sorting and display options
    ///
    /// # Returns
    /// A `Result` containing an `AssetList` detailing the assets managed by the specified authority. It can also return `None` if no assets are managed by the given authority
    pub async fn get_assets_by_authority(&self, request: GetAssetsByAuthority) -> Result<AssetList> {
        self.post_rpc_request("getAssetsByAuthority", request).await
    }

    /// Gets a list of assets of a given creator
    ///
    /// # Arguments
    /// * `request` - A struct containing the creator's address and optional filters for verification, sorting, and display options
    ///
    /// # Returns
    /// A `Result` containing an `AssetList` of assets created by the specified address. It can also return `None` if the specified address hasn't created any assets
    pub async fn get_assets_by_creator(&self, request: GetAssetsByCreator) -> Result<AssetList> {
        self.post_rpc_request("getAssetsByCreator", request).await
    }

    /// Gets a list of assets by a group key and value
    ///
    /// # Arguments
    /// * `request` - A struct that defines a group key and value to query, along with other optional sorting and display options
    ///
    /// # Returns
    /// A `Result` containing an `AssetList` that matches the group criteria. It can also return `None` if no assets match the group criteria
    pub async fn get_assets_by_group(&self, request: GetAssetsByGroup) -> Result<AssetList> {
        self.post_rpc_request("getAssetsByGroup", request).await
    }

    /// Gets a list of assets owned by a given address
    ///
    /// # Arguments
    /// * `request` - A struct containing the owner's address, along with optional sorting, pagination, and display options
    ///
    /// # Returns
    /// A `Result` containing an `AssetList` of assets owned by the specified address. It can also return `None` if the specified address doesn't own any assets
    pub async fn get_assets_by_owner(&self, request: GetAssetsByOwner) -> Result<AssetList> {
        self.post_rpc_request("getAssetsByOwner", request).await
    }

    /// Gets assets based on the custom search criteria passed in
    ///
    /// # Arguments
    /// * `request` - A struct that specifies the search conditions, filtering options, and sorting preferences
    ///
    /// # Returns
    /// A `Result` containing an `AssetList` of assets that meet the search criteria. It can also return `None` if no assets match the search criteria
    pub async fn search_assets(&self, request: SearchAssets) -> Result<AssetList> {
        self.post_rpc_request("searchAssets", request).await
    }

    /// Gets transaction signatures for a given asset
    ///
    /// # Arguments
    /// * `request` - A struct that includes the asset's ID and optional pagination settings
    ///
    /// # Returns
    /// A `Result` containing a `TransactionSignatureList` detailing the transactions involving the specified asset. It can also return `None` if any transactions involving the specified asset cannot be retrieved
    pub async fn get_signatures_for_asset(&self, request: GetAssetSignatures) -> Result<TransactionSignatureList> {
        self.post_rpc_request("getSignaturesForAsset", request).await
    }

    /// Gets information about all token accounts for a specific mint or owner
    ///
    /// # Arguments
    /// * `request` - A struct that includes the owner or mint address, along with optional sorting, pagination, and display options
    ///
    /// # Returns
    /// A `Result` containing a `TokenAccountsList` detailing the token accounts matching the request parameters. It can also return `None` if the specified address has no token accounts
    pub async fn get_token_accounts(&self, request: GetTokenAccounts) -> Result<TokenAccountsList> {
        self.post_rpc_request("getTokenAccounts", request).await
    }

    /// Gets all the NFT editions  associated with a specific master NFT
    ///
    /// # Arguments
    /// * `request` - A struct that includes the master NFT's ID and optional pagination settings
    ///
    /// # Returns
    /// A `Result` containing an `EditionsList` of all the editions linked to the master NFT. It can also return `None` if there aren't any editions linked to the specified master NFT
    pub async fn get_nft_editions(&self, request: GetNftEditions) -> Result<EditionsList> {
        self.post_rpc_request("getNftEditions", request).await
    }

    /// Gets an estimate of the priority fees required for a transaction to be processed more quickly
    ///
    /// This method calculates varying levels of transaction fees that can influence the priority of a transaction, based on current network conditions
    ///
    /// # Arguments
    /// * `request` - A struct that includes the following:
    /// * `transaction` - Optionally, the serialized transaction for which the fee estimate is requested
    /// * `account_key` - Optionally, a list of account public keys involved in a given transaction to help determine the necessary priority fee based on the accounts' recent activity
    /// * `options` - Additional options for fine-tuning the request, such as the desired priority level or the number of slots to look back and consider for the estimate
    ///
    /// # Returns
    /// A `Result` that, if successful, wraps the `GetPriorityFeeEstimateResponse` struct, containing:
    /// - `priority_fee_estimate`: The estimated priority fee in micro lamports
    /// - `priority_fee_levels`: A detailed breakdown of potential priority fees at various levels
    pub async fn get_priority_fee_estimate(
        &self,
        request: GetPriorityFeeEstimateRequest,
    ) -> Result<GetPriorityFeeEstimateResponse> {
        self.post_rpc_request("getPriorityFeeEstimate", vec![request]).await
    }

    /// An enhanced version of getProgramAccounts with cursor-based pagination and changedSlotSince support for efficiently querying large sets of accounts owned by specific Solana
    /// programs with incremental updates.
    ///
    /// # Arguments
    /// * `program_id` - The given program's public key to query accounts for, as a base58 encoded string
    /// * `config` - A config struct that controls the encoding, pagination, memcmp/data size filters, and incremental updates defined by the type `GetProgramAccountsV2Config`
    ///
    /// # Returns
    /// A `GetProgramAccountsV2Response` with:
    /// * `accounts` - The page of program accounts, each with a pubkey and the relevant account info
    /// * `pagination_key` - The cursor for the next page
    /// * `total_results` - The total matches, if available. Otherwise, it returns `None`
    ///
    /// # Pagination
    /// * If `pagination_key` is `Some`, pass it into the **next** request to continue
    /// * If `pagination_key` is `None`, ypu've reached the end
    /// * Note that if there are fewer than `limit` accounts in a given page it does not imply the end; always check the cursor
    ///
    /// # Incremental Updates
    /// * Set `config.changed_since_slot = Some(slot)` to return only the accounts modified at or after that slot
    /// * Omit `changed_since_slot` for a full scan
    pub async fn get_program_accounts_v2(
        &self,
        program_id: String,
        config: GetProgramAccountsV2Config,
    ) -> Result<GetProgramAccountsV2Response> {
        let params: GetProgramAccountsV2Request = (program_id, config);
        self.post_rpc_request("getProgramAccountsV2", params).await
    }

    /// An enhanced version of getTokenAccountsByOwner with cursor-based pagination and changedSinceSlot support to incrementally
    /// retrieve SPL token accounts owned by a given wallet.
    ///
    /// # Arguments
    /// * `owner` - The Base58 wallet address whose token accounts you want to fetch
    /// * `filter` - The filtering options for the token accounts fetched defined by `TokenAccountsOwnerFilter` (e.g., limiting the mint or program ID)
    /// * `config` - A config struct that controls the encoding, pagination, and `changed_since_slot` defined by the type `GetTokenAccountsByOwnerV2Config`
    ///
    /// # Returns
    /// A `GetTokenAccountsByOwnerV2Response` with:
    /// * `context` - An optional RPC context (i.e., slot and API version)
    /// * `value` - The page of token accounts, each with a pubkey and the relevant account info
    /// * `pagination_key` - The cursor for the next page
    /// * `total_results` - The total matches, if available. Otherwise, it returns `None`
    ///     
    /// # Pagination
    /// * If `pagination_key` is `Some`, pass it into the **next** request to continue
    /// * If `pagination_key` is `None`, ypu've reached the end
    /// * Note that if there are fewer than `limit` accounts in a given page it does not imply the end; always check the cursor
    ///
    /// # Incremental Updates
    /// * Set `config.changed_since_slot = Some(slot)` to return only the accounts modified at or after that slot
    /// * Omit `changed_since_slot` for a full scan
    pub async fn get_token_accounts_by_owner_v2(
        &self,
        owner: String,
        filter: TokenAccountsOwnerFilter,
        config: GetTokenAccountsByOwnerV2Config,
    ) -> Result<GetTokenAccountsByOwnerV2Response> {
        let params: GetTokenAccountsByOwnerV2Request = (owner, filter, config);
        self.post_rpc_request("getTokenAccountsByOwnerV2", params).await
    }

    /// Get all program accounts by auto-paginating through results
    ///
    /// Automatically handles pagination to fetch all accounts. Please use with caution for programs with many accounts
    ///
    /// # Arguments
    /// * `program_id` - The program ID to query
    /// * `config` - A config struct that controls the encoding, pagination, memcmp/data size filters, and incremental updates defined by the type `GetProgramAccountsV2Config`.
    ///   Note `limit` defaults to `10000`, if not provided, and the `pagination_key` is ignored since the method auto-paginates
    ///
    /// # Returns
    /// A vector of all program accounts
    pub async fn get_all_program_accounts(
        &self,
        program_id: String,
        mut config: GetProgramAccountsV2Config,
    ) -> Result<Vec<GpaAccount>> {
        // Ignore the user-provided pagination_key since we auto-paginate
        config.pagination_key = None;

        // Default to 10k
        if config.limit.is_none() {
            config.limit = Some(10000);
        }

        let mut all_accounts: Vec<GpaAccount> = Vec::new();
        loop {
            let response: GetProgramAccountsV2Response =
                self.get_program_accounts_v2(program_id.clone(), config.clone()).await?;
            all_accounts.extend(response.accounts);

            println!("Fetched {} accounts so far", all_accounts.len());

            if let Some(key) = response.pagination_key {
                config.pagination_key = Some(key);
            } else {
                break;
            }
        }

        Ok(all_accounts)
    }

    /// Get all token accounts by owner by auto-paginating through results
    ///
    /// # Arguments
    /// * `owner` - The Base58 wallet address whose token accounts you want to fetch
    /// * `filter` - Filter by mint or programId
    /// * `config` - A config struct that controls the encoding, pagination, and `changed_since_slot` defined by the type `GetTokenAccountsByOwnerV2Config`.
    ///   Note `limit` defaults to `10000`, if not provided, and the `pagination_key` is ignored since the method auto-paginates
    ///
    /// # Returns
    /// A vector of all token account records
    pub async fn get_all_token_accounts_by_owner(
        &self,
        owner: String,
        filter: TokenAccountsOwnerFilter,
        mut config: GetTokenAccountsByOwnerV2Config,
    ) -> Result<Vec<TokenAccountRecord>> {
        // Ignore the user-provided pagination_key since we auto-paginate
        config.pagination_key = None;

        // Default to 10k
        if config.limit.is_none() {
            config.limit = Some(10000);
        }

        let mut all_accounts: Vec<TokenAccountRecord> = Vec::new();
        loop {
            let response: GetTokenAccountsByOwnerV2Response = self
                .get_token_accounts_by_owner_v2(owner.clone(), filter.clone(), config.clone())
                .await?;
            all_accounts.extend(response.value.accounts);

            if let Some(key) = response.value.pagination_key {
                config.pagination_key = Some(key);
            } else {
                break;
            }
        }

        Ok(all_accounts)
    }

    /// Gets transactions for a specific address with advanced filtering and sorting
    ///
    /// # Arguments
    /// * `address` - The base58 encoded public key of the account
    /// * `options` - Options for filtering, sorting, and pagination
    ///
    /// # Returns
    /// A `Result` containing the transaction data and an optional pagination token.
    pub async fn get_transactions_for_address(
        &self,
        address: String,
        options: GetTransactionsForAddressOptions,
    ) -> Result<GetTransactionsForAddressResponse> {
        let params: GetTransactionsForAddressRequest = (address, options);
        self.post_rpc_request("getTransactionsForAddress", params).await
    }
}
