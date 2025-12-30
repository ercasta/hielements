# External Library Plugin Guide

This guide explains how to create custom libraries (plugins) for Hielements. Hielements supports two types of plugins:

1. **WASM Plugins** - Sandboxed, portable, high-performance WebAssembly modules
2. **External Process Plugins** - Flexible programs that communicate via JSON-RPC

Choose WASM for security and performance, or external processes for maximum flexibility.

---

## Table of Contents

1. [Overview](#1-overview)
2. [WASM Plugins](#2-wasm-plugins)
3. [External Process Plugins](#3-external-process-plugins)
4. [Configuration](#4-configuration)
5. [Best Practices](#5-best-practices)
6. [Troubleshooting](#6-troubleshooting)

---

## 1. Overview

Hielements supports two plugin architectures, each with different tradeoffs:

### WASM Plugins

WebAssembly plugins run in a sandboxed environment with near-native performance.

**Benefits:**
- **Security**: Strong sandboxing, no system access by default
- **Performance**: Near-native speed, no IPC overhead
- **Portability**: Single .wasm file works on all platforms
- **Size**: Typically 10-50 KB compiled
- **Distribution**: Easy to version control and share

**Limitations:**
- Limited file system access (requires WASI configuration)
- Must be compiled to WASM (Rust, C, C++, AssemblyScript, Go)
- More complex to debug than scripts

**Use WASM for:**
- Performance-critical checks
- Portable plugins that work everywhere
- Plugins that don't need system access
- Production deployments

### External Process Plugins

External plugins are standalone programs that communicate via JSON-RPC 2.0 over stdin/stdout.

**Benefits:**
- **Language Independence**: Write in Python, JavaScript, Go, or any language
- **Flexibility**: Leverage existing analysis tools and libraries
- **Simplicity**: No compilation needed for scripts
- **Debugging**: Use standard debugging tools

**Limitations:**
- Process spawning overhead
- IPC latency for each call
- Requires managing external dependencies

**Use External Processes for:**
- Maximum flexibility
- Existing tools in any language
- Plugins that need full system access
- Development and prototyping

---

## 2. WASM Plugins

### Building a WASM Plugin

WASM plugins are compiled from languages like Rust, C, C++, or AssemblyScript.

#### Example: Rust WASM Plugin

**Cargo.toml:**
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
```

**src/lib.rs:**
```rust
use serde_json::{json, Value};
use std::alloc::{alloc, Layout};

#[no_mangle]
pub extern "C" fn allocate(size: i32) -> *mut u8 {
    let layout = Layout::from_size_align(size as usize, 1).unwrap();
    unsafe { alloc(layout) }
}

#[no_mangle]
pub extern "C" fn library_call(ptr: *const u8, len: i32) -> (i32, i32) {
    // Parse input JSON
    let input = unsafe {
        std::slice::from_raw_parts(ptr, len as usize)
    };
    let request: Value = serde_json::from_slice(input).unwrap();
    
    // Handle the call
    let result = match request["function"].as_str() {
        Some("my_selector") => {
            json!({
                "Scope": {
                    "kind": {"Folder": "src"},
                    "paths": ["/path/to/src"],
                    "resolved": true
                }
            })
        }
        _ => json!({"Error": "Unknown function"})
    };
    
    // Return result as (ptr, len)
    let result_str = result.to_string();
    let bytes = result_str.as_bytes();
    let ptr = allocate(bytes.len() as i32);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
    }
    (ptr as i32, bytes.len() as i32)
}

#[no_mangle]
pub extern "C" fn library_check(ptr: *const u8, len: i32) -> (i32, i32) {
    // Similar to library_call but returns CheckResult
    // ...
}
```

#### Building

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Build the plugin
cargo build --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/my_plugin.wasm
```

### WASM Plugin Interface

WASM plugins must export three functions:

1. **`allocate(size: i32) -> *mut u8`** - Allocate memory for input/output
2. **`library_call(ptr: *const u8, len: i32) -> (i32, i32)`** - Handle selectors
3. **`library_check(ptr: *const u8, len: i32) -> (i32, i32)`** - Handle checks

Input and output are JSON strings serialized to memory.

### WASM Configuration

Add to `hielements.toml`:

```toml
[libraries]
# Explicit WASM type
my_plugin = { type = "wasm", path = "plugins/my_plugin.wasm" }

# Auto-detected (by .wasm extension)
another = { path = "plugins/another.wasm" }
```

---

## 3. External Process Plugins

### How It Works

1. You configure external libraries in `hielements.toml`
2. When a library is imported, Hielements spawns the plugin process
3. The interpreter sends JSON-RPC requests to the plugin via stdin
4. The plugin responds with results via stdout
5. The process stays running for subsequent calls

### Configuration

Create a `hielements.toml` file in your workspace root:

```toml
[libraries]
# Basic plugin
mylib = { executable = "path/to/mylib" }

# Plugin with arguments
python_checks = { executable = "python3", args = ["scripts/checks.py"] }

# Explicit external type (optional)
custom = { type = "external", executable = "./plugins/custom", args = ["--workspace"] }
```

### Configuration Options

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `type` | String | No | "external" or "wasm" (auto-detected if omitted) |
| `executable` | String | Yes* | Path to the plugin executable (*for external) |
| `args` | Array | No | Command-line arguments to pass |
| `path` | String | Yes* | Path to WASM file (*for WASM) |

### Path Resolution

- Relative paths are resolved from the workspace directory
- Use absolute paths for system-installed tools

---

## 4. Configuration

External libraries use JSON-RPC 2.0 over stdio. Each request is a single JSON line, and each response is a single JSON line.

### Request Format

```json
{
    "jsonrpc": "2.0",
    "method": "library.call",
    "params": {
        "function": "selector_name",
        "args": [...],
        "workspace": "/path/to/workspace"
    },
    "id": 1
}
```

### Methods

#### `library.metadata` (optional)

Returns information about the library.

**Request:**
```json
{"jsonrpc": "2.0", "method": "library.metadata", "id": 1}
```

**Response:**
```json
{
    "jsonrpc": "2.0",
    "result": {
        "name": "mylibrary",
        "version": "1.0.0",
        "functions": ["custom_selector"],
        "checks": ["custom_check"]
    },
    "id": 1
}
```

#### `library.call`

Calls a selector function and returns a Value.

**Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "library.call",
    "params": {
        "function": "custom_selector",
        "args": [{"String": "src/"}],
        "workspace": "/home/user/project"
    },
    "id": 2
}
```

**Response (Scope):**
```json
{
    "jsonrpc": "2.0",
    "result": {
        "Scope": {
            "kind": {"Folder": "src/"},
            "paths": ["/home/user/project/src/main.py", "/home/user/project/src/utils.py"],
            "resolved": true
        }
    },
    "id": 2
}
```

#### `library.check`

Executes a check function and returns a CheckResult.

**Request:**
```json
{
    "jsonrpc": "2.0",
    "method": "library.check",
    "params": {
        "function": "custom_check",
        "args": [{"Scope": {...}}, {"Int": 100}],
        "workspace": "/home/user/project"
    },
    "id": 3
}
```

**Response (Pass):**
```json
{
    "jsonrpc": "2.0",
    "result": {"Pass": null},
    "id": 3
}
```

**Response (Fail):**
```json
{
    "jsonrpc": "2.0",
    "result": {"Fail": "Check failed: reason here"},
    "id": 3
}
```

**Response (Error):**
```json
{
    "jsonrpc": "2.0",
    "result": {"Error": "Could not evaluate check"},
    "id": 3
}
```

### Value Types

Values are JSON objects with type tags:

| Type | JSON Format | Example |
|------|-------------|---------|
| Null | `null` | `null` |
| Bool | `{"Bool": true}` or `true` | `{"Bool": false}` |
| Int | `{"Int": 42}` | `{"Int": 100}` |
| Float | `{"Float": 3.14}` | `{"Float": 2.5}` |
| String | `{"String": "text"}` | `{"String": "src/"}` |
| List | `{"List": [...]}` | `{"List": [{"String": "a"}, {"String": "b"}]}` |
| Scope | `{"Scope": {...}}` | See below |

#### Scope Format

```json
{
    "Scope": {
        "kind": {"File": "path"} | {"Folder": "path"} | {"Glob": "pattern"},
        "paths": ["list", "of", "resolved", "paths"],
        "resolved": true
    }
}
```

### Error Responses

```json
{
    "jsonrpc": "2.0",
    "error": {
        "code": -32000,
        "message": "Error description"
    },
    "id": 3
}
```

Standard JSON-RPC error codes apply:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error
- `-32000` to `-32099`: Server error (custom)

---

## 4. Implementing a Plugin

### Minimal Implementation

A plugin must:

1. Read JSON-RPC requests from stdin (line by line)
2. Parse the JSON request
3. Handle `library.call` and `library.check` methods
4. Write JSON-RPC responses to stdout (one line per response)
5. Keep running (process stays alive for multiple requests)

### Template (Pseudocode)

```
while line = read_line(stdin):
    request = parse_json(line)
    
    if request.method == "library.call":
        result = handle_call(request.params.function, request.params.args)
        response = {"jsonrpc": "2.0", "result": result, "id": request.id}
    
    else if request.method == "library.check":
        result = handle_check(request.params.function, request.params.args)
        response = {"jsonrpc": "2.0", "result": result, "id": request.id}
    
    write_line(stdout, to_json(response))
    flush(stdout)
```

---

## 5. Example: Python Plugin

Here's a complete Python plugin:

```python
#!/usr/bin/env python3
"""Custom Hielements library plugin."""

import json
import sys
import os

def handle_call(function, args, workspace):
    """Handle selector function calls."""
    if function == "module_selector":
        # Extract path from first argument
        path = args[0].get("String", "") if args else ""
        full_path = os.path.join(workspace, path)
        
        # Find Python files
        files = []
        if os.path.isdir(full_path):
            for root, _, filenames in os.walk(full_path):
                for f in filenames:
                    if f.endswith(".py"):
                        files.append(os.path.join(root, f))
        
        return {
            "Scope": {
                "kind": {"Folder": path},
                "paths": files,
                "resolved": True
            }
        }
    raise ValueError(f"Unknown function: {function}")

def handle_check(function, args, workspace):
    """Handle check function calls."""
    if function == "has_init":
        scope = args[0].get("Scope", {}) if args else {}
        paths = scope.get("paths", [])
        
        # Check if __init__.py exists
        for path in paths:
            if path.endswith("__init__.py"):
                return {"Pass": None}
        return {"Fail": "Missing __init__.py"}
    
    raise ValueError(f"Unknown check: {function}")

def main():
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        
        try:
            request = json.loads(line)
            method = request.get("method", "")
            params = request.get("params", {})
            req_id = request.get("id", 1)
            
            if method == "library.call":
                result = handle_call(
                    params.get("function"),
                    params.get("args", []),
                    params.get("workspace", ".")
                )
            elif method == "library.check":
                result = handle_check(
                    params.get("function"),
                    params.get("args", []),
                    params.get("workspace", ".")
                )
            else:
                result = None
            
            response = {"jsonrpc": "2.0", "result": result, "id": req_id}
            
        except Exception as e:
            response = {
                "jsonrpc": "2.0",
                "error": {"code": -32000, "message": str(e)},
                "id": request.get("id", 0)
            }
        
        print(json.dumps(response), flush=True)

if __name__ == "__main__":
    main()
```

---

## 6. Best Practices

### Performance

- **Keep the process running**: Don't exit after each request
- **Cache expensive operations**: Store analysis results between calls
- **Use lazy evaluation**: Only scan files when actually needed

### Error Handling

- **Return meaningful error messages**: Include what went wrong and how to fix it
- **Handle missing files gracefully**: Return empty scopes instead of crashing
- **Log to stderr**: Plugin errors go to stderr, not stdout

### Security

- **Validate all inputs**: Don't trust paths or arguments
- **Use workspace-relative paths**: Stay within the workspace directory
- **Don't execute arbitrary code**: Be careful with user-provided patterns

### Testing

- **Test the protocol manually**: Use `echo '{"jsonrpc":"2.0","method":"library.call","params":{"function":"test","args":[],"workspace":"."},"id":1}' | ./my-plugin`
- **Write unit tests**: Test selectors and checks independently
- **Test error cases**: Ensure graceful handling of invalid inputs

---

## 7. Troubleshooting

### Common Issues

**Plugin doesn't start:**
- Check that the executable path is correct
- Verify the executable has execute permissions (`chmod +x`)
- Check for missing dependencies

**No response from plugin:**
- Ensure stdout is flushed after each response
- Check that you're writing to stdout, not stderr
- Verify the JSON is on a single line (no pretty printing)

**Invalid JSON errors:**
- Ensure responses are valid JSON
- Check for trailing commas or unquoted strings
- Use a JSON validator to check output

**Scope paths not found:**
- Verify paths are absolute (combine workspace + relative path)
- Check that `resolved` is set to `true`
- Ensure the `kind` field matches the actual type

### Debugging

Add debug logging to stderr (won't interfere with the protocol):

```python
import sys

def debug(msg):
    print(f"[DEBUG] {msg}", file=sys.stderr)

debug(f"Received request: {line}")
```

Run the plugin manually to test:

```bash
echo '{"jsonrpc":"2.0","method":"library.call","params":{"function":"test","args":[],"workspace":"."},"id":1}' | ./my-plugin
```

---

## Appendix: Full Protocol Reference

### Value Serialization

| Hielements Type | JSON Representation |
|-----------------|---------------------|
| `null` | `null` |
| `true`/`false` | `true`/`false` or `{"Bool": true}` |
| `42` (int) | `42` or `{"Int": 42}` |
| `3.14` (float) | `3.14` or `{"Float": 3.14}` |
| `"text"` (string) | `"text"` or `{"String": "text"}` |
| `[a, b, c]` (list) | `[...]` or `{"List": [...]}` |
| Scope | `{"Scope": {"kind": {...}, "paths": [...], "resolved": true}}` |

### Check Result Serialization

| Result | JSON Representation |
|--------|---------------------|
| Pass | `{"Pass": null}` or `{"result": "pass"}` |
| Fail | `{"Fail": "message"}` or `{"result": "fail", "message": "..."}` |
| Error | `{"Error": "message"}` or `{"result": "error", "message": "..."}` |
