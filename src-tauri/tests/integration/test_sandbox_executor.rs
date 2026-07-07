//! Integration tests for the sandbox subsystem.
//!
//! Tests the public APIs of [`zs_note_lib::sandbox::capability`] and
//! [`zs_note_lib::sandbox::network`]: capability checking, path access
//! control, glob matching and network proxy policy.

use std::path::Path;

use zs_note_lib::sandbox::capability::{glob_match, resolve_virtual_path, CapabilityChecker};
use zs_note_lib::sandbox::network::SandboxNetworkProxy;

// ---------------------------------------------------------------------------
// Capability check
// ---------------------------------------------------------------------------

#[test]
fn test_capability_check_basic() {
    let checker = CapabilityChecker::new().with_permissions(vec!["read".into(), "write".into()]);

    assert!(checker.check("read", "file").is_ok());
    assert!(checker.check("write", "file").is_ok());
}

#[test]
fn test_capability_check_denied() {
    let checker = CapabilityChecker::new().with_permissions(vec!["read".into()]);

    let result = checker.check("delete", "file");
    assert!(result.is_err(), "delete should not be permitted");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("delete"),
        "error should mention the missing permission"
    );
}

#[test]
fn test_capability_check_wildcard() {
    // When no permissions are set, the `*` wildcard is always allowed
    let checker = CapabilityChecker::new();
    assert!(
        checker.check("*", "anything").is_ok(),
        "wildcard should always pass"
    );
}

#[test]
fn test_capability_check_empty_permissions() {
    let checker = CapabilityChecker::new().with_permissions(vec![]);
    assert!(checker.check("read", "file").is_err());
}

// ---------------------------------------------------------------------------
// Capability – network
// ---------------------------------------------------------------------------

#[test]
fn test_capability_network_allowed() {
    let checker = CapabilityChecker::new()
        .with_domains(vec!["*.example.com".into(), "api.trusted.org".into()]);

    assert!(
        checker
            .check_network("https://api.example.com/data")
            .is_ok(),
        "subdomain of *.example.com should be allowed"
    );
    assert!(
        checker.check_network("https://api.trusted.org/v1").is_ok(),
        "exact domain should be allowed"
    );
}

#[test]
fn test_capability_network_blocked() {
    let checker = CapabilityChecker::new().with_domains(vec!["*.example.com".into()]);

    let result = checker.check_network("https://evil.com/pwn");
    assert!(result.is_err(), "evil.com should be blocked");
}

#[test]
fn test_capability_network_no_domains() {
    let checker = CapabilityChecker::new(); // empty domains
    let result = checker.check_network("https://anywhere.com");
    assert!(
        result.is_err(),
        "network should be disabled when no domains are configured"
    );
}

#[test]
fn test_capability_network_invalid_url() {
    let checker = CapabilityChecker::new().with_domains(vec!["*".into()]);
    let result = checker.check_network("not-a-url");
    assert!(result.is_err(), "invalid URL should fail");
}

// ---------------------------------------------------------------------------
// Capability – path access
// ---------------------------------------------------------------------------

#[test]
fn test_capability_path_access() {
    let dir = tempfile::tempdir().expect("temp dir");
    let vault_root = dir.path();
    std::fs::write(vault_root.join("note.md"), "hello").ok();

    let checker = CapabilityChecker::new().with_paths(vec![vault_root.to_path_buf()]);

    let result = checker.check_path_access(Path::new("note.md"), vault_root);
    assert!(
        result.is_ok(),
        "read access to file inside vault should be allowed"
    );
}

#[test]
fn test_capability_path_access_traversal_rejected() {
    let dir = tempfile::tempdir().expect("temp dir");
    let vault_root = dir.path();

    // Don't set allowed_paths, so any path inside vault is fine
    let checker = CapabilityChecker::new().with_paths(vec![vault_root.to_path_buf()]);

    // ../etc/passwd should be rejected
    let result = checker.check_path_access(Path::new("../etc/passwd"), vault_root);
    assert!(
        result.is_err(),
        "path traversal outside vault should be rejected"
    );
}

// ---------------------------------------------------------------------------
// Glob match (re‑test the public function)
// ---------------------------------------------------------------------------

#[test]
fn test_glob_match_exact() {
    assert!(glob_match("example.com", "example.com"));
}

#[test]
fn test_glob_match_subdomain_wildcard() {
    assert!(glob_match("*.example.com", "api.example.com"));
    assert!(glob_match("*.example.com", "sub.api.example.com"));
}

