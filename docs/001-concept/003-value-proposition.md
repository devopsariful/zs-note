# 003-value-proposition.md
## ZarishNote Value Proposition
### vs. Typora, Obsidian, Moraya, VS Code, and Notion

**Document type:** Concept — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## 1. Competitive Landscape

| Product | Category | Tech | Price | AI | Sandbox |
|---|---|---|---|---|---|
| **ZarishNote** | AI-native Markdown editor | Tauri + Svelte | Free | Built-in | Wasmtime |
| **Typora** | WYSIWYG Markdown | Electron | $15 one-time | None | None |
| **Obsidian** | Knowledge base | Electron | Free (core) | Community plugins | None |
| **Moraya** | AI writing tool | Tauri + Svelte | Free | Built-in | None |
| **VS Code** | Code editor | Electron | Free | Extensions | None |
| **Notion** | All-in-one workspace | Web | Free tier | Built-in | None |

---

## 2. Where ZarishNote Wins

### 2.1 vs. Typora

| Dimension | Typora | ZarishNote |
|---|---|---|
| RAM usage | 200-400MB (Electron) | ~15MB (Tauri) |
| Installer size | ~150MB | ~12MB |
| AI | None | Built-in, multi-provider |
| Sandbox | None | Wasmtime isolation |
| Ingestion | None | 20+ formats → Markdown |
| Voice | None | Whisper.cpp offline |
| Price | $15 one-time | Free |

ZarishNote is what Typora would be if it were rebuilt today: open source, lightweight, and AI-native.

### 2.2 vs. Obsidian

| Dimension | Obsidian | ZarishNote |
|---|---|---|
| Framework | Electron | Tauri v2 |
| RAM idle | ~300MB | ~15MB |
| Installer | ~100MB | ~12MB |
| AI | Community plugins only | Built-in, sandboxed |
| Plugin safety | Full filesystem access | Wasmtime sandbox |
| Mobile | Electron-based | Tauri native |
| Price | Free (core), $25+ Sync | Free (everything) |

Obsidian is a knowledge graph tool. ZarishNote is a writing tool with AI. The sandbox is the critical differentiator: Obsidian plugins can read your entire disk; ZarishNote tools cannot.

### 2.3 vs. Moraya

| Dimension | Moraya | ZarishNote |
|---|---|---|
| Sandbox | None | Wasmtime full-stack sandbox |
| Ingestion | None | MarkItDown + 20 formats |
| Knowledge bases | None | LanceDB RAG |
| Voice | None | Whisper.cpp |
| Mobile | Desktop only | iOS + Android (Phase 2) |

Moraya pioneered the Tauri+Svelte+Milkdown editor stack. ZarishNote builds on that foundation by adding sandbox isolation, document ingestion, knowledge bases, voice transcription, and mobile support.

### 2.4 vs. VS Code

| Dimension | VS Code | ZarishNote |
|---|---|---|
| Primary purpose | Code editor | Markdown editor |
| Framework | Electron | Tauri v2 |
| RAM idle | ~400MB | ~15MB |
| Markdown preview | Read-only preview | WYSIWYG live editing |
| AI | Extensions (Copilot, etc.) | Built-in, sandboxed |
| Bundle | Bundles Node.js | No bundled runtime |

VS Code is for code. ZarishNote is for writing. The Markdown experience in ZarishNote is fundamentally richer (WYSIWYG, diagrams live, voice input, AI-native) while using a fraction of the resources.

### 2.5 vs. Notion

| Dimension | Notion | ZarishNote |
|---|---|---|
| Architecture | Web app + Electron | Native Tauri |
| Offline | Limited | Full offline-first |
| Privacy | Cloud | Local-first, optional Git |
| Format | Proprietary blocks | Plain Markdown |
| AI | $10/month add-on | Free, bring your own key |

Notion solves team collaboration and databases. ZarishNote solves personal writing and private AI. They are complementary rather than competitive.

---

## 3. The Sandbox Advantage

The single feature no other Markdown editor offers: **every piece of non-core code runs in Wasmtime.**

This means:
- AI tools cannot read your SSH keys or `.env` files
- MCP servers cannot make network calls to unauthorized domains
- Plugins cannot consume unlimited memory or CPU
- Third-party code cannot access the OS keychain

In Obsidian, VS Code, or Typora, a plugin can exfiltrate your entire filesystem. In ZarishNote, it simply cannot.

---

## 4. The Cost Advantage

| Product | 3-year cost | Data control |
|---|---|---|
| Typora | $15 | Full local |
| Obsidian + Sync | $100+ | Varies |
| Notion AI | $360+ | Cloud only |
| VS Code + Copilot | $300+ | Varies |
| **ZarishNote** | **$0** | **Full local** |

ZarishNote is free forever. No freemium tier, no enterprise upsell, no cloud dependency.

---

## 5. The Privacy Advantage

| Scenario | Other editors | ZarishNote |
|---|---|---|
| AI chat | Data goes to app vendor + AI provider | Your device → AI provider directly |
| Document storage | Cloud by default | Local by default |
| Telemetry | Often on by default | None |
| API keys | Config files | OS Keychain only |

ZarishNote has no intermediary servers. There is no "ZarishNote cloud." Your documents never touch a server owned by the project.

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
