# 001-job-description-rust.md
## Job Description: Rust/Tauri Backend Developer
### ZarishNote — ZarishSphere Foundation

**Document type:** Asset — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0

---

## Position

**Title:** Rust/Tauri Backend Developer
**Type:** Contract (3–6 months, extendable)
**Location:** Remote
**Time commitment:** Full-time or part-time (20+ hrs/week)

---

## About ZarishNote

ZarishNote is a free, open-source, ultra-lightweight (~12MB) WYSIWYG Markdown editor with a sandboxed private AI assistant — built on Tauri v2 + Svelte 5 + Rust. See the full blueprint at `docs/` in the zs-note repo.

---

## Responsibilities

- Implement Tauri v2 backend commands and plugins
- Integrate Wasmtime sandbox for AI tool execution
- Build MCP client (stdio + HTTP transports) with JSON-RPC 2.0
- Implement Git engine (auto-commit, history, diff) via git2-rs
- Integrate LanceDB + FastEmbed for local RAG vector store
- Optimize for low-memory environments (8GB RAM target)
- Write tests and security regression checks for sandbox

---

## Requirements

### Required

- 3+ years Rust experience
- Familiarity with Tauri v2 (or willing to learn quickly)
- Experience with async Rust (tokio)
- Understanding of WebAssembly and WASI
- Comfortable with Git internals (libgit2/git2-rs)
- Linux development experience (target platform)

### Nice to Have

- Experience with Wasmtime or similar WASM runtimes
- Knowledge of MCP protocol (Model Context Protocol)
- Experience with vector databases (LanceDB, Qdrant)
- Cross-platform development (Windows, macOS, Linux)
- Tauri mobile (iOS/Android) experience

---

## Deliverables

| Item | Timeline |
|---|---|
| Sandbox engine integration (Wasmtime) | Week 1–2 |
| MCP client implementation | Week 2–3 |
| Git engine (auto-commit + history) | Week 2–4 |
| Vector store integration (LanceDB + FastEmbed) | Week 4–6 |
| AI provider clients (OpenAI, Claude, Gemini) | Week 4–8 |
| Performance optimization + testing | Ongoing |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
