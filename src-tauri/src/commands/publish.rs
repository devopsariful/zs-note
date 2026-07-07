use std::path::Path;
use std::sync::{LazyLock, Mutex};

use base64::Engine;
use chrono::Utc;
use quick_xml::events::{BytesCData, BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use reqwest::Client;
use serde_json::Value;
use tauri::State;
use uuid::Uuid;

use crate::types::*;
use crate::AppState;

// ---------------------------------------------------------------------------
// In-memory publication history (persisted to .znrc in a real build)
// ---------------------------------------------------------------------------
static PUBLICATION_HISTORY: LazyLock<Mutex<Vec<PublicationRecord>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

// ---------------------------------------------------------------------------
// publish_now – dispatch to the correct backend
// ---------------------------------------------------------------------------
#[tauri::command]
pub async fn publish_now(
    state: State<'_, AppState>,
    target: PublishTarget,
    file_path: String,
    options: PublishOptions,
) -> Result<String, String> {
    match target.target_type.as_str() {
        "github" => publish_to_github(&state, &target, &file_path, &options).await,
        "custom_api" => publish_to_custom_api(&target, &file_path, &options).await,
        "rss" => generate_rss_feed(&state, &target, &file_path, &options).await,
        _ => Err(format!(
            "Unknown publish target type: {}",
            target.target_type
        )),
    }
}

// ---------------------------------------------------------------------------
// publish_preview – return the rendered Markdown with options applied
// ---------------------------------------------------------------------------
#[tauri::command]
pub async fn publish_preview(file_path: String) -> Result<String, String> {
    let content = tokio::fs::read_to_string(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Apply basic transformations for preview
    let preview = apply_publish_transforms(&content, false, false, false);
    Ok(preview)
}

// ---------------------------------------------------------------------------
// upload_image – upload an image file to the configured host
// ---------------------------------------------------------------------------
#[tauri::command]
pub async fn upload_image(
    state: State<'_, AppState>,
    file_path: String,
    _target_id: String,
) -> Result<String, String> {
    let config = state.config.read().await;
    let image_host = config
        .image_host
        .as_ref()
        .ok_or("No image host configured")?;

    match image_host.host_type {
        ImageHostType::GitHub => upload_to_github(image_host, &file_path).await,
        ImageHostType::Cloudflare => upload_to_cloudflare(image_host, &file_path).await,
    }
}

// ---------------------------------------------------------------------------
// generate_rss – standalone RSS feed generation from vault content
// ---------------------------------------------------------------------------
#[tauri::command]
pub async fn generate_rss(
    state: State<'_, AppState>,
    _target_id: String,
) -> Result<String, String> {
    let vault_path = state.vault_path.clone();
    let mut entries = Vec::new();

    // Walk the vault for .md files
    let walker = walkdir::WalkDir::new(&vault_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false));

    for entry in walker {
        let path = entry.path();
        let content = std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;

        if let Some((frontmatter, _body)) = parse_frontmatter(&content) {
            let title = frontmatter
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(
                    path.file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("Untitled"),
                )
                .to_string();

            let date_str = frontmatter
                .get("date")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tags: Vec<String> = frontmatter
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let relative = path
                .strip_prefix(&vault_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            entries.push(RssEntry {
                title,
                date: date_str,
                tags,
                path: relative,
                content: _body.clone(),
            });
        }
    }

    // Sort by date descending
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    build_rss_xml("ZarishNote Vault", &vault_path.to_string_lossy(), &entries)
}

// ---------------------------------------------------------------------------
// list_publications – return publication history
// ---------------------------------------------------------------------------
#[tauri::command]
pub fn list_publications() -> Result<Vec<PublicationRecord>, String> {
    let history = PUBLICATION_HISTORY
        .lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    Ok(history.clone())
}

// ===========================================================================
// Internal helpers
// ===========================================================================

struct RssEntry {
    title: String,
    date: String,
    tags: Vec<String>,
    path: String,
    content: String,
}

/// Parse YAML frontmatter from a Markdown string.
/// Returns `(frontmatter_map, body_without_frontmatter)`.
fn parse_frontmatter(content: &str) -> Option<(serde_json::Map<String, Value>, String)> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }

    let end = content[3..].find("---")?;
    let yaml_str = &content[3..3 + end];
    let body = content[3 + end + 3..].trim().to_string();

    let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_str).ok()?;
    let json_value = serde_json::to_value(yaml_value).ok()?;
    let map = json_value.as_object()?.clone();

    Some((map, body))
}

