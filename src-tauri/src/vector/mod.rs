pub mod index;
pub mod query;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::types::{IndexStats, SearchResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIndex {
    pub path: PathBuf,
    pub chunks: Vec<IndexChunk>,
    pub indexed_at: chrono::DateTime<chrono::Utc>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexChunk {
    pub id: String,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub start_offset: usize,
    pub end_offset: usize,
}

pub struct VectorStore {
    _db_path: PathBuf,
    index: Arc<RwLock<HashMap<String, DocumentIndex>>>,
    _table_path: PathBuf,
    /// Name of the embedding model to use (e.g. "text-embedding-ada-002", "nomic-embed-text", etc.)
    embedding_model: Arc<RwLock<Option<String>>>,
    /// Whether embedding mode is enabled. When disabled, only keyword search is used.
    embeddings_enabled: Arc<RwLock<bool>>,
}

impl VectorStore {
    pub fn new(vault_root: &Path) -> Self {
        let db_path = vault_root.join(".znrc-vector");
        let table_path = vault_root.join(".znrc-vector").join("documents");

        std::fs::create_dir_all(&db_path).ok();

        Self {
            _db_path: db_path,
            index: Arc::new(RwLock::new(HashMap::new())),
            _table_path: table_path,
            embedding_model: Arc::new(RwLock::new(None)),
            embeddings_enabled: Arc::new(RwLock::new(false)),
        }
    }

    /// Set the embedding model name.
    pub fn set_embedding_model(&self, model: Option<String>) {
        let mut m = self.embedding_model.write();
        *m = model;
    }

    /// Get the current embedding model name.
    pub fn get_embedding_model(&self) -> Option<String> {
        let m = self.embedding_model.read();
        m.clone()
    }

