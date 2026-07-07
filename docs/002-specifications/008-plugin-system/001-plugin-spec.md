# 001-plugin-spec.md
## ZarishNote Plugin System Specification
### WASM plugin API, marketplace, and signing

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Plugin Model](#1-plugin-model)
2. [WASM Plugin API](#2-wasm-plugin-api)
3. [Plugin Manifest](#3-plugin-manifest)
4. [Plugin Lifecycle](#4-plugin-lifecycle)
5. [Plugin Sandboxing](#5-plugin-sandboxing)
6. [Plugin Marketplace](#6-plugin-marketplace)
7. [Plugin Signing and Verification](#7-plugin-signing-and-verification)
8. [GUI: Plugin Manager](#8-gui-plugin-manager)
9. [Plugin Development Guide (Sketch)](#9-plugin-development-guide-sketch)

---

## 1. Plugin Model

ZarishNote plugins are **WASM modules** compiled to `wasm32-wasi` that run inside the Wasmtime sandbox. They extend the editor with custom functionality.

### 1.1 What Plugins Can Do

| Capability | V1 | Phase 2 |
|---|---|---|
| Add toolbar buttons | ✅ | ✅ |
| Modify editor content | ✅ | ✅ |
| Register AI tools | ✅ | ✅ |
| Register keyboard shortcuts | ✅ | ✅ |
| Custom UI panels | ❌ | ✅ |
| Custom renderers | ❌ | ✅ |

### 1.2 Plugin Types

| Type | Description | Example |
|---|---|---|
| `tool` | Adds an AI-callable tool | `web-fetch`, `run-code` |
| `action` | Adds toolbar button or menu item | `insert-date`, `toggle-todo` |
| `renderer` | Custom content renderer (Phase 2) | `vega-lite-charts` |
| `panel` | Custom sidebar/panel (Phase 2) | `dictionary-lookup` |

---

## 2. WASM Plugin API

### 2.1 Host Functions

ZarishNote exposes these host functions to WASM modules:

```wit
// zarishnote-plugin.wit — WASM Interface Type

/// Get the current document content
get-document: func() -> string

/// Get the currently selected text
get-selection: func() -> string

/// Insert text at cursor position
insert-text: func(text: string)

/// Replace selected text
replace-selection: func(text: string)

/// Get a setting value by key
get-setting: func(key: string) -> option<string>

/// Log a message (appears in ZarishNote developer console)
log: func(level: string, message: string)

/// Register a new AI tool
register-tool: func(name: string, description: string, schema: string)

/// Make an HTTP request (sandboxed)
http-request: func(request: http-request) -> http-response

/// Read a workspace file (sandboxed)
read-file: func(path: string) -> result<string, string>

/// Write a workspace file (sandboxed)
write-file: func(path: string, content: string) -> result<_, string>

/// Show a notification to the user
show-notification: func(message: string, type: string)
```

### 2.2 Plugin Entry Point

```rust
// Example: plugin written in Rust, compiled to wasm32-wasi

#[no_mangle]
pub extern "C" fn zn_plugin_init() {
    // Called once when plugin is loaded
    // Register tools, actions, etc.
    let api = ZarishNoteAPI::new();
    api.register_tool(
        "current-date",
        "Insert current date at cursor",
        r#"{"type":"object","properties":{}}"#,
    );
}

#[no_mangle]
pub extern "C" fn zn_plugin_exec(action: *const u8, len: u32) -> u32 {
    // Called when user triggers an action or AI calls a tool
    let action_str = unsafe { std::slice::from_raw_parts(action, len as usize) };
    let action: serde_json::Value = serde_json::from_slice(action_str).unwrap();

    if action["tool"] == "current-date" {
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        ZarishNoteAPI::new().insert_text(&date);
    }

    0 // return 0 for success
}
```

---

## 3. Plugin Manifest

Each plugin must declare a manifest in its WASM module or as a separate JSON file:

```json
{
  "name": "date-inserter",
  "version": "1.0.0",
  "description": "Insert current date at cursor position",
  "author": "ZarishSphere",
  "license": "MIT",
  "entry_point": "zn_plugin_init",
  "permissions": [
    "read:workspace",
    "write:workspace"
  ],
  "checksum_sha256": "abc123def456..."
}
```

Required fields: `name`, `version`, `entry_point`.

---

## 4. Plugin Lifecycle

```
1. INSTALL: Plugin binary downloaded to .znrc-plugins/{name}/{version}/
2. VERIFY: SHA256 checksum matched (if signed)
3. REGISTER: Plugin's zn_plugin_init() called
   - Plugin registers tools, actions with host
4. ENABLE: User enables plugin in Plugin Manager (default: enabled)
5. EXECUTE: User triggers action or AI calls tool
   - Wasmtime instance spawned
   - zn_plugin_exec() called with action payload
   - Result returned
   - Instance destroyed
6. DISABLE: Plugin tools hidden, actions removed
7. UNINSTALL: Plugin binary removed from disk
```

---

## 5. Plugin Sandboxing

All plugins run inside Wasmtime with the same sandbox model as AI tools:

| Capability | Default | Configurable |
|---|---|---|
| Memory limit | 64MB | Per plugin |
| Timeout | 10s | Per plugin |
| Network access | Blocked | Per domain, per plugin |
| Filesystem access | Vault root only | Scoped per path |
| Keychain access | Blocked | Per entry, per plugin |

Plugin permissions declared in manifest are presented to user on install:

```
Plugin "date-inserter" requests:
✅ Read workspace files
✅ Write workspace files
❌ Network access
❌ Keychain access

[Install] [Show details] [Cancel]
```

---

## 6. Plugin Marketplace

### 6.1 Registry

- Community registry at `https://registry.zarishsphere.com/plugins/`
- Plugins are WASM binaries + manifest
- Each plugin has: name, version, description, author, permissions, SHA256

> **Setup:** Deploy from a `registry` GitHub repo to Cloudflare Pages. Free tier (unlimited bandwidth).

### 6.2 V1 Scope

- Manual install: download `.wasm` file, place in `.znrc-plugins/`
- Add to `.znrc` under `plugins.installed[]`

### 6.3 Phase 2 Scope

- One-click install from marketplace browser
- Version updates with changelog
- User ratings and reports
- Automated WASM binary scanning for known vulnerabilities

---

## 7. Plugin Signing and Verification

### 7.1 Signing (Phase 2)

- Plugins signed by ZarishSphere signing key
- Signature stored in manifest as `signature` field
- Verification on load: signature checked against public key bundled in ZarishNote

### 7.2 Checksum Verification (V1)

- SHA256 checksum computed on plugin binary
- Compared against `checksum_sha256` in manifest
- Mismatch → warning shown, plugin disabled

---

## 8. GUI: Plugin Manager

```
Settings → Plugins:

┌────────────────────────────────────────────────────────┐
│  Plugin Manager                            [+ Install]  │
│                                                        │
│  ✅ date-inserter v1.0.0             [Config] [Disable] │
│     Insert current date at cursor                      │
│     Permissions: read:workspace, write:workspace       │
│                                                        │
│  ✅ mermaid-extra v1.2.0             [Config] [Disable] │
│     Additional Mermaid diagram types                   │
│     Permissions: read:workspace                        │
│                                                        │
│  ❌ vega-lite v0.3.0 (disabled)      [Enable] [Remove] │
│     Chart rendering plugin                              │
│                                                        │
│  [Browse Marketplace →]             [Manual Install]   │
└────────────────────────────────────────────────────────┘
```

---

## 9. Plugin Development Guide (Sketch)

### 9.1 Prerequisites

- Rust with `wasm32-wasi` target: `rustup target add wasm32-wasi`
- WASI SDK (optional, for C/C++ plugins)

### 9.2 Minimal Plugin (Rust)

```rust
use serde::{Deserialize, Serialize};

#[no_mangle]
pub extern "C" fn zn_plugin_init() {
    // Registration happens automatically via export
}

#[no_mangle]
pub extern "C" fn zn_plugin_exec(input: *const u8, len: u32) -> u32 {
    // Plugin logic here
    0
}
```

### 9.3 Build

```bash
cargo build --target wasm32-wasi --release
# Output: target/wasm32-wasi/release/my-plugin.wasm
```

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
