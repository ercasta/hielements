# WASM Plugins Guide

**Status**: 🚧 Infrastructure Ready - Full Implementation Coming Soon

This guide explains the planned WebAssembly (WASM) plugin support for Hielements. WASM plugins provide strong security sandboxing and near-native performance for library extensions.

---

## Table of Contents

1. [Overview](#overview)
2. [Why WASM Plugins?](#why-wasm-plugins)
3. [Current Status](#current-status)
4. [Configuration](#configuration)
5. [WASM Plugin Architecture](#wasm-plugin-architecture)
6. [Writing WASM Plugins](#writing-wasm-plugins)
7. [Build Process](#build-process)
8. [Security Model](#security-model)
9. [Performance Considerations](#performance-considerations)
10. [Migration from External Process Plugins](#migration-from-external-process-plugins)

---

## Overview

WASM plugins are an alternative to external process plugins that provide:
- **Strong sandboxing** via WebAssembly's capability-based security
- **Near-native performance** (typically 10-50% slower than native, but much faster than interpreted languages)
- **Easy distribution** (single `.wasm` file works on all platforms)
- **Controlled resource access** (explicit permissions for filesystem, network, etc.)

WASM plugins use the same protocol as external process plugins (JSON-RPC style) but run within a sandboxed WASM runtime instead of separate processes.

---

## Why WASM Plugins?

### Security

**External Process Plugins:**
- Run with full operating system permissions
- Can access any file, network resource, or system API
- Trust model: "trust the plugin completely"

**WASM Plugins:**
- Run in sandboxed environment with no default permissions
- Filesystem access limited to workspace directory (when granted)
- No network access unless explicitly allowed
- Trust model: "trust but verify - sandbox everything"

### Performance

**External Process Plugins:**
- Process spawning overhead (~10-50ms)
- JSON serialization over pipes
- Context switching between processes

**WASM Plugins:**
- No process spawning (loaded once, reused)
- Direct memory access for data passing
- Minimal context switching

### Distribution

**External Process Plugins:**
- Separate binary per platform (Linux, macOS, Windows)
- Dependencies must be installed
- Version compatibility challenges

**WASM Plugins:**
- Single `.wasm` file works everywhere
- Self-contained (no external dependencies)
- Version agnostic

---

## Current Status

### ✅ Implemented

- **Configuration format**: `hielements.toml` supports WASM plugin entries
- **Type system**: `LibraryType::Wasm` enum variant
- **Library interface**: `WasmLibrary` struct implements `Library` trait
- **Type inference**: Automatic detection from `.wasm` file extension
- **Loading logic**: Parse and load WASM libraries from config
- **Error messages**: Clear feedback when WASM features are used

### 🚧 In Progress

- **Runtime integration**: Wasmtime runtime for executing WASM modules
- **FFI protocol**: Memory management between host and WASM
- **WASI support**: Controlled filesystem and stdio access
- **Resource limits**: Memory and execution time constraints

### 📋 Planned

- **Example plugins**: Sample WASM plugin in Rust
- **Build tooling**: Scripts and documentation for building plugins
- **Testing infrastructure**: Integration tests with WASM modules
- **Performance benchmarks**: Comparison with external process plugins
- **Plugin marketplace**: Registry of community WASM plugins

---

## Configuration

### Basic Configuration

```toml
[libraries]
# Explicit WASM type
typescript = { type = "wasm", path = "lib/typescript.wasm" }

# Inferred from .wasm extension
golang = { path = "lib/golang_analyzer.wasm" }

# Multiple WASM plugins
docker = { path = "plugins/docker.wasm" }
terraform = { path = "plugins/terraform.wasm" }
```

### Mixed Configuration

You can use both WASM and external process plugins together:

```toml
[libraries]
# WASM plugins (sandboxed, fast)
typescript = { path = "lib/typescript.wasm" }
golang = { path = "lib/golang.wasm" }

# External process plugins (flexible)
python = { executable = "python3", args = ["plugins/python_plugin.py"] }
legacy_tool = { executable = "./tools/analyzer" }
```

---

## WASM Plugin Architecture

### Memory Model

```
┌─────────────────────────────────────┐
│    Hielements Interpreter (Host)   │
│  ┌───────────────────────────────┐  │
│  │   WasmLibrary                 │  │
│  │  ┌─────────────────────────┐  │  │
│  │  │ Wasmtime Runtime        │  │  │
│  │  │  ┌─────────────────┐    │  │  │
│  │  │  │ WASM Module     │    │  │  │
│  │  │  │ - Linear Memory │    │  │  │
│  │  │  │ - Functions     │    │  │  │
│  │  │  │ - Exports       │    │  │  │
│  │  │  └─────────────────┘    │  │  │
│  │  └─────────────────────────┘  │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
         │              ▲
         │ JSON input   │ JSON result
         ▼              │
  ┌──────────────────────────┐
  │ WASM Linear Memory       │
  │ [input] ... [result]     │
  └──────────────────────────┘
```

### Function Exports

Every WASM plugin must export these functions:

```rust
// Memory allocation (host calls to allocate space for input)
pub extern "C" fn alloc(size: i32) -> *mut u8;

// Handle selector function calls
pub extern "C" fn library_call(input_ptr: i32, input_len: i32) -> (i32, i32);

// Handle check function calls  
pub extern "C" fn library_check(input_ptr: i32, input_len: i32) -> (i32, i32);
```

### Data Flow

1. **Host → WASM**: Interpreter serializes request to JSON
2. **Host calls** `alloc(json_len)` to get memory pointer
3. **Host writes** JSON to WASM linear memory at pointer
4. **Host calls** `library_call(ptr, len)` or `library_check(ptr, len)`
5. **WASM processes** request, allocates result memory
6. **WASM returns** `(result_ptr, result_len)` tuple
7. **Host reads** JSON result from WASM memory
8. **Host parses** JSON and returns to interpreter

---

## Writing WASM Plugins

### Language Support

| Language | Support Level | Toolchain |
|----------|--------------|-----------|
| **Rust** | ⭐⭐⭐ Excellent | `cargo build --target wasm32-unknown-unknown` |
| **AssemblyScript** | ⭐⭐⭐ Excellent | AssemblyScript compiler |
| **C/C++** | ⭐⭐ Good | Emscripten or clang with wasm target |
| **Go** | ⭐⭐ Good | TinyGo compiler |
| **Python** | ⭐ Limited | PyScript (experimental) |

### Example: Rust WASM Plugin

```rust
use std::alloc::{alloc, Layout};
use std::slice;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Deserialize)]
struct Request {
    function: String,
    args: Vec<serde_json::Value>,
    workspace: String,
}

#[derive(Serialize)]
struct ScopeResult {
    Scope: Scope,
}

#[derive(Serialize)]
struct Scope {
    kind: ScopeKind,
    paths: Vec<String>,
    resolved: bool,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ScopeKind {
    File(String),
    Folder(String),
    Glob(String),
}

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> *mut u8 {
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { alloc(layout) }
}

#[no_mangle]
pub extern "C" fn library_call(input_ptr: i32, input_len: i32) -> (i32, i32) {
    // Read input JSON from memory
    let input_bytes = unsafe {
        slice::from_raw_parts(input_ptr as *const u8, input_len as usize)
    };
    
    let request: Request = match serde_json::from_slice(input_bytes) {
        Ok(req) => req,
        Err(e) => return error_result(&format!("Failed to parse request: {}", e)),
    };
    
    // Handle the function call
    let result = match request.function.as_str() {
        "module_selector" => handle_module_selector(&request),
        _ => error_result(&format!("Unknown function: {}", request.function)),
    };
    
    result
}

fn handle_module_selector(request: &Request) -> (i32, i32) {
    // Extract path from args
    let path = request.args.get(0)
        .and_then(|v| v.as_str())
        .unwrap_or("");
    
    // Build scope result
    let scope = ScopeResult {
        Scope: Scope {
            kind: ScopeKind::Folder(path.to_string()),
            paths: vec![], // Would scan filesystem here
            resolved: true,
        },
    };
    
    // Serialize to JSON
    let json = serde_json::to_string(&scope).unwrap();
    let bytes = json.as_bytes();
    
    // Allocate memory for result
    let result_ptr = alloc(bytes.len() as i32);
    unsafe {
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            result_ptr,
            bytes.len()
        );
    }
    
    (result_ptr as i32, bytes.len() as i32)
}

fn error_result(message: &str) -> (i32, i32) {
    let error = serde_json::json!({ "Error": message });
    let json = error.to_string();
    let bytes = json.as_bytes();
    
    let result_ptr = alloc(bytes.len() as i32);
    unsafe {
        std::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            result_ptr,
            bytes.len()
        );
    }
    
    (result_ptr as i32, bytes.len() as i32)
}

#[no_mangle]
pub extern "C" fn library_check(input_ptr: i32, input_len: i32) -> (i32, i32) {
    // Similar implementation for checks
    // Return {"Pass": null}, {"Fail": "message"}, or {"Error": "message"}
    (0, 0) // Placeholder
}
```

### Building

```bash
# Install Rust and wasm32 target
rustup target add wasm32-unknown-unknown

# Build the plugin
cargo build --target wasm32-unknown-unknown --release

# Output will be in target/wasm32-unknown-unknown/release/
# Copy to your lib directory
cp target/wasm32-unknown-unknown/release/my_plugin.wasm lib/
```

---

## Build Process

### Prerequisites

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Or AssemblyScript
npm install -g assemblyscript
```

### Project Structure

```
my_hielements_plugin/
├── Cargo.toml
├── src/
│   └── lib.rs
└── build.sh
```

### Cargo.toml

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Enable link-time optimization
strip = true        # Strip symbols
```

### Build Script

```bash
#!/bin/bash
cargo build --target wasm32-unknown-unknown --release
wasm-opt -Oz target/wasm32-unknown-unknown/release/my_plugin.wasm \
    -o target/my_plugin_optimized.wasm
```

---

## Security Model

### Capabilities

WASM plugins have **zero access by default**. Access must be explicitly granted:

| Capability | Default | Configurable |
|-----------|---------|--------------|
| Filesystem | ❌ No | ✅ Can grant workspace access |
| Network | ❌ No | ✅ Can allow specific hosts |
| System calls | ❌ No | ❌ Not available |
| Other processes | ❌ No | ❌ Not available |
| Environment vars | ❌ No | ✅ Can expose specific vars |

### Example Security Configuration (Planned)

```toml
[libraries.typescript]
path = "lib/typescript.wasm"

[libraries.typescript.permissions]
filesystem = ["read", "workspace-only"]
environment = ["HOME", "USER"]
# Network access not granted
```

### Comparison with External Process

| Attack Vector | External Process | WASM Plugin |
|--------------|-----------------|-------------|
| Read sensitive files | ✅ Full access | ❌ Sandboxed |
| Network requests | ✅ Unrestricted | ❌ No access by default |
| Fork bomb | ✅ Can spawn processes | ❌ Cannot spawn |
| Resource exhaustion | ⚠️ OS limits | ✅ Memory limits enforced |
| Code injection | ⚠️ Shell vulnerabilities | ✅ Memory safe |

---

## Performance Considerations

### Benchmarks (Estimated)

| Operation | External Process | WASM | Native |
|-----------|-----------------|------|--------|
| Plugin startup | ~10-50ms | ~1-2ms | ~0.1ms |
| Function call | ~1ms | ~0.1ms | ~0.01ms |
| Compute (1M ops) | ~100ms (Python) | ~15ms | ~10ms |
| Memory overhead | ~5-50MB | ~1-5MB | ~500KB |

### When to Use WASM

✅ **Use WASM when:**
- Performance is critical (called frequently)
- Strong security needed (untrusted code)
- Easy distribution important (single file)
- Cross-platform support required

⚠️ **Use External Process when:**
- Need to call existing tools (no rewrite needed)
- Need filesystem/network access (full capabilities)
- Using interpreted languages (Python, JS)
- Rapid prototyping (simpler development)

---

## Migration from External Process Plugins

### Step-by-Step Guide

1. **Assess**: Determine which plugins benefit from WASM
2. **Rewrite**: Port plugin logic to Rust/AssemblyScript/C
3. **Build**: Compile to WASM using appropriate toolchain
4. **Test**: Validate behavior matches external version
5. **Update Config**: Change `hielements.toml` entry
6. **Benchmark**: Verify performance improvements

### Example Migration

**Before (external_libraries.md example):**
```toml
[libraries]
sample = { executable = "python3", args = ["plugins/sample_plugin.py"] }
```

**After (WASM):**
```toml
[libraries]
sample = { path = "plugins/sample_plugin.wasm" }
```

### Compatibility

- No changes to `.hie` files needed
- Same `import sample` statement works
- Same function calls: `sample.simple_selector('path')`
- JSON protocol remains unchanged

---

## Next Steps

Once full WASM support is implemented:

1. **Try the examples**: Start with provided sample plugins
2. **Read the full guide**: Comprehensive tutorial on writing plugins
3. **Join the community**: Share your WASM plugins
4. **Contribute**: Help improve WASM runtime integration

---

## References

- [WebAssembly Official Site](https://webassembly.org/)
- [Wasmtime Runtime](https://wasmtime.dev/)
- [WASI (WebAssembly System Interface)](https://wasi.dev/)
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [AssemblyScript](https://www.assemblyscript.org/)
