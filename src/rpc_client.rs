use std::fmt::Debug;
use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;
use crate::request_handler::RequestHandler;
use crate::types::types::{RpcRequest, RpcResponse};
use crate::types::{
    Asset, AssetList, GetAsset, GetAssetBatch, GetAssetsByAuthority, GetAssetsByCreator, GetAssetsByOwner,
};

use reqwest::{Client, Method, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct RpcClient {
    pub handler: RequestHandler,
    pub config: Arc<Config>,
}

impl RpcClient {
    /// Initializes a new RpcClient instance
    pub fn new(client: Arc<Client>, config: Arc<Config>) -> Result<Self> {
        let handler: RequestHandler = RequestHandler::new(client)?;
        Ok(RpcClient { handler, config })
    }

    /// Streamlines an RPC POST request
    pub async fn post_rpc_request<R, T>(&self, method: &str, request: R) -> Result<T>
    where
        R: Debug + Serialize + Debug + Send + Sync,
        T: Debug + DeserializeOwned + Default,
    {
        let base_url: String = format!("{}/?api-key={}", self.config.endpoints.rpc, self.config.api_key);
        let url: Url = Url::parse(&base_url).expect("Failed to parse URL");

        println!("{}", base_url);
        println!("{}", url);

        let rpc_request: RpcRequest<R> = RpcRequest::new(method.to_string(), request);
        println!("Serialized Request: {:?}", serde_json::to_string(&rpc_request));

        let rpc_response: RpcResponse<T> = self.handler.send(Method::POST, url, Some(&rpc_request)).await?;
        println!("RPCRESPONSE {:?}", rpc_response.result);
        Ok(rpc_response.result)
    }

    /// Gets an asset by its ID
    pub async fn get_asset(&self, request: GetAsset) -> Result<Option<Asset>> {
        self.post_rpc_request("getAsset", request).await
    }

    /// Gets multiple assets by their ID
    pub async fn get_asset_batch(&self, request: GetAssetBatch) -> Result<Vec<Option<Asset>>> {
        self.post_rpc_request("getAssetBatch", request).await
    }

    /// Gets a list of assets owned by a given address
    pub async fn get_assets_by_owner(&self, request: GetAssetsByOwner) -> Result<AssetList> {
        self.post_rpc_request("getAssetsByOwner", request).await
    }

    /// Gets a list of assets of a given authority
    pub async fn get_assets_by_authority(&self, request: GetAssetsByAuthority) -> Result<AssetList> {
        self.post_rpc_request("getAssetsByAuthority", request).await
    }

    /// Gets a list of assets of a given creator
    pub async fn get_assets_by_creator(&self, request: GetAssetsByCreator) -> Result<AssetList> {
        self.post_rpc_request("getAssetsByCreator", request).await
    }
}
