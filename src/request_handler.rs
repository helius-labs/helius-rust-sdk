use crate::error::{HeliusError, Result};
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode, Url};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Clone)]
pub struct RequestHandler {
    pub http_client: Arc<Client>,
}

impl RequestHandler {
    pub fn new(client: Arc<Client>) -> Result<Self> {
        Ok(Self { http_client: client })
    }

    async fn send_request(&self, request_builder: RequestBuilder) -> Result<Response> {
        let response: Response = request_builder.send().await.map_err(|e| HeliusError::Network(e))?;
        Ok(response)
    }

    pub async fn send<R, T>(&self, method: Method, url: Url, body: Option<&R>) -> Result<T>
    where
        R: Serialize + ?Sized + Send + Sync + Debug,
        T: for<'de> Deserialize<'de> + Default,
    {
        let mut request_builder: RequestBuilder = self.http_client.request(method, url.clone());

        if let Some(body) = body {
            request_builder = request_builder.json(body);
        }

        let response: Response = self.send_request(request_builder).await?;
        let path: String = url.path().to_string();
        self.handle_response(path, response).await
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(&self, path: String, response: Response) -> Result<T> {
        let status: StatusCode = response.status();

        match status {
            StatusCode::OK | StatusCode::CREATED => response.json::<T>().await.map_err(HeliusError::SerdeJson),
            _ => Err(HeliusError::from_response_status(status, path, response).await),
        }
    }
}
