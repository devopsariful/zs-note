# 001-sandbox-spec.md
## ZarishNote Sandbox Engine Specification
### Wasmtime-backed isolation for all AI tools, plugins, and MCP servers

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

- [001-sandbox-spec.md](#001-sandbox-specmd)
  - [ZarishNote Sandbox Engine Specification](#zarishnote-sandbox-engine-specification)
    - [Wasmtime-backed isolation for all AI tools, plugins, and MCP servers](#wasmtime-backed-isolation-for-all-ai-tools-plugins-and-mcp-servers)
  - [Table of Contents](#table-of-contents)
  - [1. Why a Sandbox](#1-why-a-sandbox)
  - [2. Architecture](#2-architecture)
  - [3. Capability Model](#3-capability-model)
    - [3.1 Filesystem Capabilities](#31-filesystem-capabilities)
    - [3.2 Network Capabilities](#32-network-capabilities)
    - [3.3 System Capabilities](#33-system-capabilities)
    - [3.4 Capability Declaration Example (in `.znrc`)](#34-capability-declaration-example-in-znrc)
  - [4. Full-Stack Sandbox Features](#4-full-stack-sandbox-features)
    - [4.1 HTTP (Sandboxed)](#41-http-sandboxed)
    - [4.2 Filesystem Access (Sandboxed)](#42-filesystem-access-sandboxed)
    - [4.3 State (Sandboxed)](#43-state-sandboxed)
    - [4.4 Tool Calling (AI → Sandbox → Result)](#44-tool-calling-ai--sandbox--result)
  - [5. Resource Limits](#5-resource-limits)
  - [6. Network Policy](#6-network-policy)
  - [7. Filesystem Scope](#7-filesystem-scope)
  - [8. Sandbox Execution Lifecycle](#8-sandbox-execution-lifecycle)
  - [9. Interoperability Across OS](#9-interoperability-across-os)
  - [10. Rust Implementation Sketch](#10-rust-implementation-sketch)

---

## 1. Why a Sandbox

Without a sandbox, an AI tool or plugin can:
- Read any file on your system (including SSH keys, `.env`, passwords)
- Make outbound network requests to arbitrary servers
- Execute arbitrary shell commands
- Consume all available memory/CPU

ZarishNote's sandbox makes these attacks **technically impossible**, not just policy-prohibited.

**Design axiom:** Every piece of code that is not part of ZarishNote's compiled Rust core runs inside Wasmtime.

---

## 2. Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    ZarishNote (Tauri v2)                      │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │              Rust Backend (Sandbox Manager)            │  │
│  │                                                        │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐  │  │
│  │  │ Tool Executor │  │ MCP Gateway  │  │ Plugin Host │  │  │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬──────┘  │  │
│  │         │                 │                  │         │  │
│  │  ┌──────▼─────────────────▼──────────────────▼──────┐  │  │
│  │  │            Wasmtime Sandbox Engine               │  │  │
│  │  │                                                  │  │  │
│  │  │  ┌──────────┐  ┌──────────┐  ┌──────────────┐   │  │  │
│  │  │  │ WASM     │  │ WASM     │  │ WASM Plugin  │   │  │  │
│  │  │  │ Tool A   │  │ Tool B   │  │ Module       │   │  │  │
│  │  │  │          │  │          │  │              │   │  │  │
│  │  │  │ 256MB    │  │ 128MB    │  │ 64MB         │   │  │  │
│  │  │  │ 30s      │  │ 15s      │  │ 10s          │   │  │  │
│  │  │  │ net:off  │  │ net:on   │  │ net:off      │   │  │  │
│  │  │  └──────────┘  └──────────┘  └──────────────┘   │  │  │
│  │  │                                                  │  │  │
│  │  │           Capability Firewall                    │  │  │
│  │  │  (rejects any call not declared in .znrc)        │  │  │
│  │  └──────────────────────────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

## 3. Capability Model

Every tool or plugin declares its required capabilities in `.znrc`. The sandbox enforces these at runtime — no capability declared = no access granted.

### 3.1 Filesystem Capabilities

| Capability | What it allows |
|---|---|
| `read:workspace` | Read any file under vault root |
| `read:workspace:docs/**` | Read only files matching a glob pattern |
| `write:workspace` | Write to vault root (scoped) |
| `write:workspace:inbox/**` | Write to specific subfolder |
| `read:stdout` | Read from standard output of subprocesses |
| `write:stdout` | Write to standard output (captured by host) |

Absolute paths are **always rejected**. Path traversal (`../../`) is **always rejected**.

### 3.2 Network Capabilities

| Capability | What it allows |
|---|---|
| `network:false` | No network (default) |
| `network:true` | Network allowed to domains in `.znrc`'s `allowed_outbound` list |
| `network:github.com` | Specific domain override for this tool only |
| `network:localhost:*` | Access to local services |

### 3.3 System Capabilities

| Capability | What it allows |
|---|---|
| `env:none` | No environment variables (default) |
| `env:allow:VAR_NAME` | Access to a specific env var |
| `process:none` | No subprocess spawning (default) |
| `keychain:key-id` | Access to one specific OS Keychain entry |

### 3.4 Capability Declaration Example (in `.znrc`)

```yaml
tools:
  - name: "my-tool"
    type: "wasm"
    wasm_path: "tools/my-tool.wasm"
    sandbox:
      permissions:
        - "read:workspace:docs/**"
        - "write:workspace:output/**"
        - "network:api.example.com"
```

---

## 4. Full-Stack Sandbox Features

ZarishNote's sandbox is not just isolation — it is a complete execution environment:

### 4.1 HTTP (Sandboxed)

Tools can make HTTP requests via a host-provided HTTP interface:
- Request goes through ZarishNote's Rust backend
- Backend checks domain against `allowed_outbound` list
- Blocked requests return an error to the WASM module
- All requests are logged to `.znrc-audit.log`

### 4.2 Filesystem Access (Sandboxed)

Tools see a **virtual filesystem** that maps to a scoped path on disk:
- Tool requests path `workspace://docs/report.md`
- ZarishNote resolves this to the actual path: `/home/ariful/vault/docs/report.md`
- Access outside the declared scope returns `permission denied`

### 4.3 State (Sandboxed)

Tools can maintain state between calls within a session:
- Ephemeral state: cleared when the tool session ends
- Persistent state: written to a scoped `.znrc-state/{tool-name}/` folder
- State is isolated per tool — no cross-tool state access

### 4.4 Tool Calling (AI → Sandbox → Result)

When the AI requests a tool call:
1. ZarishNote validates the tool call against `.znrc`
2. Spawns a Wasmtime instance for the tool
3. Passes arguments as JSON
4. Collects stdout/return value
5. Returns to AI as tool result
6. Logs execution to audit log

---

## 5. Resource Limits

Set per-tool in `.znrc`, with global defaults as fallback:

| Limit | Default | Configurable per tool |
|---|---|---|
| Memory | 256MB | Yes |
| CPU time | 30s | Yes |
| Stack depth | 64 frames | No (fixed) |
| WASM module size | 50MB | No (fixed) |
| Output size | 4MB | Yes |

Exceeding any limit causes the sandbox to kill the WASM instance and return an error.

---

## 6. Network Policy

Default: **all network blocked**.

Tools must declare specific domains. The host (Rust backend) acts as a network proxy:

```
WASM Module → [network:api.github.com request] → Sandbox Manager
→ [check against .znrc allowed_outbound]
→ [if allowed] → Rust reqwest → api.github.com
→ [response] → WASM Module
```

TLS is always used for remote connections. Self-signed certs require explicit opt-in.

---

## 7. Filesystem Scope

The sandbox maps virtual paths to real paths:

| Virtual path (tool sees) | Real path (host resolves) |
|---|---|
| `workspace://` | `/path/to/vault/` |
| `workspace://docs/` | `/path/to/vault/docs/` |
| `workspace://output/` | `/path/to/vault/output/` |
| `system://temp/` | OS temp dir (scoped) |

**No access to:**
- Any path outside vault root (unless explicitly in capability list)
- System files (`/etc/`, `C:\Windows\`, etc.)
- Other users' files
- SSH keys, `.env` files, browser profiles

---

## 8. Sandbox Execution Lifecycle

```
1. Tool invocation request arrives (from AI or user)
2. Validate: tool is declared in .znrc? → else error
3. Validate: caller has permission to invoke? → else error
4. Load WASM binary from declared path
5. Verify SHA256 checksum (if signed plugin) → else warn
6. Configure Wasmtime engine:
   - Set memory limit
   - Set CPU timeout
   - Mount virtual filesystem
   - Wire HTTP proxy (if network: true)
7. Execute WASM entry point with JSON args
8. Capture stdout, return value, errors
9. Log execution: {tool, timestamp, duration, exit_code} → audit.log
10. Return result to caller
11. Destroy Wasmtime instance (no state leak between calls)
```

---

## 9. Interoperability Across OS

Wasmtime is the same engine on all platforms. The WASM binary is **identical** across Windows, macOS, Linux, Android, and iOS.

| Platform | Wasmtime version | Notes |
|---|---|---|---|
| Windows x64 | 45.x | Native |
| macOS arm64 | 45.x | Native (Apple Silicon) |
| macOS x64 | 45.x | Native (Intel) |
| Linux x64 | 45.x | Native |
| Linux arm64 | 45.x | Native (Raspberry Pi 4+) |
| Android arm64 | 45.x | Via Tauri v2 mobile |
| iOS arm64 | 45.x | Via Tauri v2 mobile |

Tools and plugins compiled to `wasm32-wasi` run identically on all platforms.

---

## 10. Rust Implementation Sketch

```rust
// src-tauri/src/sandbox.rs

use wasmtime::{Engine, Store, Module, Instance, Config};
use wasmtime_wasi::{WasiCtxBuilder, WasiCtx};

pub struct SandboxConfig {
    pub memory_limit_bytes: u64,   // e.g., 256 * 1024 * 1024
    pub timeout_secs: u64,
    pub allowed_domains: Vec<String>,
    pub vault_root: PathBuf,
    pub permissions: Vec<String>,
}

pub struct SandboxEngine {
    engine: Engine,
}

impl SandboxEngine {
    pub fn new() -> Result<Self, anyhow::Error> {
        let mut config = Config::new();
        config.strategy(wasmtime::Strategy::Cranelift);
        config.wasm_component_model(true);
        Ok(Self { engine: Engine::new(&config)? })
    }

    pub async fn execute(
        &self,
        wasm_bytes: &[u8],
        func_name: &str,
        args_json: &str,
        cfg: &SandboxConfig,
    ) -> Result<String, SandboxError> {
        let module = Module::new(&self.engine, wasm_bytes)?;

        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()   // captured by host
            .preopened_dir(
                Dir::open_ambient_dir(&cfg.vault_root, ambient_authority())?,
                "workspace",
            )?
            .build();

        // Resource limits via StoreLimitsBuilder
        let limits = StoreLimitsBuilder::new()
            .memory_size(cfg.memory_limit_bytes as usize)
            .build();

        let mut store = Store::new(&self.engine, wasi);
        store.limiter(|s| s);

        // CPU timeout via epoch interruption
        store.set_epoch_deadline(cfg.timeout_secs);

        let instance = Instance::new(&mut store, &module, &[])?;

        let func = instance
            .get_func(&mut store, func_name)
            .ok_or(SandboxError::FunctionNotFound(func_name.to_string()))?;

        // Execute and capture result
        let mut results = vec![Val::I32(0)];
        func.call(&mut store, &[Val::I32(0)], &mut results)?;

        // TODO: read output from shared memory / stdout capture
        Ok("{}".to_string())
    }
}
```

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*