# 001-threat-model.md
## ZarishNote Threat Model
### STRIDE analysis and mitigations

**Document type:** Security — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Scope](#1-scope)
2. [Assets](#2-assets)
3. [Trust Boundaries](#3-trust-boundaries)
4. [STRIDE Analysis](#4-stride-analysis)
5. [Mitigation Summary](#5-mitigation-summary)

---

## 1. Scope

This threat model covers:
- ZarishNote desktop application (Windows, macOS, Linux)
- Vault data on local filesystem
- Communication with AI providers and MCP servers
- Plugin/tool execution via Wasmtime sandbox

**Out of scope:**
- Physical device security
- OS-level vulnerabilities (assume OS is trusted)
- Network-level attacks on the user's LAN

---

## 2. Assets

| Asset | Sensitivity | Location |
|---|---|---|
| Document content | Varies (user data) | Vault files on disk |
| API keys | High | OS Keychain |
| AI conversation history | Medium | `.znrc-history/` |
| Vector embeddings | Medium | `.znrc-vectors/` |
| Git history | Varies | `.git/` |
| `.znrc` config | Medium | Vault root |

---

## 3. Trust Boundaries

```
[User] ←→ [ZarishNote UI] ←→ [Rust Backend]
                                 ↓
                       [Wasmtime Sandbox] ─→ [AI Tools / Plugins / MCP]
                                 ↓
                       [OS Filesystem + Keychain]
```

Boundary 1: Frontend ↔ Backend (IPC)
- Frontend cannot directly access filesystem, keychain, or network
- All access goes through typed Tauri commands

Boundary 2: Backend ↔ Wasmtime Sandbox
- WASM modules cannot access OS directly
- All syscalls filtered through capability model

Boundary 3: Backend ↔ External APIs
- API keys injected by host, never exposed to WASM
- Network requests proxied through capability firewall

---

## 4. STRIDE Analysis

### 4.1 Spoofing

| Threat | Vector | Mitigation |
|---|---|---|
| Plugin pretends to be a different plugin | Unsigned WASM binary | SHA256 checksum verification on install |
| MCP server spoofs identity | Unverified HTTP endpoint | User must explicitly add remote MCP servers |
| AI provider impersonation | DNS/phishing of API endpoint | TLS + user-verified endpoint URL |

### 4.2 Tampering

| Threat | Vector | Mitigation |
|---|---|---|
| Plugin modifies files outside workspace | Capability model bypass | Wasmtime filesystem scoping enforced at runtime |
| AI tool exfiltrates data via network | Unauthorized outbound request | Network policy: default-deny, allow-list only |
| Malicious MCP server modifies documents | Injecting content via AI tool calls | Human-in-the-loop for destructive operations |

### 4.3 Repudiation

| Threat | Vector | Mitigation |
|---|---|---|
| Plugin denies making a network call | No audit trail | All sandbox executions logged (`.znrc-audit.log`) |
| User denies a publish action | No change tracking | Git history records every file change |

### 4.4 Information Disclosure

| Threat | Vector | Mitigation |
|---|---|---|
| Plugin reads files outside vault | Path traversal | Wasmtime resolves all paths, rejects `../` |
| API key leaked to plugin | Key exposed in env or args | Keys stored in OS Keychain, injected only on demand |
| AI provider logs prompts | Data sent to third-party | User chooses provider; no ZarishNote intermediary |
| Plugin reads other plugin's state | Cross-plugin state access | State isolated per plugin in `.znrc-state/{name}/` |

### 4.5 Denial of Service

| Threat | Vector | Mitigation |
|---|---|---|
| Plugin consumes all memory | Infinite loop or large allocation | Memory limit per plugin (default 64MB) |
| Plugin runs forever | Infinite loop | CPU timeout per plugin (default 10s) |
| MCP server hangs | Unresponsive subprocess | Stdio timeout (default 30s), process killed |
| Ingestion fills disk | Very large file conversion | Max output size limit (default 10MB) |

### 4.6 Elevation of Privilege

| Threat | Vector | Mitigation |
|---|---|---|
| Plugin escapes Wasmtime sandbox | WASM VM vulnerability | Wasmtime is production-hardened; ZarishNote pins versions |
| Plugin accesses Tauri APIs | IPC from WASM to host | WASM has no Tauri IPC access — only host functions |
| MCP server executes shell commands | Arbitrary command injection | MCP servers run as subprocesses with restricted permissions |

---

## 5. Mitigation Summary

| Control | Mechanism | Where |
|---|---|---|
| Sandbox isolation | Wasmtime | All plugins, tools, MCP |
| Capability model | Per-tool permission list | `.znrc` + runtime enforcement |
| Network firewall | Domain allow-list | Rust HTTP proxy |
| Key isolation | OS Keychain | API keys |
| Audit logging | Append-only log | `.znrc-audit.log` |
| Version control | Git auto-commit | `.git/` |
| Input validation | Type-checked `.znrc` | Config load |
| Human-in-the-loop | Confirmation dialogs | Destructive tool calls |
| Memory limits | Wasmtime `StoreLimits` | Per-tool config |
| Timeouts | Epoch-based interruption | Per-tool config |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
