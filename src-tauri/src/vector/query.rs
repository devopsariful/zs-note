use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::SearchResult;
use crate::vector::{DocumentIndex, IndexChunk, VectorStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptions {
    pub top_k: usize,
    pub min_score: f64,
    pub include_snippet: bool,
    pub include_full_content: bool,
    pub kb_filter: Option<Vec<String>>,
    pub path_filter: Option<Vec<String>>,
    /// Whether to use embedding-based search (if available).
    pub use_embeddings: bool,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            top_k: 10,
            min_score: 0.0,
            include_snippet: true,
            include_full_content: false,
            kb_filter: None,
            path_filter: None,
            use_embeddings: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub results: Vec<SearchResult>,
    pub total_hits: usize,
    pub query_time_ms: u64,
}

impl VectorStore {
    pub fn search(&self, query: &str, options: QueryOptions) -> QueryResult {
        let start = std::time::Instant::now();

        // Choose search strategy based on embedding availability and options
        let mut results = if options.use_embeddings && self.is_embeddings_enabled() {
            self.query_with_embeddings(query, "default", options.top_k, options.min_score)
        } else {
            self.query(query, "default", options.top_k)
        };

        if options.min_score > 0.0 {
            results.retain(|r| r.score >= options.min_score);
        }

        if let Some(ref kbs) = options.kb_filter {
            results.retain(|r| kbs.iter().any(|kb| r.path.to_string_lossy().contains(kb)));
        }

        if let Some(ref paths) = options.path_filter {
            results.retain(|r| paths.iter().any(|p| r.path.to_string_lossy().contains(p)));
        }

        let total = results.len();

        QueryResult {
            results,
            total_hits: total,
            query_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    pub fn similarity_search(&self, query: &str, top_k: usize) -> Vec<(f64, SearchResult)> {
        let results = self.query(query, "default", top_k);
        results.into_iter().map(|r| (r.score, r)).collect()
    }

    pub fn search_with_citations(&self, query: &str, top_k: usize) -> String {
        let results = self.query(query, "default", top_k);

        if results.is_empty() {
            return "No relevant documents found.".to_string();
        }

        let mut output = String::from("### Search Results\n\n");
        for (i, result) in results.iter().enumerate() {
            let title = &result.title;
            let path = result.path.display();
            let score = result.score;

            output.push_str(&format!("**{}.** {} (score: {:.3})\n", i + 1, title, score));
            output.push_str(&format!("   `{}`\n", path));
            output.push_str(&format!("   > {}\n\n", result.snippet));
        }

        output
    }

    /// Perform a hybrid search that combines keyword and embedding-based scores.
    pub fn hybrid_search(
        &self,
        query: &str,
        keyword_weight: f64,
        semantic_weight: f64,
        top_k: usize,
    ) -> Vec<SearchResult> {
        let semantic_results = if self.is_embeddings_enabled() {
            self.query_with_embeddings(query, "default", top_k * 2, 0.0)
        } else {
            self.query(query, "default", top_k * 2)
        };

        let keyword_results = self.keyword_search(query, top_k * 2);
        let mut combined: HashMap<String, (SearchResult, f64)> = HashMap::new();

        for (i, result) in semantic_results.iter().enumerate() {
            let key = result.path.to_string_lossy().to_string();
            let score = result.score * semantic_weight * (1.0 / (i as f64 + 1.0));
            combined
                .entry(key)
                .or_insert_with(|| (result.clone(), 0.0))
                .1 += score;
        }

        for (i, result) in keyword_results.iter().enumerate() {
            let key = result.path.to_string_lossy().to_string();
            let score = keyword_weight * (1.0 / (i as f64 + 1.0));
            combined
                .entry(key)
                .or_insert_with(|| (result.clone(), 0.0))
                .1 += score;
        }

        let mut ranked: Vec<(SearchResult, f64)> = combined.into_values().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        ranked.into_iter().take(top_k).map(|(r, _)| r).collect()
    }

    pub fn keyword_search(&self, query: &str, top_k: usize) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let index = self.index.read();
        let mut scored = Vec::new();

        for (_, doc) in index.iter() {
            let full_text: String = doc
                .chunks
                .iter()
                .map(|c| c.content.as_str())
                .collect::<Vec<&str>>()
                .join(" ");

            let text_lower = full_text.to_lowercase();

            let mut term_freq = 0;
            for term in &query_terms {
                term_freq += text_lower.matches(term).count();
            }

            if term_freq > 0 {
                let title = doc
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                let snippet = doc
                    .chunks
                    .first()
                    .map(|c| {
                        let text = &c.content;
                        if text.len() > 200 {
                            format!("{}...", &text[..200])
                        } else {
                            text.clone()
                        }
                    })
                    .unwrap_or_default();

                scored.push((
                    SearchResult {
                        path: doc.path.clone(),
                        title,
                        snippet,
                        score: term_freq as f64,
                    },
                    term_freq,
                ));
            }
        }

        scored.sort_by_key(|b| std::cmp::Reverse(b.1));
        scored.into_iter().take(top_k).map(|(r, _)| r).collect()
    }

    /// Semantic search using only embedding similarity.
    /// Returns results ranked by cosine similarity.
    pub fn semantic_search(&self, query: &str, top_k: usize) -> Vec<SearchResult> {
        if !self.is_embeddings_enabled() {
            return self.query(query, "default", top_k);
        }

        let query_embedding = match self.generate_embedding(query) {
            Ok(e) => e,
            Err(_) => return self.query(query, "default", top_k),
        };

        let index = self.index.read();
        let mut scored: Vec<(f64, &DocumentIndex, &IndexChunk)> = Vec::new();

        for (_, doc) in index.iter() {
            for chunk in &doc.chunks {
                let score = chunk
                    .embedding
                    .as_ref()
                    .map(|emb| crate::vector::cosine_similarity(&query_embedding, emb))
                    .unwrap_or(0.0);

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
                snippet: crate::vector::truncate_snippet(&chunk.content, 200),
                score: (score * 1000.0).round() / 1000.0,
            })
            .collect()
    }
}

pub fn format_citation(result: &SearchResult, style: &str) -> String {
    match style {
        "inline" => {
            format!("[{}]({})", result.title, result.path.display())
        }
        "footnote" => {
            format!(
                "{} [^{}]",
                result.snippet,
                result.title.replace(' ', "-").to_lowercase()
            )
        }
        "academic" => {
            format!(
                "\"{}\" ({})",
                result.title,
                result
                    .path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
            )
        }
        _ => {
            format!("{} ({})", result.title, result.path.display())
        }
    }
}

pub fn rerank_results(query: &str, results: Vec<SearchResult>) -> Vec<SearchResult> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    let mut scored: Vec<(SearchResult, f64)> = results
        .into_iter()
        .map(|r| {
            let title_lower = r.title.to_lowercase();
            let snippet_lower = r.snippet.to_lowercase();

            let title_bonus = query_terms
                .iter()
                .filter(|t| title_lower.contains(*t))
                .count() as f64
                * 2.0;

            let snippet_bonus = query_terms
                .iter()
                .filter(|t| snippet_lower.contains(*t))
                .count() as f64;

            (r, title_bonus + snippet_bonus)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    scored.into_iter().map(|(r, _)| r).collect()
}