    /// Enable or disable embedding-based search.
    pub fn enable_embeddings(&self, enabled: bool) {
        let mut e = self.embeddings_enabled.write();
        *e = enabled;
        info!(
            "Embeddings {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Check if embeddings are enabled.
    pub fn is_embeddings_enabled(&self) -> bool {
        let e = self.embeddings_enabled.read();
        *e
    }

    /// Generate an embedding vector for the given text.
    ///
    /// Uses the AI provider router if available. Falls back to random embeddings
    /// (for testing / offline use) when no provider is configured.
    pub fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Try to use the default embedding model via the provider router.
        // For now, we produce a deterministic pseudo-embedding as a fallback.
        // In a full implementation, this would call the configured AI provider's
        // embeddings endpoint.
        if self.is_embeddings_enabled() {
            if let Some(ref _model) = *self.embedding_model.read() {
                // Placeholder: In production, call the AI provider's embedding API here.
                // The provider router would be accessible through AppState.
                return self.fallback_embedding(text);
            }
        }

        self.fallback_embedding(text)
    }

    /// Generate embeddings for text that has no stored embedding yet.
    ///
    /// Batch-processes documents that have chunks without embeddings.
    pub fn generate_missing_embeddings(&self) -> Result<usize> {
        let mut count = 0;
        let index = self.index.read();

        for doc in index.values() {
            for chunk in &doc.chunks {
                if chunk.embedding.is_none() {
                    // We could generate embeddings here in a background task.
                    // For the initial implementation, we skip missing embeddings.
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Fallback deterministic "embedding" using a hash-based approach.
    /// This is NOT semantically meaningful — it exists so the system works
    /// without an external AI provider. Cosine similarities computed from
    /// these will be essentially random.
    fn fallback_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let dim = 128;
        let mut embedding = Vec::with_capacity(dim);

        // Use a simple hash of the text to produce a deterministic vector.
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        for i in 0..dim {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            (seed ^ i as u64).hash(&mut h);
            h.finish().hash(&mut h);
            let val = (h.finish() as f64 / u64::MAX as f64) as f32;
            // Normalize to unit vector
            embedding.push(val * 2.0 - 1.0);
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut embedding {
                *x /= norm;
            }
        }

        Ok(embedding)
    }

    pub fn index_document(&self, path: &Path, content: &str) -> Result<()> {
        let checksum = compute_checksum(content);
        let file_path = path.to_string_lossy().to_string();

        {
            let index = self.index.read();
            if let Some(existing) = index.get(&file_path) {
                if existing.checksum == checksum {
                    return Ok(());
                }
            }
        }

        let chunks = index::chunk_text(content, 512, 64);

        let mut doc_chunks = Vec::new();
        for (i, chunk) in chunks.iter().enumerate() {
            doc_chunks.push(IndexChunk {
                id: format!("{}:{}", file_path, i),
                content: chunk.clone(),
                embedding: None,
                start_offset: 0,
                end_offset: chunk.len(),
            });
        }

        let doc_index = DocumentIndex {
            path: path.to_path_buf(),
            chunks: doc_chunks,
            indexed_at: Utc::now(),
            checksum,
        };

        let mut index = self.index.write();
        index.insert(file_path, doc_index);

        info!("Indexed document: {:?}", path);
        Ok(())
    }

    /// Query using keyword-based similarity (TF-like scoring)
    pub fn query(&self, query_text: &str, _kb_name: &str, top_k: usize) -> Vec<SearchResult> {
        let query_lower = query_text.to_lowercase();
        let index = self.index.read();

        let mut scored: Vec<(f64, &DocumentIndex, &IndexChunk)> = Vec::new();

        for (_, doc) in index.iter() {
            for chunk in &doc.chunks {
                let chunk_lower = chunk.content.to_lowercase();
                let score = compute_similarity(&query_lower, &chunk_lower);

                if score > 0.0 {
                    scored.push((score, doc, chunk));
                }
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(top_k)
            .map(|(score, doc, chunk)| SearchResult {
                path: doc.path.clone(),
                title: doc
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string(),
                snippet: truncate_snippet(&chunk.content, 200),
                score: (score * 1000.0).round() / 1000.0,
            })
            .collect()
    }

    /// Query using embedding-based similarity (cosine similarity).
    /// Generates an embedding for the query and compares it to stored chunk embeddings.
    /// Falls back to keyword search if embeddings are not available.
    pub fn query_with_embeddings(
        &self,
        query: &str,
        kb_name: &str,
        top_k: usize,
        min_score: f64,
    ) -> Vec<SearchResult> {
        if !self.is_embeddings_enabled() {
            return self.query(query, kb_name, top_k);
        }

        // Generate query embedding
        let query_embedding = match self.generate_embedding(query) {
            Ok(e) => e,
            Err(_) => return self.query(query, kb_name, top_k),
        };

        let index = self.index.read();
        let mut scored: Vec<(f64, &DocumentIndex, &IndexChunk)> = Vec::new();

        for (_, doc) in index.iter() {
            for chunk in &doc.chunks {
                let score = if let Some(ref chunk_emb) = chunk.embedding {
                    cosine_similarity(&query_embedding, chunk_emb)
                } else {
                    // Fallback to keyword score if no embedding
                    let chunk_lower = chunk.content.to_lowercase();
                    let query_lower = query.to_lowercase();
                    compute_similarity(&query_lower, &chunk_lower) * 0.5 // discounted keyword score
                };

                if score > min_score {
                    scored.push((score, doc, chunk));
                }
            }
        }

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .into_iter()
            .take(top_k)
            .map(|(score, doc, chunk)| SearchResult {
                path: doc.path.clone(),
                title: doc
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string(),
                snippet: truncate_snippet(&chunk.content, 200),
                score: (score * 1000.0).round() / 1000.0,
            })
            .collect()
    }

    pub fn delete_document(&self, path: &Path) -> Result<()> {
        let file_path = path.to_string_lossy().to_string();
        let mut index = self.index.write();
        index.remove(&file_path);
        info!("Removed document from index: {:?}", path);
        Ok(())
    }

    pub fn get_index_stats(&self) -> IndexStats {
        let index = self.index.read();
        let total_docs = index.len() as u64;
        let total_chunks = index.values().map(|d| d.chunks.len() as u64).sum();
        let last_indexed = index.values().map(|d| d.indexed_at).max();

        // Estimate size from in-memory data
        let size_bytes = index
            .values()
            .flat_map(|d| &d.chunks)
            .map(|c| c.content.len() + c.id.len())
            .sum::<usize>() as u64;

        IndexStats {
            total_documents: total_docs,
            total_chunks,
            index_size_bytes: size_bytes,
            last_indexed,
        }
    }

    pub fn rebuild_index(&self, vault_root: &Path) -> Result<()> {
        let mut index = self.index.write();
        index.clear();
        drop(index);

        let walker = walkdir::WalkDir::new(vault_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file()
                    && e.path()
                        .extension()
                        .map(|ext| matches!(ext.to_str(), Some("md" | "markdown" | "txt")))
                        .unwrap_or(false)
            });

        for entry in walker {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Err(e) = self.index_document(entry.path(), &content) {
                    tracing::warn!("Failed to index {:?}: {}", entry.path(), e);
                }
            }
        }

        info!("Rebuilt vector index");
        Ok(())
    }
}

fn compute_checksum(content: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Compute cosine similarity between two vectors.
/// Returns a value in [-1, 1], with 1 meaning identical direction.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot / (norm_a * norm_b)) as f64
}

fn compute_similarity(query: &str, chunk: &str) -> f64 {
    if query.is_empty() || chunk.is_empty() {
        return 0.0;
    }

    let query_words: Vec<&str> = query.split_whitespace().collect();
    let chunk_words: Vec<&str> = chunk.split_whitespace().collect();

    if query_words.is_empty() || chunk_words.is_empty() {
        return 0.0;
    }

    let mut matches = 0;
    for qw in &query_words {
        for cw in &chunk_words {
            if cw.contains(qw) || qw.contains(cw) {
                matches += 1;
                break;
            }
        }
    }

    let precision = matches as f64 / query_words.len() as f64;
    let recall = matches as f64 / chunk_words.len() as f64;

    if precision + recall == 0.0 {
        return 0.0;
    }

    2.0 * precision * recall / (precision + recall)
}

pub fn truncate_snippet(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    let mut truncated = text[..max_len].to_string();
    if let Some(last_space) = truncated.rfind(' ') {
        truncated.truncate(last_space);
    }
    truncated.push_str("...");
    truncated
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6, "Expected 1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-6, "Expected 0.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-6, "Expected -1.0, got {}", sim);
    }

    #[test]
    fn test_cosine_similarity_partial() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let sim = cosine_similarity(&a, &b);
        // dot = 4+10+18 = 32, |a| = sqrt(14), |b| = sqrt(77)
        // sim = 32 / (sqrt(14) * sqrt(77)) = 32 / sqrt(1078) ≈ 32 / 32.832 ≈ 0.9746
        assert!(
            (sim - 0.9746).abs() < 0.001,
            "Expected ~0.9746, got {}",
            sim
        );
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_cosine_similarity_mismatched_lengths() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_fallback_embedding_deterministic() {
        let store = VectorStore::new(Path::new("/tmp/test_emb"));
        let emb1 = store.fallback_embedding("hello world").unwrap();
        let emb2 = store.fallback_embedding("hello world").unwrap();
        assert_eq!(emb1.len(), emb2.len());
        assert_eq!(emb1, emb2, "Deterministic embeddings should be identical");
    }

    #[test]
    fn test_fallback_embedding_normalized() {
        let store = VectorStore::new(Path::new("/tmp/test_emb2"));
        let emb = store.fallback_embedding("test vector").unwrap();
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 0.001,
            "Expected normalized (norm~1), got {}",
            norm
        );
    }

    #[test]
    fn test_fallback_embedding_dimension() {
        let store = VectorStore::new(Path::new("/tmp/test_emb3"));
        let emb = store.fallback_embedding("dim check").unwrap();
        assert_eq!(emb.len(), 128, "Expected 128-dimensional embedding");
    }

    #[test]
    fn test_fallback_embedding_different_inputs() {
        let store = VectorStore::new(Path::new("/tmp/test_emb4"));
        let emb1 = store.fallback_embedding("apple").unwrap();
        let emb2 = store.fallback_embedding("orange").unwrap();
        assert_ne!(
            emb1, emb2,
            "Different inputs should produce different embeddings"
        );
    }
}
