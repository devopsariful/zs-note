use std::path::PathBuf;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use tauri::State;

use crate::types::AppState;

fn ensure_assets_dir(vault_path: &std::path::Path) -> Result<PathBuf, String> {
    let assets_dir = vault_path.join("assets");
    std::fs::create_dir_all(&assets_dir)
        .map_err(|e| format!("Failed to create assets dir: {}", e))?;
    Ok(assets_dir)
}

#[tauri::command]
pub fn import_image(
    state: State<'_, AppState>,
    data_url: String,
    file_name: String,
) -> Result<String, String> {
    let assets_dir = ensure_assets_dir(&state.vault_path)?;

    let sanitized_name =
        file_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|', ' '], "_");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let unique_name = format!("{}_{}", timestamp, sanitized_name);
    let dest_path = assets_dir.join(&unique_name);

    let parts: Vec<&str> = data_url.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err("Invalid data URL".into());
    }

    let image_data = BASE64
        .decode(parts[1])
        .map_err(|e| format!("Failed to decode image data: {}", e))?;

    std::fs::write(&dest_path, &image_data)
        .map_err(|e| format!("Failed to write image file: {}", e))?;

    let rel_path = format!("assets/{}", unique_name);
    Ok(format!("![{}]({})", file_name, rel_path))
}

#[tauri::command]
pub fn import_files(
    state: State<'_, AppState>,
    source_paths: Vec<String>,
    target_dir: String,
) -> Result<Vec<String>, String> {
    let vault = &state.vault_path;
    let dest_dir = if target_dir.is_empty() || target_dir == "/" {
        vault.clone()
    } else {
        let cleaned = target_dir.trim_start_matches('/');
        vault.join(cleaned)
    };

    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| format!("Failed to create target directory: {}", e))?;

    let mut imported: Vec<String> = Vec::new();

    for src_str in &source_paths {
        let src = std::path::Path::new(src_str);
        if !src.exists() {
            continue;
        }

        let file_name = src
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let dest = dest_dir.join(&file_name);

        if dest.exists() {
            let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
            let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("");
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let dedup_name = format!("{}_{}.{}", stem, timestamp, ext);
            let dest = dest_dir.join(&dedup_name);
            std::fs::copy(src, &dest)
                .map_err(|e| format!("Failed to copy file {}: {}", file_name, e))?;
            imported.push(dest.to_string_lossy().to_string());
        } else {
            std::fs::copy(src, &dest)
                .map_err(|e| format!("Failed to copy file {}: {}", file_name, e))?;
            imported.push(dest.to_string_lossy().to_string());
        }
    }

    Ok(imported)
}
