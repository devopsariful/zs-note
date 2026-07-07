use tauri::State;

use crate::config::Config;
use crate::types::AppState;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<Config, String> {
    let config = state.config.blocking_read();
    Ok(config.clone())
}

#[tauri::command]
pub fn update_config(state: State<'_, AppState>, new_config: Config) -> Result<(), String> {
    new_config
        .validate()
        .map_err(|e| format!("Invalid config: {}", e))?;

    let mut config = state.config.blocking_write();
    *config = new_config.clone();

    config
        .save(&state.vault_path)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn reset_config(state: State<'_, AppState>) -> Result<(), String> {
    let default = Config::default();
    default
        .save(&state.vault_path)
        .map_err(|e| format!("Failed to reset config: {}", e))?;

    let mut config = state.config.blocking_write();
    *config = default;

    Ok(())
}

#[tauri::command]
pub fn reload_config(state: State<'_, AppState>) -> Result<Config, String> {
    let vault = &state.vault_path;
    let loaded = Config::load(vault).map_err(|e| format!("Failed to reload config: {}", e))?;

    let mut config = state.config.blocking_write();
    *config = loaded.clone();

    Ok(loaded)
}

#[tauri::command]
pub fn get_providers(
    state: State<'_, AppState>,
) -> Result<Vec<crate::types::ProviderConfig>, String> {
    let config = state.config.blocking_read();
    Ok(config.ai.providers.clone())
}

#[tauri::command]
pub fn save_providers(
    state: State<'_, AppState>,
    providers: Vec<crate::types::ProviderConfig>,
) -> Result<(), String> {
    let mut config = state.config.blocking_write();
    config.ai.providers = providers;

    config
        .save(&state.vault_path)
        .map_err(|e| format!("Failed to save providers: {}", e))?;

    Ok(())
}
