pub mod marketplace;
pub mod protocol;
pub mod router;
pub mod transport;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::info;

use crate::mcp::transport::{McpTransport, TransportType};
use crate::types::McpToolInfo;

pub struct McpServerInstance {
    pub name: String,
    pub transport: Box<dyn McpTransport + Send + Sync>,
    pub tools: Vec<McpToolInfo>,
    pub running: bool,
}

pub struct MCPClient {
    servers: Arc<RwLock<HashMap<String, McpServerInstance>>>,
}

impl MCPClient {
    pub fn new(_name: &str) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_server(
        &self,
        name: &str,
        transport_type: TransportType,
        config: Value,
    ) -> Result<()> {
        let transport: Box<dyn McpTransport + Send + Sync> = match transport_type {
            TransportType::Stdio => {
                let command = config["command"]
                    .as_str()
                    .context("MCP stdio config missing 'command'")?
                    .to_string();
                let args = config["args"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();
                Box::new(transport::StdioTransport::new(&command, args))
            }
            TransportType::Http => {
                let url = config["url"]
                    .as_str()
                    .context("MCP HTTP config missing 'url'")?
                    .to_string();
                Box::new(transport::HttpTransport::new(&url))
            }
        };

        let tools = transport.list_tools().await?;

        let instance = McpServerInstance {
            name: name.to_string(),
            transport,
            tools,
            running: true,
        };

        let mut servers = self.servers.write().await;
        servers.insert(name.to_string(), instance);

        info!("Registered MCP server: {}", name);
        Ok(())
    }

    pub async fn unregister_server(&self, name: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        servers.remove(name);
        info!("Unregistered MCP server: {}", name);
        Ok(())
    }

    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        args: Value,
    ) -> Result<Value> {
        let servers = self.servers.read().await;
        let instance = servers
            .get(server_name)
            .with_context(|| format!("MCP server '{}' not found", server_name))?;

        if !instance.running {
            bail!("MCP server '{}' is not running", server_name);
        }

        instance.transport.call_tool(tool_name, args).await
    }

    pub async fn list_servers(&self) -> Result<Vec<String>> {
        let servers = self.servers.read().await;
        Ok(servers.keys().cloned().collect())
    }

    pub async fn list_tools(&self, server_name: &str) -> Result<Vec<McpToolInfo>> {
        let servers = self.servers.read().await;
        let instance = servers
            .get(server_name)
            .with_context(|| format!("MCP server '{}' not found", server_name))?;
        Ok(instance.tools.clone())
    }

    pub async fn start_server(&self, name: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(instance) = servers.get_mut(name) {
            instance.running = true;
            info!("Started MCP server: {}", name);
        }
        Ok(())
    }

    pub async fn stop_server(&self, name: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(instance) = servers.get_mut(name) {
            instance.running = false;
            info!("Stopped MCP server: {}", name);
        }
        Ok(())
    }

    pub async fn restart_server(&self, name: &str) -> Result<()> {
        self.stop_server(name).await?;
        self.start_server(name).await
    }

    pub async fn test_connection(&self) -> Result<bool> {
        Ok(true)
    }
}
