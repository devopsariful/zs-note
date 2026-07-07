use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

use crate::mcp::marketplace::MarketplaceEntry;
use crate::types::AppState;

/// Information about an installed WASM plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub enabled: bool,
    pub wasm_path: String,
    pub permissions: Vec<String>,
    pub homepage: Option<String>,
    pub source: Option<String>,
}

/// Manifest file parsed from a plugin directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub homepage: Option<String>,
    pub wasm: Option<String>,
    pub source: Option<String>,
}

/// Path to the plugin directory inside the vault.
fn plugins_dir(vault_path: &std::path::Path) -> PathBuf {
    vault_path.join(".znrc-plugins")
}

/// Path to a plugin's directory.
fn plugin_dir(vault_path: &std::path::Path, id: &str) -> PathBuf {
    plugins_dir(vault_path).join(id)
}

/// Path to a plugin's manifest file.
fn plugin_manifest_path(vault_path: &std::path::Path, id: &str) -> PathBuf {
    plugin_dir(vault_path, id).join("plugin.toml")
}

/// Read and parse a plugin manifest from disk.
fn read_manifest(manifest_path: &std::path::Path) -> Result<PluginManifest, String> {
    let contents = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;
    toml::from_str::<PluginManifest>(&contents)
        .map_err(|e| format!("Failed to parse plugin.toml: {}", e))
}

/// Build a PluginInfo from a manifest and optional directory contents.
fn manifest_to_info(
    vault_path: &std::path::Path,
    manifest: PluginManifest,
    enabled: bool,
) -> PluginInfo {
    let wasm_path = manifest
        .wasm
        .clone()
        .unwrap_or_else(|| format!("{}.wasm", manifest.id));
    let full_wasm_path = plugin_dir(vault_path, &manifest.id)
        .join(&wasm_path)
        .to_string_lossy()
        .to_string();

    PluginInfo {
        id: manifest.id,
        name: manifest.name,
        version: manifest.version,
        description: manifest.description.unwrap_or_default(),
        author: manifest.author.unwrap_or_default(),
        enabled,
        wasm_path: full_wasm_path,
        permissions: manifest.permissions.unwrap_or_default(),
        homepage: manifest.homepage,
        source: manifest.source,
    }
}

/// Get the set of enabled plugin IDs from the config.
fn get_enabled_plugins(state: &AppState) -> Vec<String> {
    let config = state.config.blocking_read();
    config.plugins.enabled.clone()
}

/// Check whether a plugin is enabled in the config.
fn is_plugin_enabled(state: &AppState, id: &str) -> bool {
    let config = state.config.blocking_read();
    config.plugins.enabled.contains(&id.to_string())
}

/// Set the enabled state of a plugin in the config.
fn set_plugin_enabled(state: &AppState, id: &str, enabled: bool) -> Result<(), String> {
    let mut config = state.config.blocking_write();
    if enabled {
        if !config.plugins.enabled.contains(&id.to_string()) {
            config.plugins.enabled.push(id.to_string());
        }
    } else {
        config.plugins.enabled.retain(|e| e != id);
    }
    config
        .save(&state.vault_path)
        .map_err(|e| format!("Failed to save config: {}", e))
}

/// Install a WASM plugin from a .wasm file path.
///
/// Copies the .wasm file into `.znrc-plugins/<id>/` and expects
/// a `plugin.toml` manifest to also exist in the source directory
/// or next to the .wasm file.
#[tauri::command]
pub fn plugin_install(path: String, state: State<'_, AppState>) -> Result<PluginInfo, String> {
    let source_path = std::path::Path::new(&path);
    let vault_path = &state.vault_path;

    // Determine the plugin directory from the source
    let source_dir = source_path
        .parent()
        .ok_or_else(|| "Cannot determine source directory".to_string())?;

    // Look for manifest next to the .wasm file
    let manifest_candidates = [
        source_dir.join("plugin.toml"),
        source_path.with_extension("toml"),
    ];

    let manifest_path = manifest_candidates
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| {
            format!(
                "No plugin.toml found alongside {:?}. Create one with id, name, version fields.",
                source_path
            )
        })?;

    let manifest = read_manifest(manifest_path)?;

    // Ensure the plugin has an id
    if manifest.id.is_empty() {
        return Err("plugin.toml must contain a non-empty 'id' field".to_string());
    }

    // Create the plugin directory
    let dest_dir = plugin_dir(vault_path, &manifest.id);
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create plugin directory: {}", e))?;

    // Determine wasm target name
    let wasm_filename = manifest
        .wasm
        .clone()
        .unwrap_or_else(|| format!("{}.wasm", manifest.id));

    let dest_wasm = dest_dir.join(&wasm_filename);

    // Copy the .wasm file
    std::fs::copy(source_path, &dest_wasm)
        .map_err(|e| format!("Failed to copy WASM file: {}", e))?;

    // Copy the manifest
    let dest_manifest = dest_dir.join("plugin.toml");
    std::fs::copy(manifest_path, &dest_manifest)
        .map_err(|e| format!("Failed to copy manifest: {}", e))?;

    info!(
        "Installed plugin '{}' from {:?}",
        manifest.name, source_path
    );

    // Validate with sandbox engine
    let wasm_bytes =
        std::fs::read(&dest_wasm).map_err(|e| format!("Failed to read installed WASM: {}", e))?;
    state
        .sandbox
        .test_module(&wasm_bytes)
        .map_err(|e| format!("WASM validation failed: {}", e))?;

    // Enable by default
    set_plugin_enabled(&state, &manifest.id, true)?;

    let info = manifest_to_info(vault_path, manifest, true);
    Ok(info)
}

