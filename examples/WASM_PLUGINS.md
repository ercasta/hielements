# WASM Plugin Development Guide (Experimental)

This guide explains how to create WebAssembly (WASM) plugins for Hielements.

## Status

**⚠️ Experimental**: The WASM plugin infrastructure is in place, but full execution with wasmtime runtime will be added in a future release. For production use, please use [external process plugins](../doc/external_libraries.md).

## What's Implemented

The following WASM plugin infrastructure is currently available:

- ✅ Configuration support in `hielements.toml`
- ✅ Capability-based security model (fs access, workspace restrictions, env access)
- ✅ Type conversion and serialization (Value ↔ JSON)
- ✅ Plugin discovery and loading
- ✅ WasmLibrary implementation of Library trait
- ⏳ Full WASM execution (coming soon)

## Configuration

WASM plugins are configured in `hielements.toml`:

```toml
[libraries.mylib]
type = "wasm"
path = "plugins/mylib.wasm"
capabilities = {
    fs = "read",            # File system access: "none", "read", or "write"
    workspace_only = true,  # Restrict to workspace directory only
    env_access = false      # Allow reading environment variables
}
```

## Security Model

WASM plugins use capability-based security:

### File System Access

- **`fs = "none"`** (default): No file system access
- **`fs = "read"`**: Read-only access to workspace
- **`fs = "write"`**: Read/write access (future)

### Workspace Restriction

- **`workspace_only = true`** (default): Only access files within workspace
- **`workspace_only = false`**: Access all files (requires explicit grant)

### Environment Variables

- **`env_access = false`** (default): No environment variable access
- **`env_access = true`**: Can read environment variables

## Plugin Interface

WASM plugins must implement two functions:

### library_call

Execute a selector function and return a Value:

```rust
// Conceptual interface (actual implementation pending)
fn library_call(function: String, args: Vec<Value>, workspace: String) -> Result<Value>
```

### library_check

Execute a check function and return a CheckResult:

```rust
// Conceptual interface (actual implementation pending)
fn library_check(function: String, args: Vec<Value>, workspace: String) -> Result<CheckResult>
```

## Building WASM Plugins

Future releases will include:

1. **Rust template**: Cargo project for building WASM plugins
2. **Build instructions**: Using `wasm32-wasi` target
3. **Example plugins**: Demonstrating selectors and checks
4. **Memory management**: String passing between host and WASM

## Why WASM?

Compared to external process plugins, WASM offers:

| Feature | External Process | WASM |
|---------|------------------|------|
| **Security** | Process isolation | Capability-based sandbox |
| **Performance** | Process spawn overhead | Near-native, in-process |
| **Portability** | Platform-specific scripts | Single .wasm binary |
| **Dependencies** | Must be installed | Self-contained |
| **File Access** | Full system access | Restricted by capabilities |

## Migration Path

When WASM execution is fully enabled:

1. Keep existing external plugins working (backward compatible)
2. Optionally migrate high-security or performance-critical plugins to WASM
3. Use the same Library trait interface (no .hie file changes needed)

## Timeline

- **Current (v0.1)**: Infrastructure and configuration ✅
- **Next release**: Wasmtime integration and WASI file access
- **Future**: Example plugins and development tooling

## Alternatives

Until WASM plugins are fully functional, use:

- **External process plugins**: Full-featured, production-ready
- See the [External Libraries Guide](../doc/external_libraries.md) for details

## Security Considerations

WASM provides strong security guarantees:

1. **Memory isolation**: WASM linear memory is separate from host
2. **No direct syscalls**: Must go through WASI or host functions
3. **Capability-based**: Only explicitly granted capabilities work
4. **Deterministic**: No hidden state or side effects

This makes WASM ideal for running untrusted or third-party plugins safely.

## Contributing

Interested in WASM plugin development?

- Review the infrastructure in `crates/hielements-core/src/stdlib/wasm.rs`
- Track progress in `agent-changelog/wasm-extensibility.md`
- Contribute examples or improvements via pull requests

## Questions?

- 📖 Read the [External Libraries Guide](../doc/external_libraries.md)
- 🏗️ Check [Technical Architecture](../doc/technical_architecture.md)
- 💬 Open a GitHub discussion
