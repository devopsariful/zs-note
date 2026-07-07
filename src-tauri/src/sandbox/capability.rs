use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

pub struct CapabilityChecker {
    allowed_domains: Vec<String>,
    allowed_paths: Vec<PathBuf>,
    enabled_permissions: Vec<String>,
}

impl CapabilityChecker {
    pub fn new() -> Self {
        Self {
            allowed_domains: Vec::new(),
            allowed_paths: Vec::new(),
            enabled_permissions: Vec::new(),
        }
    }

    pub fn with_domains(mut self, domains: Vec<String>) -> Self {
        self.allowed_domains = domains;
        self
    }

    pub fn with_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.allowed_paths = paths;
        self
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.enabled_permissions = permissions;
        self
    }

    pub fn check(&self, permission: &str, target: &str) -> Result<()> {
        if !self.enabled_permissions.contains(&permission.to_string()) {
            if permission == "*" {
                return Ok(());
            }
            bail!(
                "Permission '{}' is not granted. Required: {}",
                permission,
                target
            );
        }
        Ok(())
    }

    pub fn check_path_access(&self, requested_path: &Path, vault_root: &Path) -> Result<PathBuf> {
        let resolved = resolve_virtual_path(requested_path, vault_root)?;

        if !self.allowed_paths.is_empty() {
            let canonical = resolved.canonicalize()?;
            let allowed = self.allowed_paths.iter().any(|p| {
                p.canonicalize()
                    .map(|cp| canonical.starts_with(&cp))
                    .unwrap_or(false)
            });
            if !allowed {
                bail!(
                    "Access to path '{}' is not allowed by sandbox policy",
                    resolved.display()
                );
            }
        }

        Ok(resolved)
    }

    pub fn check_network(&self, url: &str) -> Result<()> {
        if self.allowed_domains.is_empty() {
            bail!("Network access is disabled in sandbox config");
        }

        let url_parsed = url::Url::parse(url).context("Invalid URL")?;
        let host = url_parsed.host_str().unwrap_or("");

        for pattern in &self.allowed_domains {
            if glob_match(pattern, host) {
                return Ok(());
            }
        }

        bail!(
            "Network access to '{}' is not allowed (allowed: {:?})",
            host,
            self.allowed_domains
        );
    }
}

impl Default for CapabilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

pub fn resolve_virtual_path(requested: &Path, vault_root: &Path) -> Result<PathBuf> {
    let requested_str = requested.to_string_lossy().replace('\\', "/");
    let cleaned = requested_str.trim_start_matches('/');
    let resolved = vault_root.join(cleaned);

    let vault_canonical = vault_root
        .canonicalize()
        .context("Vault root does not exist")?;

    let normalized = resolved
        .canonicalize()
        .unwrap_or_else(|_| normalize_path(&resolved));

    if !normalized.starts_with(&vault_canonical) {
        bail!(
            "Path traversal detected: {:?} resolves outside vault {:?}",
            requested,
            vault_root
        );
    }

    Ok(normalized)
}

fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                components.pop();
            }
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => {
                components.push(component.as_os_str());
            }
            _ => {}
        }
    }
    if components.is_empty() {
        path.to_path_buf()
    } else {
        components.iter().collect()
    }
}

pub fn glob_match(pattern: &str, host: &str) -> bool {
    if pattern == "*" || pattern == "*.*" {
        return true;
    }

    let pattern_lower = pattern.to_lowercase();
    let host_lower = host.to_lowercase();

    if pattern_lower == host_lower {
        return true;
    }

    if let Some(rest) = pattern_lower.strip_prefix("*.") {
        return host_lower.ends_with(&format!(".{}", rest)) || host_lower == rest;
    }

    if let Some(rest) = pattern_lower.strip_prefix("**.") {
        return host_lower.ends_with(&format!(".{}", rest)) || host_lower == rest;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("example.com", "example.com"));
    }

    #[test]
    fn test_glob_match_subdomain() {
        assert!(glob_match("*.example.com", "api.example.com"));
        assert!(glob_match("*.example.com", "sub.api.example.com"));
    }

    #[test]
    fn test_glob_match_wildcard_all() {
        assert!(glob_match("*", "anything.com"));
    }

    #[test]
    fn test_glob_no_match() {
        assert!(!glob_match("*.example.com", "other.com"));
    }
}
