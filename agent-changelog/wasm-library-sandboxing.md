# WASM-based Library Sandboxing

**Issue:** Evolve extensibility feature for safer library execution  
**Date:** 2025-12-30

## Summary

This document extends the existing extensibility mechanism (external process plugins via JSON-RPC) with WebAssembly (WASM) support to provide safer, sandboxed library execution while maintaining the ability to call external tools when needed.

## Current State

The Hielements system currently supports external libraries through:
- External process plugins communicating via JSON-RPC over stdio
- Configuration via `hielements.toml`
- Process isolation for security
- Language-agnostic plugin development

This works well but has some limitations:
- Process spawning overhead
- No fine-grained sandboxing control
- Distribution complexity (managing multiple binaries)
- Limited ability to restrict library capabilities

## Goals

1. **Safety**: Provide strong sandboxing for untrusted library code
2. **Performance**: Reduce overhead compared to external processes
3. **Portability**: Single WASM binary works across platforms
4. **Flexibility**: Support both WASM (sandboxed) and external process (full access) modes
5. **Capability-based security**: Fine-grained control over what libraries can access

## Design: Hybrid Approach

### Architecture Overview

```
┌─────────────────────────────────────────────────┐
│          Hielements Interpreter (Rust)          │
│                                                 │
│  ┌─────────────┐          ┌─────────────┐      │
│  │  Library    │          │  Library    │      │
│  │  Registry   │          │  Loader     │      │
│  └──────┬──────┘          └──────┬──────┘      │
│         │                        │             │
│         └────────┬───────────────┘             │
│                  │                             │
│       ┌──────────┴──────────┐                 │
│       │                     │                 │
│   ┌───▼────┐         ┌─────▼────┐            │
│   │  WASM  │         │ External │            │
│   │Library │         │ Library  │            │
│   │(New)   │         │(Existing)│            │
│   └───┬────┘         └─────┬────┘            │
│       │                    │                 │
└───────┼────────────────────┼─────────────────┘
        │                    │
    ┌───▼────┐          ┌───▼────┐
    │ WASM   │          │External│
    │Runtime │          │Process │
    │(wasmer)│          │(JSON-  │
    │        │          │ RPC)   │
    └───┬────┘          └───┬────┘
        │                   │
        │    Sandboxed      │    Full System
        │    File Access    │    Access
        │                   │
        ▼                   ▼
    ┌───────────────────────────┐
    │    File System / Tools    │
    └───────────────────────────┘
```

### Library Types

1. **WASM Libraries** (New)
   - Compiled to `.wasm` files
   - Run in sandboxed environment
   - Capability-based file system access
   - Fast in-process execution
   - Cannot spawn external processes
   - Recommended for pure analysis code

2. **External Process Libraries** (Existing)
   - Separate processes via JSON-RPC
   - Full system access
   - Can call external tools
   - Process isolation
   - Recommended when need to invoke external tools (e.g., running `ast-grep`, `cargo`, etc.)

### Configuration Format

Extend `hielements.toml` to support both types:

```toml
[libraries]
# WASM library (sandboxed)
python_analyzer = { type = "wasm", path = "libraries/python_analyzer.wasm" }

# External process library (full access)
docker = { type = "external", executable = "hielements-docker-plugin" }

# Backward compatibility: no type means external
rust_legacy = { executable = "./plugins/rust.py" }

# WASM library with explicit capabilities
custom = { 
    type = "wasm", 
    path = "libraries/custom.wasm",
    capabilities = {
        file_read = true,
        file_write = false,
        network = false
    }
}
```

### WASM Interface Design

Use WASM Component Model / WIT (WebAssembly Interface Types) for clean interface definition:

```wit
// library.wit
package hielements:library

interface library {
    // Get library metadata
    record metadata {
        name: string,
        version: string,
        functions: list<string>,
        checks: list<string>
    }
    
    get-metadata: func() -> metadata
    
    // Value types
    variant value {
        null,
        bool(bool),
        int(s64),
        float(float64),
        string(string),
        list(list<value>),
        scope(scope)
    }
    
    record scope {
        kind: scope-kind,
        paths: list<string>,
        resolved: bool
    }
    
    variant scope-kind {
        file(string),
        folder(string),
        glob(string)
    }
    
    // Check result
    variant check-result {
        pass,
        fail(string),
        error(string)
    }
    
    // Main library interface
    call: func(function: string, args: list<value>, workspace: string) -> result<value, string>
    check: func(function: string, args: list<value>, workspace: string) -> result<check-result, string>
}

// File system capabilities (provided by host)
interface filesystem {
    read-file: func(path: string) -> result<list<u8>, string>
    list-directory: func(path: string) -> result<list<string>, string>
    file-exists: func(path: string) -> bool
    // Write operations only if capability granted
    write-file: func(path: string, content: list<u8>) -> result<_, string>
}
```

