# WASM-Based Sandbox Extensibility Evolution

**Issue:** Evolve extensibility to add safer libraries (WASM sandboxed) while maintaining external tool calling
**Date:** 2025-12-30

## Problem Statement

Ease of extensibility and library sharing is fundamental. The current external process plugin mechanism (JSON-RPC over stdio) works well but has security and distribution concerns. This change evolves the extensibility feature by:
1. Adding WASM-based sandboxed library support for safer execution
2. Keeping external tool calling for flexibility when needed
3. Providing a hybrid approach that balances security, performance, and usability

## Current State

The current implementation supports:
- External process plugins via JSON-RPC over stdio (implemented in `stdlib/external.rs`)
- Configuration via `hielements.toml`
- `Library` trait for extensibility
- Two built-in libraries: `files` and `rust`

Pros of current approach:
- Language agnostic (Python, JS, Go, etc.)
- Process isolation
- Easy to develop
- Can leverage existing tools

Cons:
- Process spawning overhead
- Distribution complexity (users install binaries)
- Security model depends on external process behavior
- No strong sandboxing guarantees

## Proposed Solution: Hybrid Extensibility Model

### Three-Tier Plugin System

1. **Built-in Libraries** (Rust, compiled into interpreter)
   - Highest performance
   - Most trusted
   - Examples: `files`, `rust`

2. **WASM Libraries** (New) - **Primary Addition**
   - Strong sandboxing via WASM runtime
   - Near-native performance
   - Single binary works on all platforms
   - Capability-based security (WASI)
   - Easy distribution (single .wasm file)
   - Languages: Rust, AssemblyScript, C/C++, Go (TinyGo)

3. **External Process Libraries** (Existing)
   - Maximum flexibility
   - Can call any external tool
   - Use when WASM limitations prevent implementation
   - Examples: complex analysis tools, legacy integrations

### WASM Library Architecture

```
┌─────────────────────────────────────────────┐
│         Hielements Interpreter              │
│  ┌────────────────────────────────────┐     │
│  │    LibraryRegistry                 │     │
│  │  ┌──────────┬──────────┬─────────┐│     │
│  │  │ Built-in │   WASM   │ External││     │
│  │  │ Libraries│ Libraries│ Process ││     │
│  │  └──────────┴──────────┴─────────┘│     │
│  └────────────────────────────────────┘     │
│           │          │          │           │
│           │   ┌──────▼─────┐    │           │
│           │   │ WasmRuntime│    │           │
│           │   │  (wasmer/  │    │           │
│           │   │  wasmtime) │    │           │
│           │   └────────────┘    │           │
└───────────┼──────────┼──────────┼───────────┘
            │          │          │
            │   ┌──────▼─────┐    │
            │   │.wasm files │    │
            │   │(sandboxed) │    │
            │   └────────────┘    │
            │                     │
      ┌─────▼────┐          ┌────▼────┐
      │Built-in  │          │External │
      │Functions │          │Process  │
      └──────────┘          └─────────┘
```

### Configuration Format Evolution

```toml
# hielements.toml

[libraries]
# External process plugins (existing)
python = { type = "external", executable = "python3", args = ["lib/python_plugin.py"] }
legacy_tool = { type = "external", executable = "./tools/analyzer" }

# WASM plugins (new)
typescript = { type = "wasm", path = "lib/typescript_plugin.wasm" }
docker = { type = "wasm", path = "lib/docker_plugin.wasm" }

# Type can be inferred from file extension
golang = { path = "lib/golang_plugin.wasm" }  # .wasm -> type="wasm"
custom = { executable = "./custom.py" }        # executable -> type="external"
```

### WASM Plugin Interface

WASM plugins export standardized functions:
```rust
// Plugin interface (Rust example)
#[no_mangle]
pub extern "C" fn library_name() -> *const u8 { ... }

#[no_mangle]
pub extern "C" fn library_call(function_ptr: *const u8, args_ptr: *const u8, workspace_ptr: *const u8) -> *const u8 { ... }

#[no_mangle]
pub extern "C" fn library_check(function_ptr: *const u8, args_ptr: *const u8, workspace_ptr: *const u8) -> *const u8 { ... }
```

Data is passed as JSON-encoded strings for simplicity.

### Security Model

**WASM Libraries:**
- Run in sandboxed environment
- No direct system access by default
- Filesystem access via WASI (controlled by interpreter)
- Limited to workspace directory
- No network access (unless explicitly granted)
- Memory isolated from host

