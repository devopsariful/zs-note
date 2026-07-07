//! Image generation commands for ZarishNote.
//!
//! Supports DALL-E 3, DALL-E 2, and Stability AI image generation APIs.
//! Generated images are returned as base64-encoded strings.

use reqwest::Client;
use serde_json::Value;
use tauri::State;

use crate::types::{AppState, GeneratedImage};

/// Generate an image using an AI image model.
///
/// * `prompt` – text description of the desired image.
/// * `model` – model identifier (e.g. "dall-e-3", "dall-e-2", "stability-ai").
/// * `size` – image size as "WxH" (e.g. "1024x1024", "1792x1024", "1024x1792").
/// * `quality` – quality level ("standard" or "hd"; DALL-E 3 only).
#[tauri::command]
pub async fn generate_image(
    state: State<'_, AppState>,
    prompt: String,
    model: String,
    size: String,
    quality: String,
) -> Result<GeneratedImage, String> {
    let config = state.config.read().await;
    let api_key = config.ai.api_key.as_deref().unwrap_or("").to_string();
    drop(config);

    if api_key.is_empty() {
        return Err("API key not configured. Set your API key in Settings > API Keys.".into());
    }

    match model.as_str() {
        "dall-e-3" | "dall-e-2" => generate_dalle(&api_key, &prompt, &model, &size, &quality).await,
        "stability-ai" => generate_stability(&api_key, &prompt, &size).await,
        _ => Err(format!(
            "Unsupported image model: {}. Supported: dall-e-3, dall-e-2, stability-ai",
            model
        )),
    }
}

/// Generate image via OpenAI DALL-E API.
async fn generate_dalle(
    api_key: &str,
    prompt: &str,
    model: &str,
    size: &str,
    quality: &str,
) -> Result<GeneratedImage, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = "https://api.openai.com/v1/images/generations";

    let mut payload = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "n": 1,
        "size": size,
        "response_format": "b64_json",
    });

    // DALL-E 3 supports quality parameter
    if model == "dall-e-3" && (quality == "hd" || quality == "standard") {
        payload["quality"] = serde_json::json!(quality);
    }

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !status.is_success() {
        let err_msg = body["error"]["message"]
            .as_str()
            .unwrap_or("Unknown API error");
        return Err(format!("DALL-E API error ({}): {}", status, err_msg));
    }

    let data = body["data"][0]
        .as_object()
        .ok_or_else(|| "No image data in response".to_string())?;

    let b64 = data["b64_json"]
        .as_str()
        .ok_or_else(|| "Missing b64_json in response".to_string())?
        .to_string();

    let revised_prompt = data["revised_prompt"].as_str().map(String::from);

    Ok(GeneratedImage {
        data: b64,
        mime_type: "image/png".into(),
        model: model.to_string(),
        prompt: prompt.to_string(),
        seed: None,
        revised_prompt,
    })
}

/// Generate image via Stability AI API (Stable Diffusion).
async fn generate_stability(
    api_key: &str,
    prompt: &str,
    size: &str,
) -> Result<GeneratedImage, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Parse size
    let (width, height) = parse_size(size)?;

    let url = "https://api.stability.ai/v2beta/stable-image/generate/sd3";

    let form = reqwest::multipart::Form::new()
        .text("prompt", prompt.to_string())
        .text("output_format", "png")
        .text("width", width.to_string())
        .text("height", height.to_string())
        .text("mode", "text-to-image")
        .text("model", "sd3-large");

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Accept", "image/png")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Stability AI request failed: {}", e))?;

    let status = response.status();

    if !status.is_success() {
        let err_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".into());
        return Err(format!("Stability AI error ({}): {}", status, err_text));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read image bytes: {}", e))?;

    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

    Ok(GeneratedImage {
        data: b64,
        mime_type: "image/png".into(),
        model: "stability-ai".into(),
        prompt: prompt.to_string(),
        seed: None,
        revised_prompt: None,
    })
}

/// Parse a "WxH" size string into (width, height).
fn parse_size(size: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = size.split('x').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid size format '{}'. Expected WxH (e.g. 1024x1024)",
            size
        ));
    }
    let width = parts[0]
        .parse::<u32>()
        .map_err(|_| format!("Invalid width: {}", parts[0]))?;
    let height = parts[1]
        .parse::<u32>()
        .map_err(|_| format!("Invalid height: {}", parts[1]))?;

    let valid_sizes = [
        (1024, 1024),
        (1792, 1024),
        (1024, 1792),
        (768, 768),
        (512, 512),
    ];
    if !valid_sizes.contains(&(width, height)) {
        return Err(format!(
            "Unsupported size {}x{}. Supported sizes: 1024x1024, 1792x1024, 1024x1792, 768x768, 512x512",
            width, height
        ));
    }

    Ok((width, height))
}
