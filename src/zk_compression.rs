/// # ZK Compression API
///
/// This module provides methods for interacting with the ZK Compression protocol on Solana.
///
/// ZK Compression reduces on-chain storage costs by compressing account state into concurrent
/// Merkle trees. Instead of storing full account data in individual Solana accounts, compressed
/// state is verified via zero-knowledge proofs and stored in shared tree structures, dramatically
/// lowering rent costs while preserving the same security guarantees.
///
/// All methods in this module are implemented on the [`Helius`] client and use JSON-RPC to
/// communicate with the ZK Compression RPC endpoints.
///
/// ## Errors
///
/// Methods return `Result<T, HeliusError>` where `HeliusError` can be:
/// - `BadRequest`: Incorrect request format or parameters
/// - `Unauthorized`: Missing or invalid API key
/// - `NotFound`: The requested resource was not found
/// - `RateLimitExceeded`: Too many requests; consider exponential backoff
/// - `InternalError`: Server-side errors
/// - `Network`: Underlying HTTP communication errors
/// - `SerdeJson`: Serialization or deserialization errors
use crate::error::Result;
use crate::types::{
    AddressWithTree, GetCompressedAccountProofRequest, GetCompressedAccountProofResponse, GetCompressedAccountRequest,
    GetCompressedAccountResponse, GetCompressedAccountsByOwnerRequest, GetCompressedAccountsByOwnerResponse,
    GetCompressedBalanceByOwnerRequest, GetCompressedBalanceResponse, GetCompressedMintTokenHoldersRequest,
    GetCompressedMintTokenHoldersResponse, GetCompressedTokenAccountBalanceResponse,
    GetCompressedTokenAccountsByDelegateRequest, GetCompressedTokenAccountsByDelegateResponse,
    GetCompressedTokenAccountsByOwnerRequest, GetCompressedTokenAccountsByOwnerResponse,
    GetCompressedTokenBalancesByOwnerRequest, GetCompressedTokenBalancesByOwnerResponse,
    GetCompressedTokenBalancesByOwnerV2Response, GetCompressionSignaturesForAccountRequest,
    GetCompressionSignaturesForAccountResponse, GetCompressionSignaturesForAddressRequest,
    GetCompressionSignaturesForAddressResponse, GetCompressionSignaturesForOwnerRequest,
    GetCompressionSignaturesForOwnerResponse, GetCompressionSignaturesForTokenOwnerResponse,
    GetLatestCompressionSignaturesRequest, GetLatestCompressionSignaturesResponse,
    GetLatestNonVotingSignaturesResponse, GetMultipleCompressedAccountProofsResponse,
    GetMultipleCompressedAccountsRequest, GetMultipleCompressedAccountsResponse, GetMultipleNewAddressProofsResponse,
    GetMultipleNewAddressProofsV2Response, GetTransactionWithCompressionInfoRequest,
    GetTransactionWithCompressionInfoResponse, GetValidityProofRequest, GetValidityProofResponse,
};
use crate::Helius;

