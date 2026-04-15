use crate::error::Result;
use crate::types::ProjectUsage;
use crate::Helius;

use reqwest::{Method, Url};

const ADMIN_API_BASE_URL: &str = "https://admin-api.helius.xyz/";

impl Helius {
    /// Get the appropriate base URL for Admin API calls.
    ///
    /// Uses the config's API endpoint for testing/custom URLs, otherwise uses the
    /// production Admin API endpoint.
    fn get_admin_api_base_url(&self) -> String {
        let api_url = &self.config.endpoints.api;
        if self.config.custom_url.is_some()
            || api_url.starts_with("http://localhost")
            || api_url.starts_with("http://127.0.0.1")
        {
            let mut local_url = self.config.endpoints.api.clone();
            if !local_url.ends_with('/') {
                local_url.push('/');
            }
            local_url
        } else {
            ADMIN_API_BASE_URL.to_string()
        }
    }

    /// Retrieves current billing-period usage for a project tied to the API key.
    ///
    /// Returns the included credit balance, prepaid credit balance, subscription details,
    /// and a per-product usage breakdown for the given project.
    ///
    /// # Arguments
    /// * `project_id` - The Helius project ID to inspect
    ///
    /// # Returns
    /// A `Result` wrapping a `ProjectUsage` summary for the project
    ///
    /// # Errors
    /// Returns a `HeliusError` if:
    /// - The API key is missing
    /// - The project ID is invalid
    /// - The API key does not have access to the project
    /// - The API request fails
    ///
    /// # Example
    /// ```ignore
    /// use helius::Helius;
    /// use helius::types::Cluster;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let helius = Helius::new("your_api_key", Cluster::MainnetBeta).unwrap();
    ///     let usage = helius.get_project_usage("your_project_id").await.unwrap();
    ///     println!("Credits remaining: {}", usage.credits_remaining);
    /// }
    /// ```
    pub async fn get_project_usage(&self, project_id: &str) -> Result<ProjectUsage> {
        let api_key = self.config.require_api_key("admin project usage")?;
        let base_url = self.get_admin_api_base_url();
        let url: String = format!(
            "{}v0/admin/projects/{}/usage?api-key={}",
            base_url,
            project_id,
            api_key.as_str()
        );
        let parsed_url: Url = Url::parse(&url).expect("Failed to parse URL");

        self.rpc_client.handler.send(Method::GET, parsed_url, None::<&()>).await
    }
}
