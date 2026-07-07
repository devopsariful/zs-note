# ZarishNote — Master Project Blueprint
### Complete Specification for V1 Development

**Document type:** Master Reference — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** Apache 2.0 (code) · CC BY 4.0 (documentation)
**Status:** V1 — Authoritative. All development must comply.
**Part of:** [`zarishsphere/zs-note`](https://github.com/zarishsphere/zs-note) — `docs/` directory

---

## Table of Contents

1. [What is ZarishNote](#1-what-is-zarishnote)
2. [Why ZarishNote Exists](#2-why-zarishnote-exists)
3. [Core Principles](#3-core-principles)
4. [Blueprint Structure](#4-blueprint-structure)
5. [Quick Reference — Feature Matrix](#5-quick-reference--feature-matrix)
6. [Technology Stack](#6-technology-stack)
7. [For Developers: Where to Start](#7-for-developers-where-to-start)

---

## 1. What is ZarishNote

ZarishNote is a **free, open-source, ultra-lightweight (~12MB) WYSIWYG Markdown editor** that doubles as a **private AI assistant** — all running natively on your device.

It is built on **Tauri v2 + Svelte 5 + Rust** and runs on Windows, macOS, Linux, Android, and iOS.

ZarishNote is not a note-taking app trying to add AI. It is an **AI-native writing environment** that keeps Markdown at its core. The AI panel, sandbox, ingestion engine, MCP support, voice transcription, and publishing tools are all first-class features — not bolt-ons.

**One sentence:** A clean Markdown editor for everyday writing that becomes a fully sandboxed private AI agent when you need one.

---

## 2. Why ZarishNote Exists

| The Problem | ZarishNote's Answer |
|---|---|
| Electron editors waste 200–400MB RAM just to edit text | Tauri v2: <15MB RAM idle, ~12MB installer |
| AI editors phone home — prompts, documents, secrets leave your machine | No intermediary servers. Your device → provider API directly |
| Context is manual — you pick files for the AI to read | G2A Context Engine: `.znrc` config auto-injects context |
| Sandboxing is absent — AI tools run with full system access | Built-in WASM sandbox (Wasmtime). Every tool is isolated |
| Document ingestion requires separate tools | Integrated MarkItDown engine: 20+ formats → Markdown |
| MCP is fragmented across apps | First-class MCP client + marketplace + knowledge bases |
| Publishing requires leaving the editor | One-click GitHub, custom API, RSS, image host |
| Voice transcription requires external apps | Whisper-backed voice → Markdown with speaker labels |
| Mobile and desktop notes are separate | Tauri v2 iOS/Android + offline-first sync |

---

## 3. Core Principles

| Principle | What it means in practice |
|---|---|
| **Sandbox by default** | Every AI tool, plugin, and MCP server runs in Wasmtime. No direct OS access |
| **Full-stack inside the sandbox** | The sandbox includes HTTP, filesystem scoping, tool calling, and state — not just isolation |
| **Interoperable** | All sandbox tools work identically on Windows, macOS, Linux, Android, iOS |
| **Offline-first** | Every feature works without internet. AI, sync, ingestion — all degrade gracefully |
| **GUI-first** | Every configuration has a GUI. CLI and config files are secondary |
| **Universal standards** | FHIR, MCP, Markdown, OPDS, RSS, JSON-LD — open specs only |
| **Lightweight** | Zero Electron. No Java. No Python runtime bundled. Rust + native WebView |
| **Zero vendor lock-in** | All data in Markdown + plain files. Export any time. No proprietary format |
| **Zero cost** | Free forever. No freemium expiry. Apache 2.0 + CC BY 4.0 |
| **Doc as Code** | Documents are Git commits. Every save is versioned |
| **Diagram as Code** | Mermaid, PlantUML, D2 — all rendered live from text |
| **Context as Code** | `.znrc` config defines AI behavior, tools, and rules |
| **Data as Code** | YAML front matter, structured tables, CSV — all queryable |

---

## 4. Blueprint Structure

```
docs/                                   # Blueprint specifications (merged into zs-note)
├── README.md                           ← You are here
├── 001-concept/
│   ├── 001-vision.md                   ← Full vision and positioning
│   ├── 002-user-personas.md            ← Who ZarishNote is for
│   ├── 003-value-proposition.md        ← vs. Typora, Obsidian, Moraya, VS Code
│   └── 004-glossary.md                 ← Definitions of all ZarishNote terms
├── 002-specifications/
│   ├── 001-core-editor/
│   │   ├── 001-editor-spec.md          ← WYSIWYG, source, split view, formatting
│   │   ├── 002-file-manager-spec.md    ← Sidebar, tree, search, tags
│   │   └── 003-znrc-schema.md          ← Workspace config schema (full spec)
│   ├── 002-sandbox-engine/
│   │   └── 001-sandbox-spec.md         ← Wasmtime integration, capability model
│   ├── 003-ai-engine/
│   │   ├── 001-ai-chat-spec.md         ← Multi-provider, streaming, actions
│   │   ├── 002-ai-providers.md         ← All supported providers + config
│   │   └── 003-ai-templates.md         ← Reusable prompt templates spec
│   ├── 004-ingestion-engine/
│   │   ├── 001-ingestion-spec.md       ← Converter registry, priority model
│   │   ├── 002-format-matrix.md        ← All supported formats + strategies
│   │   └── 003-web-converters.md       ← YouTube, Wikipedia, RSS, SERP
│   ├── 005-mcp-engine/
│   │   ├── 001-mcp-spec.md             ← MCP client spec, transport, auth
│   │   ├── 002-mcp-marketplace.md      ← Discovery, install, sandboxing
│   │   └── 003-knowledge-bases.md      ← Local RAG + vector store spec
│   ├── 006-voice-engine/
│   │   └── 001-voice-spec.md           ← Transcription, speaker labels, export
│   ├── 007-publish-sync/
│   │   ├── 001-publish-spec.md         ← GitHub, custom API, RSS, image hosting
│   │   └── 002-sync-spec.md            ← Git auto-commit, conflict resolution
│   └── 008-plugin-system/
│       └── 001-plugin-spec.md          ← WASM plugin API, marketplace, signing
├── 003-architecture/
│   ├── 001-system-architecture.md      ← Tauri layers, IPC, data flow
│   ├── 002-directory-structure.md      ← Full codebase directory layout
│   ├── 003-tech-stack.md               ← All libraries, versions, rationale
│   └── 004-data-model.md               ← Local DB, vector store, Git history
├── 004-security/
│   ├── 001-threat-model.md             ← STRIDE analysis, mitigations
│   ├── 002-security-policy.md          ← Responsible disclosure, SLA
│   └── 003-sandbox-implementation.md  ← Wasmtime config, resource limits
├── 005-roadmap/
│   ├── 001-phase-one.md                ← Weeks 1–8: Core MVP
│   ├── 002-phase-two.md                ← Weeks 9–16: Full feature set
│   └── 003-phase-three.md              ← Weeks 17+: Ecosystem
├── 006-assets/
│   ├── 001-job-description-rust.md     ← For Tauri/Rust developer hire
│   ├── 002-job-description-python.md   ← For ingestion engine developer
│   └── 003-brand-guidelines.md         ← Name, colors, tone
└── 007-prototypes/
    ├── 001-zrc-examples/
    │   ├── default.znrc                ← Starter workspace config
    │   └── advanced.znrc               ← Full-featured config example
    └── 002-sandbox-hello/              ← hello.wasm source + instructions
```

---

## 5. Quick Reference — Feature Matrix

| Feature Group | Feature | V1 MVP | Phase 2 |
|---|---|---|---|
| **Editor** | WYSIWYG Markdown | ✅ | |
| | Source / split / preview view | ✅ | |
| | Tables, tasks, math, code blocks | ✅ | |
| | Mermaid, PlantUML, D2 diagrams | ✅ | |
| | Images (paste, drag, embed) | ✅ | |
| | Search and replace | ✅ | |
| | YAML front matter | ✅ | |
| | Multi-window, multi-tab | | ✅ |
| **File Manager** | Sidebar file tree | ✅ | |
| | Tags and folders | ✅ | |
| | Full-text search | ✅ | |
| | Drag-and-drop | ✅ | |
| **AI Chat** | Multi-provider streaming | ✅ | |
| | OpenAI, Claude, Gemini, DeepSeek, Ollama | ✅ | |
| | Insert / replace / copy / rewrite | ✅ | |
| | Translate, summarize, generate | ✅ | |
| | Reusable templates | ✅ | |
| | Image generation (DALL-E, Stable Diffusion) | | ✅ |
| | API key storage (OS Keychain) | ✅ | |
| **Sandbox** | Wasmtime isolation | ✅ | |
| | Per-tool capability model | ✅ | |
| | Network allow/deny list | ✅ | |
| | Memory + time limits | ✅ | |
| | Full-stack sandbox (HTTP, FS scope, state) | ✅ | |
| **Ingestion** | PDF, DOCX, PPTX, XLSX | ✅ | |
| | EPUB, CSV, Jupyter, ZIP | ✅ | |
| | YouTube, Wikipedia, RSS | ✅ | |
| | Image OCR + vision description | | ✅ |
| | Audio transcription | | ✅ |
| **MCP** | MCP client (stdio + HTTP) | ✅ | |
| | Tool calling | ✅ | |
| | MCP marketplace | | ✅ |
| | Knowledge bases (local RAG) | ✅ | |
| **Voice** | Whisper transcription | ✅ | |
| | Speaker labels | | ✅ |
| | Markdown export | ✅ | |
| **Publishing** | GitHub Pages / Repos | ✅ | |
| | Custom API endpoints | ✅ | |
| | RSS generation | ✅ | |
| | Image hosting (GitHub, Cloudflare) | ✅ | |
| **Sync** | Git auto-commit | ✅ | |
| | Offline-first | ✅ | |
| | Cross-device sync | | ✅ |
| **Platform** | Windows, macOS, Linux | ✅ | |
| | Android, iOS (Tauri v2) | | ✅ |
| | i18n (Bangla, English, Arabic, others) | | ✅ |

---

## 6. Technology Stack

| Layer | Technology | Version | Rationale |
|---|---|---|---|
| **Framework** | Tauri | v2.11+ | Rust backend, native WebView, 5–15MB binary, iOS/Android |
| **Frontend** | Svelte | 5.x | Reactive, zero-overhead, TypeScript-first |
| **Editor core** | Milkdown (ProseMirror) | 7.x | WYSIWYG Markdown, extensible, no heavy deps |
| **Syntax highlight** | Shiki | 1.x | Zero-runtime, accurate, supports 200+ languages |
| **Diagrams** | Mermaid.js | 11.x | Lazy-loaded, industry-standard |
| **Sandbox** | Wasmtime (Rust crate) | 45.x | WASM isolation, Cranelift JIT, memory limits |
| **Vector store** | LanceDB (local) | 0.x | Embedded, zero server, Rust native |
| **Embeddings** | FastEmbed-rs | latest | Local ONNX embeddings, no server |
| **Git engine** | git2-rs (libgit2) | 0.19+ | Auto-commit, diff, branch |
| **Key storage** | keytar (via Tauri plugin) | — | OS Keychain on all platforms |
| **Voice** | Whisper.cpp (via Tauri) | latest | On-device, no API needed for base model |
| **Ingestion (Python)** | markitdown (Microsoft) | 0.1.x | 139K+ stars, 15+ formats, MIT licensed |
| **Document convert** | mammoth, pdfminer.six | latest | DOCX → HTML, PDF text extraction |
| **XML safety** | defusedxml | latest | XXE prevention |
| **HTTP** | reqwest (Rust) | 0.12+ | Async HTTP with TLS |
| **State** | Svelte stores + Tauri state | — | No Redux, no extra runtime |
| **Build** | Vite | 8.x | Fast, ESM-native (Rolldown) |
| **Package** | pnpm | 11.x | Fast, disk-efficient |
| **CI/CD** | GitHub Actions | — | Cross-platform builds |

---

## 7. For Developers: Where to Start

If you are a developer picking up this blueprint:

1. Read `001-concept/001-vision.md` — understand the "why" first.
2. Read `003-architecture/001-system-architecture.md` — understand the overall shape.
3. Read `002-specifications/002-sandbox-engine/001-sandbox-spec.md` — the sandbox is the most critical differentiator; all AI tools must run through it.
4. Read `002-specifications/001-core-editor/003-znrc-schema.md` — the `.znrc` file is the "brain" of a workspace.
5. Start building Phase 1 tasks from `005-roadmap/001-phase-one.md`.

---

*ZarishSphere Foundation · V1 · June 2026*
*License: Apache 2.0 (code) · CC BY 4.0 (documentation)*
*GitHub: https://github.com/zarishsphere*
*Docs: https://zarishsphere.github.io/zs-note*