/// Remove a plugin's directory from `.znrc-plugins/`.
#[tauri::command]
pub fn plugin_uninstall(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let dir = plugin_dir(&state.vault_path, &id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)
            .map_err(|e| format!("Failed to remove plugin directory: {}", e))?;
    }

    // Also remove from enabled list
    set_plugin_enabled(&state, &id, false)?;

    info!("Uninstalled plugin '{}'", id);
    Ok(())
}

/// List all installed plugins by scanning `.znrc-plugins/`.
#[tauri::command]
pub fn plugin_list(state: State<'_, AppState>) -> Result<Vec<PluginInfo>, String> {
    let dir = plugins_dir(&state.vault_path);
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let enabled_set = get_enabled_plugins(&state);

    let mut plugins = Vec::new();
    let entries =
        std::fs::read_dir(&dir).map_err(|e| format!("Failed to read plugins directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("plugin.toml");
        if !manifest_path.exists() {
            continue;
        }

        let plugin_id = entry.file_name().to_string_lossy().to_string();
        match read_manifest(&manifest_path) {
            Ok(manifest) => {
                let enabled =
                    enabled_set.contains(&plugin_id) || enabled_set.contains(&manifest.id);
                plugins.push(manifest_to_info(&state.vault_path, manifest, enabled));
            }
            Err(e) => {
                tracing::warn!("Skipping plugin at {:?}: {}", path, e);
            }
        }
    }

    Ok(plugins)
}

/// Enable or disable a plugin.
#[tauri::command]
pub fn plugin_toggle(id: String, enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    set_plugin_enabled(&state, &id, enabled)?;
    info!("Plugin '{}' toggled to {}", id, enabled);
    Ok(())
}

/// Get detailed information about a single plugin.
#[tauri::command]
pub fn plugin_get_info(id: String, state: State<'_, AppState>) -> Result<PluginInfo, String> {
    let manifest_path = plugin_manifest_path(&state.vault_path, &id);
    if !manifest_path.exists() {
        return Err(format!("Plugin '{}' is not installed", id));
    }

    let manifest = read_manifest(&manifest_path)?;
    let enabled = is_plugin_enabled(&state, &id);
    Ok(manifest_to_info(&state.vault_path, manifest, enabled))
}

/// Fetch available MCP servers from the marketplace registry.
#[tauri::command]
pub async fn marketplace_fetch(
    registry_url: Option<String>,
) -> Result<Vec<MarketplaceEntry>, String> {
    let url =
        registry_url.unwrap_or_else(|| "https://marketplace.zarishsphere.com/api/mcp".to_string());
    let mut registry = crate::mcp::marketplace::MarketplaceRegistry::new(&url);
    registry
        .fetch_servers()
        .await
        .map_err(|e| format!("Failed to fetch marketplace: {}", e))
}

/// Install an MCP server from the marketplace into the config.
#[tauri::command]
pub async fn marketplace_install(
    server_id: String,
    registry_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<MarketplaceEntry, String> {
    let url =
        registry_url.unwrap_or_else(|| "https://marketplace.zarishsphere.com/api/mcp".to_string());
    let mut registry = crate::mcp::marketplace::MarketplaceRegistry::new(&url);

    // Fetch fresh listing
    registry
        .fetch_servers()
        .await
        .map_err(|e| format!("Failed to fetch marketplace: {}", e))?;

    let mut config = state.config.blocking_write();

    registry
        .install_server(server_id, &mut config)
        .await
        .map_err(|e| format!("Failed to install server: {}", e))
}

/// Check for updates to installed marketplace servers.
#[tauri::command]
pub async fn marketplace_check_updates(
    registry_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::mcp::marketplace::UpdateInfo>, String> {
    let url =
        registry_url.unwrap_or_else(|| "https://marketplace.zarishsphere.com/api/mcp".to_string());
    let registry = crate::mcp::marketplace::MarketplaceRegistry::new(&url);

    let config = state.config.blocking_read();
    registry
        .check_for_updates(&config)
        .await
        .map_err(|e| format!("Failed to check updates: {}", e))
}

/// Uninstall an MCP server that was installed from the marketplace.
#[tauri::command]
pub fn marketplace_uninstall(server_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.config.blocking_write();
    crate::mcp::marketplace::MarketplaceRegistry::uninstall_server(&server_id, &mut config)
        .map_err(|e| format!("Failed to uninstall server: {}", e))
}
