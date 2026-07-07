use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: i64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Success {
        jsonrpc: String,
        id: i64,
        result: Value,
    },
    Error {
        jsonrpc: String,
        id: i64,
        error: JsonRpcError,
    },
    Request {
        jsonrpc: String,
        id: i64,
        method: String,
        params: Option<Value>,
    },
    Notification {
        jsonrpc: String,
        method: String,
        params: Option<Value>,
    },
}

pub fn parse_jsonrpc_message(data: &str) -> Result<JsonRpcMessage> {
    let value: Value = serde_json::from_str(data)
        .with_context(|| format!("Invalid JSON-RPC message: {}", data))?;

    let has_id = value.get("id").is_some();
    let has_method = value.get("method").is_some();
    let has_result = value.get("result").is_some();
    let has_error = value.get("error").is_some();

    match (has_id, has_method, has_result, has_error) {
        (true, true, _, _) => {
            let msg: JsonRpcRequest = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Request {
                jsonrpc: msg.jsonrpc,
                id: msg.id,
                method: msg.method,
                params: msg.params,
            })
        }
        (true, false, true, false) => {
            let jsonrpc = value["jsonrpc"].as_str().unwrap_or("2.0").to_string();
            let id = value["id"].as_i64().unwrap_or(0);
            let result = value["result"].clone();
            Ok(JsonRpcMessage::Success {
                jsonrpc,
                id,
                result,
            })
        }
        (true, false, true, true) => {
            // Both result and error present — prefer error by spec
            let jsonrpc = value["jsonrpc"].as_str().unwrap_or("2.0").to_string();
            let id = value["id"].as_i64().unwrap_or(0);
            let error: JsonRpcError = serde_json::from_value(value["error"].clone())?;
            Ok(JsonRpcMessage::Error { jsonrpc, id, error })
        }
        (true, false, false, true) => {
            let jsonrpc = value["jsonrpc"].as_str().unwrap_or("2.0").to_string();
            let id = value["id"].as_i64().unwrap_or(0);
            let error: JsonRpcError = serde_json::from_value(value["error"].clone())?;
            Ok(JsonRpcMessage::Error { jsonrpc, id, error })
        }
        (false, true, false, false) => {
            let msg: JsonRpcNotification = serde_json::from_value(value)?;
            Ok(JsonRpcMessage::Notification {
                jsonrpc: msg.jsonrpc,
                method: msg.method,
                params: msg.params,
            })
        }
        _ => bail!("Unrecognized JSON-RPC message structure: {}", data),
    }
}

pub fn create_initialize_request(id: i64) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id,
        method: "initialize".into(),
        params: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
            },
            "clientInfo": {
                "name": "zarishnote",
                "version": "1.0.0"
            }
        })),
    }
}

pub fn create_list_tools_request(id: i64) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id,
        method: "tools/list".into(),
        params: Some(serde_json::json!({})),
    }
}

pub fn create_call_tool_request(id: i64, name: &str, arguments: Value) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id,
        method: "tools/call".into(),
        params: Some(serde_json::json!({
            "name": name,
            "arguments": arguments,
        })),
    }
}

pub mod notifications {
    use serde_json::Value;

    pub fn create_cancelled_notification(id: i64) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/cancelled",
            "params": {
                "requestId": id,
            }
        })
    }

    pub fn create_initialized_notification() -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
        })
    }

    pub fn create_progress_notification(token: i64, progress: f64, total: Option<f64>) -> Value {
        let mut params = serde_json::json!({
            "progressToken": token,
            "progress": progress,
        });
        if let Some(t) = total {
            params["total"] = serde_json::json!(t);
        }
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/progress",
            "params": params,
        })
    }
}
