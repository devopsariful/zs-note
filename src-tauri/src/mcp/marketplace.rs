use std::collections::HashMap;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::Config;

/// An entry from the MCP server marketplace registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub transport: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub downloads: u64,
    pub rating: Option<f64>,
}

/// Registry for fetching and installing MCP servers from a remote marketplace.
pub struct MarketplaceRegistry {
    registry_url: String,
    cache: HashMap<String, MarketplaceEntry>,
    http_client: reqwest::Client,
}

impl MarketplaceRegistry {
    /// Create a new registry pointing to the given URL.
    pub fn new(registry_url: &str) -> Self {
        Self {
            registry_url: registry_url.to_string(),
            cache: HashMap::new(),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent("ZarishNote/0.1")
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    /// Fetch the list of available servers from the registry API.
    /// Results are cached in memory.
    pub async fn fetch_servers(&mut self) -> Result<Vec<MarketplaceEntry>> {
        let resp = self
            .http_client
            .get(&self.registry_url)
            .send()
            .await
            .context("Failed to fetch marketplace listing")?;

        if !resp.status().is_success() {
            bail!("Marketplace returned HTTP {}", resp.status().as_u16());
        }

        let entries: Vec<MarketplaceEntry> = resp
            .json()
            .await
            .context("Failed to parse marketplace response")?;

        // Populate cache
        for entry in &entries {
            self.cache.insert(entry.id.clone(), entry.clone());
        }

        info!("Fetched {} servers from marketplace", entries.len());
        Ok(entries)
    }

    /// Return cached entries without making a network request.
    pub fn get_cached_servers(&self) -> Vec<&MarketplaceEntry> {
        self.cache.values().collect()
    }

    /// Look up a single entry from the cache by id.
    pub fn get_cached(&self, server_id: &str) -> Option<&MarketplaceEntry> {
        self.cache.get(server_id)
    }

    /// Download and install a server by its marketplace id.
    ///
    /// Fetches the server's configuration manifest from the registry,
    /// appends it to the MCP servers list in `config`, and saves.
    pub async fn install_server(
        &mut self,
        server_id: String,
        config: &mut Config,
    ) -> Result<MarketplaceEntry> {
        // Fetch fresh metadata if not cached
        let entry = match self.cache.get(&server_id) {
            Some(e) => e.clone(),
            None => {
                let manifest_url = format!(
                    "{}/{}/manifest.json",
                    self.registry_url.trim_end_matches('/'),
                    server_id
                );
                let resp = self
                    .http_client
                    .get(&manifest_url)
                    .send()
                    .await
                    .context("Failed to fetch server manifest")?;
                if !resp.status().is_success() {
                    bail!("Server '{}' not found in marketplace", server_id);
                }
                let entry: MarketplaceEntry = resp
                    .json()
                    .await
                    .context("Failed to parse server manifest")?;
                self.cache.insert(server_id.clone(), entry.clone());
                entry
            }
        };

        // Build the MCP config entry
        let server_config = serde_json::json!({
            "name": entry.name,
            "transport": entry.transport,
            "command": entry.command,
            "args": entry.args,
            "env": entry.env,
            "version": entry.version,
            "marketplaceId": entry.id,
        });

        // Append to the MCP servers list in plugins.settings
        let mcp_entry = config
            .plugins
            .settings
            .entry("mcp".to_string())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));

        if let Some(arr) = mcp_entry.as_array_mut() {
            // Avoid duplicate installs
            if !arr
                .iter()
                .any(|v| v.get("marketplaceId").and_then(|id| id.as_str()) == Some(&entry.id))
            {
                arr.push(server_config);
                info!("Installed MCP server '{}' from marketplace", entry.name);
            } else {
                info!("MCP server '{}' already installed, skipping", entry.name);
            }
        }

        Ok(entry)
    }

    /// Check which installed servers have newer versions in the marketplace.
    pub async fn check_for_updates(&self, config: &Config) -> Result<Vec<UpdateInfo>> {
        let mut updates = Vec::new();

        let mcp_servers = config
            .plugins
            .settings
            .get("mcp")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        for installed in &mcp_servers {
            let installed_version = installed
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0");
            let marketplace_id = installed
                .get("marketplaceId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let Some(ref id) = marketplace_id {
                // Try cache first, then fetch
                let latest = if let Some(cached) = self.cache.get(id) {
                    cached.clone()
                } else {
                    let manifest_url = format!(
                        "{}/{}/manifest.json",
                        self.registry_url.trim_end_matches('/'),
                        id
                    );
                    match self.http_client.get(&manifest_url).send().await {
                        Ok(resp) if resp.status().is_success() => {
                            if let Ok(entry) = resp.json::<MarketplaceEntry>().await {
                                entry
                            } else {
                                continue;
                            }
                        }
                        _ => continue,
                    }
                };

                if latest.version != installed_version {
                    updates.push(UpdateInfo {
                        server_id: id.clone(),
                        name: latest.name.clone(),
                        installed_version: installed_version.to_string(),
                        latest_version: latest.version.clone(),
                    });
                }
            }
        }

        Ok(updates)
    }

    /// Remove a marketplace-installed server from the config.
    pub fn uninstall_server(server_id: &str, config: &mut Config) -> Result<()> {
        if let Some(arr) = config
            .plugins
            .settings
            .get_mut("mcp")
            .and_then(|v| v.as_array_mut())
        {
            arr.retain(|entry| {
                entry.get("marketplaceId").and_then(|id| id.as_str()) != Some(server_id)
            });
            info!("Uninstalled MCP server '{}' from marketplace", server_id);
        }
        Ok(())
    }

    /// Get the version string of an installed marketplace server.
    pub fn get_installed_version(server_id: &str, config: &Config) -> Option<String> {
        let mcp_servers = config
            .plugins
            .settings
            .get("mcp")
            .and_then(|v| v.as_array())?;

        for entry in mcp_servers {
            if entry.get("marketplaceId").and_then(|id| id.as_str()) == Some(server_id) {
                return entry
                    .get("version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
        }
        None
    }
}

/// Describes an available update from the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub server_id: String,
    pub name: String,
    pub installed_version: String,
    pub latest_version: String,
}

impl Default for MarketplaceRegistry {
    fn default() -> Self {
        Self::new("https://marketplace.zarishsphere.com/api/mcp")
    }
}
