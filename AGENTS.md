# ZarishNote — Agent Guide

Source: <https://github.com/zarishsphere/zs-note>
Blueprint specs: `docs/` (39 Markdown files, read-only reference)

## Developer commands

| Command | What it does |
|---|---|
| `pnpm dev` | Vite dev server only (port 1420) — no Tauri window |
| `pnpm tauri dev` | Full Tauri dev (Vite + native window) |
| `pnpm build` | `vite build && tauri build` — produces installer bundles |
| `pnpm typecheck` | `tsc --noEmit` |
| `pnpm lint` | `tsc --noEmit && eslint src/` (no ESLint config exists yet) |
| `cargo test` | Rust unit + integration tests (from `src-tauri/`) |
| `cargo test --test integration` | Integration tests only |
| `cargo test --test integration <module>` | Single module: `test_config`, `test_editor`, `test_git_engine`, `test_mcp_protocol`, `test_sandbox_executor`, `test_vector_store` |
| `pip install -e ".[all]"` | Install Python ingestion CLI (from `ingestion/`) |
| `zarishnote-ingest` | Python CLI entry point |
| `pytest -v --tb=short` | Python tests (from `ingestion/`) |
| `ruff check src/` | Python lint (from `ingestion/`) |

**CI gate (run before push):** `cargo fmt --check && cargo clippy && cargo test && pnpm typecheck`

## Repo structure

Three disconnected sub-projects (no cross-building):

- `src/` — Svelte 5 + TypeScript 6 frontend
- `src-tauri/` — Rust backend (Tauri v2, Wasmtime 45, LanceDB)
- `ingestion/` — Python CLI (MarkItDown + custom converters)

Each has its own test/lint config. CI runs all three as parallel jobs.

## Svelte 5 + TypeScript 6 quirks

- **Svelte 5 runes mode** — stores use `.svelte.ts` extension with `$state`, `$derived`, `$effect` runes. Do not import from `svelte/store`.
- **Mount API:** `import { mount } from 'svelte'` — no `new App()` or `createApp`.
- **`compilerOptions.runes: true`** in `svelte.config.js` — Svelte 4 legacy syntax will fail.
- **TypeScript 6 strictness:** `noUncheckedIndexedAccess` (array access returns `T | undefined`), `noImplicitOverride` (must use `override` keyword), `noPropertyAccessFromIndexSignature`, `verbatimModuleSyntax` (requires `import type` for type-only imports).
- **Alias:** `$lib/*` → `src/lib/*`.
- **Svelte files included in tsconfig:** `src/**/*.svelte` and `src/**/*.svelte.ts`.

## Rust / Tauri v2 quirks

- **Crate types:** `["lib", "cdylib", "staticlib"]` — needed for mobile (iOS/Android) targets.
- **Lib entry:** `src-tauri/src/lib.rs` exports `pub fn run()`; `main.rs` calls it.
- **Devtools feature:** `tauri = { features = ["devtools"] }` — enables Tauri devtools in dev mode.
- **CSP:** `null` in `tauri.conf.json` — wide open for dev; restrict before release.
- **Voice feature gate:** `#[cfg(feature = "voice")]` on all voice commands. Cargo feature `voice` enables `cpal`, `hound`, `rodio`. Not in default set. `whisper-rs` is also optional but not gated behind `voice`.
- **All Tauri commands registered manually** in `invoke_handler!(generate_handler![...])` at `lib.rs:44-135`. Adding a new command requires adding to this list.
- **Shared state** via `AppState` struct managed with `Arc<RwLock<...>>`; injected via `.manage(state)`.
- **Capabilities** in `src-tauri/capabilities/default.json` control filesystem scope (`$APPDATA`, `$RESOURCE`, `$HOME`, `$TEMP`).
- **Tests** never start Tauri, never make network calls, use `tempfile::TempDir` for filesystem ops. Unit tests co-located with source as `#[cfg(test)]`.

## Ingestion / Python

- Install for dev: `pip install -e ".[all]"` from `ingestion/`.
- Test/lint from `ingestion/` directory: `pytest`, `ruff check src/`.
- Entry: `zarishnote_ingest.cli:main` → CLI binary `zarishnote-ingest`.

## Verification order (CI mirror)

```
cargo fmt --check          # from src-tauri/
cargo clippy               # from src-tauri/ (warnings allowed)
cargo test                 # from src-tauri/
pnpm typecheck             # from repo root (frozen lockfile in CI)
ruff check src/            # from ingestion/ (warnings allowed)
pytest -v --tb=short       # from ingestion/
```

## GitHub identity

- Active GH user: **codeandbrain**. Use `gh auth status` to verify.
- Org: **zarishsphere** — repo origin is `git@github.com:zarishsphere/zs-note.git`.
- All org members (arwazarish, devopsariful, healthbgd, bgd-cpms) are the same person's aliases.
- `opencode.json` and `.agent/` are **gitignored** — agent config is local-only, never committed.

## Key architecture constraints

- All AI tools, MCP servers, and plugins execute in **Wasmtime sandbox** with capability-based permissions. Never bypass.
- API keys go in **OS Keychain** (via `keyring` crate), never in config files or `.env`.
- The **editor core** is Milkdown 7 (ProseMirror) — custom plugins use Milkdown's API, not ProseMirror directly.
- `.znrc` is the workspace config file — parsed by `serde_yaml` in Rust; every setting has both GUI and `.znrc` equivalent.
