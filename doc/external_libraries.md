# Hielements Library Plugin Guide

This guide explains how to create custom libraries (plugins) for Hielements. Hielements supports two types of libraries:

1. **WASM Libraries** - Sandboxed WebAssembly modules (recommended for pure analysis)
2. **External Process Libraries** - Standalone programs via JSON-RPC (for tools needing system access)

---

## Table of Contents

1. [Overview](#1-overview)
2. [Choosing Library Type](#2-choosing-library-type)
3. [WASM Libraries](#3-wasm-libraries)
4. [External Process Libraries](#4-external-process-libraries)
5. [Configuration](#5-configuration)
6. [Best Practices](#6-best-practices)
7. [Troubleshooting](#7-troubleshooting)

---

## 1. Overview

Hielements provides two complementary approaches to extending the system:

### WASM Libraries (Recommended for most use cases)
- Compiled WebAssembly modules that run in a sandboxed environment
- **Security**: Strong isolation, capability-based file access
- **Performance**: Fast in-process execution, no IPC overhead
- **Portability**: Single `.wasm` file works across all platforms
- **Use when**: Writing pure analysis code without external tool dependencies

### External Process Libraries
- Standalone programs communicating via JSON-RPC over stdin/stdout
- **Flexibility**: Write in any language, call external tools
- **Security**: Process isolation
- **Use when**: Need to invoke external commands (e.g., `cargo`, `docker`, analysis tools)

---

## 2. Choosing Library Type

| Scenario | Recommended Type | Reason |
|----------|------------------|---------|
| Pure code analysis (parsing, pattern matching) | WASM | Better performance and security |
| Need to call external tools (git, docker, etc.) | External Process | Can spawn subprocesses |
| Rapid prototyping | External Process | Easier debugging, any language |
| Production deployment | WASM | Better security and portability |
| Working with untrusted code | WASM | Sandboxed execution |
| Complex file system operations | External Process | Full system access |

---

## 3. WASM Libraries

### Overview

WASM libraries are compiled WebAssembly modules that run inside the Hielements interpreter with controlled access to system resources.

### Creating a WASM Library

**Coming soon**: Full guide for creating WASM libraries in Rust. For now, WASM support is available but requires implementing the low-level interface manually.

The interface expects:
- `library_call(params_ptr: i32, params_len: i32) -> (result_ptr: i32, result_len: i32)`
- `library_check(params_ptr: i32, params_len: i32) -> (result_ptr: i32, result_len: i32)`
- `alloc(size: i32) -> i32` for memory management

### Configuration

```toml
[libraries]
# Basic WASM library
mylib = { type = "wasm", path = "libraries/mylib.wasm" }

# WASM library with custom capabilities
secure = { 
    type = "wasm", 
    path = "lib/secure.wasm",
    capabilities = {
        file_read = true,      # Allow reading files (default: true)
        file_write = false,    # Deny writing files (default: false)
        network = false        # Deny network access (default: false, not yet implemented)
    }
}
```

### Capabilities

WASM libraries run with restricted capabilities by default:

| Capability | Default | Description |
|------------|---------|-------------|
| `file_read` | `true` | Read files from workspace |
| `file_write` | `false` | Write files to workspace |
| `network` | `false` | Network access (not yet implemented) |

**Note**: Host function integration for file system access is planned but not yet implemented. Current WASM libraries must implement their own file access logic.

---

## 4. External Process Libraries

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

## 5. Configuration

### Unified Configuration File

Create a `hielements.toml` in your workspace root that can include both WASM and external process libraries:

```toml
[libraries]
# WASM library (recommended for pure analysis)
python_analyzer = { 
    type = "wasm", 
    path = "libraries/python_analyzer.wasm",
    capabilities = {
        file_read = true,
        file_write = false
    }
}

# External process library (for calling external tools)
docker = { 
    type = "external", 
    executable = "hielements-docker-plugin" 
}

# External process with arguments
git_analyzer = { 
    type = "external",
    executable = "python3", 
    args = ["scripts/git_checks.py"] 
}

# Legacy format (backward compatible, defaults to external)
rust_legacy = { 
    executable = "./plugins/rust.py" 
}
```

### Configuration Options

#### WASM Libraries

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `type` | String | Yes | Must be `"wasm"` |
| `path` | String | Yes | Path to `.wasm` file |
| `capabilities` | Object | No | Capability restrictions (see above) |

#### External Process Libraries

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `type` | String | No | `"external"` or omit for legacy compatibility |
| `executable` | String | Yes | Path to the plugin executable |
| `args` | Array | No | Command-line arguments to pass |

### Path Resolution

- Relative paths are resolved from the workspace directory
- Use absolute paths for system-installed tools

---

## 6. Best Practices

### Choosing Between WASM and External Process

**Use WASM when:**
- Writing pure code analysis (AST parsing, pattern matching)
- Working with untrusted libraries
- Need maximum portability
- Performance is critical (no IPC overhead)

**Use External Process when:**
- Need to call external tools (git, docker, compilers)
- Rapid prototyping in any language
- Complex file system operations
- Need full system access

### WASM Best Practices

- **Keep libraries small**: WASM is best for focused analysis tasks
- **Avoid system calls**: Use the provided host functions (when available)
- **Test thoroughly**: WASM debugging can be challenging
- **Document capabilities**: Clearly state what your library needs

### External Process Best Practices

#### Performance

- **Keep the process running**: Don't exit after each request
- **Cache expensive operations**: Store analysis results between calls
- **Use lazy evaluation**: Only scan files when actually needed

#### Error Handling

- **Return meaningful error messages**: Include what went wrong and how to fix it
- **Handle missing files gracefully**: Return empty scopes instead of crashing
- **Log to stderr**: Plugin errors go to stderr, not stdout

#### Security

**For all libraries:**
- **Validate all inputs**: Don't trust paths or arguments
- **Use workspace-relative paths**: Stay within the workspace directory
- **Don't execute arbitrary code**: Be careful with user-provided patterns

**WASM-specific security:**
- WASM libraries are sandboxed by default
- Capabilities must be explicitly granted
- Cannot spawn external processes
- Limited file system access

**External Process security:**
- Process isolation provides basic security
- Full system access requires trust
- Consider using allowlists for executables

### Testing

**For External Process Libraries:**
- **Test the protocol manually**: Use `echo '{"jsonrpc":"2.0","method":"library.call","params":{"function":"test","args":[],"workspace":"."},"id":1}' | ./my-plugin`
- **Write unit tests**: Test selectors and checks independently
- **Test error cases**: Ensure graceful handling of invalid inputs

**For WASM Libraries:**
- Test locally before deployment
- Use WASI test tools for debugging
- Verify capability requirements are correct

---

## 7. Migration Guide

### From External Process to WASM

If you have an existing external process library and want to migrate to WASM for better security and performance:

1. **Assess feasibility**: Does your library call external tools? If yes, keep it as external process.
2. **Rewrite in Rust** (or other WASM-compatible language)
3. **Remove external tool calls**: WASM can't spawn processes
4. **Implement the WASM interface**: See section 3 for details
5. **Compile to WASM**: Use `cargo build --target wasm32-unknown-unknown`
6. **Update configuration**: Change `type` from `external` to `wasm`
7. **Test thoroughly**: Verify all functionality works in sandbox

### Backward Compatibility

All existing external process libraries continue to work without changes:

```toml
# Old format (still supported)
[libraries]
mylib = { executable = "./mylib" }

# Equivalent new format
[libraries]
mylib = { type = "external", executable = "./mylib" }
```

---

## 8. Troubleshooting

### Common Issues - External Process Libraries

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

### Common Issues - WASM Libraries

**WASM module fails to load:**
- Verify the `.wasm` file exists at the specified path
- Check that it was compiled with the correct target: `wasm32-unknown-unknown`
- Ensure the WASM module exports the required functions

**Capability errors:**
- Check that required capabilities are granted in configuration
- Verify file paths are within the workspace
- Note: Host function support is not yet fully implemented

**Memory errors:**
- Ensure `alloc` function is properly implemented
- Verify memory management in WASM module
- Check for buffer overflows in string operations

**Performance issues:**
- WASM libraries should be faster than external processes
- If slow, profile your WASM code
- Consider caching results between calls

### Debugging WASM Libraries

Currently, WASM debugging is limited. For development:

1. Test your library logic in native Rust first
2. Compile to WASM only after verification
3. Use logging to stdout (will appear in Hielements output)
4. Consider building with debug symbols: `cargo build --target wasm32-unknown-unknown`

**Future improvements**: Better WASM debugging support is planned, including:
- Stack traces for WASM errors
- Step-through debugging
- Memory inspection tools

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
