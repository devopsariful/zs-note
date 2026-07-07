use std::time::Duration;

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::sandbox::capability::glob_match;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

pub struct SandboxNetworkProxy {
    client: Client,
    allowed_domains: Vec<String>,
}

impl SandboxNetworkProxy {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("ZarishNote-Sandbox/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            allowed_domains,
        }
    }

    pub fn check_network_allowed(&self, url: &str) -> Result<()> {
        if self.allowed_domains.is_empty() {
            bail!("Network access is disabled in sandbox configuration");
        }

        let parsed = url::Url::parse(url).context("Invalid URL")?;
        let host = parsed.host_str().unwrap_or("");

        let allowed = self
            .allowed_domains
            .iter()
            .any(|pattern| glob_match(pattern, host));

        if !allowed {
            bail!(
                "Network access to '{}' is not allowed by sandbox policy",
                host
            );
        }

        Ok(())
    }

    pub async fn execute_request(&self, request: NetworkRequest) -> Result<NetworkResponse> {
        self.check_network_allowed(&request.url)?;

        let parsed = url::Url::parse(&request.url)?;
        let host = parsed.host_str().unwrap_or("");

        let allowed = self
            .allowed_domains
            .iter()
            .any(|pattern| glob_match(pattern, host));

        if !allowed {
            bail!("Domain '{}' is not in the allow list", host);
        }

        let mut req_builder = match request.method.to_uppercase().as_str() {
            "GET" => self.client.get(&request.url),
            "POST" => {
                let mut r = self.client.post(&request.url);
                if let Some(body) = &request.body {
                    r = r.body(body.clone());
                }
                r
            }
            "PUT" => {
                let mut r = self.client.put(&request.url);
                if let Some(body) = &request.body {
                    r = r.body(body.clone());
                }
                r
            }
            "DELETE" => self.client.delete(&request.url),
            "PATCH" => {
                let mut r = self.client.patch(&request.url);
                if let Some(body) = &request.body {
                    r = r.body(body.clone());
                }
                r
            }
            "HEAD" => self.client.head(&request.url),
            _ => bail!("Unsupported HTTP method: {}", request.method),
        };

        for (key, value) in &request.headers {
            req_builder = req_builder.header(key, value);
        }

        let response = req_builder.send().await?;
        let status = response.status().as_u16();
        let response_headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.text().await?;

        Ok(NetworkResponse {
            status,
            headers: response_headers,
            body,
        })
    }

    pub fn allowed_domains(&self) -> &[String] {
        &self.allowed_domains
    }
}
