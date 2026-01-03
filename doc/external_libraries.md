# External Library Plugin Guide

This guide explains how to create custom libraries (plugins) for Hielements. External libraries allow you to extend Hielements with your own selectors and checks, supporting any programming language or technology.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Plugin Types](#2-plugin-types)
3. [Configuration](#3-configuration)
4. [Protocol Specification](#4-protocol-specification)
5. [Implementing an External Process Plugin](#5-implementing-an-external-process-plugin)
6. [Example: Python Plugin](#6-example-python-plugin)
7. [WASM Plugins](#7-wasm-plugins)
8. [Sharing and Distributing Libraries](#8-sharing-and-distributing-libraries)
9. [Best Practices](#9-best-practices)
10. [Troubleshooting](#10-troubleshooting)

---

## 1. Overview

Hielements supports multiple plugin architectures to balance flexibility, security, and performance:

1. **External Process Plugins** - Communicate via JSON-RPC 2.0 over stdin/stdout (Production Ready)
2. **WASM Plugins** - Sandboxed WebAssembly modules (Infrastructure Ready - Runtime Integration in Progress)

This guide focuses on external process plugins, which are fully implemented and production-ready. See [Section 7](#7-wasm-plugins) for WASM plugin status and configuration.

### External Process Plugins

External libraries are standalone programs that communicate with the Hielements interpreter via JSON-RPC 2.0 over stdin/stdout. This design provides:

- **Language Independence**: Write plugins in Python, JavaScript, Go, or any language
- **Security**: Process isolation between plugins and the interpreter
- **Flexibility**: Leverage existing analysis tools and libraries
- **Simplicity**: Simple text-based protocol
- **Easy Integration**: Wrap existing tools without modification

### How It Works

1. You configure external libraries in `hielements.toml`
2. When a library is imported, Hielements spawns the plugin process
3. The interpreter sends JSON-RPC requests to the plugin
4. The plugin responds with results (scopes, check outcomes)
5. The process stays running for subsequent calls

---

## 2. Plugin Types

Hielements supports two types of plugins, configured in `hielements.toml`:

| Type | Configuration | Status | Use When |
|------|--------------|--------|----------|
| **External Process** | `executable` | ✅ Production Ready | Maximum flexibility, existing tools, language of choice |
| **WASM** | `path` (`.wasm` file) | 🚧 Infrastructure Ready | Strong sandboxing required, performance-critical (runtime integration in progress) |

### Choosing a Plugin Type

**Use External Process plugins when:**
- You want to write plugins in Python, JavaScript, Go, etc.
- You need to leverage existing analysis tools
- You want the simplest development experience
- You need filesystem or network access

**Use WASM plugins when (future):**
- You need strong security sandboxing
- Performance is critical (near-native speed)
- You want easy distribution (single .wasm file)
- You can work within WASM's constraints

---

## 3. Configuration

### Configuration File

Create a `hielements.toml` file in your workspace root:

```toml
[libraries]
# External process plugin (explicit type)
mylib = { type = "external", executable = "path/to/mylib" }

# External process plugin (inferred from 'executable' field)
python_checks = { executable = "python3", args = ["scripts/checks.py"] }

# Plugin with multiple arguments
custom = { executable = "./plugins/custom", args = ["--workspace", "--verbose"] }

# WASM plugin (explicit type - infrastructure ready, runtime integration in progress)
typescript = { type = "wasm", path = "lib/typescript.wasm" }

# WASM plugin (inferred from .wasm extension)
docker = { path = "lib/docker.wasm" }
```

### Configuration Options

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `type` | String | No | Plugin type: "external" or "wasm". If omitted, inferred from other fields. |
| `executable` | String | For external | Path to the plugin executable |
| `path` | String | For WASM | Path to the WASM file (`.wasm` extension) |
| `args` | Array | No | Command-line arguments (external plugins only) |

### Type Inference Rules

The `type` field is optional. Hielements infers the plugin type as follows:

1. If `type` is specified explicitly, use it
2. If `path` ends with `.wasm`, infer `type = "wasm"`
3. If `executable` is specified, infer `type = "external"`
4. If `path` is specified but not `.wasm`, infer `type = "external"`

### Path Resolution

- Relative paths are resolved from the workspace directory
- Use absolute paths for system-installed tools

---

## 4. Protocol Specification

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

#### `library.doc` (optional)

Returns detailed documentation for the library, including descriptions of all functions, checks, and their parameters. This enables automatic documentation generation.

**Request:**
```json
{"jsonrpc": "2.0", "method": "library.doc", "id": 1}
```

**Response:**
```json
{
    "jsonrpc": "2.0",
    "result": {
        "name": "mylibrary",
        "description": "Custom library for Python analysis",
        "version": "1.0.0",
        "functions": [
            {
                "name": "module_selector",
                "description": "Select Python modules by path pattern",
                "parameters": [
                    {"name": "path", "type": "string", "description": "Path pattern"}
                ],
                "return_type": "Scope",
                "example": "mylibrary.module_selector('src/')"
            }
        ],
        "checks": [
            {
                "name": "has_init",
                "description": "Check if a Python package has __init__.py",
                "parameters": [
                    {"name": "scope", "type": "Scope", "description": "Package scope"}
                ],
                "return_type": "CheckResult",
                "example": "check mylibrary.has_init(src)"
            }
        ]
    },
    "id": 1
}
```

Use `hielements doc` to generate documentation catalogs that include all libraries (built-in and external) in markdown or JSON format.

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

## 9. Best Practices

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

## 7. WASM Plugins

### Overview

WASM plugin support provides a safer, more performant alternative to external process plugins. The infrastructure is in place and functional, with runtime execution integration in progress.

### Benefits of WASM Plugins

| Benefit | Description |
|---------|-------------|
| **Strong Sandboxing** | WASM provides capability-based security - plugins can only access what you explicitly allow |
| **Near-Native Performance** | WASM executes at near-native speed, much faster than interpreted languages |
| **Portable** | Single `.wasm` file works on all platforms (Linux, macOS, Windows) |
| **Small Size** | WASM binaries are typically smaller than equivalent native code |
| **Easy Distribution** | Share plugins as single files, no installation needed |

### Current Status

**✅ Ready:**
- Configuration format supports WASM plugins in `hielements.toml`
- `LibraryType::Wasm` enum variant
- `WasmLibrary` struct with `Library` trait implementation
- Type inference from `.wasm` file extension
- Loading logic from configuration files
- Clear error messages when attempting to use WASM plugins

**🚧 In Progress:**
- Wasmtime runtime integration for executing WASM modules
- WASM FFI protocol for library calls and checks
- Memory management between host and WASM
- WASI permissions configuration

**📋 Planned:**
- Example WASM plugin in Rust
- Build tooling for compiling plugins
- Complete documentation for writing WASM plugins
- Performance benchmarks vs external processes

### Configuration (When Available)

```toml
[libraries]
# Explicit WASM type
typescript = { type = "wasm", path = "lib/typescript.wasm" }

# Inferred from .wasm extension
golang = { path = "lib/golang_analyzer.wasm" }
docker = { path = "plugins/docker.wasm" }
```

### WASM Plugin Interface (Planned)

WASM plugins will export these functions:

```rust
// Allocate memory for input (returns pointer)
#[no_mangle]
pub extern "C" fn alloc(size: i32) -> *mut u8;

// Handle library calls (receives JSON, returns JSON)
#[no_mangle]
pub extern "C" fn library_call(input_ptr: i32, input_len: i32) -> (i32, i32); // returns (result_ptr, result_len)

// Handle library checks (receives JSON, returns JSON)
#[no_mangle]
pub extern "C" fn library_check(input_ptr: i32, input_len: i32) -> (i32, i32);
```

JSON format will be the same as external process plugins for consistency.

### Writing WASM Plugins

Languages that can compile to WASM:
- **Rust** (best support via `wasm32-unknown-unknown` target)
- **AssemblyScript** (TypeScript-like language for WASM)
- **C/C++** (via Emscripten or clang)
- **Go** (via TinyGo)
- **Many others** with varying levels of support

A complete WASM plugin guide will be provided when full support is available.

### Migration Path

External process plugins will continue to work indefinitely. When WASM support is ready:
1. Decide which plugins benefit from WASM (performance, sandboxing)
2. Rewrite or compile those plugins to WASM
3. Update `hielements.toml` configuration
4. Keep other plugins as external processes

No breaking changes to the plugin API are planned.

---

## 8. Sharing and Distributing Libraries

Once you've created a custom library, you can share it with the Hielements community or your organization. This section covers various distribution methods.

### Distribution Methods

#### 1. Source Code Distribution

**Best for:** Internal teams, open-source collaboration, interpreted languages

Share your plugin source code directly:

```bash
my-hielements-library/
├── README.md              # Installation and usage instructions
├── plugin.py              # Plugin implementation
├── requirements.txt       # Dependencies (if Python)
├── package.json          # Dependencies (if Node.js)
└── hielements.toml.example  # Example configuration
```

**Users configure:**
```toml
[libraries]
mylibrary = { executable = "python3", args = ["path/to/plugin.py"] }
```

**Pros:**
- Easy to modify and customize
- Transparent implementation
- Simple for scripting languages

**Cons:**
- Users need language runtime installed
- May expose implementation details

#### 2. Binary Distribution

**Best for:** Production use, compiled languages, external distribution

Compile your plugin to native executables:

```bash
# Go
go build -o mylibrary-plugin ./cmd/plugin

# Rust
cargo build --release

# Python with PyInstaller
pyinstaller --onefile plugin.py
```

Distribute platform-specific binaries:
```
releases/
├── mylibrary-v1.0.0-linux-amd64
├── mylibrary-v1.0.0-darwin-amd64
├── mylibrary-v1.0.0-darwin-arm64
└── mylibrary-v1.0.0-windows-amd64.exe
```

**Users configure:**
```toml
[libraries]
mylibrary = { executable = "./bin/mylibrary-plugin" }
```

**Pros:**
- No runtime dependencies
- Better performance
- Hides implementation details

**Cons:**
- Platform-specific builds required
- Larger file sizes

#### 3. Package Manager Distribution

**Best for:** Public libraries, version management, automatic updates

##### Python (PyPI)

```bash
# setup.py
from setuptools import setup

setup(
    name='mylibrary-hielements',
    version='1.0.0',
    py_modules=['mylibrary_plugin'],
    entry_points={
        'console_scripts': [
            'mylibrary-plugin=mylibrary_plugin:main',
        ],
    },
)

# Publish
python setup.py sdist bdist_wheel
twine upload dist/*
```

**Users install and configure:**
```bash
pip install mylibrary-hielements
```

```toml
[libraries]
mylibrary = { executable = "mylibrary-plugin" }
```

##### npm (Node.js)

```bash
# package.json
{
  "name": "mylibrary-hielements",
  "version": "1.0.0",
  "bin": {
    "mylibrary-plugin": "./index.js"
  }
}

# Publish
npm publish
```

**Users install and configure:**
```bash
npm install -g mylibrary-hielements
```

```toml
[libraries]
mylibrary = { executable = "mylibrary-plugin" }
```

**Pros:**
- Automatic dependency management
- Version control
- Wide distribution reach

**Cons:**
- Package ecosystem overhead
- Requires account/publishing setup

#### 4. Git Repository

**Best for:** Open-source projects, version control, collaborative development

Host your library in a Git repository:

```bash
# Repository structure
my-hielements-library/
├── README.md
├── LICENSE
├── CHANGELOG.md
├── src/
│   └── plugin.{py,js,go,rs}
├── tests/
├── examples/
│   ├── basic.hie
│   └── hielements.toml
└── docs/
```

**Users can:**
```bash
# Clone
git clone https://github.com/username/my-hielements-library libs/mylibrary

# Or use as submodule
git submodule add https://github.com/username/my-hielements-library libs/mylibrary
```

```toml
[libraries]
mylibrary = { executable = "python3", args = ["libs/mylibrary/src/plugin.py"] }
```

**Pros:**
- Version history
- Easy updates (git pull)
- Collaborative development

**Cons:**
- Users need git installed
- Requires manual configuration

#### 5. WASM Distribution (Future)

**Best for:** Cross-platform, security-sensitive, performance-critical (when available)

**Note:** WASM infrastructure is ready, runtime integration in progress.

Once fully available, WASM provides the best distribution experience:

```bash
# Build Rust plugin to WASM
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/mylibrary.wasm ./dist/
```

**Distribution:**
- Single `.wasm` file works on all platforms
- No runtime dependencies
- Strong security sandboxing

```toml
[libraries]
mylibrary = { path = "mylibrary.wasm" }
```

**Pros:**
- Single file, all platforms
- No dependencies
- Secure sandboxing
- Near-native performance

**Cons:**
- Requires WASM-compatible language (Rust, AssemblyScript, C/C++, Go/TinyGo)
- Runtime integration still in progress

See [WASM Plugins Guide](wasm_plugins.md) for current status.

### Documentation Best Practices

Include these in your library distribution:

#### 1. README.md

```markdown
# MyLibrary for Hielements

Brief description of what your library does.

## Installation

Instructions for installing/setting up the library.

## Configuration

\`\`\`toml
[libraries]
mylibrary = { executable = "python3", args = ["path/to/plugin.py"] }
\`\`\`

## Available Functions

### Selectors

- `mylibrary.selector_name(arg1, arg2)` - Description

### Checks

- `mylibrary.check_name(scope, arg)` - Description

## Examples

Show example .hie files using your library.

## Requirements

- Python 3.8+
- Dependencies listed in requirements.txt
```

#### 2. API Reference

Document all functions:

```markdown
## mylibrary.module_selector(path: string) -> Scope

Selects Python modules in the specified path.

**Parameters:**
- `path` (string): Relative path from workspace

**Returns:**
- Scope containing all .py files in the path

**Example:**
\`\`\`hielements
import mylibrary

element my_app:
    scope src = mylibrary.module_selector('src/')
\`\`\`
```

#### 3. Configuration Examples

Provide `hielements.toml.example`:

```toml
# Example Hielements configuration for MyLibrary

[libraries]
mylibrary = { 
    executable = "python3", 
    args = ["path/to/mylibrary_plugin.py"]
}

# Alternative: if installed as package
# mylibrary = { executable = "mylibrary-plugin" }
```

#### 4. Example .hie Files

Include working examples:

```hielements
# examples/basic_usage.hie
import mylibrary

element example:
    scope src = mylibrary.module_selector('src/')
    check mylibrary.has_tests(src)
```

### Version Management

Use semantic versioning for your library:

- **MAJOR**: Breaking changes to plugin API
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes

Document breaking changes in CHANGELOG.md:

```markdown
# Changelog

## [2.0.0] - 2024-01-15
### Breaking Changes
- Renamed `old_function` to `new_function`
- Changed return format for `selector_name`

### Added
- New check function `new_check`

## [1.1.0] - 2023-12-01
### Added
- Support for Python 3.11
```

### Security Considerations

When distributing libraries:

1. **Sign releases**: Provide checksums or GPG signatures
2. **Document permissions**: Clearly state filesystem/network access needs
3. **Security policy**: Include SECURITY.md with vulnerability reporting process
4. **License**: Include LICENSE file with clear terms

### Community Best Practices

1. **Open source when possible**: Increase trust and adoption
2. **Provide tests**: Show your library works correctly
3. **CI/CD**: Automate testing and releases
4. **Issue tracker**: Enable users to report problems
5. **Community support**: Monitor issues and questions
6. **Keep updated**: Maintain compatibility with Hielements updates

---

## 9. Troubleshooting

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
