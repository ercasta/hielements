# WASM Extensibility: Safer Plugin Architecture

**Issue:** Evolve extensibility with safer libraries  
**Date:** 2025-12-30

## Summary

This document tracks the design and implementation of WebAssembly (WASM) plugin support in Hielements. WASM provides a safer, sandboxed execution environment for plugins while maintaining the existing external process plugin system for maximum flexibility.

## Problem Statement

The current external process plugin system (JSON-RPC over stdio) is flexible but has security concerns:
- Plugins have full access to the system via the spawned process
- No fine-grained capability control
- Trust boundary is at the process level

The goal is to add WASM as an alternative that provides:
- **Sandboxed execution**: Plugins run in a secure sandbox with limited capabilities
- **Performance**: Near-native execution speed
- **Portability**: Single binary works across all platforms
- **Capability-based security**: Fine-grained control over what plugins can access

## Architecture Design

### Hybrid Plugin System

Hielements will support three types of libraries:
1. **Built-in libraries** (files, rust) - Native Rust, full trust
2. **External process plugins** - JSON-RPC over stdio, process isolation
3. **WASM plugins** (NEW) - Sandboxed WASM modules with capability-based security

### WASM Runtime Choice

**Selected: wasmtime**
- Official Bytecode Alliance project
- Excellent Rust integration
- Mature WASI (WebAssembly System Interface) support
- Strong security and sandboxing
- Good performance

### Capability System

WASM plugins use a capability-based security model:
- Plugins declare required capabilities in configuration
- Capabilities include:
  - `fs:read` - Read access to workspace files
  - `fs:write` - Write access (for code generation plugins)
  - `env:read` - Read environment variables
  - `network:none` - No network access (default)
  
Example configuration:
```toml
[libraries.mylib]
type = "wasm"
path = "plugins/mylib.wasm"
capabilities = { fs = "read", workspace_only = true }
```

### WASM Plugin Interface

WASM plugins export functions that match the Library trait:
- `library_name() -> String`
- `library_call(function: String, args: Value, workspace: String) -> Value`
- `library_check(function: String, args: Value, workspace: String) -> CheckResult`

File system access is provided via WASI with restricted directory mappings.

## Implementation Plan

### Phase 1: Core WASM Infrastructure
1. Add wasmtime dependency to Cargo.toml
2. Create `crates/hielements-core/src/stdlib/wasm.rs`
3. Implement WasmLibrary struct with Library trait
4. Add WASI configuration with directory preopen for workspace

### Phase 2: Configuration and Loading
1. Extend HielementsConfig to support WASM library entries
2. Add capability parsing and validation
3. Update load_workspace_libraries to handle WASM plugins
4. Add type discriminator to library config

### Phase 3: Example and Documentation
1. Create example WASM plugin in Rust
2. Add build instructions for WASM plugins
3. Update external_libraries.md with WASM section
4. Update technical_architecture.md

### Phase 4: Testing
1. Add integration tests for WASM plugins
2. Test capability restrictions
3. Verify backward compatibility

## Changes to hielements.hie

```hielements
## Standard Library
element stdlib:
    # ... existing content ...
    
    ## External library support - allows user-defined plugins
    element external:
        scope external_module = files.file_selector('crates/hielements-core/src/stdlib/external.rs')
        
        check files.exists(stdlib_src, 'external.rs')
        check rust.struct_exists('ExternalLibrary')
        check rust.implements('ExternalLibrary', 'Library')
        check rust.struct_exists('ExternalLibraryConfig')
        check rust.function_exists('load_external_libraries')
    
    ## WASM library support - sandboxed plugins (NEW)
    element wasm:
        scope wasm_module = files.file_selector('crates/hielements-core/src/stdlib/wasm.rs')
        
        check files.exists(stdlib_src, 'wasm.rs')
        check rust.struct_exists('WasmLibrary')
        check rust.implements('WasmLibrary', 'Library')
        check rust.struct_exists('WasmCapabilities')
        check rust.function_exists('load_wasm_library')
```

## Security Considerations

### WASM Sandbox Benefits
1. **Memory isolation**: WASM has its own linear memory, isolated from host
2. **No direct system access**: Must go through WASI or host functions
3. **Capability-based**: Only granted capabilities are accessible
4. **Deterministic**: Same inputs produce same outputs (no hidden state)

### Capability Model
- Default: WASM plugins have NO file system access
- Explicit grant: Must request `fs:read` capability
- Workspace restriction: Even with `fs:read`, only workspace is accessible
- No network: WASM plugins cannot make network requests

### WASI File System Access
WASI provides sandboxed file system access through "preopened directories":
- Host maps workspace directory to WASI `/workspace`
- Plugin can only access files under `/workspace`
- Symbolic links outside workspace are rejected

Example:
```rust
// In WasmLibrary initialization
let mut config = wasmtime::Config::new();
let engine = wasmtime::Engine::new(&config)?;
let mut linker = wasmtime::Linker::new(&engine);
wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

// Preopen workspace with read-only access
let wasi = WasiCtxBuilder::new()
    .preopened_dir(Dir::open_ambient_dir(workspace, ambient_authority())?, "/workspace")?
    .build();
```

## Backward Compatibility

- Existing external process plugins continue to work unchanged
- Configuration format remains compatible (type field is optional, defaults to "external")
- Library trait interface unchanged
- No breaking changes to .hie file syntax

## Example WASM Plugin

A simple WASM plugin in Rust:

```rust
// plugins/simple/src/lib.rs
#[no_mangle]
pub extern "C" fn library_name() -> *const u8 {
    "simple\0".as_ptr()
}

#[no_mangle]
pub extern "C" fn library_call(
    function_ptr: *const u8,
    function_len: usize,
    args_ptr: *const u8,
    args_len: usize,
    workspace_ptr: *const u8,
    workspace_len: usize,
) -> *const u8 {
    // Implementation
}
```

Build:
```bash
cargo build --target wasm32-wasi --release
```

## Timeline

1. Phase 1 (Core): 2-3 hours
2. Phase 2 (Config): 1-2 hours  
3. Phase 3 (Example/Docs): 2-3 hours
4. Phase 4 (Testing): 1-2 hours

Total: ~6-10 hours of development

## Alternatives Considered

### 1. Only WASM (Remove External Plugins)
**Rejected**: External plugins are valuable for:
- Wrapping existing tools
- Languages that don't compile to WASM easily
- Rapid prototyping without compilation

### 2. Embedded Scripting (Rhai/Lua)
**Rejected for now**: 
- Another language to learn
- Limited ecosystem compared to Rust/WASM
- Can add later if there's demand

### 3. Shared Libraries (DLLs/SOs)
**Rejected**:
- No stable Rust ABI
- Platform-specific builds
- Security risks (no sandboxing)

## Success Criteria

- [ ] WASM plugins can be loaded from .wasm files
- [ ] WASM plugins can read workspace files through WASI
- [ ] WASM plugins cannot access files outside workspace
- [ ] WASM plugins implement Library trait correctly
- [ ] Example WASM plugin demonstrates usage
- [ ] Documentation explains how to create WASM plugins
- [ ] All tests pass
- [ ] Backward compatibility maintained

## References

- [WASI Documentation](https://wasi.dev/)
- [wasmtime Rust API](https://docs.wasmtime.dev/)
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
- [Bytecode Alliance Security](https://bytecodealliance.org/articles/security-and-correctness-in-wasmtime)
