# Sandbox Hello World

A minimal WASM module for testing ZarishNote's Wasmtime sandbox.

## Prerequisites

- Rust with `wasm32-wasi` target:
  ```bash
  rustup target add wasm32-wasi
  ```

## Build

```bash
cargo init --lib --name hello hello-world
cd hello-world
```

Replace `src/lib.rs` with:

```rust
#[no_mangle]
pub extern "C" fn hello() -> i32 {
    42
}

#[no_mangle]
pub extern "C" fn greet(name_ptr: *const u8, name_len: u32) -> i32 {
    let name = unsafe {
        std::slice::from_raw_parts(name_ptr, name_len as usize)
    };
    let greeting = format!("Hello, {}!", std::str::from_utf8(name).unwrap());
    let bytes = greeting.as_bytes();
    // In production, copy to shared memory; for now return length
    bytes.len() as i32
}
```

Build:

```bash
cargo build --target wasm32-wasi --release
cp target/wasm32-wasi/release/hello.wasm ../hello.wasm
```

## Usage in ZarishNote

Place `hello.wasm` in your vault's `tools/` folder and configure in `.znrc`:

```yaml
tools:
  - name: "hello"
    description: "Sandbox hello world test"
    type: "wasm"
    wasm_path: "tools/hello.wasm"
    sandbox:
      memory_limit: "16MB"
      timeout: "5s"
      network: false
      permissions:
        - "write:stdout"
```

## Expected result

Calling the `hello` function returns `42`, confirming the sandbox is working.
