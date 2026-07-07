use tauri::State;
use walkdir::WalkDir;

use crate::types::{AppState, SearchResult};

#[derive(Debug, serde::Deserialize)]
pub struct SearchFilters {
    pub file_types: Option<Vec<String>>,
    pub max_size: Option<u64>,
    pub min_size: Option<u64>,
    pub modified_after: Option<String>,
    pub modified_before: Option<String>,
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingConfig {
    pub enabled: bool,
    pub model: Option<String>,
}

#[tauri::command]
pub fn search_files(
    state: State<'_, AppState>,
    query: String,
    scope: Option<String>,
    filters: Option<SearchFilters>,
) -> Result<Vec<SearchResult>, String> {
    let vault = &state.vault_path;
    let search_dir = match &scope {
        Some(s) => vault.join(s.trim_start_matches('/')),
        None => vault.clone(),
    };

    let case_sensitive = filters
        .as_ref()
        .and_then(|f| f.case_sensitive)
        .unwrap_or(false);
    let file_types = filters.as_ref().and_then(|f| f.file_types.clone());

    let query_lower = query.to_lowercase();
    let mut results: Vec<(SearchResult, u32)> = Vec::new();

    for entry in WalkDir::new(&search_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        if let Some(types) = &file_types {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !types.iter().any(|t| t == ext || t == &format!(".{}", ext)) {
                continue;
            }
        }

        if let Some(ext) = path.extension() {
            if ext == "md" || ext == "markdown" || ext == "txt" {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let content_for_search = if case_sensitive {
                        content.clone()
                    } else {
                        content.to_lowercase()
                    };
                    let search_term = if case_sensitive { &query } else { &query_lower };

                    if let Some(pos) = content_for_search.find(search_term) {
                        let snippet_start = pos.saturating_sub(80);
                        let snippet_end = (pos + query.len() + 80).min(content.len());
                        let snippet = if snippet_start > 0 {
                            format!("...{}...", &content[snippet_start..snippet_end])
                        } else {
                            content[snippet_start..snippet_end].to_string()
                        };

                        let count = content_for_search.matches(search_term).count() as u32;
                        let title = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();

                        results.push((
                            SearchResult {
                                path: path.to_path_buf(),
                                title,
                                snippet,
                                score: count as f64,
                            },
                            count,
                        ));
                    }
                }
            }
        }
    }

    results.sort_by_key(|b| std::cmp::Reverse(b.1));
    Ok(results.into_iter().map(|(r, _)| r).take(50).collect())
}

/// Enable or disable embedding-based search in the vector store.
#[tauri::command]
pub fn set_embeddings_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    state.vector.enable_embeddings(enabled);
    Ok(())
}

/// Set the embedding model name.
#[tauri::command]
pub fn set_embedding_model(state: State<'_, AppState>, model: String) -> Result<(), String> {
    state.vector.set_embedding_model(Some(model));
    Ok(())
}

/// Get the current embedding configuration.
#[tauri::command]
pub fn get_embedding_config(state: State<'_, AppState>) -> Result<EmbeddingConfig, String> {
    Ok(EmbeddingConfig {
        enabled: state.vector.is_embeddings_enabled(),
        model: state.vector.get_embedding_model(),
    })
}

/// Perform a vector search query with optional embedding support.
#[tauri::command]
pub fn vector_search(
    state: State<'_, AppState>,
    query: String,
    kb_name: String,
    top_k: usize,
    use_embeddings: bool,
    min_score: f64,
) -> Result<Vec<SearchResult>, String> {
    if use_embeddings && state.vector.is_embeddings_enabled() {
        Ok(state
            .vector
            .query_with_embeddings(&query, &kb_name, top_k, min_score))
    } else {
        Ok(state.vector.query(&query, &kb_name, top_k))
    }
}

/// Perform a hybrid search combining keyword and embedding scores.
#[tauri::command]
pub fn hybrid_search(
    state: State<'_, AppState>,
    query: String,
    keyword_weight: f64,
    semantic_weight: f64,
    top_k: usize,
) -> Result<Vec<SearchResult>, String> {
    Ok(state
        .vector
        .hybrid_search(&query, keyword_weight, semantic_weight, top_k))
}
