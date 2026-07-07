# 001-phase-one.md
## ZarishNote Phase 1 Roadmap
### Weeks 1–8: Core MVP

**Document type:** Roadmap — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Scope](#1-scope)
2. [Week 1–2: Project Scaffold + Core Editor](#2-week-1-2-project-scaffold--core-editor)
3. [Week 3–4: File Manager + Git Engine + Config](#3-week-3-4-file-manager--git-engine--config)
4. [Week 5–6: Sandbox + AI + Ingestion Foundations](#4-week-5-6-sandbox--ai--ingestion-foundations)
5. [Week 7–8: MCP + Voice + Publish + Integration](#5-week-7-8-mcp--voice--publish--integration)
6. [Phase 1 Deliverable](#6-phase-1-deliverable)

---

## 1. Scope

Phase 1 delivers the core MVP of ZarishNote for **desktop (Windows, macOS, Linux)**. Mobile (iOS, Android) is Phase 2.

### 1.1 What Phase 1 Includes

| Feature | Status |
|---|---|
| WYSIWYG Markdown editor (Milkdown) | ✅ |
| Source / split / preview views | ✅ |
| File sidebar with tree, tags, search | ✅ |
| Git auto-commit + history browsing | ✅ |
| `.znrc` config with full GUI | ✅ |
| Wasmtime sandbox with capability model | ✅ |
| AI chat (multi-provider) with streaming | ✅ |
| Document ingestion (10+ formats) | ✅ |
| MCP client (stdio + HTTP) | ✅ |
| Knowledge bases (local RAG) | ✅ |
| Voice transcription (Whisper.cpp) | ✅ |
| GitHub + API publishing | ✅ |

---

## 2. Project Scaffold + Core Editor

### Tasks

- [x] Initialize Tauri v2 project with Svelte 5 + Vite 8 + TypeScript 6
- [ ] Configure pnpm workspace, `Cargo.toml`, `tauri.conf.json`
- [ ] Set up CI/CD pipeline (GitHub Actions: lint, test, build per platform)
- [ ] Configure Dependabot for dependency vulnerability scanning
- [ ] Integrate Milkdown v7 with CommonMark + GFM presets
- [ ] Implement WYSIWYG mode (default)
- [ ] Implement Source mode with Shiki syntax highlighting
- [ ] Implement Split mode (side-by-side)
- [ ] Basic formatting toolbar (H1–H3, B, I, code, link, list)
- [ ] KaTeX math rendering (inline + block)
- [ ] Mermaid diagram rendering (fenced ` ```mermaid ` blocks)
- [ ] Table editor with context toolbar
- [ ] Image paste/drag-and-drop
- [ ] Keyboard shortcuts table (see editor spec §6)
- [ ] Editor settings panel (theme, font, etc.)

**Deliverable:** Functional Markdown editor with three view modes, math, diagrams, and image support.

---

## 3. File Manager + Git Engine + Config

### Tasks

- [ ] File tree sidebar with folder navigation
- [ ] File operations: create, rename, delete, duplicate, move
- [ ] File import dialog (single + bulk)
- [ ] Tag system (read from front matter, manual assign)
- [ ] Tag sidebar with filter
- [ ] Full-text search with SQLite FTS5
- [ ] Git engine: auto-commit on save (debounced)
- [ ] Git engine: commit history browser
- [ ] Git engine: diff view
- [ ] `.znrc` schema parser in Rust (serde_yaml)
- [ ] `.znrc` validation on vault open
- [ ] Settings GUI bound to `.znrc` fields
- [ ] Hot-reload `.znrc` on file change

**Deliverable:** Working file manager with Git version history and fully configured settings system.

---

## 4. Sandbox + AI + Ingestion Foundations

### Tasks

- [ ] Wasmtime engine integration (create engine, compile module, execute)
- [ ] Capability model (permissions parsing + runtime check)
- [ ] Network proxy with domain allow-list
- [ ] Filesystem scoping with virtual `workspace://` paths
- [ ] Resource limits (memory, timeout, output size)
- [ ] Audit logging (`.znrc-audit.log`)
- [ ] AI chat panel UI (message list, input, streaming display)
- [ ] OpenAI provider client (streaming chat completion)
- [ ] Anthropic Claude provider client
- [ ] Google Gemini provider client
- [ ] Ollama provider client
- [ ] Provider switching in panel
- [ ] API key management via OS Keychain (Tauri plugin)
- [ ] Context injection (current doc, selection, core files)
- [ ] Python ingestion CLI (`zarishnote-ingest`)
- [ ] Tauri command wrapper for ingestion subprocess
- [ ] Drag-and-drop file → ingestion

**Deliverable:** Sandbox running WASM tools, AI chat with 4 providers, document ingestion working.

---

## 5. MCP + Voice + Publish + Integration

### Tasks

- [ ] MCP stdio transport (spawn subprocess, JSON-RPC framing)
- [ ] MCP HTTP transport (SSE streaming, JSON-RPC)
- [ ] Tool routing (AI → MCP server → result)
- [ ] Human-in-the-loop confirmation UI
- [ ] MCP server configuration GUI
- [ ] Knowledge base indexing (LanceDB + FastEmbed)
- [ ] Knowledge base query (RAG for AI panel)
- [ ] Whisper.cpp integration (recording + transcription)
- [ ] Voice dictation into editor
- [ ] Audio file import transcription
- [ ] GitHub publishing (API push to repo)
- [ ] Custom API publishing (POST to endpoint)
- [ ] RSS feed generation
- [ ] Image hosting (GitHub, Cloudflare)
- [ ] Settings → Publish panel
- [ ] Integration testing across all modules
- [ ] Integration test suite (unit + E2E for all modules)
- [ ] Cross-platform build verification
- [ ] Installer generation (NSIS, DMG, AppImage)

**Deliverable:** Complete V1 MVP with all features integrated and cross-platform installers.

---

## 6. Phase 1 Deliverable

### 6.1 Release Artifacts

| Platform | Format |
|---|---|
| Windows | `.msi` installer + portable `.exe` |
| macOS | `.dmg` (Apple Silicon + Intel universal) |
| Linux | `.AppImage` + `.deb` + `.rpm` |

### 6.2 Acceptance Criteria

- [ ] Editor: open, edit, save Markdown files in all three modes
- [ ] File manager: browse, search, tag, organize vault
- [ ] Git: every save committed, history browsable
- [ ] AI: chat with any provider, insert/replace text
- [ ] Sandbox: WASM tool executes with declared capabilities
- [ ] Ingestion: PDF, DOCX, PPTX, XLSX, EPUB, YouTube → Markdown
- [ ] MCP: connect GitHub server, list issues via AI
- [ ] Knowledge base: index "knowledge/" folder, query via AI
- [ ] Voice: record and transcribe into document
- [ ] Publishing: push document to GitHub repo
- [ ] Installer: works on clean Windows, macOS, Linux

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
