use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use tracing::info;

use crate::vector::VectorStore;

pub fn chunk_text(text: &str, max_chunk_size: usize, overlap: usize) -> Vec<String> {
    if text.len() <= max_chunk_size {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < text.len() {
        let end = (start + max_chunk_size).min(text.len());

        if end < text.len() {
            let search_start = end.saturating_sub(max_chunk_size / 4);
            let search_slice = &text[search_start..end];
            if let Some(newline_pos) = search_slice.rfind('\n') {
                let chunk_end = search_start + newline_pos;
                chunks.push(text[start..chunk_end].to_string());
                start = chunk_end.saturating_sub(overlap);
                continue;
            }
            if let Some(space_pos) = search_slice.rfind(' ') {
                let chunk_end = search_start + space_pos;
                chunks.push(text[start..chunk_end].to_string());
                start = chunk_end.saturating_sub(overlap);
                continue;
            }
        }

        chunks.push(text[start..end].to_string());

        if end >= text.len() {
            break;
        }

        start = end.saturating_sub(overlap);

        if start + max_chunk_size >= text.len() && start < text.len() {
            chunks.push(text[start..].to_string());
            break;
        }
    }

    chunks
}

pub fn chunk_by_paragraphs(text: &str, max_chunk_size: usize) -> Vec<String> {
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    let mut chunks = Vec::new();
    let mut current = String::new();

    for para in paragraphs {
        if current.len() + para.len() + 2 > max_chunk_size && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current = String::new();
        }

        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(para);
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    if chunks.is_empty() && !text.is_empty() {
        chunks.push(text.to_string());
    }

    chunks
}

pub fn chunk_semantic(text: &str, max_chunk_size: usize) -> Vec<String> {
    let boundaries = ["\n## ", "\n### ", "\n---\n", "\n\n", "\n", ". ", "! ", "? "];

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < text.len() {
        let end = (start + max_chunk_size).min(text.len());
        let search_section = &text[start..end];
        let mut best_boundary = None;

        for boundary in &boundaries {
            if let Some(pos) = search_section.rfind(boundary) {
                let absolute_pos = start + pos + boundary.len();
                if absolute_pos > start && absolute_pos < text.len() {
                    best_boundary = Some(absolute_pos);
                    break;
                }
            }
        }

        if let Some(boundary_pos) = best_boundary {
            chunks.push(text[start..boundary_pos].to_string());
            start = boundary_pos;
            continue;
        }

        chunks.push(text[start..end].to_string());
        start = end;
    }

    if chunks.is_empty() && !text.is_empty() {
        chunks.push(text.to_string());
    }

    chunks
}

pub fn chunk_hierarchical(text: &str, max_chunk_size: usize) -> Vec<Vec<String>> {
    let sections = chunk_semantic(text, max_chunk_size);
    sections
        .into_iter()
        .map(|section| {
            if section.len() > max_chunk_size {
                chunk_text(&section, max_chunk_size, max_chunk_size / 8)
            } else {
                vec![section]
            }
        })
        .collect()
}

pub fn deduplicate_chunks(chunks: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for chunk in chunks {
        let normalized: String = chunk.chars().filter(|c| !c.is_whitespace()).collect();

        if seen.insert(normalized) {
            result.push(chunk);
        }
    }

    result
}

impl VectorStore {
    pub fn batch_index(&self, vault_root: &Path) -> Result<usize> {
        let mut count = 0;
        let walker = walkdir::WalkDir::new(vault_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file()
                    && e.path()
                        .extension()
                        .map(|ext| {
                            matches!(
                                ext.to_str(),
                                Some("md" | "markdown" | "txt" | "json" | "yaml" | "yml" | "toml")
                            )
                        })
                        .unwrap_or(false)
            });

        for entry in walker {
            match std::fs::read_to_string(entry.path()) {
                Ok(content) => {
                    if let Err(e) = self.index_document(entry.path(), &content) {
                        tracing::warn!("Failed to index {:?}: {}", entry.path(), e);
                    } else {
                        count += 1;
                    }
                }
                Err(e) => {
                    tracing::debug!("Skipping {:?}: {}", entry.path(), e);
                }
            }
        }

        info!("Batch indexed {} documents", count);
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_basic() {
        let text = "Hello world this is a test.";
        let chunks = chunk_text(text, 10, 2);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_chunk_text_small() {
        let text = "Short text";
        let chunks = chunk_text(text, 100, 10);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Short text");
    }

    #[test]
    fn test_chunk_by_paragraphs() {
        let text = "Para one.\n\nPara two.\n\nPara three.";
        let chunks = chunk_by_paragraphs(text, 20);
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_chunk_semantic() {
        let text = "## Section 1\nContent here.\n## Section 2\nMore content.";
        let chunks = chunk_semantic(text, 100);
        assert!(chunks.len() >= 2);
    }
}
