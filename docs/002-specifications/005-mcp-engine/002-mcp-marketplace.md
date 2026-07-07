# 002-mcp-marketplace.md
## ZarishNote MCP Marketplace Specification
### Server discovery, installation, and sandboxing

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Marketplace Overview](#1-marketplace-overview)
2. [Server Discovery](#2-server-discovery)
3. [Server Installation](#3-server-installation)
4. [Server Sandboxing](#4-server-sandboxing)
5. [GUI: Marketplace Browser](#5-gui-marketplace-browser)
6. [Registry API](#6-registry-api)

---

## 1. Marketplace Overview

The MCP Marketplace is a community registry of MCP servers that ZarishNote users can discover and install. It is the primary channel for extending ZarishNote's AI tool capabilities.

**V1 scope:** Browse + manual install by command.
**Phase 2 scope:** One-click install, ratings, reviews, automatic updates.

---

## 2. Server Discovery

### 2.1 Registry Source

The default registry is the community MCP server list at `https://registry.zarishsphere.com/mcp/servers.json` (Phase 2). In V1, ZarishNote ships with a curated list of popular servers hardcoded.

> **Setup:** Deploy a static `registry` repo to Cloudflare Pages (free tier). Add `CNAME registry.zarishsphere.com` → Pages site in Cloudflare DNS.

### 2.2 Curated Server List (V1)

| Server | Package | What it does |
|---|---|---|
| GitHub | `@modelcontextprotocol/server-github` | Search code, issues, PRs, repos |
| Filesystem | `@modelcontextprotocol/server-filesystem` | Read/write files (scoped) |
| Google Drive | `@modelcontextprotocol/server-google-drive` | Search and read Drive documents |
| Slack | `@modelcontextprotocol/server-slack` | Search Slack messages and channels |
| Fetch | `@modelcontextprotocol/server-fetch` | Fetch any URL as text |
| Memory | `@modelcontextprotocol/server-memory` | Persistent memory for AI across sessions |
| Sequential Thinking | `@modelcontextprotocol/server-sequential-thinking` | Chain-of-thought reasoning |
| PostgreSQL | `@modelcontextprotocol/server-postgres` | Query PostgreSQL databases |
| SQLite | `@modelcontextprotocol/server-sqlite` | Query local SQLite databases |
| Brave Search | `@modelcontextprotocol/server-brave-search` | Web search via Brave |

### 2.3 Discovery (Phase 2)

- Community registry with user-submitted servers
- Search by name, description, tags
- "Featured" and "New" sections
- GitHub stars and download count
- Client-side verification of server metadata

---

## 3. Server Installation

### 3.1 Installation Methods

| Method | V1 | Phase 2 |
|---|---|---|
| Manual CLI (`npx`, `uvx`, binary) | ✅ | ✅ |
| One-click from marketplace | ❌ | ✅ |
| Docker (for complex servers) | ❌ | ✅ |

### 3.2 V1 Manual Installation

User copies the command from the marketplace listing and runs it in ZarishNote's MCP settings:

```
Settings → MCP → Servers → Add → Paste command
```

Example:
```
npx -y @modelcontextprotocol/server-github
```

ZarishNote parses the command into `transport`, `command`, and `args` for `.znrc`.

### 3.3 One-Click Installation (Phase 2)

1. Browse marketplace → click "Install"
2. ZarishNote downloads server metadata
3. If npm package: runs `npm install -g` or uses `npx` on demand
4. If binary: downloads to `.znrc-plugins/mcp/` with checksum verification
5. If Docker: pulls image, configures Docker execution
6. Server appears in MCP server list, enabled by default

---

## 4. Server Sandboxing

All MCP servers run sandboxed regardless of installation method:

| Restriction | stdio servers | HTTP servers |
|---|---|---|
| Process isolation | Subprocess with resource limits | N/A (remote) |
| Filesystem limits | Declared paths only | N/A |
| Network limits | Declared domains only | TLS required |
| Timeout | 30s default | 30s default |
| Keychain access | Injected by host, never read directly | N/A |
| Audit | All calls logged | All calls logged |

The sandbox configuration for each server is stored in `.znrc`:

```yaml
mcp:
  servers:
    - name: "github"
      transport: "stdio"
      command: "npx"
      args: ["-y", "@modelcontextprotocol/server-github"]
      sandbox: true
      memory_limit: "128MB"
      timeout: "15s"
```

---

## 5. GUI: Marketplace Browser

```
┌──────────────────────────────────────────────────────────┐
│  MCP Marketplace                              [Search]   │
│                                                          │
│  📂 All Categories                                       │
│  🔧 Featured  📊 Data  🔗 API  🛠️ Dev Tools             │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │ github                                   ★ 12.4k  │  │
│  │ Search code, manage issues and PRs                 │  │
│  │ stdio · npm · [Install] [Details]                  │  │
│  ├────────────────────────────────────────────────────┤  │
│  │ filesystem                               ★ 8.1k   │  │
│  │ Read/write files with path restrictions           │  │
│  │ stdio · npm · [Install] [Details]                  │  │
│  ├────────────────────────────────────────────────────┤  │
│  │ brave-search                             ★ 3.2k   │  │
│  │ Web search via Brave Search API                    │  │
│  │ stdio · npm · [Install] [Details]                  │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  [Refresh] [Report Server]                               │
└──────────────────────────────────────────────────────────┘
```

---

## 6. Registry API

### 6.1 Server Manifest Format

```json
{
  "name": "github",
  "version": "1.0.0",
  "display_name": "GitHub MCP Server",
  "description": "Search code, manage issues, PRs, and repos",
  "categories": ["dev-tools", "api"],
  "transport": ["stdio"],
  "install_command": "npx -y @modelcontextprotocol/server-github",
  "homepage": "https://github.com/modelcontextprotocol/servers",
  "license": "MIT",
  "icon_url": "https://registry.zarishsphere.com/icons/github.png",
  "tags": ["github", "git", "code"]
}
```

### 6.2 API Endpoints (Phase 2)

| Endpoint | Description |
|---|---|
| `GET /mcp/servers.json` | Full server list |
| `GET /mcp/servers/{name}.json` | Single server detail |
| `GET /mcp/search?q=github` | Search servers |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