#[test]
fn test_glob_match_double_wildcard() {
    assert!(glob_match("**.example.com", "deep.sub.example.com"));
    assert!(glob_match("**.example.com", "example.com"));
}

#[test]
fn test_glob_match_wildcard_all() {
    assert!(glob_match("*", "anything.whatever.com"));
    assert!(glob_match("*.*", "foo.bar"));
}

#[test]
fn test_glob_match_no_match() {
    assert!(!glob_match("*.example.com", "other.com"));
    assert!(!glob_match("example.com", "notexample.com"));
}

#[test]
fn test_glob_match_case_insensitive() {
    assert!(glob_match("EXAMPLE.COM", "example.com"));
    assert!(glob_match("example.com", "EXAMPLE.COM"));
}

#[test]
fn test_glob_match_empty_pattern() {
    assert!(!glob_match("", "example.com"));
}

// ---------------------------------------------------------------------------
// Network proxy – allowed
// ---------------------------------------------------------------------------

#[test]
fn test_network_proxy_allowed_check() {
    let proxy = SandboxNetworkProxy::new(vec!["*.example.com".into()]);

    assert!(
        proxy
            .check_network_allowed("https://api.example.com")
            .is_ok(),
        "api.example.com should be allowed"
    );
    assert!(
        proxy
            .check_network_allowed("https://sub.api.example.com")
            .is_ok(),
        "deep subdomain should be allowed"
    );
}

#[test]
fn test_network_proxy_allowed_exact_domain() {
    let proxy = SandboxNetworkProxy::new(vec!["trusted.org".into()]);
    assert!(
        proxy
            .check_network_allowed("https://trusted.org/path")
            .is_ok(),
        "exact domain match should be allowed"
    );
}

#[test]
fn test_network_proxy_allowed_wildcard() {
    let proxy = SandboxNetworkProxy::new(vec!["*".into()]);
    assert!(
        proxy
            .check_network_allowed("https://absolutely-anything.com")
            .is_ok(),
        "wildcard * should allow everything"
    );
}

// ---------------------------------------------------------------------------
// Network proxy – blocked
// ---------------------------------------------------------------------------

#[test]
fn test_network_proxy_blocked_domain() {
    let proxy = SandboxNetworkProxy::new(vec!["*.example.com".into()]);

    let result = proxy.check_network_allowed("https://malicious.net");
    assert!(result.is_err(), "malicious.net should be blocked");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("malicious"),
        "error should mention the blocked host"
    );
}

#[test]
fn test_network_proxy_blocked_empty_allowlist() {
    let proxy = SandboxNetworkProxy::new(vec![]);
    let result = proxy.check_network_allowed("https://anywhere.com");
    assert!(
        result.is_err(),
        "empty allowlist should block all network access"
    );
}

#[test]
fn test_network_proxy_blocked_no_match() {
    let proxy = SandboxNetworkProxy::new(vec!["specific.com".into()]);
    let result = proxy.check_network_allowed("https://other.com");
    assert!(result.is_err(), "non‑matching domain should be blocked");
}

#[test]
fn test_network_proxy_blocked_invalid_url() {
    let proxy = SandboxNetworkProxy::new(vec!["*".into()]);
    let result = proxy.check_network_allowed("not-a-valid-url");
    assert!(result.is_err(), "invalid URL should fail");
}

// ---------------------------------------------------------------------------
// Allowed domains accessor
// ---------------------------------------------------------------------------

#[test]
fn test_network_proxy_allowed_domains() {
    let proxy = SandboxNetworkProxy::new(vec!["a.com".into(), "b.org".into()]);
    let domains = proxy.allowed_domains();
    assert_eq!(domains.len(), 2);
    assert!(domains.contains(&"a.com".to_string()));
}

// ---------------------------------------------------------------------------
// Resolve virtual path
// ---------------------------------------------------------------------------

#[test]
fn test_resolve_virtual_path_valid() {
    let dir = tempfile::tempdir().expect("temp dir");
    let p =
        resolve_virtual_path(Path::new("note.md"), dir.path()).expect("valid path should resolve");
    assert!(p.starts_with(dir.path()));
}

#[test]
fn test_resolve_virtual_path_traversal() {
    let dir = tempfile::tempdir().expect("temp dir");
    let result = resolve_virtual_path(Path::new("../escape"), dir.path());
    assert!(result.is_err(), "traversal should be rejected");
}
