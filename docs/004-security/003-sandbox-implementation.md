# 003-sandbox-implementation.md
## ZarishNote Sandbox Implementation
### Wasmtime configuration, resource limits, and capability enforcement

**Document type:** Security — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Wasmtime Configuration](#1-wasmtime-configuration)
2. [Resource Limits Implementation](#2-resource-limits-implementation)
3. [Network Proxy Implementation](#3-network-proxy-implementation)
4. [Filesystem Scoping Implementation](#4-filesystem-scoping-implementation)
5. [Capability Enforcement](#5-capability-enforcement)
6. [Audit Logging](#6-audit-logging)
7. [Testing and Verification](#7-testing-and-verification)

---

## 1. Wasmtime Configuration

### 1.1 Engine Setup

```rust
use wasmtime::{Config, Engine, Module, Store, Instance};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use std::time::Duration;

pub fn create_engine() -> Engine {
    let mut config = Config::new();

    // Cranelift JIT compiler
    config.strategy(wasmtime::Strategy::Cranelift);

    // Enable WASI
    config.wasm_component_model(true);

    // Enable epoch-based interruption for timeouts
    config.epoch_interruption(true);

    // Memory limits
    config.max_wasm_stack(512 * 1024);   // 512KB stack
    config.static_memory_maximum_size(256 * 1024 * 1024); // 256MB

    Engine::new(&config).unwrap()
}
```

### 1.2 Epoch-Based Interruption

Timeouts use epoch-based interruption rather than `set_timeout`:

```rust
// On each store creation:
store.set_epoch_deadline(cfg.timeout_secs);

// In a background thread, increment epoch every second:
std::thread::spawn(|| {
    loop {
        std::thread::sleep(Duration::from_secs(1));
        engine.increment_epoch();
    }
});
```

---

## 2. Resource Limits Implementation

### 2.1 Memory Limits

```rust
use wasmtime::StoreLimitsBuilder;

let limits = StoreLimitsBuilder::new()
    .memory_size(cfg.memory_limit_bytes as usize)  // e.g., 256MB
    .table_elements(1024)                           // max table entries
    .instances(1)                                   // one instance per store
    .tables(1)                                      // one table per store
    .build();

store.limiter(|state| state);
```

### 2.2 CPU Timeout

```rust
// Configured in Wasmtime engine config
config.epoch_interruption(true);

// Per-instance deadline
store.set_epoch_deadline(cfg.timeout_secs);  // e.g., 30

// Background epoch tick
engine.increment_epoch();  // called every second
```

### 2.3 Output Size Limit

```rust
// WASM output captured via stdout
let stdout_limit = 4 * 1024 * 1024; // 4MB max output

let mut stdout = Vec::new();
let mut captured_bytes = 0;

// During execution, track output size
if captured_bytes > stdout_limit {
    kill_instance(&mut store);
    return Err(SandboxError::OutputLimitExceeded);
}
```

### 2.4 Module Size Limit

```rust
// Reject WASM modules larger than 50MB
const MAX_MODULE_SIZE: usize = 50 * 1024 * 1024;

if wasm_bytes.len() > MAX_MODULE_SIZE {
    return Err(SandboxError::ModuleTooLarge(wasm_bytes.len()));
}
```

---

## 3. Network Proxy Implementation

### 3.1 Architecture

```
WASM Module → WASI HTTP request → Rust Host → DNS resolution
    → TLS handshake → HTTP request via reqwest
    → Response filtered → Return to WASM
```

### 3.2 Domain Allow-List Check

```rust
pub fn check_network_allowed(url: &Url, cfg: &SandboxConfig) -> Result<(), SandboxError> {
    if !cfg.network_enabled {
        return Err(SandboxError::NetworkDisabled);
    }

    let host = url.host_str()
        .ok_or(SandboxError::InvalidUrl)?;

    let allowed = cfg.allowed_domains.iter().any(|pattern| {
        if pattern.starts_with("*.") {
            let suffix = &pattern[1..]; // .example.com
            host.ends_with(suffix)
        } else {
            host == pattern.as_str()
        }
    });

    if allowed {
        Ok(())
    } else {
        Err(SandboxError::DomainNotAllowed(host.to_string()))
    }
}
```

### 3.3 TLS Enforcement

All remote HTTP requests require TLS. Self-signed certificates require explicit opt-in via `.znrc`:

```yaml
sandbox:
  network:
    allow_self_signed_certs: false
```

---

## 4. Filesystem Scoping Implementation

### 4.1 Virtual Path Resolution

```rust
pub fn resolve_path(
    virtual_path: &str,
    vault_root: &Path,
    permissions: &[String],
) -> Result<PathBuf, SandboxError> {
    let path_str = virtual_path
        .strip_prefix("workspace://")
        .ok_or(SandboxError::InvalidVirtualPath)?;

    // Reject path traversal
    let canonical = std::fs::canonicalize(vault_root.join(path_str))?;
    let vault_canonical = std::fs::canonicalize(vault_root)?;

    if !canonical.starts_with(&vault_canonical) {
        return Err(SandboxError::PathTraversalDetected);
    }

    // Check permission
    let relative = canonical.strip_prefix(&vault_canonical).unwrap();
    let allowed = permissions.iter().any(|perm| {
        let glob_pattern = perm.strip_prefix("read:workspace:").or_else(|| {
            perm.strip_prefix("write:workspace:")
        }).unwrap_or("");
        glob_match::glob_match(glob_pattern, relative.to_str().unwrap_or(""))
    });

    if allowed { Ok(canonical) }
    else { Err(SandboxError::PermissionDenied) }
}
```

### 4.2 WASI Preopened Directories

```rust
use cap_std::fs::Dir;
use wasmtime_wasi::ambient_authority;

let vault_dir = Dir::open_ambient_dir(&cfg.vault_root, ambient_authority())?;

let wasi = WasiCtxBuilder::new()
    .preopened_dir(vault_dir, "workspace")?
    .build();
```

---

## 5. Capability Enforcement

Capabilities are checked at the host function level. Each host function (HTTP request, file read, file write) checks the tool's declared permissions before executing.

```rust
pub struct CapabilityChecker {
    tool_config: ToolConfig,
}

impl CapabilityChecker {
    pub fn check(&self, capability: &str) -> Result<(), CapabilityError> {
        if self.tool_config.sandbox.permissions.contains(&capability.to_string()) {
            Ok(())
        } else {
            Err(CapabilityError::MissingCapability(capability.to_string()))
        }
    }
}
```

---

## 6. Audit Logging

Every sandbox execution is logged:

```rust
#[derive(Serialize)]
struct SandboxAuditEntry {
    timestamp: String,          // ISO 8601
    tool_name: String,
    action: String,             // "execute", "network_request", "file_read"
    target: String,             // URL, file path, function name
    duration_ms: u64,
    exit_code: i32,
    allowed: bool,
    error: Option<String>,
}
```

Audit log format: JSONL, appended to `.znrc-audit.log` in vault root.

---

## 7. Testing and Verification

### 7.1 Test Cases

| Test | What it verifies |
|---|---|
| Path traversal rejection | `workspace://../../etc/passwd` blocked |
| Domain allow-list | `network:example.com` blocks `evil.com` |
| Memory limit enforcement | WASM allocation over limit → killed |
| Timeout enforcement | WASM infinite loop → killed after N seconds |
| Cross-tool isolation | State from tool A not visible to tool B |
| Stack overflow protection | Deep recursion → trapped |
| Module size limit | WASM >50MB → rejected on load |

### 7.2 Security Regression Testing

- All sandbox tests run before every release
- Test suite includes known WASM escape payloads
- Wasmtime version pinned and reviewed for CVEs before update

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