### Implementation Components

#### 1. WasmLibrary Struct

```rust
// crates/hielements-core/src/stdlib/wasm.rs

use wasmer::{Store, Module, Instance, imports, Function};

pub struct WasmLibrary {
    name: String,
    instance: Instance,
    store: Store,
    capabilities: WasmCapabilities,
}

#[derive(Debug, Clone)]
pub struct WasmCapabilities {
    pub file_read: bool,
    pub file_write: bool,
    pub network: bool,
}

impl WasmLibrary {
    pub fn load(path: &Path, capabilities: WasmCapabilities) -> LibraryResult<Self> {
        // Load WASM module
        // Set up imports (file system access, etc.)
        // Instantiate module
        // Return WasmLibrary
    }
}

impl Library for WasmLibrary {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn call(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value> {
        // Call WASM function
    }
    
    fn check(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult> {
        // Call WASM check function
    }
}
```

#### 2. Extended Configuration

```rust
// crates/hielements-core/src/stdlib/external.rs (extend existing)

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LibraryConfig {
    External {
        executable: String,
        #[serde(default)]
        args: Vec<String>,
    },
    Wasm {
        path: String,
        #[serde(default)]
        capabilities: WasmCapabilitiesConfig,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct WasmCapabilitiesConfig {
    #[serde(default = "default_true")]
    pub file_read: bool,
    #[serde(default)]
    pub file_write: bool,
    #[serde(default)]
    pub network: bool,
}

fn default_true() -> bool { true }
```

#### 3. Library Loading

```rust
// Update load_external_libraries to handle both types

pub fn load_libraries(config_path: &Path) -> LibraryResult<Vec<Box<dyn Library>>> {
    let config: HielementsConfig = load_config(config_path)?;
    
    let mut libraries: Vec<Box<dyn Library>> = Vec::new();
    
    for (name, lib_config) in config.libraries {
        match lib_config {
            LibraryConfig::External { executable, args } => {
                let ext_lib = ExternalLibrary::new(ExternalLibraryConfig {
                    name: name.clone(),
                    executable,
                    args,
                });
                libraries.push(Box::new(ext_lib));
            }
            LibraryConfig::Wasm { path, capabilities } => {
                let wasm_lib = WasmLibrary::load(
                    Path::new(&path),
                    capabilities.into()
                )?;
                libraries.push(Box::new(wasm_lib));
            }
        }
    }
    
    Ok(libraries)
}
```

## Security Model

### WASM Libraries
- **Sandboxed by default**: Cannot access file system unless granted
- **Capability-based**: Must explicitly request permissions
- **No network access**: Unless explicitly granted (future)
- **No process spawning**: Cannot execute external commands
- **Memory isolation**: Limited to WASM linear memory

### External Process Libraries
- **Process isolation**: Run in separate process
- **Full system access**: Can read/write files, spawn processes
- **Use when**: Need to call external tools or do system-level operations
- **Trust required**: User must trust the executable

## Implementation Phases

### Phase 1: Core WASM Support ✓ (This PR)
- [ ] Add wasmer dependency
- [ ] Implement WasmLibrary struct
- [ ] Define WASM interface (WIT or manual)
- [ ] Capability-based file system access
- [ ] Update configuration format
- [ ] Basic example WASM library

### Phase 2: Enhanced Features (Future)
- [ ] Debugging support for WASM libraries
- [ ] Better error messages with stack traces
- [ ] Hot-reloading of WASM modules
- [ ] WASM compilation from Rust source
- [ ] Performance optimizations

### Phase 3: Ecosystem (Future)
- [ ] Template project for creating WASM libraries
- [ ] Standard library of WASM helpers
- [ ] Registry for sharing WASM libraries
- [ ] Documentation and tutorials

## WASM Runtime Choice: wasmer vs wasmtime

