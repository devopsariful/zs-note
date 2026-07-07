use anyhow::{bail, Result};
use async_trait::async_trait;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::mcp::protocol::{parse_jsonrpc_message, JsonRpcMessage, JsonRpcRequest};
use crate::types::McpToolInfo;

#[derive(Debug, Clone, PartialEq)]
pub enum TransportType {
    Stdio,
    Http,
}

#[async_trait]
pub trait McpTransport {
    async fn send_request(&self, method: &str, params: Value) -> Result<Value>;
    async fn call_tool(&self, name: &str, args: Value) -> Result<Value>;
    async fn list_tools(&self) -> Result<Vec<McpToolInfo>>;
    async fn initialize(&self) -> Result<Value>;
    async fn close(&mut self) -> Result<()>;
}

pub struct StdioTransport {
    process: Mutex<Child>,
    stdin: Mutex<tokio::process::ChildStdin>,
    stdout: Mutex<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>,
    request_id: Mutex<u64>,
}

impl StdioTransport {
    pub fn new(command: &str, args: Vec<String>) -> Self {
        let mut child = Command::new(command)
            .args(&args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .unwrap_or_else(|e| {
                // Log the error rather than panicking — callers handle connection failure
                tracing::error!("Failed to spawn MCP subprocess '{}': {}", command, e);
                panic!("MCP subprocess spawn failed: {}", e)
            });

        let stdin = child.stdin.take().expect("Failed to capture stdin");
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let reader = BufReader::new(stdout).lines();

        Self {
            process: Mutex::new(child),
            stdin: Mutex::new(stdin),
            stdout: Mutex::new(reader),
            request_id: Mutex::new(0),
        }
    }

    async fn next_id(&self) -> u64 {
        let mut id = self.request_id.lock().await;
        *id += 1;
        *id
    }

    async fn send_raw(&self, message: &str) -> Result<()> {
        let mut stdin = self.stdin.lock().await;
        stdin.write_all(message.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn read_line(&self) -> Result<String> {
        let mut stdout = self.stdout.lock().await;
        match stdout.next_line().await {
            Ok(Some(line)) => Ok(line),
            Ok(None) => bail!("MCP subprocess closed stdout"),
            Err(e) => bail!("MCP read error: {}", e),
        }
    }
}

#[async_trait]
impl McpTransport for StdioTransport {
    async fn send_request(&self, _method: &str, params: Value) -> Result<Value> {
        let id = self.next_id().await;
        let request = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: id as i64,
            method: _method.to_string(),
            params: Some(params),
        };

        let request_str = serde_json::to_string(&request)?;
        self.send_raw(&request_str).await?;

        let response_line = self.read_line().await?;
        let response: JsonRpcMessage = parse_jsonrpc_message(&response_line)?;

        match response {
            JsonRpcMessage::Success { result, .. } => Ok(result),
            JsonRpcMessage::Error { error, .. } => {
                bail!("MCP error {}: {}", error.code, error.message)
            }
            _ => bail!("Unexpected JSON-RPC message type"),
        }
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<Value> {
        let params = serde_json::json!({
            "name": name,
            "arguments": args,
        });
        self.send_request("tools/call", params).await
    }

    async fn list_tools(&self) -> Result<Vec<McpToolInfo>> {
        let result = self
            .send_request("tools/list", serde_json::json!({}))
            .await?;

        let tools = result["tools"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|t| McpToolInfo {
                        name: t["name"].as_str().unwrap_or("").to_string(),
                        description: t["description"].as_str().unwrap_or("").to_string(),
                        input_schema: t["inputSchema"].clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(tools)
    }

    async fn initialize(&self) -> Result<Value> {
        self.send_request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "zarishnote",
                    "version": "1.0.0"
                }
            }),
        )
        .await
    }

    async fn close(&mut self) -> Result<()> {
        let mut process = self.process.lock().await;
        process.kill().await?;
        process.wait().await?;
        Ok(())
    }
}

pub struct HttpTransport {
    client: reqwest::Client,
    base_url: String,
    request_id: std::sync::atomic::AtomicU64,
    _sse_url: Option<String>,
}

impl HttpTransport {
    pub fn new(url: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: url.trim_end_matches('/').to_string(),
            request_id: std::sync::atomic::AtomicU64::new(0),
            _sse_url: None,
        }
    }

    fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait]
impl McpTransport for HttpTransport {
    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id();
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let response = self
            .client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let data: Value = response.json().await?;

        if let Some(error) = data.get("error") {
            bail!(
                "MCP error {}: {}",
                error["code"].as_i64().unwrap_or(-1),
                error["message"].as_str().unwrap_or("unknown")
            );
        }

        Ok(data["result"].clone())
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<Value> {
        let params = serde_json::json!({
            "name": name,
            "arguments": args,
        });
        self.send_request("tools/call", params).await
    }

    async fn list_tools(&self) -> Result<Vec<McpToolInfo>> {
        let result = self
            .send_request("tools/list", serde_json::json!({}))
            .await?;

        let tools = result["tools"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|t| McpToolInfo {
                        name: t["name"].as_str().unwrap_or("").to_string(),
                        description: t["description"].as_str().unwrap_or("").to_string(),
                        input_schema: t["inputSchema"].clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(tools)
    }

    async fn initialize(&self) -> Result<Value> {
        self.send_request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "zarishnote",
                    "version": "1.0.0"
                }
            }),
        )
        .await
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}
