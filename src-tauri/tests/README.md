# ZarishNote Integration Tests

This directory contains the integration test suite for the ZarishNote
back‑end (the `zs_note_lib` crate).  Unit tests are co‑located with
source files as `#[cfg(test)] mod tests { … }` blocks.

## Prerequisites

- Rust toolchain (edition 2024, minimum Rust 1.85+)
- The `tempfile` crate (listed in `[dev-dependencies]`)

## Running Tests

```bash
# Run ALL integration tests
cargo test --test integration

# Run a specific test module
cargo test --test integration test_vector_store
cargo test --test integration test_config
cargo test --test integration test_editor
cargo test --test integration test_git_engine
cargo test --test integration test_mcp_protocol
cargo test --test integration test_sandbox_executor

# Run a single test by name
cargo test --test integration test_vector_store_index_and_query
cargo test --test integration test_config_validate

# Run unit tests too
cargo test

# Run with full output (no capture)
cargo test -- --nocapture

# Run with verbose logging (requires RUST_LOG env)
RUST_LOG=debug cargo test --test integration
```

## Test Architecture

| Test module | What it covers |
|---|---|
| `test_config` | `Config` default construction, validation, YAML round‑trip, save/load |
| `test_editor` | Vault path traversal prevention via `resolve_virtual_path` |
| `test_git_engine` | Commit‑message generation (`generate_commit_message`), change‑type detection |
| `test_mcp_protocol` | JSON‑RPC parsing (request, response, error, notification), helper builders, `McpToolRouter` routing and confirmation logic |
| `test_sandbox_executor` | `CapabilityChecker` (permissions, network, path), `glob_match`, `SandboxNetworkProxy` allow/block logic |
| `test_vector_store` | `VectorStore` index/query/delete/rebuild, chunking, deduplication, empty‑store behaviour |

## Design Principles

1. **No external runtime** — tests never start Tauri, open a window, or
   make network calls.
2. **No hard‑coded paths** — all filesystem operations use `tempfile::TempDir`.
3. **Public API only** — integration tests exercise the public interface of
   `zs_note_lib`; private implementation details are left to unit tests.
4. **Focused** — each test covers exactly one behaviour.
5. **Deterministic** — tests are self‑contained and do not rely on global
   state or environment variables.
