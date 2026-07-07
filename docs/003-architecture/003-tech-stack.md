# 003-tech-stack.md
## ZarishNote Technology Stack
### All libraries, versions, and rationale

**Document type:** Architecture — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Runtime Layer](#1-runtime-layer)
2. [Backend (Rust) Libraries](#2-backend-rust-libraries)
3. [Frontend Libraries](#3-frontend-libraries)
4. [Python Dependencies](#4-python-dependencies)
5. [Development Toolchain](#5-development-toolchain)
6. [Rationale Notes](#6-rationale-notes)

---

## 1. Runtime Layer

| Technology | Version | Purpose |
|---|---|---|
| **Tauri** | v2.11+ | Desktop/mobile framework: Rust backend + native WebView |
| **Rust** | 2024 edition | Backend language |
| **TypeScript** | 6.x | Frontend language |
| **Svelte** | 5.x | Reactive UI framework |
| **Vite** | 8.x | Build tool, HMR dev server (Rolldown) |
| **pnpm** | 11.x | Package manager |
| **Node.js** | 24 LTS | Required for build tooling only |

---

## 2. Backend (Rust) Libraries

### 2.1 Core

| Crate | Version | Purpose |
|---|---|---|
| `tauri` | 2.x | Application framework |
| `tauri-plugin-shell` | 2.x | Subprocess spawning (MCP, ingestion) |
| `tauri-plugin-dialog` | 2.x | File open/save dialogs |
| `tauri-plugin-fs` | 2.x | Filesystem access with capabilities |
| `tauri-plugin-notification` | 2.x | Desktop notifications |
| `serde` / `serde_json` | 1.x | Serialization |
| `serde_yaml` | 0.9 | `.znrc` parsing |
| `anyhow` | 1.x | Error handling |
| `thiserror` | 1.x | Custom error types |
| `tokio` | 1.x | Async runtime |
| `tracing` | 0.1 | Structured logging |

### 2.2 Editor

| Crate | Version | Purpose |
|---|---|---|
| `milkdown` | 7.x | ProseMirror-based Markdown editor framework (frontend) |
| `shiki` | 1.x | Syntax highlighting (frontend, WASM build) |

### 2.3 Sandbox

| Crate | Version | Purpose |
|---|---|---|
| `wasmtime` | 45.x | WebAssembly runtime |
| `wasmtime-wasi` | 45.x | WASI support for sandboxed tools |
| `wasmtime-wasi-http` | 45.x | Sandboxed HTTP access |

### 2.4 AI

| Crate | Version | Purpose |
|---|---|---|
| `reqwest` | 0.12 | Async HTTP client with TLS |
| `reqwest-eventsource` | 0.5 | SSE streaming for AI responses |
| `tiktoken-rs` | 0.5 | Token counting (OpenAI-compatible) |

### 2.5 Vector Store

| Crate | Version | Purpose |
|---|---|---|
| `lancedb` | 0.x | Embedded vector database |
| `fastembed-rs` | latest | Local ONNX embeddings |

### 2.6 Git

| Crate | Version | Purpose |
|---|---|---|
| `git2` (libgit2) | 0.19 | Git operations |
| `ssh-keychain-agent` | custom | SSH key management via OS keychain |

### 2.7 Voice

| Crate | Version | Purpose |
|---|---|---|
| `whisper-rs` | 0.13 | Whisper.cpp Rust bindings |
| `hound` | 3.x | WAV file reading/writing |
| `cpal` | 0.15 | Audio capture (optional, for live recording) |

### 2.8 MCP

| Crate | Version | Purpose |
|---|---|---|
| `jsonrpsee` | 0.24 | JSON-RPC 2.0 client for HTTP transport |
| `tokio-process` | (stdlib) | stdio subprocess management |

---

## 3. Frontend Libraries

| Package | Version | Purpose |
|---|---|---|
| `@tauri-apps/api` | 2.x | Tauri IPC bindings |
| `@tauri-apps/plugin-*` | 2.x | Tauri plugin client stubs |
| `milkdown` | 7.x | WYSIWYG Markdown editor |
| `@milkdown/*` | 7.x | Milkdown presets and plugins |
| `prosemirror-*` | (bundled) | ProseMirror core |
| `mermaid` | 11.x | Diagram rendering |
| `katex` | 0.17 | Math rendering |
| `shiki` | 1.x | Syntax highlighting |
| `@shikijs/transformers` | 1.x | Shiki transformers |

---

## 4. Python Dependencies

| Package | Version | Purpose |
|---|---|---|
| `markitdown[all]` | 0.1.x | Core document conversion engine |
| `defusedxml` | latest | XML/XXE security |
| `charset-normalizer` | latest | Text encoding detection |
| `markdownify` | latest | HTML → Markdown |
| `pdfminer.six` | latest | PDF text extraction |
| `mammoth` | latest | DOCX → HTML conversion |
| `python-pptx` | latest | PPTX slide extraction |
| `pandas` | latest | XLSX/XLS data handling |
| `openpyxl` | latest | XLSX workbook parsing |
| `xlrd` | latest | Legacy XLS parsing |
| `ebooklib` | latest | EPUB extraction |
| `youtube-transcript-api` | latest | YouTube transcript download |
| `feedparser` | latest | RSS/Atom feed parsing |
| `beautifulsoup4` | latest | HTML parsing for web converters |

---

## 5. Development Toolchain

| Tool | Purpose |
|---|---|
| `cargo` | Rust build/test |
| `cargo-tauri` | Tauri dev/build commands |
| `pnpm dev` | Frontend dev server |
| `pnpm build` | Production build |
| `cargo clippy` | Rust linting |
| `cargo fmt` | Rust formatting |
| `pnpm lint` | TypeScript linting |
| `pnpm typecheck` | TypeScript type checking |
| `GitHub Actions` | CI/CD for all platforms |
| `dprint` | Rust + TypeScript formatting (optional) |

---

## 6. Rationale Notes

### 6.1 Why Tauri vs. Electron

| Factor | Tauri v2 | Electron |
|---|---|---|
| Binary size | 5-15MB | 150-200MB |
| RAM idle | ~15MB | 200-400MB |
| Memory safety | Rust (safe by default) | Node.js (GC, unsafe) |
| Mobile | iOS + Android | None |
| Update mechanism | Built-in | External |

### 6.2 Why Svelte vs. React/Vue

- Smaller bundle: no virtual DOM runtime
- Less boilerplate: runes vs. hooks vs. reactivity system
- Closer to vanilla JS/TS mental model
- Tauri examples predominantly use Svelte

### 6.3 Why Milkdown vs. TipTap/Codemirror

- ProseMirror-based (same engine as TipTap)
- No React dependency (TipTap requires React)
- Built-in Markdown parsing and serialization
- Plugin system compatible with ZarishNote's WASM model

### 6.4 Why LanceDB vs. Qdrant/Chroma

- Embedded: no server process, no network
- Rust-native: direct integration without FFI
- Zero external dependencies for basic usage
- Lance columnar format optimized for ANN search

### 6.5 Why Wasmtime vs. wasm3/wasm-micro-runtime

- Cranelift JIT: good balance of startup speed and execution speed
- Full WASI support: required for sandboxed HTTP, filesystem
- Production-proven: used by Fastly, Cloudflare Workers
- Rust-native: maintainable by the same team writing the backend

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
