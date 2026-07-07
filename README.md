# ZarishNote

> **A free, open-source, ultra-lightweight WYSIWYG Markdown editor with a sandboxed private AI assistant.**

[![GitHub](https://img.shields.io/badge/zarishsphere/zs--note-181717?logo=github)](https://github.com/zarishsphere/zs-note)
[![CI](https://github.com/zarishsphere/zs-note/actions/workflows/ci.yml/badge.svg)](https://github.com/zarishsphere/zs-note/actions/workflows/ci.yml)

ZarishNote is a desktop note-taking application that combines a rich Markdown editor (Milkdown/ProseMirror) with a sandboxed Wasmtime-based AI assistant — all running locally on your machine.

---

## Repository Structure

| Directory | Purpose |
|---|---|
| `src/` | Svelte 5 frontend |
| `src-tauri/` | Rust backend (Tauri v2) |
| `ingestion/` | Python document ingestion engine |
| `docs/` | Blueprint specifications, roadmaps, architecture docs |

---

## Tech Stack

| Layer | Technology |
|---|---|
| Framework | Tauri v2 (Rust backend, native WebView) |
| Frontend | Svelte 5 + TypeScript 6 + Vite 8 |
| Package manager | pnpm 11.x |
| Editor core | Milkdown 7.x (ProseMirror) |
| Sandbox | Wasmtime 46.x (Rust crate) |
| Vector store | In-memory HashMap + fastembed |
| Ingestion | Python CLI (MarkItDown + custom converters) |
| CI/CD | GitHub Actions |

---

## Getting Started

*Prerequisites: Node.js ≥24, pnpm ≥11, Rust toolchain (rustup), Tauri system dependencies.*

```bash
# Clone
git clone git@github.com:zarishsphere/zs-note.git
cd zs-note

# Install frontend deps
pnpm install

# Run in dev mode
pnpm tauri dev
```

> **Status:** CI pipeline is fully green — `cargo fmt`, `cargo clippy`, `cargo test` (107/107), `pnpm typecheck`, `pytest` (41/41), and `ruff` all pass. The project compiles and tests pass on all three platforms (ubuntu-latest, macos-latest, windows-latest).
>
> **Note:** v0.2.0 Stable Release — see [GitHub Releases](https://github.com/zarishsphere/zs-note/releases).

---

## Project Map

```
zs-note/
├── src/                          # Svelte 5 frontend
│   ├── main.ts                   # Entry point
│   ├── app.css                   # Global styles / CSS variables
│   └── lib/
│       ├── components/           # Svelte components
│       │   ├── App.svelte        # Root layout
│       │   ├── Editor.svelte     # Editor orchestrator (WYSIWYG/Source/Split)
│       │   ├── MilkdownEditor.svelte
│       │   ├── SourceEditor.svelte
│       │   ├── Sidebar.svelte    # File tree + search + tags
│       │   ├── AIPanel.svelte    # AI chat panel
│       │   ├── Settings.svelte   # Settings modal
│       │   ├── ...               # 16 more components
│       │   └── Modal.svelte      # Reusable modal wrapper
│       ├── stores/               # Svelte 5 rune stores (.svelte.ts)
│       ├── commands/             # Tauri invoke wrappers
│       ├── milkdown/             # Milkdown editor setup
│       └── types.ts              # TypeScript interfaces
├── src-tauri/                    # Rust backend (Tauri)
│   └── src/
│       ├── main.rs / lib.rs      # Entry + plugin registration
│       ├── commands/             # Tauri command handlers
│       │   ├── editor.rs         # File CRUD
│       │   ├── ai.rs             # AI chat (OpenAI, Claude, Gemini, Ollama)
│       │   ├── sandbox.rs        # WASM execution
│       │   ├── git.rs            # Git operations
│       │   ├── ingest.rs         # Ingestion subprocess
│       │   ├── mcp.rs            # MCP tool routing
│       │   └── search.rs / voice.rs / config.rs
│       ├── sandbox/              # Wasmtime sandbox engine
│       ├── ai/                   # AI provider implementations
│       ├── git/                  # Git engine (commit, history, sync)
│       ├── mcp/                  # MCP client (transport, protocol, router)
│       ├── vector/               # Vector store (index, query)
│       ├── config.rs / types.rs / logging.rs
│       └── capabilities/         # Tauri v2 capability permissions
├── ingestion/                    # Python ingestion engine
│   └── src/zarishnote_ingest/
│       ├── cli.py                # CLI entry point
│       ├── markitdown.py         # MarkItDown wrapper
│       ├── converters/           # 12 format converters
│       └── ...
├── package.json / tsconfig.json / vite.config.ts / svelte.config.js
└── Cargo.toml / tauri.conf.json / build.rs
```

---

## Build Status

All CI checks pass on `main`:

| Check | Status |
|---|---|
| Format (`cargo fmt --check`) | ✅ Passes |
| Lint (`cargo clippy`) | ✅ Passes (warnings allowed) |
| Tests (`cargo test`) | ✅ 107/107 pass |
| TypeScript (`pnpm typecheck`) | ✅ Passes |
| Python lint (ruff) | ✅ Passes |

## Features (Planned / In Progress)

See the [blueprint TODO](docs/TODO.md) for a complete status breakdown.

| Feature | Status |
|---|---|
| WYSIWYG Markdown editor | 🏗 Scaffolded |
| Source + Split modes | 🏗 Scaffolded |
| File tree + vault manager | 🏗 Scaffolded |
| Git auto-commit + history | 🏗 Scaffolded |
| Wasmtime sandbox | 🏗 Scaffolded |
| AI chat (OpenAI, Claude, Gemini, Ollama) | 🏗 Scaffolded |
| Document ingestion (PDF, DOCX, ...) | 🏗 Scaffolded |
| MCP tool integration | 🏗 Scaffolded |
| Vector store / RAG | 🏗 Scaffolded |
| Voice dictation | 🏗 Scaffolded |
| Publish to GitHub | 🏗 Scaffolded |

---

## Architecture

The application follows a three-layer architecture:

1. **Rust Backend** (`src-tauri/`) — Tauri v2 commands handle file I/O, Git operations, sandboxed WASM execution, AI provider communication, MCP tool routing, and vector store indexing.
2. **Svelte Frontend** (`src/`) — Reactive UI with Milkdown/ProseMirror for the editor, file tree sidebar, AI chat panel, settings, and modals.
3. **Python Ingestion Engine** (`ingestion/`) — Standalone CLI that converts documents (PDF, DOCX, PPTX, XLSX, EPUB, HTML, CSV, Jupyter, YouTube, Wikipedia, RSS, SERP) to Markdown via MarkItDown + custom converters.

All AI tools, MCP servers, and plugins execute inside a Wasmtime sandbox with configurable capabilities (filesystem scoping, network allow-list, memory limits, timeouts).

---

## Contributing

1. Read the [blueprint specifications](docs/README.md) to understand the design.
2. Check the [TODO](docs/TODO.md) for open tasks.
3. File issues at [github.com/zarishsphere/zs-note/issues](https://github.com/zarishsphere/zs-note/issues).

---

## License

MIT — see [LICENSE](LICENSE) (pending).

---

*Built by ZarishSphere Foundation*
