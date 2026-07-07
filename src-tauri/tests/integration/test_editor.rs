//! Integration tests for vault path resolution and path‑traversal prevention.
//!
//! The editor module has a private `resolve_vault_path` helper.  The same
//! logic is exposed publicly via [`zs_note_lib::sandbox::capability::resolve_virtual_path`].
//! We test that public API here to cover path‑traversal prevention.

use std::fs;
use std::path::Path;

use zs_note_lib::sandbox::capability::resolve_virtual_path;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temporary vault directory and return its path.
fn setup_vault() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp vault");
    // Make sure the directory exists on disk (canonicalize needs it)
    let _ = fs::create_dir_all(dir.path());
    dir
}

// ---------------------------------------------------------------------------
// Valid paths
// ---------------------------------------------------------------------------

#[test]
fn test_resolve_vault_path_valid() {
    let vault = setup_vault();

    // A file in the vault root
    let resolved = resolve_virtual_path(Path::new("note.md"), vault.path())
        .expect("valid path should resolve");
    assert!(
        resolved.starts_with(vault.path()),
        "resolved path must be inside vault"
    );

    // A nested path
    let nested = resolve_virtual_path(Path::new("subdir/note.md"), vault.path())
        .expect("nested path should resolve");
    assert!(nested.starts_with(vault.path()));
}

#[test]
fn test_resolve_vault_path_absolute_like() {
    let vault = setup_vault();

    // Paths that start with `/` are treated as relative inside the vault
    let resolved = resolve_virtual_path(Path::new("/note.md"), vault.path())
        .expect("leading slash should be stripped");
    assert!(resolved.starts_with(vault.path()));
}

#[test]
fn test_resolve_vault_path_deeply_nested() {
    let vault = setup_vault();
    let resolved = resolve_virtual_path(Path::new("a/very/deep/nested/path/file.md"), vault.path())
        .expect("deeply nested path should resolve");
    assert!(resolved.starts_with(vault.path()));
    assert!(resolved.to_string_lossy().contains("a/very/deep/nested"));
}

// ---------------------------------------------------------------------------
// Path traversal – rejected
// ---------------------------------------------------------------------------

#[test]
fn test_resolve_vault_path_traversal() {
    let vault = setup_vault();

    // `../` should be rejected because it would escape the vault
    let result = resolve_virtual_path(Path::new("../etc/passwd"), vault.path());
    assert!(
        result.is_err(),
        "path traversal with ../ should be rejected"
    );
}

#[test]
fn test_resolve_vault_path_traversal_nested() {
    let vault = setup_vault();
    let result = resolve_virtual_path(Path::new("subdir/../../etc/passwd"), vault.path());
    assert!(result.is_err(), "nested path traversal should be rejected");
}

#[test]
fn test_resolve_vault_path_traversal_encoded() {
    let vault = setup_vault();

    // Even with extra segments, traversal must be blocked
    let result = resolve_virtual_path(Path::new("valid/../../outside"), vault.path());
    assert!(result.is_err(), "encoded traversal should be rejected");
}

// ---------------------------------------------------------------------------
// Outside vault
// ---------------------------------------------------------------------------

#[test]
fn test_resolve_vault_path_outside() {
    let vault = setup_vault();

    // An absolute path that points completely outside the vault should be
    // resolved relative to the vault root (leading `/` is stripped) and
    // therefore still be inside the vault – unless it contains `..`.
    let result = resolve_virtual_path(Path::new("/tmp/foo"), vault.path());
    // Since `/tmp/foo` is treated as `tmp/foo` relative to vault, it should
    // resolve inside (no traversal segments), so it succeeds.
    assert!(
        result.is_ok(),
        "absolute‑like path without .. resolves inside vault"
    );

    // Now truly outside: use `..` to escape the vault root.
    let result = resolve_virtual_path(Path::new("../outside_vault"), vault.path());
    assert!(
        result.is_err(),
        "explicit traversal outside vault must be rejected"
    );
}

#[test]
fn test_resolve_vault_path_symlink_outside() {
    // Symlink to a path outside the vault should be caught by canonicalization
    let vault = setup_vault();
    let outside = vault.path().join("outside_link");
    // Create a symlink that points outside if we can
    #[cfg(unix)]
    {
        let target = Path::new("/tmp");
        std::os::unix::fs::symlink(target, &outside).ok();
        if outside.exists() {
            let result = resolve_virtual_path(Path::new("outside_link"), vault.path());
            // Depending on whether the symlink target canonicalizes outside vault
            if outside.canonicalize().ok().map_or(false, |c| {
                c.starts_with(&vault.path().canonicalize().unwrap())
            }) {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
            }
            let _ = fs::remove_file(&outside);
        }
    }
}
