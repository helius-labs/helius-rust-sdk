use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;
use crate::request_handler::RequestHandler;

use reqwest::Client;

pub struct RpcClient {
    pub handler: RequestHandler,
    pub config: Arc<Config>,
}

impl RpcClient {
    pub fn new(client: Arc<Client>, config: Arc<Config>) -> Result<Self> {
        let handler: RequestHandler = RequestHandler::new(client)?;
        Ok(RpcClient { handler, config })
    }
}
