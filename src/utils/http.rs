use crate::prelude::*;
use reqwest::blocking::Client;
use std::time::Duration;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new(timeout_secs: u64) -> Fallible<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(crate::USER_AGENT)
            .build()?;
        Ok(HttpClient { client })
    }

    pub fn get(&self, url: &str) -> Fallible<String> {
        let response = self.client.get(url).send()?;
        let status = response.status();
        let body = response.text()?;

        if !status.is_success() {
            anyhow::bail!("HTTP GET failed: {} - {}", status, body);
        }

        Ok(body)
    }

    pub fn post_json(&self, url: &str, json: &serde_json::Value) -> Fallible<String> {
        let response = self.client.post(url).json(json).send()?;
        let status = response.status();
        let body = response.text()?;

        if !status.is_success() {
            anyhow::bail!("HTTP POST failed: {} - {}", status, body);
        }

        Ok(body)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new(30).expect("Failed to create default HTTP client")
    }
}