### wasmer
**Pros:**
- Mature and production-ready
- Multiple backends (LLVM, Singlepass, Cranelift)
- Good Rust integration
- Active development

**Cons:**
- Larger dependency tree
- More complex API

### wasmtime
**Pros:**
- Bytecode Alliance project
- Excellent Component Model support
- Lighter weight
- Good security track record

**Cons:**
- Less flexible backend options

**Recommendation:** Start with **wasmer** for:
1. Better Rust ecosystem integration
2. More flexible backends
3. Mature production use
4. Can switch to wasmtime later if Component Model becomes critical

## Testing Strategy

1. **Unit Tests**
   - WASM loading and initialization
   - Capability checks
   - Value serialization/deserialization
   - Error handling

2. **Integration Tests**
   - Load WASM library from config
   - Call selector functions
   - Execute check functions
   - File system access with/without capabilities

3. **Example Libraries**
   - Simple WASM library in Rust
   - Python analyzer as WASM
   - Comparison with external process version

## Migration Path

### For Users
1. Existing external libraries continue to work (backward compatible)
2. Opt-in to WASM by changing `type = "wasm"`
3. Gradually migrate performance-critical libraries to WASM

### For Library Authors
1. External process plugins remain supported
2. WASM provides new option for better performance and distribution
3. Choose based on needs:
   - Pure analysis → WASM
   - Need external tools → External process

## Documentation Updates

### external_libraries.md
- Add WASM section
- Explain when to use WASM vs external
- Provide examples of both
- Document capability system
- Show how to compile libraries to WASM

### New: wasm_library_guide.md
- Complete guide to writing WASM libraries
- Rust project setup
- Building to WASM
- Testing locally
- Debugging tips

## Example: Simple WASM Library

```rust
// examples/wasm_libraries/simple/src/lib.rs

use hielements_wasm_sdk::*;

#[wasm_library]
pub struct SimpleLibrary;

impl Library for SimpleLibrary {
    fn metadata() -> Metadata {
        Metadata {
            name: "simple".to_string(),
            version: "1.0.0".to_string(),
            functions: vec!["module_selector".to_string()],
            checks: vec!["has_tests".to_string()],
        }
    }
    
    fn call(function: &str, args: Vec<Value>, workspace: &str) -> Result<Value, String> {
        match function {
            "module_selector" => {
                let path = args[0].as_string()?;
                let files = list_directory(&format!("{}/{}", workspace, path))?;
                Ok(Value::scope(ScopeKind::Folder(path), files, true))
            }
            _ => Err(format!("Unknown function: {}", function))
        }
    }
    
    fn check(function: &str, args: Vec<Value>, workspace: &str) -> Result<CheckResult, String> {
        match function {
            "has_tests" => {
                let scope = args[0].as_scope()?;
                let has_test = scope.paths.iter()
                    .any(|p| p.contains("test"));
                if has_test {
                    Ok(CheckResult::Pass)
                } else {
                    Ok(CheckResult::Fail("No test files found".to_string()))
                }
            }
            _ => Err(format!("Unknown check: {}", function))
        }
    }
}
```

## Benefits Summary

### For Users
- **Safer**: Sandboxed libraries can't damage the system
- **Faster**: No process spawning overhead
- **Easier Distribution**: Single WASM file works everywhere
- **Fine-grained Control**: Choose capabilities per library

### For Library Authors
- **Simpler Development**: No need to implement JSON-RPC
- **Better Debugging**: In-process debugging
- **Portable**: Write once, run anywhere
- **Standard Interface**: WIT provides clear contract

### For Hielements Project
- **Security**: Safer to run untrusted libraries
- **Performance**: Faster execution
- **Ecosystem**: Easier to build library marketplace
- **Future-proof**: WASM is growing standard

## Backward Compatibility

- All existing external process libraries continue to work
- Configuration without `type` defaults to external process
- No breaking changes to existing APIs
- Migration is opt-in

## Open Questions

1. Should we support mixed libraries (WASM + external process fallback)?
2. How to handle WASM libraries that need database access?
3. Should we build a WASM library SDK/helpers package?
4. What's the story for debugging WASM libraries?

## Conclusion

Adding WASM support alongside external process plugins provides the best of both worlds:
- **WASM** for safe, fast, portable pure analysis code
- **External processes** when you need to call existing tools

This hybrid approach maximizes flexibility while improving safety and performance where possible.
