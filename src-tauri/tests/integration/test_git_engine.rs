//! Integration tests for the Git engine subsystem.
//!
//! Tests the public API of [`zs_note_lib::git::commit`] for commit‑message
//! generation and change‑type detection.  Full repository operations
//! (init / commit / history) require a real git repo on disk and are
//! exercised in unit tests inside ``src/git/*.rs``.

use std::path::Path;

use zs_note_lib::git::commit::generate_commit_message;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn p(s: &str) -> &Path {
    Path::new(s)
}

// ---------------------------------------------------------------------------
// Auto commit message generation – conventional style
// ---------------------------------------------------------------------------

#[test]
fn test_auto_commit_message_generation_conventional() {
    let msg = generate_commit_message(&[p("notes/meeting.md")], "conventional");

    assert!(
        msg.starts_with("docs(notes):"),
        "conventional message should start with 'docs(notes):', got: {}",
        msg
    );
    assert!(
        msg.contains("meeting"),
        "message should contain the file stem 'meeting'"
    );
}

#[test]
fn test_auto_commit_message_generation_conventional_multiple() {
    let msg = generate_commit_message(&[p("src/main.rs"), p("src/lib.rs")], "conventional");

    assert!(
        msg.starts_with("feat(src):"),
        "rs files should produce 'feat(src):', got: {}",
        msg
    );
    assert!(msg.contains("main") || msg.contains("lib"));
}

#[test]
fn test_auto_commit_message_generation_conventional_config() {
    let msg = generate_commit_message(&[p(".znrc")], "conventional");
    assert!(
        msg.starts_with("config("),
        "config files should produce 'config(...):'"
    );
}

// ---------------------------------------------------------------------------
// Generate commit message – every style
// ---------------------------------------------------------------------------

#[test]
fn test_generate_commit_message_simple() {
    let msg = generate_commit_message(&[p("readme.md")], "simple");
    assert_eq!(msg, "Update readme.md");
}

#[test]
fn test_generate_commit_message_simple_multiple() {
    let msg = generate_commit_message(&[p("a.md"), p("b.txt")], "simple");
    assert!(msg.starts_with("Update"));
    assert!(msg.contains("a.md"));
    assert!(msg.contains("b.txt"));
}

#[test]
fn test_generate_commit_message_descriptive() {
    let msg = generate_commit_message(&[p("foo.md")], "descriptive");
    assert!(msg.starts_with("Auto-save from ZarishNote on "));
    // Should contain a date like 2026-06-08
    assert!(msg.len() > 30);
}

#[test]
fn test_generate_commit_message_unknown_style_fallback() {
    let msg = generate_commit_message(&[p("a.md"), p("b.md")], "unknown_style");
    assert_eq!(msg, "Update 2 files");
}

#[test]
fn test_generate_commit_message_empty_paths() {
    let msg = generate_commit_message(&[], "conventional");
    // With no paths, it should produce some fallback
    assert!(!msg.is_empty());
}

#[test]
fn test_generate_commit_message_no_extension() {
    let msg = generate_commit_message(&[p("Makefile")], "conventional");
    assert!(
        msg.contains("chore("),
        "no‑extension files should be 'chore'"
    );
}

// ---------------------------------------------------------------------------
// Detect change types
// ---------------------------------------------------------------------------

#[test]
fn test_detect_change_types_markdown() {
    // We can't call `detect_change_types` directly because it's private.
    // Instead, verify its effect through generated commit messages.

    let msg = generate_commit_message(&[p("notes/meeting.md")], "conventional");
    assert!(msg.starts_with("docs("), "markdown → docs type");
}

#[test]
fn test_detect_change_types_rust_source() {
    let msg = generate_commit_message(&[p("src/main.rs")], "conventional");
    assert!(msg.starts_with("feat("), "rust source → feat type");
}

#[test]
fn test_detect_change_types_typescript_source() {
    let msg = generate_commit_message(&[p("components/app.ts")], "conventional");
    assert!(msg.starts_with("feat("), "typescript source → feat type");
}

#[test]
fn test_detect_change_types_json_config() {
    let msg = generate_commit_message(&[p("tsconfig.json")], "conventional");
    assert!(msg.starts_with("chore("), "json → chore type");
}

#[test]
fn test_detect_change_types_yaml_config() {
    let msg = generate_commit_message(&[p("deploy.yaml")], "conventional");
    assert!(msg.starts_with("chore("), "yaml → chore type");
}

#[test]
fn test_detect_change_types_mixed() {
    // Multiple file types should use the "highest priority" type
    let msg = generate_commit_message(&[p("main.rs"), p("notes.md")], "conventional");
    // One is feat, one is docs → both appear, but the first one in sorted order wins
    assert!(msg.starts_with("docs(") || msg.starts_with("feat("));
}