/// Apply transformations for publishing (wikilinks, private tags, etc.)
fn apply_publish_transforms(
    content: &str,
    convert_wikilinks: bool,
    strip_private: bool,
    _upload_images: bool,
) -> String {
    let mut result = content.to_string();

    // Convert [[wikilinks]] to [wikilinks](wikilinks.md)
    if convert_wikilinks {
        // Matches [[link]] and [[link|display]]
        let re = regex_lite::Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
        result = re
            .replace_all(&result, |caps: &regex_lite::Captures| {
                let link = &caps[1];
                let display = caps.get(2).map(|m| m.as_str()).unwrap_or(link);
                format!("[{}]({}.md)", display, link)
            })
            .to_string();
    }

    // Strip lines with `#private` tag
    if strip_private {
        let lines: Vec<&str> = result.lines().collect();
        let mut filtered = Vec::new();
        let mut in_private = false;
        for line in &lines {
            if line.contains("#private") {
                in_private = true;
            }
            if !in_private {
                filtered.push(*line);
            }
            if in_private && line.trim().is_empty() {
                in_private = false;
            }
        }
        result = filtered.join("\n");
    }

    result
}

/// Build an RSS 2.0 XML string from vault entries.
fn build_rss_xml(title: &str, link: &str, entries: &[RssEntry]) -> Result<String, String> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    // XML declaration
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
        .map_err(|e| format!("XML write error: {}", e))?;

    // <rss version="2.0">
    let mut rss_start = BytesStart::new("rss");
    rss_start.push_attribute(("version", "2.0"));
    rss_start.push_attribute(("xmlns:atom", "http://www.w3.org/2005/Atom"));
    rss_start.push_attribute(("xmlns:content", "http://purl.org/rss/1.0/modules/content/"));
    rss_start.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
    writer
        .write_event(Event::Start(rss_start))
        .map_err(|e| format!("XML write error: {}", e))?;

    // <channel>
    writer
        .write_event(Event::Start(BytesStart::new("channel")))
        .map_err(|e| format!("XML write error: {}", e))?;

    // Required channel elements
    write_text_element(&mut writer, "title", title)?;
    write_text_element(&mut writer, "link", link)?;
    write_text_element(&mut writer, "description", "ZarishNote published vault")?;
    write_text_element(&mut writer, "language", "en-us")?;

    // Atom self-link
    let mut atom_link = BytesStart::new("atom:link");
    let href = format!("{}/rss.xml", link);
    atom_link.push_attribute(("href", href.as_str()));
    atom_link.push_attribute(("rel", "self"));
    atom_link.push_attribute(("type", "application/rss+xml"));
    writer
        .write_event(Event::Empty(atom_link))
        .map_err(|e| format!("XML write error: {}", e))?;

    // Items
    for entry in entries {
        writer
            .write_event(Event::Start(BytesStart::new("item")))
            .map_err(|e| format!("XML write error: {}", e))?;

        write_text_element(&mut writer, "title", &entry.title)?;
        write_text_element(&mut writer, "link", &format!("{}/{}", link, entry.path))?;
        write_text_element(&mut writer, "guid", &format!("{}/{}", link, entry.path))?;

        // Date (RFC 2822)
        if !entry.date.is_empty() {
            if let Ok(parsed) = chrono::NaiveDate::parse_from_str(&entry.date, "%Y-%m-%d") {
                let dt = parsed.and_hms_opt(0, 0, 0).unwrap().and_utc();
                write_text_element(&mut writer, "pubDate", &dt.to_rfc2822())?;
            } else {
                write_text_element(&mut writer, "pubDate", &entry.date)?;
            }
        }

        // dc:creator
        write_text_element(&mut writer, "dc:creator", "ZarishNote User")?;

        // Tags as categories
        for tag in &entry.tags {
            write_text_element(&mut writer, "category", tag)?;
        }

        // Description (first 500 chars of body)
        let description = entry.content.chars().take(500).collect::<String>();
        write_text_element(&mut writer, "description", &description)?;

        // content:encoded with full body (CDATA)
        let cdata_start = BytesStart::new("content:encoded");
        writer
            .write_event(Event::Start(cdata_start))
            .map_err(|e| format!("XML write error: {}", e))?;
        let cdata = BytesCData::new(entry.content.as_str());
        writer
            .write_event(Event::CData(cdata))
            .map_err(|e| format!("XML write error: {}", e))?;
        writer
            .write_event(Event::End(BytesEnd::new("content:encoded")))
            .map_err(|e| format!("XML write error: {}", e))?;

        writer
            .write_event(Event::End(BytesEnd::new("item")))
            .map_err(|e| format!("XML write error: {}", e))?;
    }

    // Close channel
    writer
        .write_event(Event::End(BytesEnd::new("channel")))
        .map_err(|e| format!("XML write error: {}", e))?;
    // Close rss
    writer
        .write_event(Event::End(BytesEnd::new("rss")))
        .map_err(|e| format!("XML write error: {}", e))?;

    String::from_utf8(writer.into_inner()).map_err(|e| format!("UTF-8 conversion error: {}", e))
}

