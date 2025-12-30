# WASM Library Support: Safer Extensibility

**Issue:** Enhance extensibility with safer libraries (WASM) while maintaining external tool support  
**Date:** 2025-12-30

## Summary

This document describes the implementation of WebAssembly (WASM) plugin support for Hielements, providing a safer alternative to external process plugins while maintaining backward compatibility with the existing external tool calling mechanism.

## Problem Statement

The current external library system uses JSON-RPC over stdio with external processes, which provides excellent flexibility but has some limitations:
- Performance overhead from process spawning and IPC
- Security concerns with executing arbitrary external code
- Difficulty distributing and managing external dependencies

## Solution: Hybrid Plugin Architecture

Implement a hybrid approach that supports both:
1. **WASM plugins** - Sandboxed, portable, near-native performance
2. **External process plugins** - Maximum flexibility, language-agnostic

## Architecture Changes

### 1. Library Types

Support three types of libraries:
- **Built-in libraries** (files, rust) - Native Rust implementation
- **WASM libraries** - Compiled WebAssembly modules
- **External libraries** - Separate processes via JSON-RPC

### 2. Configuration Format (hielements.toml)

```toml
[libraries]
# External process plugin (existing)
python = { type = "external", executable = "python3", args = ["plugins/python_lib.py"] }

# WASM plugin (new)
mylib = { type = "wasm", path = "plugins/mylib.wasm" }

# Auto-detect based on extension (new)
auto1 = { path = "plugins/sample.wasm" }  # Detected as WASM
auto2 = { executable = "python3", args = ["plugin.py"] }  # Detected as external
```

### 3. WASM Host Interface

WASM plugins export functions that match the Library trait:
- `library_call(function: string, args: string, workspace: string) -> string`
- `library_check(function: string, args: string, workspace: string) -> string`

Arguments and results are serialized as JSON strings for WASM compatibility.

### 4. Security & Sandboxing

WASM provides strong sandboxing:
- No access to file system by default
- Explicit capability grants via WASI
- Limited to computational operations unless granted specific permissions
- Cannot spawn processes or access network

External plugins remain isolated via process boundaries.

## Implementation Plan

### Changes to hielements.hie

Add WASM library support to the external element:

```hielements
element external:
    scope external_module = files.file_selector('crates/hielements-core/src/stdlib/external.rs')
    scope wasm_module = files.file_selector('crates/hielements-core/src/stdlib/wasm.rs')
    
    check files.exists(stdlib_src, 'external.rs')
    check files.exists(stdlib_src, 'wasm.rs')
    
    check rust.struct_exists('ExternalLibrary')
    check rust.implements('ExternalLibrary', 'Library')
    
    check rust.struct_exists('WasmLibrary')
    check rust.implements('WasmLibrary', 'Library')
    
    check rust.enum_exists('LibraryType')
    check rust.function_exists('load_library')
```

### Code Changes

1. **Add WASM runtime dependency** (wasmtime)
   - Mature, production-ready
   - Strong sandboxing and WASI support
   - Good Rust integration

2. **Create WasmLibrary struct**
   - Implements Library trait
   - Manages WASM module lifecycle
   - Handles serialization/deserialization

3. **Enhance configuration parsing**
   - Support type field: "wasm" | "external"
   - Auto-detect from file extension (.wasm)
   - Backward compatible with existing configs

4. **Update LibraryRegistry**
   - Support loading both WASM and external libraries
   - Unified interface for all library types

## Benefits

### WASM Benefits
| Benefit | Description |
|---------|-------------|
| **Security** | Strong sandboxing, no system access by default |
| **Performance** | Near-native speed, no IPC overhead |
| **Portability** | Single .wasm file works on all platforms |
| **Distribution** | Easy to share and version control |
| **Language Support** | Rust, C, C++, AssemblyScript, Go, etc. |

### External Process Benefits  
| Benefit | Description |
|---------|-------------|
| **Flexibility** | Any language, any tooling |
| **Ecosystem** | Leverage existing analysis tools |
| **No Compilation** | Scripts work directly |
| **Debugging** | Standard debugging tools |

## Usage Examples

### WASM Plugin Example

Rust code compiled to WASM:

```rust
// mylib.rs
use serde_json::Value;

#[no_mangle]
pub extern "C" fn library_call(
    function_ptr: *const u8,
    function_len: usize,
    args_ptr: *const u8,
    args_len: usize,
    workspace_ptr: *const u8,
    workspace_len: usize,
) -> i32 {
    // Implementation
}
```

Configuration:

```toml
[libraries]
mylib = { type = "wasm", path = "plugins/mylib.wasm" }
```

Usage in .hie:

```hielements
import mylib

element component:
    scope src = mylib.custom_selector('src/')
    check mylib.custom_check(src)
```

### Mixed Usage Example

```toml
[libraries]
# Fast WASM plugin for hot path
performance = { type = "wasm", path = "plugins/perf.wasm" }

# Python plugin for flexibility
integration = { type = "external", executable = "python3", args = ["plugins/int.py"] }
```

```hielements
import performance
import integration

element system:
    scope src = performance.fast_selector('.')
    check integration.external_tool_check(src)
```

## Migration Path

1. **Phase 1** (Current PR): Add WASM support alongside external plugins
2. **Phase 2**: Create WASM versions of common plugins
3. **Phase 3**: Provide tooling to help convert external plugins to WASM
4. **Long-term**: Encourage WASM for performance-critical plugins, external for flexibility

## Testing Strategy

1. Create sample WASM plugin in Rust
2. Test WASM plugin loading and execution
3. Test hybrid usage (WASM + external)
4. Test error handling and sandboxing
5. Performance benchmarks (WASM vs external)

## Security Considerations

### WASM Security
- Sandboxed execution by default
- No file system access unless granted via WASI
- Cannot spawn processes
- Memory isolated from host

### External Process Security
- Process isolation
- Workspace path explicitly passed
- Consider adding executable allowlist
- Document security model

### Comparison
WASM provides stronger security guarantees but less flexibility. External processes provide maximum flexibility but require trust in the plugin code.

## Documentation Updates

1. Update external_libraries.md with WASM section
2. Add WASM plugin development guide
3. Update technical_architecture.md with hybrid architecture
4. Create example WASM plugin with full walkthrough

## Future Enhancements

1. **WASI support** - File system access for WASM plugins
2. **Component Model** - Better interface types when stable
3. **Hot reloading** - Reload WASM modules without restart
4. **Plugin marketplace** - Share and discover plugins
5. **Plugin bundling** - Package multiple WASM modules together

## Conclusion

Adding WASM support provides a safer, more performant option for Hielements plugins while maintaining the flexibility of external process plugins. This hybrid approach gives users the best of both worlds: security and performance when needed, flexibility and ease of development when preferred.
