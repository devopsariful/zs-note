# ZarishNote

> **A free, open-source, ultra-lightweight WYSIWYG Markdown editor with a sandboxed private AI assistant.**

[![GitHub](https://img.shields.io/badge/devopsariful/zs--note-181717?logo=github)](https://github.com/devopsariful/zs-note)
[![CI](https://github.com/devopsariful/zs-note/actions/workflows/ci.yml/badge.svg)](https://github.com/devopsariful/zs-note/actions/workflows/ci.yml)

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

*Prerequisites: Node.js ≥24.15, pnpm ≥11.5, Rust toolchain (rustup), Tauri system dependencies.*

```bash
# Clone
git clone https://github.com/devopsariful/zs-note.git
cd zs-note

# Install frontend deps
pnpm install

# Run in dev mode
pnpm dev:tauri

# Or just run Vite dev server (without Tauri window)
pnpm dev
```

> **Status:** Current version — v0.2.0  
> **CI Status:** See [GitHub Actions](https://github.com/devopsariful/zs-note/actions) for latest build status.

---

## Project Map

```
zs-note/
├── src/                          # Svelte 5 frontend
│   ├── main.ts                   # Entry point
│   ├── app.css                   # Global styles / CSS variables
│   └── lib/
│       ├── components/           # Svelte components
│       ├── stores/               # Svelte 5 rune stores (.svelte.ts)
│       ├── commands/             # Tauri invoke wrappers
│       ├── milkdown/             # Milkdown editor setup
│       └── types.ts              # TypeScript interfaces
├── src-tauri/                    # Rust backend (Tauri v2)
│   ├── src/
│   │   ├── main.rs / lib.rs      # Entry + plugin registration
│   │   ├── commands/             # Tauri command handlers
│   │   ├── sandbox/              # Wasmtime sandbox engine
│   │   ├── ai/                   # AI provider implementations
│   │   ├── git/                  # Git engine
│   │   ├── mcp/                  # MCP client
│   │   ├── vector/               # Vector store
│   │   ├── config.rs / types.rs  # Configuration and types
│   │   └── capabilities/         # Tauri v2 capability permissions
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── tests/                    # Integration tests
├── ingestion/                    # Python ingestion engine
│   ├── src/zarishnote_ingest/
│   │   ├── cli.py                # CLI entry point
│   │   ├── converters/           # Format converters
│   │   └── ...
│   └── pyproject.toml
├── docs/                         # Documentation
│   ├── README.md                 # Blueprint specification
│   ├── TODO.md                   # Implementation status
│   ├── ZARISHNOTE-COMPLETE-GUIDE.md
│   └── [subdirectories for specs]
├── scripts/
│   └── check-consistency.sh      # Repository consistency checker
├── package.json                  # Frontend package config
├── tsconfig.json                 # TypeScript config
├── pnpm-workspace.yaml           # pnpm workspace config
├── vite.config.ts                # Vite build config
└── svelte.config.js              # Svelte config
```

---

## Build Status

See [CI Status](https://github.com/devopsariful/zs-note/actions/workflows/ci.yml) for latest results.

| Check | Tool |
|---|---|
| Format | `cargo fmt --check` |
| Lint | `cargo clippy` |
| Tests | `cargo test` |
| TypeScript | `pnpm typecheck` |
| Python | `ruff check src/` |

## Features (Planned / In Progress)

See the [blueprint TODO](docs/TODO.md) for detailed status breakdown.

| Feature | Status |
|---|---|
| WYSIWYG Markdown editor | 🏗 Scaffolded |
| Source + Split modes | 🏗 Scaffolded |
| File tree + vault manager | 🏗 Scaffolded |
| Git auto-commit + history | 🏗 Scaffolded |
| Wasmtime sandbox | 🏗 Scaffolded |
| AI chat (OpenAI, Claude, Gemini, Ollama) | 🏗 Scaffolded |
| Document ingestion | 🏗 Scaffolded |
| MCP tool integration | 🏗 Scaffolded |
| Vector store / RAG | 🏗 Scaffolded |
| Voice dictation | 🏗 Scaffolded |
| Publish to GitHub | 🏗 Scaffolded |

---

## Architecture

The application follows a three-layer architecture:

1. **Rust Backend** (`src-tauri/`) — Tauri v2 commands handle file I/O, Git operations, sandboxed WASM execution, AI provider communication, MCP tool routing, and vector store indexing.
2. **Svelte Frontend** (`src/`) — Reactive UI with Milkdown/ProseMirror for the editor, file tree sidebar, AI chat panel, settings, and modals.
3. **Python Ingestion Engine** (`ingestion/`) — Standalone CLI that converts documents (PDF, DOCX, PPTX, XLSX, EPUB, HTML, CSV, Jupyter, YouTube, Wikipedia, RSS, SERP) to Markdown.

All AI tools, MCP servers, and plugins execute inside a Wasmtime sandbox with configurable capabilities (filesystem scoping, network allow-list, memory limits, timeouts).

---

## Development

### Run consistency checks

```bash
# Check for inconsistencies
pnpm check:consistency

# Auto-fix common issues
pnpm check:consistency --fix
```

### Run all tests

```bash
# Frontend
pnpm typecheck

# Backend
cd src-tauri
cargo test

# Python ingestion
cd ingestion
pytest -v
```

---

## Contributing

1. Read the [blueprint specifications](docs/README.md) to understand the design.
2. Check the [TODO](docs/TODO.md) for open tasks.
3. File issues at [github.com/devopsariful/zs-note/issues](https://github.com/devopsariful/zs-note/issues).
4. Run `pnpm check:consistency` before committing.

---

## License

MIT — see [LICENSE](LICENSE)

---

*Maintained by devopsariful*
