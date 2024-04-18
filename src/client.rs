#![allow(dead_code)]

use crate::config::Config;
use crate::error::Result;
use crate::types::Cluster;

use reqwest::Client;

pub struct Helius {
    config: Config,
    client: Client,
}

impl Helius {
    pub fn new(api_key: &str, cluster: Cluster) -> Result<Self> {
        let config: Config = Config::new(api_key, cluster)?;
        let client: Client = Client::new();

        Ok(Helius {config, client})
    }
}