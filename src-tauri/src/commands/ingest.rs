use std::path::PathBuf;

use tauri::State;
use tokio::process::Command;

use crate::types::{AppState, ConverterInfo};

#[tauri::command]
pub async fn ingest_file(
    state: State<'_, AppState>,
    source: String,
    output_path: String,
    mime_hint: Option<String>,
) -> Result<(), String> {
    let source_path = PathBuf::from(&source);
    let output = PathBuf::from(&output_path);

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    let vault = &state.vault_path;
    let ingest_script = vault.join("scripts").join("ingest.py");

    let mut cmd = Command::new("python3");
    cmd.arg(&ingest_script)
        .arg("--source")
        .arg(&source_path)
        .arg("--output")
        .arg(&output);

    if let Some(mime) = mime_hint {
        cmd.arg("--mime").arg(mime);
    }

    let result = cmd
        .output()
        .await
        .map_err(|e| format!("Ingest process failed: {}", e))?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("Ingestion failed: {}", stderr));
    }

    let content = std::fs::read_to_string(&output)
        .map_err(|e| format!("Failed to read ingested output: {}", e))?;

    let _ = state.vector.index_document(&output, &content);

    Ok(())
}

#[tauri::command]
pub async fn ingest_url(url: String) -> Result<String, String> {
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    let content = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    Ok(content)
}

#[tauri::command]
pub fn list_converters() -> Vec<ConverterInfo> {
    vec![
        ConverterInfo {
            name: "markdown".into(),
            description: "Converts various formats to Markdown using Pandoc".into(),
            input_formats: vec![
                "docx".into(),
                "html".into(),
                "epub".into(),
                "rst".into(),
                "latex".into(),
                "odt".into(),
            ],
            output_formats: vec!["md".into(), "markdown".into()],
        },
        ConverterInfo {
            name: "images".into(),
            description: "Extracts text from images via OCR (Tesseract)".into(),
            input_formats: vec![
                "png".into(),
                "jpg".into(),
                "jpeg".into(),
                "tiff".into(),
                "bmp".into(),
            ],
            output_formats: vec!["md".into(), "txt".into()],
        },
        ConverterInfo {
            name: "pandoc".into(),
            description: "Universal document converter via Pandoc".into(),
            input_formats: vec![
                "docx".into(),
                "html".into(),
                "epub".into(),
                "rst".into(),
                "latex".into(),
                "odt".into(),
                "org".into(),
                "csv".into(),
                "tsv".into(),
            ],
            output_formats: vec![
                "md".into(),
                "html".into(),
                "pdf".into(),
                "latex".into(),
                "docx".into(),
            ],
        },
    ]
}
