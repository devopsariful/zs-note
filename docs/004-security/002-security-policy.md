# 002-security-policy.md
## ZarishNote Security Policy
### Responsible disclosure, SLA, and vulnerability handling

**Document type:** Security — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Reporting a Vulnerability](#1-reporting-a-vulnerability)
2. [Response SLA](#2-response-sla)
3. [Responsible Disclosure](#3-responsible-disclosure)
4. [Supported Versions](#4-supported-versions)
5. [Security Posture](#5-security-posture)

---

## 1. Reporting a Vulnerability

Security vulnerabilities should be reported privately to the ZarishSphere Foundation.

**Preferred method:** Email `security@zarishsphere.com` (requires email forwarding setup — see repo maintainer) with:
- Project name and version
- Type of vulnerability (sandbox escape, XSS, privilege escalation, etc.)
- Steps to reproduce
- Proof of concept (if available)
- Impact assessment

**Alternative method:** GitHub Security Advisory at `https://github.com/zarishsphere/zs-note/security/advisories`

**Do not:**
- Open a public GitHub issue for security vulnerabilities
- Disclose the vulnerability publicly before the fix is released

---

## 2. Response SLA

| Severity | Initial response | Fix target |
|---|---|---|
| **Critical** (sandbox escape, RCE, data exfiltration) | 24 hours | 7 days |
| **High** (privilege escalation, sensitive data leak) | 48 hours | 14 days |
| **Medium** (limited information disclosure, DoS) | 5 days | 30 days |
| **Low** (minor config leakage, cosmetic issues) | 14 days | Next release |

All vulnerabilities will receive a CVE identifier.

---

## 3. Responsible Disclosure

ZarishSphere Foundation follows coordinated disclosure:

1. Reporter submits vulnerability (private)
2. Foundation acknowledges receipt (within SLA)
3. Foundation develops fix and prepares release
4. Fix is released with advisory
5. Reporter notified and credited (if desired)
6. Advisory published 30 days after fix

---

## 4. Supported Versions

| Version | Supported |
|---|---|
| Latest release | ✅ Security patches |
| Previous release | ✅ Critical fixes only |
| Older releases | ❌ |

Users are encouraged to always run the latest version. ZarishNote's built-in update mechanism (Tauri updater) notifies users of available updates.

---

## 5. Security Posture

### 5.1 Design Principles

- **Sandbox by default:** All non-core code runs in Wasmtime
- **Least privilege:** Tools request only the capabilities they need
- **Defense in depth:** Multiple layers (OS keychain, sandbox, audit, network firewall)
- **No cloud dependency:** No ZarishNote servers to attack

### 5.2 Known Security Limitations

| Limitation | Mitigation |
|---|---|
| Ingestion engine (Python) runs as subprocess, not sandboxed | Python process has restricted filesystem and no network access |
| MCP stdio servers run as subprocesses | Sandboxed via OS process isolation (seccomp on Linux) |
| `.znrc` is plain YAML on disk | Vault permissions control access; Git tracks changes |
| Frontend runs JavaScript in WebView | No access to filesystem/network except via IPC |

### 5.3 Dependency Security

- WASM modules pinned to specific SHA256 checksums
- Rust crates pinned in `Cargo.lock`
- Python dependencies pinned in `pyproject.toml`
- Dependabot enabled on GitHub repository
- Regular dependency audits before each release

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
