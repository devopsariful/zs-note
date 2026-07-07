# 001-mcp-spec.md
## ZarishNote MCP Engine Specification
### Model Context Protocol client implementation

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

- [001-mcp-spec.md](#001-mcp-specmd)
  - [ZarishNote MCP Engine Specification](#zarishnote-mcp-engine-specification)
    - [Model Context Protocol client implementation](#model-context-protocol-client-implementation)
  - [Table of Contents](#table-of-contents)
  - [1. MCP Overview](#1-mcp-overview)
  - [2. ZarishNote as MCP Client](#2-zarishnote-as-mcp-client)
    - [2.1 MCP Primitives Supported](#21-mcp-primitives-supported)
  - [3. Supported Transports](#3-supported-transports)
    - [3.1 OAuth 2.1 Support](#31-oauth-21-support)
  - [4. Server Configuration](#4-server-configuration)
    - [4.1 Popular Servers (Pre-configured in Marketplace)](#41-popular-servers-pre-configured-in-marketplace)
  - [5. Tool Calling Flow](#5-tool-calling-flow)
    - [5.1 Human-in-the-Loop](#51-human-in-the-loop)
  - [6. Security: MCP in the Sandbox](#6-security-mcp-in-the-sandbox)
  - [7. Knowledge Bases](#7-knowledge-bases)
    - [7.1 Knowledge Base Indexing](#71-knowledge-base-indexing)
    - [7.2 Knowledge Base Query (via MCP or AI panel)](#72-knowledge-base-query-via-mcp-or-ai-panel)
  - [8. GUI: MCP Manager](#8-gui-mcp-manager)

---

## 1. MCP Overview

MCP (Model Context Protocol) is the industry-standard protocol (adopted December 2025 by the Linux Foundation, supported by Anthropic, OpenAI, Google DeepMind, Microsoft) for connecting AI models to external tools.

ZarishNote is a **first-class MCP client**, meaning:
- It implements the full MCP specification (2025-11-25 version)
- It can connect to any MCP server
- All MCP tool calls are sandboxed
- The AI in the chat panel can invoke MCP tools natively

MCP enables ZarishNote to connect to:
- GitHub (search code, create issues, read PRs)
- Google Drive (search and read documents)
- Slack (search messages)
- Local filesystem (beyond vault root, with permission)
- Custom internal APIs
- Any of 10,000+ community MCP servers

---

## 2. ZarishNote as MCP Client

ZarishNote implements the MCP client role:
- Discovers available tools from connected servers
- Injects tool descriptions into the AI system prompt
- Routes AI tool calls to the appropriate server
- Returns tool results back to the AI
- Presents tool results in the chat panel

### 2.1 MCP Primitives Supported

| Primitive | Support | Notes |
|---|---|---|
| **Tools** | ✅ Full | AI-callable functions |
| **Resources** | ✅ Full | Data sources (files, APIs) |
| **Prompts** | ✅ Full | Reusable prompt templates from server |
| **Sampling** | ✅ V1 | Human-in-the-loop confirmation |
| **Elicitation** | Phase 2 | Server asks for user input |

---

## 3. Supported Transports

| Transport | Description | Use case |
|---|---|---|
| `stdio` | Server runs as subprocess, ZarishNote communicates via stdin/stdout | Local tools, `npx` servers |
| `http` (Streamable HTTP) | Remote server over HTTP with SSE streaming | Remote APIs, team servers |
| `sse` | Legacy SSE transport | Older servers |

### 3.1 OAuth 2.1 Support

For remote MCP servers that require authentication:
- ZarishNote handles OAuth 2.1 flow natively
- Access tokens stored in OS Keychain
- Auto-refresh on expiry
- No OAuth implementation required in server for basic usage

---

## 4. Server Configuration

Servers configured in `.znrc` or via GUI (Settings → MCP → Servers):

```yaml
mcp:
  servers:
    # stdio server (most common)
    - name: "github"
      transport: "stdio"
      command: "npx"
      args: ["-y", "@modelcontextprotocol/server-github"]
      env:
        GITHUB_TOKEN: "${keychain:github-token}"
      sandbox: true
      enabled: true
      description: "GitHub file search, issue management"

    # HTTP server (remote)
    - name: "team-kb"
      transport: "http"
      url: "https://api.example.com/mcp"
      auth:
        type: "bearer"
        key_id: "team-kb-token"
      sandbox: true
      enabled: true

    # Local filesystem
    - name: "filesystem"
      transport: "stdio"
      command: "npx"
      args: ["-y", "@modelcontextprotocol/server-filesystem", "${vault_path}"]
      sandbox: true
      enabled: true
```

### 4.1 Popular Servers (Pre-configured in Marketplace)

| Server | What it does | Transport |
|---|---|---|
| `@mcp/github` | GitHub: search code, issues, PRs | stdio |
| `@mcp/filesystem` | File read/write (scoped) | stdio |
| `@mcp/google-drive` | Search and read Google Drive | stdio |
| `@mcp/slack` | Search Slack messages | stdio |
| `@mcp/fetch` | Fetch any URL as text | stdio |
| `@mcp/memory` | Persistent memory across sessions | stdio |
| `@mcp/sequential-thinking` | Chain-of-thought reasoning | stdio |
| `@mcp/postgres` | Query a PostgreSQL database | stdio |
| `@mcp/sqlite` | Query a local SQLite database | stdio |
| `@mcp/brave-search` | Web search via Brave | stdio |

---

## 5. Tool Calling Flow

```
1. User asks: "Find all open GitHub issues tagged 'bug'"
2. AI decides to call tool: github.list_issues(state="open", labels=["bug"])
3. ZarishNote intercepts tool call
4. Sandbox check: is 'github' server enabled? does it have 'list_issues' tool?
5. If server requires confirmation (sampled): show user confirmation dialog
6. Execute: ZarishNote routes call to GitHub MCP server via stdio
7. GitHub MCP server returns JSON result
8. ZarishNote formats result for AI
9. AI continues response with tool result context
10. Result optionally inserted into document
```

### 5.1 Human-in-the-Loop

For sensitive tool calls (file writes, API POSTs), ZarishNote shows confirmation:

```
┌────────────────────────────────────────────┐
│  🔧 Tool Request                           │
│                                            │
│  AI wants to call:                         │
│  github.create_issue                       │
│                                            │
│  Arguments:                                │
│  title: "Bug: Login fails on mobile"      │
│  body: "..."                               │
│  labels: ["bug"]                           │
│                                            │
│  [Allow once] [Allow always] [Deny]        │
└────────────────────────────────────────────┘
```

---

## 6. Security: MCP in the Sandbox

All MCP server executions (stdio) run as subprocesses with restricted permissions:
- Network access only to declared domains
- Filesystem access only to declared paths
- No access to OS keychain (ZarishNote injects secrets directly)
- stdout/stderr captured by ZarishNote, not visible to other processes
- Process killed on timeout (default 30s)

MCP HTTP servers:
- All requests proxied through ZarishNote's Rust HTTP client
- TLS always required
- No direct browser/WebView connection to MCP servers

---

## 7. Knowledge Bases

ZarishNote supports **local knowledge bases** — indexed collections of documents queryable via RAG:

```yaml
mcp:
  knowledge_bases:
    - name: "rohingya-health-protocols"
      path: "knowledge/rohingya-protocols"
      description: "WHO and UNHCR health guidelines for Rohingya FDMN population"
      formats: ["*.md", "*.pdf", "*.docx"]
      index_on_start: true
      auto_reindex: true          # re-index on file changes
```

### 7.1 Knowledge Base Indexing

1. On vault open, ZarishNote scans declared KB folders
2. New/modified files are ingested via the Ingestion Engine
3. Markdown output is chunked (512 tokens, 50 overlap)
4. Chunks embedded via FastEmbed-rs (local ONNX, no API required)
5. Embeddings stored in LanceDB at `.znrc-vectors/`

### 7.2 Knowledge Base Query (via MCP or AI panel)

When AI needs information:
- "Query knowledge base: rohingya-health-protocols" → retrieves top-5 chunks
- Results injected into AI context
- Source citations included in response

---

## 8. GUI: MCP Manager

Settings → MCP shows:

```
┌──────────────────────────────────────────────────────┐
│  MCP Servers                              [+ Add]    │
│                                                      │
│  ✅ github            stdio   [Configure] [Test] [×] │
│  ✅ filesystem        stdio   [Configure] [Test] [×] │
│  ❌ team-kb (off)     http    [Configure] [Test] [×] │
│                                                      │
│  Knowledge Bases                          [+ Add]    │
│                                                      │
│  📚 rohingya-health-protocols                        │
│     path: knowledge/rohingya-protocols               │
│     2,340 chunks indexed · Last: 2 min ago           │
│     [Re-index] [Browse] [Delete]                     │
│                                                      │
│  Marketplace                                         │
│  [Browse MCP Marketplace →]                          │
└──────────────────────────────────────────────────────┘
```

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*