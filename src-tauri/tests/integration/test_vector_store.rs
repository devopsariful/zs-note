//! Integration tests for the vector store subsystem.
//!
//! Tests the public API of [`zs_note_lib::vector`]:
//! indexing, querying, deletion, rebuild, deduplication and
//! empty‑store behaviour.

use std::fs;
use std::path::{Path, PathBuf};

use zs_note_lib::vector::{index, VectorStore};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temporary directory and return the path together with a
/// `VectorStore` rooted at that directory.
fn setup_temp_store() -> (tempfile::TempDir, VectorStore) {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let store = VectorStore::new(dir.path());
    (dir, store)
}

/// Write a text file inside a given directory.
fn write_doc(root: &Path, rel: &str, content: &str) -> PathBuf {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&p, content).expect("failed to write test doc");
    p
}

// ---------------------------------------------------------------------------
// Index & Query
// ---------------------------------------------------------------------------

#[test]
fn test_vector_store_index_and_query() {
    let (_dir, store) = setup_temp_store();

    // Index a document
    let content = "ZarishNote is a modern note‑taking application with \
                   sandboxed code execution and vector search capabilities.";
    let doc_path = PathBuf::from("test_doc.md");
    store
        .index_document(&doc_path, content)
        .expect("index_document should succeed");

    // Query for a term that appears in the content
    let results = store.query("ZarishNote", "default", 5);
    assert!(!results.is_empty(), "should find results for 'ZarishNote'");
    assert!(
        results[0].score > 0.0,
        "score should be > 0 for a matching query"
    );
    assert_eq!(results[0].path, doc_path);
}

#[test]
fn test_vector_store_query_multiple_docs() {
    let (_dir, store) = setup_temp_store();

    store
        .index_document(&PathBuf::from("alpha.md"), "The quick brown fox")
        .unwrap();
    store
        .index_document(&PathBuf::from("beta.md"), "jumps over the lazy dog")
        .unwrap();

    // Both documents share no common words; "fox" only appears in alpha
    let results = store.query("fox", "default", 10);
    assert_eq!(results.len(), 1, "only alpha should match 'fox'");
    assert!(results[0].path.to_string_lossy().contains("alpha"));
}

// ---------------------------------------------------------------------------
// Delete
// ---------------------------------------------------------------------------

#[test]
fn test_vector_store_delete() {
    let (_dir, store) = setup_temp_store();

    let doc_path = PathBuf::from("delete_me.md");
    store
        .index_document(&doc_path, "Content to be deleted")
        .unwrap();

    // Confirm it appears in results
    let before = store.query("deleted", "default", 5);
    assert!(!before.is_empty());

    // Delete and verify it's gone
    store
        .delete_document(&doc_path)
        .expect("delete should succeed");
    let after = store.query("deleted", "default", 5);
    assert!(after.is_empty(), "document should no longer be found");
}

#[test]
fn test_vector_store_delete_nonexistent() {
    let (_dir, store) = setup_temp_store();
    // Deleting a document that was never indexed should not error
    store
        .delete_document(&PathBuf::from("ghost.md"))
        .expect("delete of non‑existent doc should be a no‑op");
}

// ---------------------------------------------------------------------------
// Rebuild
// ---------------------------------------------------------------------------

#[test]
fn test_vector_store_rebuild() {
    let dir = tempfile::tempdir().expect("temp dir");

    // Create a few markdown files inside the temp vault
    write_doc(dir.path(), "doc1.md", "First document content about apples");
    write_doc(
        dir.path(),
        "doc2.md",
        "Second document content about oranges",
    );
    write_doc(dir.path(), "sub/doc3.md", "Nested document about bananas");

    let store = VectorStore::new(dir.path());

    // Before rebuild the store is empty
    let stats_before = store.get_index_stats();
    assert_eq!(stats_before.total_documents, 0);

    // Rebuild walks the vault directory and indexes all supported files
    store
        .rebuild_index(dir.path())
        .expect("rebuild should succeed");

    let stats_after = store.get_index_stats();
    assert_eq!(
        stats_after.total_documents, 3,
        "should have indexed 3 documents"
    );

    // Verify we can search across rebuilt documents
    let results = store.query("apples", "default", 5);
    assert!(!results.is_empty(), "should find 'apples' after rebuild");
}