impl Helius {
    /// Retrieves a compressed account by its address or data hash.
    ///
    /// Compressed accounts are stored in concurrent Merkle trees and verified via
    /// zero-knowledge proofs, offering up to 1000x cost savings compared to regular
    /// Solana accounts. Provide either an `address` or a `hash` to identify the account.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedAccountRequest`] containing either the account's
    ///   base58-encoded public key (`address`) or its data hash (`hash`)
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedAccountResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — the [`CompressedAccount`](crate::types::CompressedAccount), or `None`
    ///   if no account matches
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - Neither `address` nor `hash` is provided (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedAccountRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedAccountRequest {
    ///         hash: Some("11111111111111111111111111111111".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compressed_account(request).await.unwrap();
    ///     if let Some(account) = response.value {
    ///         println!("Owner: {}", account.owner);
    ///         println!("Lamports: {}", account.lamports);
    ///         println!("Tree: {}", account.tree);
    ///     }
    /// }
    /// ```
    pub async fn get_compressed_account(
        &self,
        request: GetCompressedAccountRequest,
    ) -> Result<GetCompressedAccountResponse> {
        self.rpc_client.post_rpc_request("getCompressedAccount", request).await
    }

    /// Retrieves the Merkle proof for a compressed account.
    ///
    /// The proof contains the full path from the leaf (compressed account) to the root of
    /// the Merkle tree, enabling cryptographic verification of the account's existence and
    /// data integrity without requiring the full tree data on-chain.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedAccountProofRequest`] containing the base58-encoded
    ///   hash of the compressed account
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedAccountProofResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — the [`MerkleProofWithContext`](crate::types::MerkleProofWithContext)
    ///   including the proof path, root hash, tree address, and leaf index
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The hash is invalid or missing (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - No account matches the provided hash (`NotFound`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedAccountProofRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedAccountProofRequest {
    ///         hash: "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
    ///     };
    ///     let response = helius.get_compressed_account_proof(request).await.unwrap();
    ///     let proof = response.value;
    ///     println!("Root: {}", proof.root);
    ///     println!("Leaf index: {}", proof.leaf_index);
    ///     println!("Proof length: {}", proof.proof.len());
    /// }
    /// ```
    pub async fn get_compressed_account_proof(
        &self,
        request: GetCompressedAccountProofRequest,
    ) -> Result<GetCompressedAccountProofResponse> {
        self.rpc_client
            .post_rpc_request("getCompressedAccountProof", request)
            .await
    }

    /// Retrieves all compressed accounts owned by a given address.
    ///
    /// Returns a paginated list of compressed accounts belonging to the specified owner,
    /// with optional filtering by account data and data slicing to reduce response size.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedAccountsByOwnerRequest`] containing:
    ///   - `owner` — the base58-encoded public key of the account owner (required)
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `data_slice` — optional sub-range of each account's data to return
    ///   - `filters` — optional memcmp filters to narrow results
    ///   - `limit` — maximum number of accounts to return
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedAccountsByOwnerResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`PaginatedAccountList`](crate::types::PaginatedAccountList) with
    ///   matching accounts and an optional cursor for the next page
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The owner address is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedAccountsByOwnerRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedAccountsByOwnerRequest {
    ///         owner: "11111114d3RrygbPdAtMuFnDmzsN8T5fYKVQ7FVr7".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compressed_accounts_by_owner(request).await.unwrap();
    ///     println!("Found {} accounts", response.value.items.len());
    ///     if let Some(cursor) = response.value.cursor {
    ///         println!("Next page cursor: {}", cursor);
    ///     }
    /// }
    /// ```
    pub async fn get_compressed_accounts_by_owner(
        &self,
        request: GetCompressedAccountsByOwnerRequest,
    ) -> Result<GetCompressedAccountsByOwnerResponse> {
        self.rpc_client
            .post_rpc_request("getCompressedAccountsByOwner", request)
            .await
    }

    /// Retrieves the lamport balance of a compressed account.
    ///
    /// Returns the SOL balance (in lamports) for a compressed account identified by
    /// either its public key or data hash.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedAccountRequest`] containing either the account's
    ///   base58-encoded public key (`address`) or its data hash (`hash`)
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedBalanceResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — the balance in lamports (`u64`)
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - Neither `address` nor `hash` is provided (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedAccountRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedAccountRequest {
    ///         address: Some("11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compressed_balance(request).await.unwrap();
    ///     println!("Balance: {} lamports", response.value);
    /// }
    /// ```
    pub async fn get_compressed_balance(
        &self,
        request: GetCompressedAccountRequest,
    ) -> Result<GetCompressedBalanceResponse> {
        self.rpc_client.post_rpc_request("getCompressedBalance", request).await
    }

    /// Retrieves the total compressed balance for all accounts owned by a given address.
    ///
    /// Returns the aggregate SOL balance (in lamports) across all compressed accounts
    /// belonging to the specified owner.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedBalanceByOwnerRequest`] containing the owner's
    ///   base58-encoded public key
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedBalanceResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — the total balance in lamports (`u64`)
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The owner address is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedBalanceByOwnerRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedBalanceByOwnerRequest {
    ///         owner: "11111113R2cuenjG5nFubqX9Wzuukdin2YfGQVzu5".to_string(),
    ///     };
    ///     let response = helius.get_compressed_balance_by_owner(request).await.unwrap();
    ///     println!("Total compressed balance: {} lamports", response.value);
    /// }
    /// ```
    pub async fn get_compressed_balance_by_owner(
        &self,
        request: GetCompressedBalanceByOwnerRequest,
    ) -> Result<GetCompressedBalanceResponse> {
        self.rpc_client
            .post_rpc_request("getCompressedBalanceByOwner", request)
            .await
    }

    /// Retrieves all token holders for a compressed mint.
    ///
    /// Returns a paginated list of owners and their balances for the specified
    /// compressed token mint.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedMintTokenHoldersRequest`] containing:
    ///   - `mint` — the base58-encoded mint address (required)
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `limit` — maximum number of holders to return
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedMintTokenHoldersResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — an [`OwnerBalanceList`](crate::types::OwnerBalanceList) with holder
    ///   balances and an optional cursor for the next page
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The mint address is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedMintTokenHoldersRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedMintTokenHoldersRequest {
    ///         mint: "111111152P2r5yt6odmBLPsFCLBrFisJ3aS7LqLAT".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compressed_mint_token_holders(request).await.unwrap();
    ///     for holder in &response.value.items {
    ///         println!("Owner: {} — Balance: {}", holder.owner, holder.balance);
    ///     }
    /// }
    /// ```
    pub async fn get_compressed_mint_token_holders(
        &self,
        request: GetCompressedMintTokenHoldersRequest,
    ) -> Result<GetCompressedMintTokenHoldersResponse> {
        self.rpc_client
            .post_rpc_request("getCompressedMintTokenHolders", request)
            .await
    }

    /// Retrieves the token balance of a compressed token account.
    ///
    /// Returns the token amount for a compressed token account identified by
    /// either its public key or data hash.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedAccountRequest`] containing either the account's
    ///   base58-encoded public key (`address`) or its data hash (`hash`)
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedTokenAccountBalanceResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`TokenAccountBalance`](crate::types::TokenAccountBalance) with the
    ///   token amount
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - Neither `address` nor `hash` is provided (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedAccountRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedAccountRequest {
    ///         hash: Some("11111111111111111111111111111111".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compressed_token_account_balance(request).await.unwrap();
    ///     println!("Token balance: {}", response.value.amount);
    /// }
    /// ```
    pub async fn get_compressed_token_account_balance(
        &self,
        request: GetCompressedAccountRequest,
    ) -> Result<GetCompressedTokenAccountBalanceResponse> {
        self.rpc_client
            .post_rpc_request("getCompressedTokenAccountBalance", request)
            .await
    }

    /// Retrieves all compressed token accounts delegated to a given address.
    ///
    /// Returns a paginated list of compressed token accounts where the specified address
    /// has been granted delegate authority, with optional filtering by mint.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedTokenAccountsByDelegateRequest`] containing:
    ///   - `delegate` — the base58-encoded public key of the delegate (required)
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `limit` — maximum number of token accounts to return
    ///   - `mint` — optional mint address to filter results
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedTokenAccountsByDelegateResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`TokenAccountList`](crate::types::TokenAccountList) with matching
    ///   compressed token accounts and an optional cursor for the next page
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The delegate address is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedTokenAccountsByDelegateRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedTokenAccountsByDelegateRequest {
    ///         delegate: "11111116EPqoQskEM2Pddp8KTL9JdYEBZMGF3aq7V".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compressed_token_accounts_by_delegate(request).await.unwrap();
    ///     for token_account in &response.value.items {
    ///         println!(
    ///             "Mint: {} — Owner: {} — Amount: {}",
    ///             token_account.token_data.mint,
    ///             token_account.token_data.owner,
    ///             token_account.token_data.amount
    ///         );
    ///     }
    /// }
    /// ```
    pub async fn get_compressed_token_accounts_by_delegate(
        &self,
        request: GetCompressedTokenAccountsByDelegateRequest,
    ) -> Result<GetCompressedTokenAccountsByDelegateResponse> {
        self.rpc_client
            .post_rpc_request("getCompressedTokenAccountsByDelegate", request)
            .await
    }

    /// Retrieves all compressed token accounts owned by a given address.
    ///
    /// Returns a paginated list of compressed token accounts belonging to the specified
    /// owner, with optional filtering by mint. This is useful for building wallet portfolio
    /// views that include compressed SPL tokens and NFTs.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedTokenAccountsByOwnerRequest`] containing:
    ///   - `owner` — the base58-encoded public key of the wallet owner (required)
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `limit` — maximum number of token accounts to return
    ///   - `mint` — optional mint address to filter results
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedTokenAccountsByOwnerResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`TokenAccountList`](crate::types::TokenAccountList) with matching
    ///   compressed token accounts and an optional cursor for the next page
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The owner address is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedTokenAccountsByOwnerRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedTokenAccountsByOwnerRequest {
    ///         owner: "11111116EPqoQskEM2Pddp8KTL9JdYEBZMGF3aq7V".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compressed_token_accounts_by_owner(request).await.unwrap();
    ///     for token_account in &response.value.items {
    ///         println!(
    ///             "Mint: {} — Amount: {}",
    ///             token_account.token_data.mint,
    ///             token_account.token_data.amount
    ///         );
    ///     }
    /// }
    /// ```
    pub async fn get_compressed_token_accounts_by_owner(
        &self,
        request: GetCompressedTokenAccountsByOwnerRequest,
    ) -> Result<GetCompressedTokenAccountsByOwnerResponse> {
        self.rpc_client
            .post_rpc_request("getCompressedTokenAccountsByOwner", request)
            .await
    }

    /// Retrieves aggregate compressed token balances for a wallet, grouped by mint.
    ///
    /// Returns a paginated list of token mints and their total balances across all
    /// compressed token accounts owned by the specified address. Useful for displaying
    /// a wallet's token portfolio without needing individual account details.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedTokenBalancesByOwnerRequest`] containing:
    ///   - `owner` — the base58-encoded public key of the wallet owner (required)
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `limit` — maximum number of token balances to return
    ///   - `mint` — optional mint address to filter results
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedTokenBalancesByOwnerResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`TokenBalanceList`](crate::types::TokenBalanceList) with token
    ///   balances grouped by mint and an optional cursor for the next page
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The owner address is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedTokenBalancesByOwnerRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedTokenBalancesByOwnerRequest {
    ///         owner: "11111113pNDtm61yGF8j2ycAwLEPsuWQXobye5qDR".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compressed_token_balances_by_owner(request).await.unwrap();
    ///     for token in &response.value.token_balances {
    ///         println!("Mint: {} — Balance: {}", token.mint, token.balance);
    ///     }
    /// }
    /// ```
    pub async fn get_compressed_token_balances_by_owner(
        &self,
        request: GetCompressedTokenBalancesByOwnerRequest,
    ) -> Result<GetCompressedTokenBalancesByOwnerResponse> {
        self.rpc_client
            .post_rpc_request("getCompressedTokenBalancesByOwner", request)
            .await
    }

    /// Retrieves aggregate compressed token balances for a wallet (V2).
    ///
    /// This is the V2 version of [`get_compressed_token_balances_by_owner`](Self::get_compressed_token_balances_by_owner).
    /// It returns the same data but uses the `items` field name in the response instead of
    /// `token_balances`, consistent with other paginated list types.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressedTokenBalancesByOwnerRequest`] containing:
    ///   - `owner` — the base58-encoded public key of the wallet owner (required)
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `limit` — maximum number of token balances to return
    ///   - `mint` — optional mint address to filter results
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressedTokenBalancesByOwnerV2Response`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`TokenBalanceListV2`](crate::types::TokenBalanceListV2) with token
    ///   balances grouped by mint and an optional cursor for the next page
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The owner address is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressedTokenBalancesByOwnerRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressedTokenBalancesByOwnerRequest {
    ///         owner: "11111114DhpssPJgSi1YU7hCMfYt1BJ334YgsffXm".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compressed_token_balances_by_owner_v2(request).await.unwrap();
    ///     for token in &response.value.items {
    ///         println!("Mint: {} — Balance: {}", token.mint, token.balance);
    ///     }
    /// }
    /// ```
    pub async fn get_compressed_token_balances_by_owner_v2(
        &self,
        request: GetCompressedTokenBalancesByOwnerRequest,
    ) -> Result<GetCompressedTokenBalancesByOwnerV2Response> {
        self.rpc_client
            .post_rpc_request("getCompressedTokenBalancesByOwnerV2", request)
            .await
    }

    /// Retrieves transaction signatures for a compressed account.
    ///
    /// Returns all transaction signatures that have affected the specified compressed
    /// account, identified by its data hash. Each signature includes the slot and
    /// block timestamp.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressionSignaturesForAccountRequest`] containing the
    ///   base58-encoded hash of the compressed account
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressionSignaturesForAccountResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`SignatureInfoList`](crate::types::SignatureInfoList) with transaction
    ///   signatures, slots, and block timestamps
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The hash is invalid or missing (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressionSignaturesForAccountRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressionSignaturesForAccountRequest {
    ///         hash: "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
    ///     };
    ///     let response = helius.get_compression_signatures_for_account(request).await.unwrap();
    ///     for sig in &response.value.items {
    ///         println!(
    ///             "Signature: {} — Slot: {} — Time: {}",
    ///             sig.signature, sig.slot, sig.block_time
    ///         );
    ///     }
    /// }
    /// ```
    pub async fn get_compression_signatures_for_account(
        &self,
        request: GetCompressionSignaturesForAccountRequest,
    ) -> Result<GetCompressionSignaturesForAccountResponse> {
        self.rpc_client
            .post_rpc_request("getCompressionSignaturesForAccount", request)
            .await
    }

    /// Retrieves transaction signatures for a compressed address.
    ///
    /// Returns a paginated list of transaction signatures that have affected
    /// compressed accounts at the specified address. Each signature includes the
    /// slot and block timestamp.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressionSignaturesForAddressRequest`] containing:
    ///   - `address` — the base58-encoded public key of the compressed address (required)
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `limit` — maximum number of signatures to return
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressionSignaturesForAddressResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`PaginatedSignatureInfoList`](crate::types::PaginatedSignatureInfoList)
    ///   with transaction signatures and an optional cursor for the next page
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The address is invalid or missing (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressionSignaturesForAddressRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressionSignaturesForAddressRequest {
    ///         address: "11111119T6fgHG3unjQB6vpWozhBdiXDbQovvFVeF".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compression_signatures_for_address(request).await.unwrap();
    ///     for sig in &response.value.items {
    ///         println!(
    ///             "Signature: {} — Slot: {} — Time: {}",
    ///             sig.signature, sig.slot, sig.block_time
    ///         );
    ///     }
    ///     if let Some(cursor) = &response.value.cursor {
    ///         println!("Next page cursor: {}", cursor);
    ///     }
    /// }
    /// ```
    pub async fn get_compression_signatures_for_address(
        &self,
        request: GetCompressionSignaturesForAddressRequest,
    ) -> Result<GetCompressionSignaturesForAddressResponse> {
        self.rpc_client
            .post_rpc_request("getCompressionSignaturesForAddress", request)
            .await
    }

    /// Retrieves transaction signatures for all compressed accounts owned by a given address.
    ///
    /// Returns a paginated list of transaction signatures that have affected any compressed
    /// account belonging to the specified owner. Each signature includes the slot and
    /// block timestamp. This is useful for building a full transaction history for a wallet's
    /// compressed account activity.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressionSignaturesForOwnerRequest`] containing:
    ///   - `owner` — the base58-encoded public key of the account owner (required)
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `limit` — maximum number of signatures to return
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressionSignaturesForOwnerResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`PaginatedSignatureInfoList`](crate::types::PaginatedSignatureInfoList)
    ///   with transaction signatures and an optional cursor for the next page
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The owner address is invalid or missing (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressionSignaturesForOwnerRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressionSignaturesForOwnerRequest {
    ///         owner: "11111119T6fgHG3unjQB6vpWozhBdiXDbQovvFVeF".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compression_signatures_for_owner(request).await.unwrap();
    ///     for sig in &response.value.items {
    ///         println!(
    ///             "Signature: {} — Slot: {} — Time: {}",
    ///             sig.signature, sig.slot, sig.block_time
    ///         );
    ///     }
    ///     if let Some(cursor) = &response.value.cursor {
    ///         println!("Next page cursor: {}", cursor);
    ///     }
    /// }
    /// ```
    pub async fn get_compression_signatures_for_owner(
        &self,
        request: GetCompressionSignaturesForOwnerRequest,
    ) -> Result<GetCompressionSignaturesForOwnerResponse> {
        self.rpc_client
            .post_rpc_request("getCompressionSignaturesForOwner", request)
            .await
    }

    /// Retrieves transaction signatures for compressed token accounts owned by a given address.
    ///
    /// Returns a paginated list of transaction signatures that have affected compressed
    /// **token** accounts belonging to the specified owner. Unlike
    /// [`get_compression_signatures_for_owner`](Self::get_compression_signatures_for_owner),
    /// which covers all compressed accounts, this method is scoped specifically to SPL
    /// token account activity.
    ///
    /// # Arguments
    /// * `request` - A [`GetCompressionSignaturesForOwnerRequest`] containing:
    ///   - `owner` — the base58-encoded public key of the token account owner (required)
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `limit` — maximum number of signatures to return
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetCompressionSignaturesForTokenOwnerResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`PaginatedSignatureInfoList`](crate::types::PaginatedSignatureInfoList)
    ///   with transaction signatures and an optional cursor for the next page
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The owner address is invalid or missing (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetCompressionSignaturesForOwnerRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetCompressionSignaturesForOwnerRequest {
    ///         owner: "11111119T6fgHG3unjQB6vpWozhBdiXDbQovvFVeF".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_compression_signatures_for_token_owner(request).await.unwrap();
    ///     for sig in &response.value.items {
    ///         println!(
    ///             "Signature: {} — Slot: {} — Time: {}",
    ///             sig.signature, sig.slot, sig.block_time
    ///         );
    ///     }
    ///     if let Some(cursor) = &response.value.cursor {
    ///         println!("Next page cursor: {}", cursor);
    ///     }
    /// }
    /// ```
    pub async fn get_compression_signatures_for_token_owner(
        &self,
        request: GetCompressionSignaturesForOwnerRequest,
    ) -> Result<GetCompressionSignaturesForTokenOwnerResponse> {
        self.rpc_client
            .post_rpc_request("getCompressionSignaturesForTokenOwner", request)
            .await
    }

    /// Checks the health status of the ZK Compression indexer.
    ///
    /// Returns `"ok"` when the compression indexer is healthy and synchronized with
    /// the Solana network. Use this for infrastructure monitoring, reliability dashboards,
    /// and pre-flight checks before performing operations on compressed accounts or tokens.
    ///
    /// # Returns
    /// A `Result` wrapping a `String` — `"ok"` when the indexer is healthy
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The indexer is experiencing issues (`InternalError`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let status = helius.get_indexer_health().await.unwrap();
    ///     println!("Indexer status: {}", status); // "ok"
    /// }
    /// ```
    pub async fn get_indexer_health(&self) -> Result<String> {
        self.rpc_client.post_rpc_request("getIndexerHealth", ()).await
    }

    /// Retrieves the current slot that the ZK Compression indexer has processed up to.
    ///
    /// Returns the latest Solana slot number that the compression indexer has indexed.
    /// Compare this against the network's current slot to gauge how far behind the
    /// indexer is. Useful for monitoring indexer lag and ensuring data freshness before
    /// querying compressed account state.
    ///
    /// # Returns
    /// A `Result` wrapping a `u64` — the latest indexed Solana slot number
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let slot = helius.get_indexer_slot().await.unwrap();
    ///     println!("Indexer is at slot: {}", slot);
    /// }
    /// ```
    pub async fn get_indexer_slot(&self) -> Result<u64> {
        self.rpc_client.post_rpc_request("getIndexerSlot", ()).await
    }

    /// Retrieves the most recent compression transaction signatures.
    ///
    /// Returns a paginated list of the latest transaction signatures involving
    /// compressed accounts and tokens across the Solana network. Useful for
    /// monitoring real-time compression activity, market analysis, and auditing
    /// compression-related transactions.
    ///
    /// # Arguments
    /// * `request` - A [`GetLatestCompressionSignaturesRequest`] containing:
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `limit` — maximum number of signatures to return
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetLatestCompressionSignaturesResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`PaginatedSignatureInfoList`](crate::types::PaginatedSignatureInfoList)
    ///   with transaction signatures and an optional cursor for the next page
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetLatestCompressionSignaturesRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetLatestCompressionSignaturesRequest {
    ///         limit: Some(10),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_latest_compression_signatures(request).await.unwrap();
    ///     for sig in &response.value.items {
    ///         println!(
    ///             "Signature: {} — Slot: {} — Time: {}",
    ///             sig.signature, sig.slot, sig.block_time
    ///         );
    ///     }
    ///     if let Some(cursor) = &response.value.cursor {
    ///         println!("Next page cursor: {}", cursor);
    ///     }
    /// }
    /// ```
    pub async fn get_latest_compression_signatures(
        &self,
        request: GetLatestCompressionSignaturesRequest,
    ) -> Result<GetLatestCompressionSignaturesResponse> {
        self.rpc_client
            .post_rpc_request("getLatestCompressionSignatures", request)
            .await
    }

    /// Retrieves the most recent non-voting transaction signatures.
    ///
    /// Returns a list of the latest transaction signatures that are not validator
    /// vote transactions. Each signature includes the slot, block timestamp, and
    /// an optional error string for failed transactions. Useful for monitoring
    /// real user and program activity on the network, excluding vote noise.
    ///
    /// # Arguments
    /// * `request` - A [`GetLatestCompressionSignaturesRequest`] containing:
    ///   - `cursor` — pagination cursor from a previous response
    ///   - `limit` — maximum number of signatures to return
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetLatestNonVotingSignaturesResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`SignatureInfoListWithError`](crate::types::SignatureInfoListWithError)
    ///   with transaction signatures, each potentially including an error message
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetLatestCompressionSignaturesRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetLatestCompressionSignaturesRequest {
    ///         limit: Some(10),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_latest_non_voting_signatures(request).await.unwrap();
    ///     for sig in &response.value.items {
    ///         println!(
    ///             "Signature: {} — Slot: {} — Error: {:?}",
    ///             sig.signature, sig.slot, sig.error
    ///         );
    ///     }
    /// }
    /// ```
    pub async fn get_latest_non_voting_signatures(
        &self,
        request: GetLatestCompressionSignaturesRequest,
    ) -> Result<GetLatestNonVotingSignaturesResponse> {
        self.rpc_client
            .post_rpc_request("getLatestNonVotingSignatures", request)
            .await
    }

    /// Retrieves Merkle proofs for multiple compressed accounts in a single request.
    ///
    /// Batch version of [`get_compressed_account_proof`](Self::get_compressed_account_proof).
    /// Returns a proof for each requested hash, in the same order. Each proof contains
    /// the full path from the leaf to the root of the Merkle tree, enabling cryptographic
    /// verification of account existence and data integrity.
    ///
    /// # Arguments
    /// * `hashes` - A vector of base58-encoded compressed account hashes to retrieve
    ///   proofs for
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetMultipleCompressedAccountProofsResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a vector of [`MerkleProofWithContext`](crate::types::MerkleProofWithContext),
    ///   one per requested hash, in the same order
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - Any hash is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - Any requested account is not found (`NotFound`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let hashes = vec![
    ///         "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
    ///         "11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3".to_string(),
    ///     ];
    ///     let response = helius.get_multiple_compressed_account_proofs(hashes).await.unwrap();
    ///     for proof in &response.value {
    ///         println!(
    ///             "Hash: {} — Root: {} — Proof length: {}",
    ///             proof.hash, proof.root, proof.proof.len()
    ///         );
    ///     }
    /// }
    /// ```
    pub async fn get_multiple_compressed_account_proofs(
        &self,
        hashes: Vec<String>,
    ) -> Result<GetMultipleCompressedAccountProofsResponse> {
        self.rpc_client
            .post_rpc_request("getMultipleCompressedAccountProofs", hashes)
            .await
    }

    /// Retrieves multiple compressed accounts by their addresses or hashes in a single request.
    ///
    /// Batch version of [`get_compressed_account`](Self::get_compressed_account). Returns
    /// a list of compressed accounts corresponding to the requested identifiers. Items are
    /// positional: each entry in the response corresponds to the address or hash at the same
    /// index in the request. Entries are `None` when no account matches a given identifier.
    ///
    /// # Arguments
    /// * `request` - A [`GetMultipleCompressedAccountsRequest`] containing:
    ///   - `addresses` — optional vector of base58-encoded public keys
    ///   - `hashes` — optional vector of base58-encoded data hashes
    ///
    ///   At least one of `addresses` or `hashes` must be provided.
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetMultipleCompressedAccountsResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — an [`AccountList`](crate::types::AccountList) where each item is either
    ///   a [`CompressedAccount`](crate::types::CompressedAccount) or `None`
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - Neither `addresses` nor `hashes` is provided (`BadRequest`)
    /// - Any address or hash is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetMultipleCompressedAccountsRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetMultipleCompressedAccountsRequest {
    ///         addresses: Some(vec![
    ///             "11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3".to_string(),
    ///             "11111114d3RrygbPdAtMuFnDmzsN8T5fYKVQ7FVr7".to_string(),
    ///         ]),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_multiple_compressed_accounts(request).await.unwrap();
    ///     for (i, account) in response.value.items.iter().enumerate() {
    ///         match account {
    ///             Some(acc) => println!("[{}] Owner: {} — Lamports: {}", i, acc.owner, acc.lamports),
    ///             None => println!("[{}] No matching account", i),
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn get_multiple_compressed_accounts(
        &self,
        request: GetMultipleCompressedAccountsRequest,
    ) -> Result<GetMultipleCompressedAccountsResponse> {
        self.rpc_client
            .post_rpc_request("getMultipleCompressedAccounts", request)
            .await
    }

    /// Retrieves new-address proofs for multiple addresses in a single request.
    ///
    /// Returns a proof for each requested address that demonstrates the address does not
    /// yet exist in the address Merkle tree. These proofs are required when creating new
    /// compressed accounts at specific addresses, ensuring no duplicate addresses are
    /// created.
    ///
    /// Each proof includes the lower and higher range addresses that bound the new address
    /// in the tree's sorted order, along with the full Merkle proof path.
    ///
    /// # Arguments
    /// * `addresses` - A vector of base58-encoded Solana public keys to retrieve
    ///   new-address proofs for
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetMultipleNewAddressProofsResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a vector of [`MerkleContextWithNewAddressProof`](crate::types::MerkleContextWithNewAddressProof),
    ///   one per requested address, in the same order
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - Any address is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let addresses = vec![
    ///         "11111117qkFjr4u54stuNNUR8fRF8dNhaP35yvANs".to_string(),
    ///         "11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3".to_string(),
    ///     ];
    ///     let response = helius.get_multiple_new_address_proofs(addresses).await.unwrap();
    ///     for proof in &response.value {
    ///         println!(
    ///             "Address: {} — Root: {} — Tree: {}",
    ///             proof.address, proof.root, proof.merkle_tree
    ///         );
    ///     }
    /// }
    /// ```
    pub async fn get_multiple_new_address_proofs(
        &self,
        addresses: Vec<String>,
    ) -> Result<GetMultipleNewAddressProofsResponse> {
        self.rpc_client
            .post_rpc_request("getMultipleNewAddressProofs", addresses)
            .await
    }

    /// Retrieves new-address proofs for multiple address-tree pairs (V2).
    ///
    /// This is the V2 version of [`get_multiple_new_address_proofs`](Self::get_multiple_new_address_proofs).
    /// Instead of plain addresses, it accepts [`AddressWithTree`] pairs that specify both the
    /// address to prove and the specific address Merkle tree to check against. This is useful
    /// when the caller knows which tree the address should belong to.
    ///
    /// Each proof demonstrates that the address does not yet exist in the specified tree,
    /// enabling safe creation of a new compressed account at that address.
    ///
    /// # Arguments
    /// * `addresses` - A vector of [`AddressWithTree`] pairs, each containing a base58-encoded
    ///   address and the base58-encoded address Merkle tree to check against
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetMultipleNewAddressProofsV2Response`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a vector of [`MerkleContextWithNewAddressProof`](crate::types::MerkleContextWithNewAddressProof),
    ///   one per requested pair, in the same order
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - Any address or tree is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, AddressWithTree};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let addresses = vec![
    ///         AddressWithTree {
    ///             address: "11111117qkFjr4u54stuNNUR8fRF8dNhaP35yvANs".to_string(),
    ///             tree: "11111118F5rixNBnFLmioWZSYzjjFuAL5dyoDVzhD".to_string(),
    ///         },
    ///     ];
    ///     let response = helius.get_multiple_new_address_proofs_v2(addresses).await.unwrap();
    ///     for proof in &response.value {
    ///         println!(
    ///             "Address: {} — Root: {} — Tree: {}",
    ///             proof.address, proof.root, proof.merkle_tree
    ///         );
    ///     }
    /// }
    /// ```
    pub async fn get_multiple_new_address_proofs_v2(
        &self,
        addresses: Vec<AddressWithTree>,
    ) -> Result<GetMultipleNewAddressProofsV2Response> {
        self.rpc_client
            .post_rpc_request("getMultipleNewAddressProofsV2", addresses)
            .await
    }

    /// Retrieves a Solana transaction along with its compression-specific information.
    ///
    /// Returns the full encoded transaction and metadata about which compressed accounts
    /// were opened (created) and closed (destroyed) by the transaction. This is useful
    /// for understanding the compression-level effects of a transaction, such as which
    /// compressed accounts were created or consumed.
    ///
    /// # Arguments
    /// * `request` - A [`GetTransactionWithCompressionInfoRequest`] containing the
    ///   base58-encoded transaction signature
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetTransactionWithCompressionInfoResponse`] that contains:
    /// - `compression_info` — a [`CompressionInfo`](crate::types::CompressionInfo) with lists
    ///   of opened and closed compressed accounts, each paired with optional token data
    /// - `transaction` — the full encoded Solana transaction with status metadata as a
    ///   JSON value
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The signature is invalid or missing (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The transaction is not found (`NotFound`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetTransactionWithCompressionInfoRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetTransactionWithCompressionInfoRequest {
    ///         signature: "5J8H5sTvEhnGcB4R8K1n7mfoiWUD9RzPVGES7e3WxC7c".to_string(),
    ///     };
    ///     let response = helius.get_transaction_with_compression_info(request).await.unwrap();
    ///     if let Some(info) = &response.compression_info {
    ///         println!("Opened accounts: {}", info.opened_accounts.len());
    ///         println!("Closed accounts: {}", info.closed_accounts.len());
    ///     }
    /// }
    /// ```
    pub async fn get_transaction_with_compression_info(
        &self,
        request: GetTransactionWithCompressionInfoRequest,
    ) -> Result<GetTransactionWithCompressionInfoResponse> {
        self.rpc_client
            .post_rpc_request("getTransactionWithCompressionInfo", request)
            .await
    }

    /// Retrieves a zero-knowledge validity proof for compressed accounts and/or new addresses.
    ///
    /// Returns the cryptographic proof components needed to verify compressed account state
    /// or prove non-existence of new addresses on-chain. The proof includes three elliptic
    /// curve elements (`a`, `b`, `c`) along with the Merkle tree context (roots, leaf indices,
    /// leaves, and tree addresses) required for on-chain verification.
    ///
    /// This is essential for building transactions that interact with compressed accounts,
    /// as the validity proof must be included in the transaction to satisfy the verifier
    /// program.
    ///
    /// # Arguments
    /// * `request` - A [`GetValidityProofRequest`] containing:
    ///   - `hashes` — optional vector of base58-encoded compressed account hashes to verify
    ///   - `new_addresses_with_trees` — optional vector of [`AddressWithTree`] pairs for
    ///     new addresses to prove non-existence of
    ///
    /// # Returns
    /// A `Result` wrapping a [`GetValidityProofResponse`] that contains:
    /// - `context` — the Solana slot at which the data was fetched
    /// - `value` — a [`CompressedProofWithContext`](crate::types::CompressedProofWithContext)
    ///   with the proof and Merkle tree metadata
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - Any hash or address is invalid (`BadRequest`)
    /// - The API key is missing or invalid (`Unauthorized`)
    /// - The API request fails due to network or server issues
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::{Cluster, GetValidityProofRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let request = GetValidityProofRequest {
    ///         hashes: Some(vec![
    ///             "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP".to_string(),
    ///         ]),
    ///         ..Default::default()
    ///     };
    ///     let response = helius.get_validity_proof(request).await.unwrap();
    ///     let proof = &response.value;
    ///     println!("Proof a: {}", proof.compressed_proof.a);
    ///     println!("Roots: {:?}", proof.roots);
    ///     println!("Merkle trees: {:?}", proof.merkle_trees);
    /// }
    /// ```
    pub async fn get_validity_proof(&self, request: GetValidityProofRequest) -> Result<GetValidityProofResponse> {
        self.rpc_client.post_rpc_request("getValidityProof", request).await
    }
}
