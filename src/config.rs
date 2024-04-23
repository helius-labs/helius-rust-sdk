use crate::error::Result;
use crate::types::{Cluster, HeliusEndpoints};

#[derive(Clone)]
pub struct Config {
    pub api_key: String,
    pub cluster: Cluster,
    pub endpoints: HeliusEndpoints,
}

impl Config {
    pub fn new(api_key: &str, cluster: Cluster) -> Result<Self> {
        let endpoints: HeliusEndpoints = HeliusEndpoints::for_cluster(&cluster);
        Ok(Config {
            api_key: api_key.to_string(),
            cluster,
            endpoints,
        })
    }
}
