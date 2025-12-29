# External Library Plugin Guide

This guide explains how to create custom libraries (plugins) for Hielements. External libraries allow you to extend Hielements with your own selectors and checks, supporting any programming language or technology.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Configuration](#2-configuration)
3. [Protocol Specification](#3-protocol-specification)
4. [Implementing a Plugin](#4-implementing-a-plugin)
5. [Example: Python Plugin](#5-example-python-plugin)
6. [Best Practices](#6-best-practices)
7. [Troubleshooting](#7-troubleshooting)

---

## 1. Overview

External libraries are standalone programs that communicate with the Hielements interpreter via JSON-RPC 2.0 over stdin/stdout. This design provides:

- **Language Independence**: Write plugins in Python, JavaScript, Go, or any language
- **Security**: Process isolation between plugins and the interpreter
- **Flexibility**: Leverage existing analysis tools and libraries
- **Simplicity**: Simple text-based protocol

### How It Works

1. You configure external libraries in `hielements.toml`
2. When a library is imported, Hielements spawns the plugin process
3. The interpreter sends JSON-RPC requests to the plugin
4. The plugin responds with results (scopes, check outcomes)
5. The process stays running for subsequent calls

---

## 2. Configuration

### Configuration File

Create a `hielements.toml` file in your workspace root:

```toml
[libraries]
# Basic plugin
mylib = { executable = "path/to/mylib" }

# Plugin with arguments
python_checks = { executable = "python3", args = ["scripts/checks.py"] }

# Plugin with multiple arguments
custom = { executable = "./plugins/custom", args = ["--workspace", "--verbose"] }
```

### Configuration Options

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `executable` | String | Yes | Path to the plugin executable |
| `args` | Array | No | Command-line arguments to pass |

### Path Resolution

- Relative paths are resolved from the workspace directory
- Use absolute paths for system-installed tools

---

## 3. Protocol Specification

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
echo '{"jsonrpc":"2.0","method":"library.call","params":{"function":"test","args":[],"workspace":"."},"id":1}' | python3 my_plugin.py
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
