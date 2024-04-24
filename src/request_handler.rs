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
        request_builder.send().await.map_err(HeliusError::Network)
    }

    pub async fn send<R, T>(&self, method: Method, url: Url, body: Option<&R>) -> Result<T>
    where
        R: Serialize + ?Sized + Send + Sync + Debug,
        T: for<'de> Deserialize<'de> + Default,
    {
        let mut request_builder: RequestBuilder = self.http_client.request(method, url);

        if let Some(body) = body {
            request_builder = request_builder.json(body);
        }

        let response: Response = self.send_request(request_builder).await?;
        self.handle_response(response).await
    }
    
    async fn handle_response<T: for<'de> Deserialize<'de>>(&self, response: Response) -> Result<T> {
        let status: StatusCode = response.status();
        let path: String = response.url().path().to_string();
        let body_text: String = response.text().await.unwrap_or_default();
    
        println!("Response status: {}, Body: {}", status, body_text);
    
        if status.is_success() {
            match serde_json::from_str::<T>(&body_text) {
                Ok(data) => Ok(data),
                Err(e) => {
                    println!("Deserialization error: {}", e);
                    println!("Raw JSON: {}", body_text);
                    Err(HeliusError::from(e))
                }
            }
        } else {
            Err(HeliusError::from_response_status(status, path.clone(), body_text))
        }
    }
}
