use keyring::Entry;

const SERVICE_NAME: &str = "zarishnote";
const TARGET_KEY: &str = "api_keys";

#[tauri::command]
pub fn store_api_key(provider: String, api_key: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, &format!("{}_{}", TARGET_KEY, provider))
        .map_err(|e| format!("Failed to create keychain entry: {}", e))?;
    entry
        .set_password(&api_key)
        .map_err(|e| format!("Failed to store API key: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_api_key(provider: String) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, &format!("{}_{}", TARGET_KEY, provider))
        .map_err(|e| format!("Failed to create keychain entry: {}", e))?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to read API key: {}", e)),
    }
}

#[tauri::command]
pub fn delete_api_key(provider: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, &format!("{}_{}", TARGET_KEY, provider))
        .map_err(|e| format!("Failed to create keychain entry: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete API key: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn list_api_keys() -> Result<Vec<String>, String> {
    let known_providers = vec!["openai", "anthropic", "google", "deepseek"];
    let mut found = Vec::new();

    for provider in known_providers {
        let entry = Entry::new(SERVICE_NAME, &format!("{}_{}", TARGET_KEY, provider))
            .map_err(|e| format!("Failed to create keychain entry: {}", e))?;
        if entry.get_password().is_ok() {
            found.push(provider.to_string());
        }
    }

    Ok(found)
}
