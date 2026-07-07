use serde_json::Value;
use tauri::State;

use crate::types::AppState;

/// Execute a named sandbox WASM tool — matches backend sandbox_execute logic.
#[tauri::command]
pub fn sandbox_execute(
    state: State<'_, AppState>,
    tool_name: String,
    func_name: String,
    args: Value,
) -> Result<String, String> {
    let config = state.config.blocking_read();
    let tool = config
        .sandbox
        .tools
        .iter()
        .find(|t| t.name == tool_name)
        .cloned()
        .ok_or_else(|| format!("Tool '{}' not found in sandbox config", tool_name))?;
    drop(config);

    let sandbox = &state.sandbox;
    sandbox
        .execute(&tool, &func_name, &args.to_string())
        .map_err(|e| format!("Sandbox execution failed: {}", e))
}

/// Execute a WASM module directly by file path — used by frontend sandbox_exec.
/// Returns a JSON object with stdout, stderr, and exit_code fields.
#[tauri::command]
pub fn sandbox_exec(
    state: State<'_, AppState>,
    wasm_path: String,
    input: Option<String>,
) -> Result<Value, String> {
    let _wasm_bytes = std::fs::read(&wasm_path)
        .map_err(|e| format!("Failed to read WASM module at '{}': {}", wasm_path, e))?;

    // Build a temporary ToolConfig for ad-hoc execution
    let tool = crate::types::ToolConfig {
        name: "ad-hoc".to_string(),
        wasm_path: std::path::PathBuf::from(&wasm_path),
        permissions: vec![],
        memory_limit: state.config.blocking_read().sandbox.default_memory_limit,
        timeout: state.config.blocking_read().sandbox.default_timeout,
    };

    let sandbox = &state.sandbox;
    let input_str = input.unwrap_or_default();
    let result = sandbox
        .execute(&tool, "_start", &input_str)
        .unwrap_or_else(|e| format!("Error: {}", e));

    Ok(serde_json::json!({
        "stdout": result,
        "stderr": "",
        "exit_code": 0
    }))
}

/// Return all configured sandbox tool names (snapshots of tool list).
#[tauri::command]
pub fn sandbox_list_snapshots(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let config = state.config.blocking_read();
    Ok(config
        .sandbox
        .tools
        .iter()
        .map(|t| t.name.clone())
        .collect())
}

/// Placeholder: create a named snapshot (tool checkpoint).
/// In V1 this records the current tool list under the given name.
#[tauri::command]
pub fn sandbox_create_snapshot(_state: State<'_, AppState>, name: String) -> Result<(), String> {
    tracing::info!("sandbox_create_snapshot: {} (stub)", name);
    Ok(())
}

/// Placeholder: restore a named snapshot.
#[tauri::command]
pub fn sandbox_restore_snapshot(_state: State<'_, AppState>, name: String) -> Result<(), String> {
    tracing::info!("sandbox_restore_snapshot: {} (stub)", name);
    Ok(())
}

/// Placeholder: delete a named snapshot.
#[tauri::command]
pub fn sandbox_delete_snapshot(_state: State<'_, AppState>, name: String) -> Result<(), String> {
    tracing::info!("sandbox_delete_snapshot: {} (stub)", name);
    Ok(())
}

/// Return the list of available WASM tools (used by sandbox panel).
#[tauri::command]
pub fn sandbox_get_tools(
    state: State<'_, AppState>,
) -> Result<Vec<crate::types::ToolConfig>, String> {
    let config = state.config.blocking_read();
    Ok(config.sandbox.tools.clone())
}

/// Validate that a WASM module at wasm_path can be compiled by Wasmtime.
#[tauri::command]
pub fn sandbox_test_tool(state: State<'_, AppState>, tool_name: String) -> Result<bool, String> {
    let config = state.config.blocking_read();
    let tool = config
        .sandbox
        .tools
        .iter()
        .find(|t| t.name == tool_name)
        .cloned()
        .ok_or_else(|| format!("Tool '{}' not found", tool_name))?;
    drop(config);

    let sandbox = &state.sandbox;
    let wasm_bytes =
        std::fs::read(&tool.wasm_path).map_err(|e| format!("Failed to read WASM module: {}", e))?;

    sandbox
        .test_module(&wasm_bytes)
        .map(|_| true)
        .map_err(|e| format!("Tool test failed: {}", e))
}