**External Process Libraries:**
- Process isolation
- Explicit workspace path
- User responsible for trust
- Documentation warns about security implications

### Implementation Plan

#### Phase 1: Core WASM Support
1. Add WASM runtime dependency (wasmer or wasmtime)
2. Create `WasmLibrary` struct implementing `Library` trait
3. Define WASM FFI interface
4. Update configuration parsing to support WASM type
5. Update `LibraryRegistry` to load WASM libraries

#### Phase 2: Example WASM Plugin
1. Create Rust-based example WASM plugin
2. Build tooling for compiling plugins to WASM
3. Documentation for writing WASM plugins
4. Test integration with existing checks

#### Phase 3: Enhanced Security
1. Capability-based permissions for WASM
2. Configurable WASI permissions
3. Resource limits (memory, execution time)
4. Plugin verification/signing (future)

## Changes to hielements.hie

```hielements
## Standard Library
element stdlib:
    scope module = rust.module_selector('stdlib')
    scope stdlib_src = files.folder_selector('crates/hielements-core/src/stdlib')
    
    # ... existing elements ...
    
    ## WASM library support (NEW)
    element wasm:
        scope wasm_module = files.file_selector('crates/hielements-core/src/stdlib/wasm.rs')
        
        check files.exists(stdlib_src, 'wasm.rs')
        check rust.struct_exists('WasmLibrary')
        check rust.implements('WasmLibrary', 'Library')
        check rust.struct_exists('WasmRuntime')
        check rust.function_exists('load_wasm_library')
        check rust.has_tests(wasm_module)
    
    ## External library support (EXISTING - enhanced)
    element external:
        scope external_module = files.file_selector('crates/hielements-core/src/stdlib/external.rs')
        
        check files.exists(stdlib_src, 'external.rs')
        check rust.struct_exists('ExternalLibrary')
        check rust.implements('ExternalLibrary', 'Library')
        check rust.struct_exists('ExternalLibraryConfig')
        check rust.function_exists('load_external_libraries')
        
        ## Enhanced to support library type discrimination
        check rust.enum_exists('LibraryType')
```

## Implementation Details

### Dependencies to Add (Cargo.toml)
```toml
[dependencies]
# WASM runtime - choose one
wasmer = "4.2"  # OR wasmtime = "15.0"
# For WASI support
wasmer-wasi = "4.2"  # OR wasmtime-wasi = "15.0"
```

### Files to Create
1. `crates/hielements-core/src/stdlib/wasm.rs` - WASM library implementation
2. `doc/wasm_plugins.md` - Documentation for writing WASM plugins
3. `examples/plugins/sample_wasm_plugin/` - Example WASM plugin in Rust

### Files to Modify
1. `crates/hielements-core/src/stdlib/mod.rs` - Register WASM module
2. `crates/hielements-core/src/stdlib/external.rs` - Enhanced config parsing
3. `crates/hielements-core/Cargo.toml` - Add WASM runtime dependency
4. `hielements.hie` - Add WASM element checks
5. `doc/external_libraries.md` - Document WASM plugin option
6. `README.md` - Update with WASM security benefits

## Benefits

1. **Security**: Strong sandboxing for untrusted plugins
2. **Performance**: Near-native execution speed
3. **Portability**: Single .wasm file works everywhere
4. **Distribution**: Easy to share (single file)
5. **Flexibility**: Keep external process option for complex tools
6. **Modern**: Aligns with industry trends (WASM in plugins)

## Migration Path

Existing external plugins continue to work. Users can:
1. Keep using external process plugins (no breaking changes)
2. Gradually migrate performance-critical plugins to WASM
3. Use WASM for new plugins with security requirements
4. Choose based on their needs (flexibility vs security)

## Testing Strategy

1. Unit tests for WASM library loading
2. Integration tests with sample WASM plugin
3. Security tests (verify sandboxing)
4. Performance benchmarks (WASM vs external process)
5. Cross-platform tests (Linux, macOS, Windows)

## Documentation Updates

1. New guide: "Writing WASM Plugins for Hielements"
2. Update: "External Libraries Guide" - add WASM section
3. Update: README.md - highlight security benefits
4. Example: Complete WASM plugin project structure
5. Security considerations for both WASM and external plugins

## Future Enhancements

1. Plugin marketplace/registry
2. Plugin signing and verification
3. Hot reloading of WASM plugins
4. Plugin dependency management
5. Performance profiling for plugins
6. Memory pooling for WASM instances
