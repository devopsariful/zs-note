use tauri::State;

use crate::mcp::MCPClient;
use crate::types::*;
use crate::AppState;

/// Helper: read MCP server configs from plugins.settings["mcp"]
fn get_mcp_configs(config: &crate::config::Config) -> Vec<serde_json::Value> {
    config
        .plugins
        .settings
        .get("mcp")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Parse a single mcp config JSON entry into McpServerInfo
fn parse_server(cfg: &serde_json::Value) -> McpServerInfo {
    let id = cfg
        .get("id")
        .and_then(|v| v.as_str())
        .or_else(|| cfg.get("name").and_then(|v| v.as_str()))
        .unwrap_or("unknown")
        .to_string();
    let name = cfg
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let transport = cfg
        .get("transport")
        .and_then(|v| v.as_str())
        .unwrap_or("stdio")
        .to_string();
    let enabled = cfg.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
    let command = cfg
        .get("command")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let url = cfg
        .get("url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let args = cfg.get("args").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect()
    });
    let tools = cfg
        .get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    Some(McpToolInfo {
                        name: t.get("name").and_then(|v| v.as_str())?.to_string(),
                        description: t
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        input_schema: t
                            .get("inputSchema")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    McpServerInfo {
        id,
        name,
        transport,
        command,
        args,
        url,
        enabled,
        status: McpStatus::Disconnected,
        error_message: None,
        tools,
    }
}

#[tauri::command]
pub async fn mcp_list_servers(state: State<'_, AppState>) -> Result<Vec<McpServerInfo>, String> {
    let config = state.config.read().await;
    let mcp_configs = get_mcp_configs(&config);
    Ok(mcp_configs.iter().map(parse_server).collect())
}

/// Frontend sends { server: McpServerInfo }
#[tauri::command]
pub async fn mcp_add_server(
    state: State<'_, AppState>,
    server: serde_json::Value,
) -> Result<(), String> {
    let mut config = state.config.write().await;
    // Ensure id field exists (derive from name if absent)
    let mut entry = server.clone();
    if entry.get("id").is_none() {
        if let Some(name) = entry.get("name").and_then(|v| v.as_str()) {
            let id = name.to_lowercase().replace(' ', "-");
            if let Some(obj) = entry.as_object_mut() {
                obj.insert("id".to_string(), serde_json::Value::String(id));
            }
        }
    }

    let mcp_entry = config
        .plugins
        .settings
        .entry("mcp".to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));

    if let Some(arr) = mcp_entry.as_array_mut() {
        arr.push(entry);
    }

    config
        .save(&state.vault_path)
        .map_err(|e| format!("Failed to save config: {}", e))
}

/// Frontend sends { id } (uses id field, not name)
#[tauri::command]
pub async fn mcp_remove_server(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut config = state.config.write().await;
    if let Some(arr) = config
        .plugins
        .settings
        .get_mut("mcp")
        .and_then(|v| v.as_array_mut())
    {
        arr.retain(|entry| {
            // Match by id OR by name (backward compat)
            let entry_id = entry.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let entry_name = entry.get("name").and_then(|v| v.as_str()).unwrap_or("");
            entry_id != id && entry_name != id
        });
    }

    config
        .save(&state.vault_path)
        .map_err(|e| format!("Failed to save config: {}", e))
}

/// Frontend sends { id, enabled }
#[tauri::command]
pub async fn mcp_toggle_server(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut config = state.config.write().await;
    if let Some(arr) = config
        .plugins
        .settings
        .get_mut("mcp")
        .and_then(|v| v.as_array_mut())
    {
        for entry in arr.iter_mut() {
            let entry_id = entry.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let entry_name = entry.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if entry_id == id || entry_name == id {
                if let Some(obj) = entry.as_object_mut() {
                    obj.insert("enabled".to_string(), serde_json::Value::Bool(enabled));
                }
            }
        }
    }

    config
        .save(&state.vault_path)
        .map_err(|e| format!("Failed to save config: {}", e))
}

/// Frontend sends { id } — test connection by id
#[tauri::command]
pub async fn mcp_test_connection(id: String) -> Result<bool, String> {
    let client = MCPClient::new(&id);
    client
        .test_connection()
        .await
        .map_err(|e| format!("Connection test failed: {}", e))
}

#[tauri::command]
pub async fn mcp_call_tool(
    server: String,
    tool: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let client = MCPClient::new(&server);
    client
        .call_tool(&server, &tool, args)
        .await
        .map_err(|e| format!("Tool call failed: {}", e))
}
