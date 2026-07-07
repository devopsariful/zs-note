pub mod capability;
pub mod executor;
pub mod network;

use std::time::Instant;
use thiserror::Error;
use wasmtime::{Engine, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

use crate::logging::{audit_log, SandboxAuditEntry};
use crate::types::ToolConfig;

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("Module too large: {size} bytes exceeds limit of {limit} bytes")]
    ModuleTooLarge { size: usize, limit: usize },
    #[error("Memory limit exceeded: {usage} bytes")]
    MemoryLimit { usage: usize },
    #[error("Execution timed out after {timeout}ms")]
    Timeout { timeout: u64 },
    #[error("Network access blocked: {url}")]
    NetworkBlocked { url: String },
    #[error("Permission denied: {details}")]
    PermissionDenied { details: String },
    #[error("WASM runtime error: {0}")]
    Runtime(#[from] wasmtime::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

pub struct SandboxEngine {
    engine: Engine,
}

impl SandboxEngine {
    pub fn new() -> Self {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_value(true);
        config.wasm_memory64(false);
        config.wasm_reference_types(true);
        config.wasm_simd(true);
        config.wasm_bulk_memory(true);
        config.consume_fuel(true);
        config.epoch_interruption(true);

        let engine = Engine::new(&config).expect("Failed to create Wasmtime engine");
        Self { engine }
    }

    pub fn execute(
        &self,
        tool: &ToolConfig,
        func_name: &str,
        args_json: &str,
    ) -> Result<String, SandboxError> {
        let start = Instant::now();

        let wasm_bytes = std::fs::read(&tool.wasm_path)?;
        let module_size = wasm_bytes.len();

        if module_size > tool.memory_limit {
            return Err(SandboxError::ModuleTooLarge {
                size: module_size,
                limit: tool.memory_limit,
            });
        }

        let module = Module::new(&self.engine, &wasm_bytes)?;

        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build();

        let mut store = Store::new(&self.engine, wasi);
        store.set_epoch_deadline(tool.timeout);
        store.set_fuel(tool.memory_limit as u64)?;

        let instance = wasmtime::Instance::new(&mut store, &module, &[])?;

        let func = instance.get_func(&mut store, func_name).ok_or_else(|| {
            SandboxError::Other(format!("Function '{}' not found in module", func_name))
        })?;

        let params: Vec<wasmtime::Val> = vec![wasmtime::Val::I64(args_json.len() as i64)];
        let mut results = vec![wasmtime::Val::I64(0)];
        func.call(&mut store, &params, &mut results)?;

        let duration = start.elapsed().as_millis() as u64;

        audit_log(SandboxAuditEntry {
            timestamp: chrono::Utc::now(),
            tool_name: tool.name.clone(),
            action: func_name.to_string(),
            target: args_json.to_string(),
            duration_ms: duration,
            exit_code: 0,
            allowed: true,
            error: None,
        });

        Ok(match results.first() {
            Some(wasmtime::Val::I64(val)) => val.to_string(),
            Some(wasmtime::Val::I32(val)) => val.to_string(),
            _ => "0".to_string(),
        })
    }

    pub fn test_module(&self, wasm_bytes: &[u8]) -> Result<(), SandboxError> {
        Module::new(&self.engine, wasm_bytes)?;
        Ok(())
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }
}

impl Default for SandboxEngine {
    fn default() -> Self {
        Self::new()
    }
}
