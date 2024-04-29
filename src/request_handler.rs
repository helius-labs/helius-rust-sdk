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
    /// Initializes a new RequestHandler instance
    pub fn new(client: Arc<Client>) -> Result<Self> {
        Ok(Self { http_client: client })
    }

    /// Sends a given request via the RequestBuilder and returns a Response
    async fn send_request(&self, request_builder: RequestBuilder) -> Result<Response> {
        request_builder.send().await.map_err(HeliusError::Network)
    }

    /// Handles the send functionality for a given HTTP request
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

        print!("RESPONSE {:?}", response);

        self.handle_response(response).await
    }

    /// Handles the Response for a given HTTP request
    async fn handle_response<T: for<'de> Deserialize<'de>>(&self, response: Response) -> Result<T> {
        let status: StatusCode = response.status();
        let path: String = response.url().path().to_string();
        let body_text: String = response.text().await.unwrap_or_default();

        println!("STATUS {}", status);
        println!("PATH {}", path);
        println!("BODY {}", body_text);

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
            let body_json: serde_json::Result<serde_json::Value> = serde_json::from_str(&body_text);
            match body_json {
                Ok(body) => {
                    let error_message: String = body["message"].as_str().unwrap_or("Unknown error").to_string();
                    Err(HeliusError::from_response_status(status, path, error_message))
                }
                Err(_) => Err(HeliusError::from_response_status(status, path, body_text)),
            }
        }
    }
}