/// Write a simple text element: <name>value</name>
fn write_text_element(writer: &mut Writer<Vec<u8>>, name: &str, value: &str) -> Result<(), String> {
    writer
        .write_event(Event::Start(BytesStart::new(name)))
        .map_err(|e| format!("XML write error: {}", e))?;
    writer
        .write_event(Event::Text(BytesText::new(value)))
        .map_err(|e| format!("XML write error: {}", e))?;
    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(|e| format!("XML write error: {}", e))?;
    Ok(())
}

// ===========================================================================
// Publishing backends
// ===========================================================================

/// Publish by committing to the vault's git repo and pushing to remote.
async fn publish_to_github(
    state: &State<'_, AppState>,
    target: &PublishTarget,
    file_path: &str,
    options: &PublishOptions,
) -> Result<String, String> {
    // Read and transform content
    let raw = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let transformed = apply_publish_transforms(
        &raw,
        options.convert_wikilinks,
        options.strip_private,
        options.upload_images,
    );

    // Write transformed content back temporarily
    tokio::fs::write(file_path, &transformed)
        .await
        .map_err(|e| format!("Failed to write temp content: {}", e))?;

    // Use the existing git engine to commit and push
    let mut git = state.git.blocking_write();

    git.commit_all(&format!("Publish: {}", file_path))
        .map_err(|e| format!("Git commit failed: {}", e))?;

    git.push().map_err(|e| format!("Git push failed: {}", e))?;

    // Restore original content
    tokio::fs::write(file_path, &raw)
        .await
        .map_err(|e| format!("Failed to restore content: {}", e))?;

    let repo_url = target.repo.clone().unwrap_or_else(|| "unknown".to_string());

    // Record publication
    let record = PublicationRecord {
        id: Uuid::new_v4().to_string(),
        target_name: target.name.clone(),
        file_path: file_path.to_string(),
        published_at: Utc::now(),
        status: "success".to_string(),
        url: Some(format!("{}/blob/main/{}", repo_url, file_path)),
    };

    if let Ok(mut history) = PUBLICATION_HISTORY.lock() {
        history.push(record.clone());
    }

    serde_json::to_string(&record).map_err(|e| format!("Serialize error: {}", e))
}

/// Publish via HTTP POST to a custom API endpoint.
async fn publish_to_custom_api(
    target: &PublishTarget,
    file_path: &str,
    options: &PublishOptions,
) -> Result<String, String> {
    let endpoint = target
        .endpoint
        .as_deref()
        .or(options.custom_endpoint.as_deref())
        .ok_or("No endpoint configured for custom API target")?;

    let raw = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let transformed = apply_publish_transforms(
        &raw,
        options.convert_wikilinks,
        options.strip_private,
        options.upload_images,
    );

    // Build payload
    let path = Path::new(file_path);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled");

    let mut payload = serde_json::Map::new();
    payload.insert("content".to_string(), Value::String(transformed));
    payload.insert("filename".to_string(), Value::String(filename.to_string()));
    payload.insert("path".to_string(), Value::String(file_path.to_string()));
    payload.insert(
        "title".to_string(),
        Value::String(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("untitled")
                .to_string(),
        ),
    );

    if let Some(frontmatter) = parse_frontmatter(&raw) {
        payload.insert("frontmatter".to_string(), Value::Object(frontmatter.0));
    }

    // Build headers
    let client = Client::new();
    let mut req = client.post(endpoint).json(&payload);

    if let Some(headers) = &options.custom_headers {
        for h in headers {
            req = req.header(&h.key, &h.value);
        }
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("API returned {}: {}", status, body));
    }

    // Record publication
    let record = PublicationRecord {
        id: Uuid::new_v4().to_string(),
        target_name: target.name.clone(),
        file_path: file_path.to_string(),
        published_at: Utc::now(),
        status: "success".to_string(),
        url: Some(endpoint.to_string()),
    };

    if let Ok(mut history) = PUBLICATION_HISTORY.lock() {
        history.push(record.clone());
    }

    serde_json::to_string(&record).map_err(|e| format!("Serialize error: {}", e))
}

