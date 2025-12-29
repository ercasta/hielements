# User-Defined Checks: Extensibility Mechanism Design

**Issue:** #User-defined checks  
**Date:** 2025-12-29

## Summary

This document evaluates different options for implementing a "plugin" / extension mechanism that allows users to define their own selectors and checks in Hielements. The goal is to enable users to create new hielements libraries (e.g., `import mylibrary`) without modifying the core framework.

## Current State

Currently, the Hielements system has:
- A `Library` trait in `crates/hielements-core/src/stdlib/mod.rs` that defines the interface for libraries
- A `LibraryRegistry` that manages available libraries
- Two built-in libraries: `files` (file system operations) and `rust` (Rust code analysis)
- Libraries are registered at interpreter construction time in `LibraryRegistry::new()`

The current architecture already provides the foundation for extensibility through the `Library` trait:

```rust
pub trait Library {
    fn name(&self) -> &str;
    fn call(&self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value>;
    fn check(&self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult>;
}
```

## Extensibility Options

### Option 1: External Process Plugins (Recommended)

**Description:** Libraries are implemented as external executables that communicate with the Hielements interpreter via JSON-RPC over stdio.

**How it works:**
1. User creates an executable (in any language) that follows a simple JSON-RPC protocol
2. Libraries are registered in a configuration file (e.g., `hielements.toml` or `hielements.yaml`)
3. The interpreter spawns the external process when the library is imported
4. Communication happens via stdin/stdout with JSON-RPC 2.0 protocol

**Protocol Example:**
```json
// Request
{"jsonrpc": "2.0", "method": "call", "params": {"function": "module_selector", "args": [{"String": "mymodule"}], "workspace": "/path/to/project"}, "id": 1}

// Response
{"jsonrpc": "2.0", "result": {"Scope": {"kind": {"File": "mymodule"}, "paths": ["/path/to/file.py"], "resolved": true}}, "id": 1}
```

**Pros:**
| Advantage | Description |
|-----------|-------------|
| **Language agnostic** | Plugins can be written in any language (Python, JavaScript, Go, etc.) |
| **Security** | Process isolation provides security boundary |
| **Stability** | Plugin crashes don't crash the interpreter |
| **Easy to develop** | Simple protocol, familiar tooling |
| **Ecosystem reuse** | Can leverage existing language-specific analysis tools |
| **Progressive adoption** | Easy to convert external tools into plugins |
| **No ABI concerns** | No shared library versioning issues |

**Cons:**
| Disadvantage | Description |
|--------------|-------------|
| **Performance** | Process spawning and IPC overhead |
| **Complexity** | Requires protocol implementation |
| **Distribution** | Users must install/manage external binaries |
| **Debugging** | Cross-process debugging is harder |

### Option 2: WebAssembly (WASM) Plugins

**Description:** Libraries are compiled to WebAssembly and loaded at runtime, providing sandboxed execution with near-native performance.

**How it works:**
1. Users write plugins in Rust, AssemblyScript, or any WASM-capable language
2. Plugins are compiled to `.wasm` files
3. The interpreter loads and executes WASM modules in a sandboxed runtime
4. Communication via WASM component model or simple function exports

**Pros:**
| Advantage | Description |
|-----------|-------------|
| **Performance** | Near-native execution speed |
| **Security** | Strong sandboxing capabilities |
| **Portability** | Single binary works on all platforms |
| **Small footprint** | WASM binaries are typically small |

**Cons:**
| Disadvantage | Description |
|--------------|-------------|
| **Ecosystem maturity** | WASM component model still evolving |
| **Language limitations** | Not all languages compile to WASM easily |
| **File system access** | Requires careful capability management |
| **Learning curve** | WASM toolchain can be complex |
| **Memory constraints** | Limited memory model |

### Option 3: Embedded Scripting Language (Rhai/Lua)

**Description:** Embed a scripting language that allows users to write libraries in a sandboxed scripting environment.

**How it works:**
1. Hielements embeds Rhai (Rust) or Lua interpreter
2. Users write library scripts in the embedded language
3. Scripts are loaded from `.rhai` or `.lua` files
4. Core types and functions are exposed to the scripting environment

**Example (Rhai):**
```rhai
// mylibrary.rhai
fn name() {
    "mylibrary"
}

fn call(function, args, workspace) {
    if function == "custom_selector" {
        let path = args[0];
        return scope_from_glob(workspace + "/" + path);
    }
}

fn check(function, args, workspace) {
    if function == "custom_check" {
        return check_pass();
    }
}
```

