//! ZarishNote integration test suite.
//!
//! This file is the top-level test module that re-exports all
//! integration test modules. Each sub‑module covers one subsystem.
//!
//! Run with:
//!   cargo test --test integration              # all integration tests
//!   cargo test --test integration test_config  # single module

#[path = "integration/test_config.rs"]
mod test_config;
#[path = "integration/test_editor.rs"]
mod test_editor;
#[path = "integration/test_git_engine.rs"]
mod test_git_engine;
#[path = "integration/test_mcp_protocol.rs"]
mod test_mcp_protocol;
#[path = "integration/test_sandbox_executor.rs"]
mod test_sandbox_executor;
#[path = "integration/test_vector_store.rs"]
mod test_vector_store;
