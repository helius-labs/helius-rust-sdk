use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;
use crate::request_handler::RequestHandler;
use crate::types::types::ApiResponse;
use crate::types::{AssetsByOwnerRequest, GetAssetRequest, AssetsByAuthorityRequest};

use reqwest::{Client, Method, Url};
use serde_json::{json, Value};

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

    /// Gets an asset by its ID
    pub async fn get_asset(&self, request: GetAssetRequest) -> Result<ApiResponse> {
        let base_url: String = format!("{}?api-key={}", self.config.endpoints.rpc, self.config.api_key);
        let url: Url = Url::parse(&base_url).expect("Failed to parse URL");

        let request_body: Value = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAsset",
            "params": request
        });

        self.handler.send(Method::POST, url, Some(&request_body)).await
    }

    /// Gets a list of assets owned by a given address
    pub async fn get_assets_by_owner(&self, request: AssetsByOwnerRequest) -> Result<ApiResponse> {
        let base_url: String = format!("{}?api-key={}", self.config.endpoints.rpc, self.config.api_key);
        let url: Url = Url::parse(&base_url).expect("Failed to parse URL");

        let request_body: Value = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAssetsByOwner",
            "params": request
        });

        self.handler.send(Method::POST, url, Some(&request_body)).await
    }

    /// Gets a list of assets of a given authority
    pub async fn get_assets_by_authority(&self, request: AssetsByAuthorityRequest) -> Result<ApiResponse> {
        let url: String = format!("{}?api-key={}", self.config.endpoints.rpc, self.config.api_key);
        let url: Url = Url::parse(&url).expect("Failed to parse URL");

        let request_body: Value = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAssetsByAuthority",
            "params": request
        });

        self.handler.send(Method::POST, url, Some(&request_body)).await
    }
}