/// Generate RSS feed from vault content and optionally save to the vault.
async fn generate_rss_feed(
    state: &State<'_, AppState>,
    target: &PublishTarget,
    _file_path: &str,
    options: &PublishOptions,
) -> Result<String, String> {
    // If a specific file path is given and generate_rss is true, regenerate the full feed
    let vault_path = state.vault_path.clone();
    let mut entries = Vec::new();

    let walker = walkdir::WalkDir::new(&vault_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false));

    for entry in walker {
        let path = entry.path();
        let content = std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;

        if let Some((frontmatter, body)) = parse_frontmatter(&content) {
            let title = frontmatter
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(
                    path.file_stem()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("Untitled"),
                )
                .to_string();

            let date_str = frontmatter
                .get("date")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tags: Vec<String> = frontmatter
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let relative = path
                .strip_prefix(&vault_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            entries.push(RssEntry {
                title,
                date: date_str,
                tags,
                path: relative,
                content: body,
            });
        }
    }

    entries.sort_by(|a, b| b.date.cmp(&a.date));

    let rss_xml = build_rss_xml("ZarishNote Vault", &vault_path.to_string_lossy(), &entries)?;

    // If generate_rss is enabled, save the feed to the vault root
    if options.generate_rss {
        let rss_path = vault_path.join("rss.xml");
        tokio::fs::write(&rss_path, &rss_xml)
            .await
            .map_err(|e| format!("Failed to write RSS file: {}", e))?;
    }

    // Record publication
    let record = PublicationRecord {
        id: Uuid::new_v4().to_string(),
        target_name: target.name.clone(),
        file_path: _file_path.to_string(),
        published_at: Utc::now(),
        status: "success".to_string(),
        url: Some(format!("{}/rss.xml", vault_path.to_string_lossy())),
    };

    if let Ok(mut history) = PUBLICATION_HISTORY.lock() {
        history.push(record.clone());
    }

    Ok(rss_xml)
}

// ===========================================================================
// Image upload backends
// ===========================================================================

/// Upload an image to a GitHub repo via the Contents API.
async fn upload_to_github(host: &ImageHost, file_path: &str) -> Result<String, String> {
    let token = host
        .token
        .as_deref()
        .ok_or("GitHub token required for image upload")?;
    let repo = host
        .repo
        .as_deref()
        .ok_or("GitHub repo required for image upload")?;
    let branch = host.branch.as_deref().unwrap_or("main");

    let path = Path::new(file_path);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file path")?;

    let dest_path = format!("images/{}", filename);

    // Read and Base64-encode the file
    let image_bytes = tokio::fs::read(file_path)
        .await
        .map_err(|e| format!("Failed to read image: {}", e))?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&image_bytes);

    // Build the GitHub Contents API request
    let api_url = format!(
        "https://api.github.com/repos/{}/contents/{}",
        repo, dest_path
    );

    let body = serde_json::json!({
        "message": format!("Upload image: {}", filename),
        "content": encoded,
        "branch": branch,
    });

    let client = Client::new();
    let resp = client
        .put(&api_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "ZarishNote")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("GitHub upload failed: {}", e))?;

    let status = resp.status();
    let json: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "GitHub upload returned {}: {}",
            status,
            json.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
        ));
    }

    // Return the download URL
    json.get("content")
        .and_then(|c| c.get("download_url"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "GitHub response missing download_url".to_string())
}

/// Upload an image to Cloudflare Images API.
async fn upload_to_cloudflare(host: &ImageHost, file_path: &str) -> Result<String, String> {
    let account_id = host
        .account_id
        .as_deref()
        .ok_or("Cloudflare account ID required")?;
    let api_token = host
        .api_token
        .as_deref()
        .ok_or("Cloudflare API token required")?;

    let api_url = format!(
        "https://api.cloudflare.com/client/v4/accounts/{}/images/v1",
        account_id
    );

    let image_bytes = tokio::fs::read(file_path)
        .await
        .map_err(|e| format!("Failed to read image: {}", e))?;

    let path = Path::new(file_path);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid file path")?;

    // Use multipart form upload
    let part = reqwest::multipart::Part::bytes(image_bytes)
        .file_name(filename.to_string())
        .mime_str("image/png") // default; could be inferred
        .map_err(|e| format!("Failed to create multipart: {}", e))?;

    let form = reqwest::multipart::Form::new().part("file", part);

    let client = Client::new();
    let resp = client
        .post(&api_url)
        .header("Authorization", format!("Bearer {}", api_token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Cloudflare upload failed: {}", e))?;

    let status = resp.status();
    let json: Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Cloudflare response: {}", e))?;

    if !status.is_success() || json.get("success").and_then(|v| v.as_bool()) != Some(true) {
        let errors = json
            .get("errors")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        return Err(format!("Cloudflare upload failed: {}", errors));
    }

    // Return the Cloudflare Images delivery URL
    json.get("result")
        .and_then(|r| r.get("variants"))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Cloudflare response missing variants URL".to_string())
}
