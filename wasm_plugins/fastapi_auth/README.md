# FastAPI Authentication Checker - WASM Plugin

This is a WebAssembly plugin for Hielements that analyzes FastAPI applications to verify that API endpoints have proper authentication.

## Overview

This plugin implements the **hybrid WASM + parsing architecture**:

1. **Host (Hielements)**: Parses Python code to AST using RustPython or external libcst
2. **This WASM Plugin**: Analyzes the JSON AST to detect authentication patterns
3. **Result**: Returns Pass/Fail/Error based on authentication coverage

## Architecture

```
┌──────────────────────────┐
│ Hielements Interpreter   │
│                          │
│ 1. Parse Python → AST    │
│    (RustPython/libcst)   │
│                          │
│ 2. Convert AST → JSON    │
└────────┬─────────────────┘
         │
         │ JSON AST
         │
         ▼
┌──────────────────────────┐
│  WASM Plugin (Sandboxed) │
│                          │
│  3. Analyze Routes       │
│  4. Check Authentication │
│  5. Return Result        │
└──────────────────────────┘
```

## Authentication Detection

The plugin detects multiple authentication patterns:

### 1. Decorator-Based

```python
@app.post("/api/payment")
@requires_auth
async def create_payment():
    pass
```

Detected decorators:
- `@requires_auth`
- `@authenticated`
- `@login_required`
- `@auth_required`
- `@protected`

### 2. Dependency Injection

```python
@app.post("/api/payment")
async def create_payment(user: User = Depends(get_current_user)):
    pass
```

Detects: `Depends(get_current_user)` and similar patterns

### 3. FastAPI Security Schemes

```python
@app.post("/api/payment")
async def create_payment(token: str = Security(oauth2_scheme)):
    pass
```

Detects: `Security()` dependencies

## Exported Functions

### `check_all_routes_authenticated(ast_json_ptr, ast_json_len) -> (result_ptr, result_len)`

Verifies that all routes (GET, POST, PUT, DELETE, etc.) have authentication.

**Returns**: CheckResult as JSON
- `{"Pass": null}` - All routes authenticated
- `{"Fail": "message"}` - Some routes missing authentication
- `{"Error": "message"}` - Analysis error

### `check_post_routes_authenticated(ast_json_ptr, ast_json_len) -> (result_ptr, result_len)`

Verifies that all POST routes have authentication.

### `check_get_routes_authenticated(ast_json_ptr, ast_json_len) -> (result_ptr, result_len)`

Verifies that all GET routes have authentication.

### `get_routes_info(ast_json_ptr, ast_json_len) -> (result_ptr, result_len)`

Returns detailed information about all routes.

**Returns**: JSON array of Route objects
```json
[
  {
    "name": "create_payment",
    "path": "/api/payment",
    "method": "POST",
    "authenticated": true,
    "auth_method": "dependency_injection",
    "line_number": 45
  }
]
```

## Building

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm32 target
rustup target add wasm32-unknown-unknown
```

### Build the WASM Module

```bash
cd wasm_plugins/fastapi_auth

# Build release version
cargo build --target wasm32-unknown-unknown --release

# Output will be in:
# target/wasm32-unknown-unknown/release/fastapi_auth_wasm.wasm
```

### Optimize (Optional)

```bash
# Install wasm-opt (from binaryen)
# Ubuntu/Debian:
sudo apt-get install binaryen

# macOS:
brew install binaryen

# Optimize the WASM file
wasm-opt -Oz target/wasm32-unknown-unknown/release/fastapi_auth_wasm.wasm \
    -o lib/fastapi_auth.wasm
```

## Usage in Hielements

### Configuration (`hielements.toml`)

```toml
[libraries]
fastapi_auth = { path = "lib/fastapi_auth.wasm" }
```

### Hielements Specification

```hielements
import fastapi_auth

element my_api {
    scope api = python.module_selector('app/api.py')
    
    # Check all routes have authentication
    check fastapi_auth.all_routes_authenticated(api)
    
    # Check only POST routes
    check fastapi_auth.post_routes_authenticated(api)
    
    # Check only GET routes  
    check fastapi_auth.get_routes_authenticated(api)
}
```

## JSON AST Format

The plugin expects a simplified Python AST in JSON format:

```json
{
  "type": "Module",
  "body": [
    {
      "type": "FunctionDef",
      "name": "create_payment",
      "lineno": 45,
      "decorators": [
        {
          "type": "Decorator",
          "func": {
            "type": "Attribute",
            "value": {"type": "Name", "id": "app"},
            "attr": "post"
          },
          "args": [
            {"type": "Constant", "value": "/api/payment"}
          ]
        }
      ],
      "parameters": [
        {
          "name": "current_user",
          "annotation": "User",
          "default": {
            "type": "Call",
            "func": {"type": "Name", "id": "Depends"},
            "args": [
              {"type": "Name", "id": "get_current_user"}
            ]
          }
        }
      ]
    }
  ]
}
```

## Testing

### Unit Tests

```bash
# Run tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Integration Tests

See `../../examples/fastapi_example/` for complete examples.

## Performance

- **Parse overhead**: ~0ms (done by host, not WASM)
- **Analysis time**: ~1-5ms per file (100+ routes)
- **Memory usage**: ~100KB - 1MB depending on AST size
- **WASM module size**: ~50-100KB (optimized)

## Security

This plugin runs in a WebAssembly sandbox with:
- ✅ No filesystem access
- ✅ No network access
- ✅ No system calls
- ✅ Memory-safe execution
- ✅ Deterministic behavior

Perfect for untrusted code analysis!

## Limitations

- Only detects common authentication patterns
- Doesn't analyze middleware-based auth (needs separate check)
- Doesn't verify actual auth implementation (only presence)
- Requires AST to follow expected JSON structure

## Future Enhancements

- [ ] Middleware authentication detection
- [ ] Custom authentication pattern configuration
- [ ] More detailed authentication analysis
- [ ] Support for class-based views
- [ ] Starlette route detection

## License

MIT

## Contributing

Contributions welcome! Please ensure:
1. All tests pass
2. Code is formatted with `cargo fmt`
3. No clippy warnings: `cargo clippy`
4. WASM builds successfully