**Pros:**
| Advantage | Description |
|-----------|-------------|
| **Integration** | Deep integration with Rust core |
| **Sandboxing** | Built-in sandbox capabilities |
| **No distribution** | Scripts are text files, easy to share |
| **Fast iteration** | No compilation needed |
| **Single process** | No IPC overhead |

**Cons:**
| Disadvantage | Description |
|--------------|-------------|
| **New language** | Users must learn another language |
| **Limited ecosystem** | Cannot easily use existing analysis libraries |
| **Performance** | Slower than native for complex analysis |
| **Capability limits** | May need to expose many core functions |

### Option 4: Rust Dynamic Libraries (FFI)

**Description:** Libraries are compiled as shared libraries (.so/.dll/.dylib) that implement a C-compatible FFI interface.

**Pros:**
| Advantage | Description |
|-----------|-------------|
| **Performance** | Native execution speed |
| **Full capabilities** | Access to all system resources |

**Cons:**
| Disadvantage | Description |
|--------------|-------------|
| **Platform specific** | Must compile for each platform |
| **ABI stability** | Rust has no stable ABI |
| **Security risks** | No isolation from host process |
| **Complexity** | Complex FFI interface design |
| **Distribution** | Binary compatibility issues |

### Option 5: Hybrid Approach

**Description:** Support multiple plugin mechanisms, starting with the simplest and adding more as needed.

**Implementation phases:**
1. **Phase 1:** External process plugins via JSON-RPC (easiest, most flexible)
2. **Phase 2:** Embedded Rhai scripts (for simple customizations)
3. **Phase 3:** WASM plugins (for performance-critical plugins)

## Recommendation: External Process Plugins (Option 1)

For the initial implementation, **External Process Plugins** is recommended because:

1. **Matches existing architecture pattern:** The technical architecture document already mentions JSON-RPC over stdio for external tools
2. **Maximum flexibility:** Users can use any language they're comfortable with
3. **Easy migration path:** Existing analysis tools can be wrapped as plugins
4. **Lower barrier to entry:** No need to learn WASM or embedded languages
5. **Alignment with LSP patterns:** Similar to how Language Server Protocol works

## Implementation Plan

### Changes to hielements.hie

```hielements
## Standard Library
element stdlib:
    # ... existing content ...
    
    ## External Library Support
    element external:
        scope module = rust.module_selector('external')
        
        check rust.trait_exists('ExternalLibrary')
        check rust.struct_exists('ExternalLibraryLoader')
        check rust.function_exists('load_external_library')
```

### Changes to Code

1. **Add configuration file support** (hielements.toml)
2. **Create ExternalLibrary wrapper** that implements the Library trait
3. **Implement JSON-RPC protocol** for external process communication
4. **Add library discovery** from configuration
5. **Update LibraryRegistry** to support loading external libraries

### Configuration Format

```toml
# hielements.toml
[libraries]
python = { executable = "hielements-python-lib", args = [] }
docker = { executable = "hielements-docker-lib" }
mylibrary = { executable = "./scripts/mylibrary.py", args = ["--workspace"] }
```

### Protocol Specification

```json
// Metadata request
{"jsonrpc": "2.0", "method": "library.metadata", "id": 1}
// Response: {"jsonrpc": "2.0", "result": {"name": "python", "version": "1.0", "functions": ["module_selector", "function_exists"]}, "id": 1}

// Call request
{"jsonrpc": "2.0", "method": "library.call", "params": {"function": "module_selector", "args": [...], "workspace": "..."}, "id": 2}

// Check request  
{"jsonrpc": "2.0", "method": "library.check", "params": {"function": "function_exists", "args": [...], "workspace": "..."}, "id": 3}
```

## Testing Strategy

1. Create a simple test plugin in Python that implements the protocol
2. Write integration tests that load and use the external plugin
3. Verify error handling for plugin failures

## Security Considerations

1. Plugins run in separate processes (isolation)
2. Workspace path is passed explicitly (no implicit access)
3. Consider adding allowlist for allowed executables
4. Document security model for users

## Conclusion

The external process plugin approach provides the best balance of flexibility, security, and ease of implementation. It allows users to leverage their existing skills and tools while maintaining a clear boundary between the core interpreter and user-defined functionality.