#[test]
fn test_vector_store_rebuild_skips_unsupported_extensions() {
    let dir = tempfile::tempdir().expect("temp dir");
    write_doc(dir.path(), "notes.md", "markdown file");
    write_doc(dir.path(), "binary.bin", "\0\0\0\0\0\0\0\0\0\0");
    write_doc(dir.path(), "image.png", "not a real png");

    let store = VectorStore::new(dir.path());
    store
        .rebuild_index(dir.path())
        .expect("rebuild should succeed");

    let stats = store.get_index_stats();
    assert_eq!(stats.total_documents, 1, "only .md files should be indexed");
}

// ---------------------------------------------------------------------------
// Dedup
// ---------------------------------------------------------------------------

#[test]
fn test_vector_store_dedup() {
    let chunks = vec![
        "apple banana".to_string(),
        "banana apple".to_string(),
        "apple banana".to_string(),
    ];

    let deduped = index::deduplicate_chunks(chunks);
    // "apple banana" normalized = "applebanana", "banana apple" = "bananaapple" → different
    assert_eq!(deduped.len(), 2);
    assert_eq!(deduped[0], "apple banana");
    assert_eq!(deduped[1], "banana apple");
}

#[test]
fn test_vector_store_dedup_preserves_order() {
    let chunks = vec![
        "first".to_string(),
        "second".to_string(),
        "first".to_string(),
        "third".to_string(),
    ];

    let deduped = index::deduplicate_chunks(chunks);
    assert_eq!(deduped.len(), 3);
    assert_eq!(deduped[0], "first");
    assert_eq!(deduped[1], "second");
    assert_eq!(deduped[2], "third");
}

#[test]
fn test_vector_store_dedup_whitespace_insensitive() {
    let chunks = vec!["hello world".to_string(), "hello  world".to_string()];
    let deduped = index::deduplicate_chunks(chunks);
    assert_eq!(
        deduped.len(),
        1,
        "dedup should be insensitive to whitespace differences"
    );
}

// ---------------------------------------------------------------------------
// Empty query
// ---------------------------------------------------------------------------

#[test]
fn test_vector_store_empty_query() {
    let (_dir, store) = setup_temp_store();

    // Query before any documents are indexed
    let results = store.query("anything", "default", 10);
    assert!(results.is_empty(), "empty store should return no results");
}

#[test]
fn test_vector_store_empty_query_after_operations() {
    let (_dir, store) = setup_temp_store();

    // Index and then delete everything
    let p = PathBuf::from("temp.md");
    store.index_document(&p, "temporary").unwrap();
    store.delete_document(&p).unwrap();

    let results = store.query("temporary", "default", 5);
    assert!(results.is_empty());
}

#[test]
fn test_vector_store_query_empty_string() {
    let (_dir, store) = setup_temp_store();
    store
        .index_document(&PathBuf::from("doc.md"), "some content")
        .unwrap();

    let results = store.query("", "default", 5);
    // An empty query should not match anything (score = 0)
    assert!(results.is_empty());
}

// ---------------------------------------------------------------------------
// Index stats
// ---------------------------------------------------------------------------

#[test]
fn test_vector_store_stats() {
    let (_dir, store) = setup_temp_store();

    let empty_stats = store.get_index_stats();
    assert_eq!(empty_stats.total_documents, 0);
    assert_eq!(empty_stats.total_chunks, 0);
    assert!(empty_stats.last_indexed.is_none());

    store
        .index_document(&PathBuf::from("a.md"), "Hello world")
        .unwrap();
    store
        .index_document(&PathBuf::from("b.md"), "Foo bar baz qux")
        .unwrap();

    let stats = store.get_index_stats();
    assert_eq!(stats.total_documents, 2);
    assert!(stats.total_chunks > 0);
    assert!(stats.last_indexed.is_some());
}

// ---------------------------------------------------------------------------
// Chunking logic (public functions from index module)
// ---------------------------------------------------------------------------

#[test]
fn test_chunk_text_boundary_respect() {
    // Verify chunking doesn't split in the middle of a word when possible
    let text = "word.word word.word word.word";
    let chunks = index::chunk_text(text, 10, 2);
    for chunk in &chunks {
        // Each chunk should be at most 10 chars
        assert!(chunk.len() <= 10, "chunk '{}' exceeds max size", chunk);
    }
}

#[test]
fn test_chunk_text_exact_fit() {
    let text = "1234567890";
    let chunks = index::chunk_text(text, 10, 0);
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0], "1234567890");
}

#[test]
fn test_chunk_semantic_respects_headings() {
    let text = "## Section 1\nContent.\n## Section 2\nMore content.";
    let chunks = index::chunk_semantic(text, 100);
    assert!(chunks.len() >= 2, "should split on heading boundaries");
    assert!(chunks[0].contains("Section 1"));
}
