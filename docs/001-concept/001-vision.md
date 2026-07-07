# 001-vision.md
## ZarishNote — Full Vision & Positioning
### What it is, what it is not, and why it matters

**Document type:** Concept — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## 1. The One-Sentence Vision

ZarishNote is a **clean, elegant Markdown editor** that — when you need it — becomes a **fully sandboxed private AI agent** running entirely on your device, with no data leaving without your explicit permission.

---

## 2. The Writing Experience First

ZarishNote begins as a great writing tool. Before the AI, before the sandbox, before the MCP — the core is an editor that respects your flow.

- You open it and see text. Not panels, not icons, not banners.
- You type `#` and see a heading instantly. You type `- ` and a list begins.
- You can split the view to see rendered Markdown live.
- You can drop into raw source when you need precision.
- The sidebar shows your files. Search finds what you wrote. Tags group what belongs together.
- Tables, tasks, math (KaTeX), code blocks, images, and Mermaid diagrams are all native.

This is the foundation. Every other feature is built on top of, not instead of, this experience.

---

## 3. The AI Layer — Private, Powerful, Sandboxed

When you need help, the AI panel slides in from the right. It is connected to:

- OpenAI (GPT-4o, o3)
- Anthropic Claude (claude-sonnet-4-6, opus)
- Google Gemini (gemini-2.5-pro)
- DeepSeek
- Ollama (any local model: Llama, Mistral, Gemma, Phi, etc.)
- Any OpenAI-compatible custom endpoint

**What you can do from the AI panel:**

- Ask questions about your document
- Insert AI-generated text directly into the editor
- Replace a selected passage with a rewrite
- Translate selected text
- Summarize sections
- Generate images (DALL-E, Stability AI) and embed them
- Use reusable prompt templates (stored locally, synced with your workspace)

**What never happens:**

- Your API key is not stored in a config file. It is in your OS Keychain.
- Your document text does not touch any intermediary server.
- The AI conversation happens: Your Device → Provider API. Nothing in between.

---

## 4. The Sandbox — Every Tool is Isolated

The most important architectural decision in ZarishNote is this:

**Every AI tool, every MCP server, every plugin runs in a Wasmtime WASM sandbox.**

This means:
- No tool can read files outside the workspace root.
- No tool can make network requests unless you explicitly allow specific domains in `.znrc`.
- No tool can consume unlimited memory. Each has a cap (default 256MB).
- No tool can run forever. Each has a timeout (default 30s).

This is not theoretical isolation. Wasmtime is the same engine used in production by Fastly, Cloudflare Workers, and the Linux Foundation's WASI spec. It is audited, mature, and enforces strict boundaries.

The full-stack sandbox means:
- Tools can do HTTP (sandboxed, domain-restricted)
- Tools can read/write files (sandboxed, workspace-scoped)
- Tools can maintain state across calls (sandboxed, ephemeral or persistent)
- Tools work identically on Windows, macOS, Linux, Android, iOS

---

## 5. The Ingestion Engine — Every Document Becomes Context

ZarishNote includes a full document ingestion engine built on Microsoft's MarkItDown (MIT, 139K+ GitHub stars) and extended for ZarishNote's needs.

You can drop any of these into ZarishNote and it becomes clean Markdown:
- PDF, DOCX, PPTX, XLSX, XLS, EPUB, CSV, Jupyter notebooks, Outlook .msg, ZIP
- YouTube URLs (title + description + transcript)
- Wikipedia pages (title + summary + content)
- RSS/Atom feeds
- Bing search results
- Any HTML page

This Markdown then becomes part of your workspace and is automatically indexed into the local RAG vector store — making it available to the AI as context.

---

## 6. MCP — Universal Tool Protocol

ZarishNote is a first-class MCP (Model Context Protocol) client.

MCP is now the industry standard for connecting AI to tools — supported by Anthropic, OpenAI, Google DeepMind, Microsoft, and over 10,000 community servers. ZarishNote plugs into this ecosystem natively.

- Connect any MCP server (GitHub, Google Drive, Slack, custom APIs)
- Discover servers from the community marketplace
- Set up simple local MCP servers via GUI (no code required)
- Define knowledge bases that the AI can search
- All MCP tool calls run through the sandbox

---

## 7. Platform — Truly Cross-Platform, Truly Offline

ZarishNote is built with Tauri v2, which means:

- **Desktop:** Windows, macOS, Linux — native performance, 5–15MB binary
- **Mobile:** iOS and Android — same code, native WebView, same features
- **Offline:** Every feature works without internet. AI uses Ollama local models if you prefer. Sync is via Git (local or any remote).

The experience on mobile is optimized:
- Touch toolbar for Markdown formatting
- Tab bar for multi-file editing
- Magic Keyboard shortcut support on iPad
- Floating toolbar on touch surfaces

---

## 8. What ZarishNote Is Not

| What ZarishNote is NOT | Why |
|---|---|
| A Notion clone | No database views, no blocks, no collaboration by default |
| A knowledge graph / Obsidian clone | No bidirectional links as a primary feature (though supported via Markdown) |
| An Electron app | Zero Electron dependency |
| A cloud service | No ZarishNote account, no ZarishNote servers |
| A paid product | Free forever. Apache 2.0 |
| A Java-based tool | No Java, no JVM, no HAPI FHIR (RAM constraint on target hardware) |
| A Python-runtime-bundled app | Python is used only for the ingestion CLI; not bundled in the desktop app |

---

## 9. Inspiration and Differentiation

| Product | What ZarishNote borrows | What ZarishNote adds |
|---|---|---|
| **Moraya** | Tauri+Svelte, Milkdown editor, OS Keychain, MCP support, publishing | Wasmtime sandbox, ingestion engine, knowledge bases, voice, mobile |
| **Obsidian** | File-based vault, sidebar, plugin ecosystem | Sandbox, AI-native, Tauri (not Electron), free |
| **Typora** | Instant WYSIWYG, clean aesthetic | Open source, AI panel, sandbox, publishing |
| **VS Code** | Source view, extensions | Markdown-first, lightweight, no Node.js bundled |
| **MarkItDown (Microsoft)** | Format conversion engine | Integrated into desktop app with sandbox |
| **Zarish Blueprint** | .zrc schema, G2A engine, ZarishSphere principles | Applied to a focused note editor |

---

## 10. Name and Identity

**ZarishNote** (project code: `zs-note`)

- **Zarish** — from ZarishSphere Foundation, meaning "golden rain" (Urdu/Persian), symbolizing knowledge that nourishes
- **Note** — the simple, honest descriptor of the core function

The app presents itself as `ZarishNote` in all UI. The Tauri bundle ID is `com.zarishsphere.note`. The GitHub repository is `zarishsphere/zs-note`.

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
*GitHub: https://github.com/zarishsphere*