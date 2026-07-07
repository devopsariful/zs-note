# 002-directory-structure.md
## ZarishNote Directory Structure
### Full codebase directory layout and ownership

**Document type:** Architecture — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Source Tree](#1-source-tree)
2. [Vault Directory Layout](#2-vault-directory-layout)
3. [Key File Descriptions](#3-key-file-descriptions)

---

## 1. Source Tree

```
zs-note/                              # GitHub: zarishsphere/zs-note
├── src-tauri/                        # Rust backend (Tauri v2)
│   ├── src/
│   │   ├── main.rs                   # Entry point, Tauri bootstrap
│   │   ├── commands/                 # Tauri command handlers
│   │   │   ├── editor.rs             # File read/write, editor state
│   │   │   ├── sandbox.rs            # WASM execution, tool registry
│   │   │   ├── ai.rs                 # Chat, streaming, provider routing
│   │   │   ├── ingest.rs             # Ingestion engine subprocess
│   │   │   ├── search.rs             # Full-text search
│   │   │   ├── git.rs                # Auto-commit, history, sync
│   │   │   ├── voice.rs              # Transcription
│   │   │   ├── mcp.rs                # MCP server management
│   │   │   └── config.rs             # .znrc load/validate
│   │   ├── sandbox/                  # Wasmtime engine
│   │   │   ├── mod.rs
│   │   │   ├── executor.rs           # WASM instance lifecycle
│   │   │   ├── capability.rs         # Permission model
│   │   │   └── network.rs            # Sandboxed HTTP proxy
│   │   ├── ai/                       # AI provider clients
│   │   │   ├── mod.rs
│   │   │   ├── openai.rs             # OpenAI-compatible API
│   │   │   ├── claude.rs             # Anthropic API
│   │   │   ├── gemini.rs             # Google API
│   │   │   └── ollama.rs             # Local Ollama client
│   │   ├── git/                      # Git engine
│   │   │   ├── mod.rs
│   │   │   ├── commit.rs             # Auto-commit logic
│   │   │   ├── history.rs            # Commit log browsing
│   │   │   └── sync.rs               # Push/pull/remote
│   │   ├── mcp/                      # MCP client
│   │   │   ├── mod.rs
│   │   │   ├── transport.rs          # stdio + HTTP transports
│   │   │   ├── protocol.rs           # JSON-RPC 2.0 framing
│   │   │   └── router.rs             # Tool routing
│   │   ├── vector/                   # LanceDB store
│   │   │   ├── mod.rs
│   │   │   ├── index.rs              # Embedding + indexing
│   │   │   └── query.rs              # ANN search
│   │   ├── config.rs                 # .znrc parser
│   │   ├── logging.rs                # Audit + structured logging
│   │   └── types.rs                  # Shared types
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                              # Frontend (Svelte 5 + TypeScript)
│   ├── lib/
│   │   ├── components/
│   │   │   ├── App.svelte
│   │   │   ├── Sidebar.svelte
│   │   │   ├── Editor.svelte
│   │   │   ├── AIPanel.svelte
│   │   │   ├── StatusBar.svelte
│   │   │   └── ...                   # Modal, Toolbar, Search, etc.
│   │   ├── stores/                   # Svelte 5 stores (runes)
│   │   │   ├── editor.ts
│   │   │   ├── files.ts
│   │   │   ├── ai.ts
│   │   │   └── config.ts
│   │   ├── commands/                 # Tauri invoke wrappers
│   │   │   ├── editor.ts
│   │   │   ├── ai.ts
│   │   │   └── sandbox.ts
│   │   ├── milkdown/                 # Milkdown editor setup
│   │   │   ├── editor.ts
│   │   │   ├── nodes/
│   │   │   └── plugins/
│   │   └── types.ts
│   ├── app.css
│   ├── main.ts                       # Vite entry
│   └── vite-env.d.ts
│
├── packages/                         # Shared packages (if any)
├── ingestion/                        # Python ingestion engine
│   ├── pyproject.toml
│   ├── src/
│   │   └── zarishnote_ingest/
│   │       ├── __init__.py
│   │       ├── markitdown.py         # Core converter wrapper
│   │       ├── converters/           # Individual converters
│   │       │   ├── pdf.py
│   │       │   ├── docx.py
│   │       │   ├── youtube.py
│   │       │   └── wikipedia.py
│   │       └── cli.py                # CLI entry point
│   └── tests/
│
├── tools/                            # Sample WASM tools
│   ├── run-code/
│   │   └── src/main.rs
│   └── web-fetch/
│       └── src/main.rs
│
├── static/                           # Static assets
├── index.html
├── package.json
├── pnpm-lock.yaml
├── vite.config.ts
├── tsconfig.json
├── svelte.config.js
└── README.md
```

---

## 2. Vault Directory Layout

When a user creates/opens a vault with ZarishNote, the vault folder contains:

```
my-vault/
├── .znrc                            # Workspace config (YAML)
├── .znrc-audit.log                  # Tool execution audit log
├── .znrc-vectors/                   # LanceDB vector store (auto)
│   └── (LanceDB tables)
├── .znrc-search/                    # Full-text search index (auto)
│   └── (SQLite FTS5 index)
├── .znrc-plugins/                   # Installed plugins (auto)
│   └── plugin-name/
│       └── plugin.wasm
├── .znrc-state/                     # Tool persistent state (auto)
│   └── tool-name/
│       └── state.json
├── .znrc-history/                   # AI conversation history (auto)
│   └── 2026-06-08-ollama.jsonl
├── templates/                       # AI prompt templates (user)
│   ├── summarize.md
│   └── clinical-case.md
├── knowledge/                       # Knowledge base folders (user)
│   ├── clinical/
│   │   └── who-protocols.md
│   └── project/
│       └── overview.md
├── inbox/                           # Imported files (auto)
├── transcripts/                     # Voice transcriptions (auto)
├── assets/                          # Images, attachments (auto)
│   └── ai-images/                   # AI-generated images
├── recordings/                      # Voice recordings (auto)
└── .git/                            # Git repository (auto)
```

---

## 3. Key File Descriptions

### 3.1 Rust Backend

| File | Purpose |
|---|---|
| `src-tauri/src/main.rs` | Tauri bootstrap, plugin registration, menu setup |
| `src-tauri/src/commands/*.rs` | One file per command group, maps to Tauri `#[tauri::command]` |
| `src-tauri/src/sandbox/*.rs` | Wasmtime lifecycle, capability enforcement |
| `src-tauri/src/ai/*.rs` | Provider-specific API clients, unified response streaming |
| `src-tauri/src/git/*.rs` | libgit2 wrapper with debounced auto-commit |
| `src-tauri/src/mcp/*.rs` | JSON-RPC transport, server lifecycle, tool routing |
| `src-tauri/src/vector/*.rs` | LanceDB + FastEmbed integration |

### 3.2 Frontend

| File | Purpose |
|---|---|
| `src/lib/components/App.svelte` | Root layout: sidebar, editor, AI panel, status bar |
| `src/lib/components/Editor.svelte` | Mode switching (WYSIWYG/source/split) |
| `src/lib/components/AIPanel.svelte` | Chat UI, template picker, context inspector |
| `src/lib/milkdown/editor.ts` | Milkdown/ProseMirror configuration, custom nodes |
| `src/lib/commands/*.ts` | Type-safe wrappers around `@tauri-apps/api` invoke calls |

### 3.3 Python Ingestion

| File | Purpose |
|---|---|
| `ingestion/src/zarishnote_ingest/cli.py` | CLI entry point: `zarishnote-ingest` |
| `ingestion/src/zarishnote_ingest/markitdown.py` | Core MarkItDown wrapper with ZarishNote extensions |
| `ingestion/src/zarishnote_ingest/converters/*.py` | One file per converter (PDF, DOCX, YouTube, etc.) |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
