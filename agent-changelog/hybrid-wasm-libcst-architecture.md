# Hybrid WASM + libcst Architecture for FastAPI Authentication Checking

## Overview

This document describes a novel hybrid architecture that combines:
1. **Built-in Python function** (in Hielements core) that calls libcst for parsing
2. **WASM plugin** (sandboxed Rust) that implements the pattern checking logic

## Architecture Goals

- ✅ **Safety**: Pattern checking logic runs in sandboxed WASM
- ✅ **Robustness**: Use libcst (mature, well-tested) for Python parsing
- ✅ **Performance**: WASM execution is near-native speed
- ✅ **Maintainability**: Complex parsing in Python, business logic in Rust
- ✅ **Extensibility**: Pattern logic can be updated without touching parser

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    Hielements Interpreter (Rust)                │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  Python Library (built-in)                                 │ │
│  │                                                            │ │
│  │  fn parse_python_ast(code: &str) -> Result<AST_JSON>     │ │
│  │    ├─> Call embedded Python interpreter (PyO3)           │ │
│  │    ├─> Execute: libcst.parse_module(code)                │ │
│  │    ├─> Convert CST to JSON representation                │ │
│  │    └─> Return JSON AST                                   │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                     │
│                            │ JSON AST                            │
│                            ▼                                     │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  WASM Plugin Executor (wasmtime)                          │ │
│  │                                                            │ │
│  │  fn check_fastapi_auth(ast_json: &str) -> CheckResult    │ │
│  │    ├─> Load WASM module (fastapi_auth.wasm)              │ │
│  │    ├─> Pass JSON AST to WASM                             │ │
│  │    ├─> WASM analyzes routes and authentication           │ │
│  │    └─> Return Pass/Fail/Error                            │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

### 1. Built-in Python Parser Function (PyO3 + libcst)

**Location**: `crates/hielements-core/src/stdlib/python.rs`

**Responsibility**: Parse Python code into a standardized JSON AST

```rust
// New function in PythonLibrary
fn parse_to_ast(&self, file_path: &str, workspace: &str) -> LibraryResult<Value> {
    // 1. Read Python file
    // 2. Call embedded Python with libcst
    // 3. Convert CST to JSON
    // 4. Return as Value::String(json)
}
```

**Usage in .hie**:
```hielements
import python
import fastapi_wasm

element my_api {
    scope api = python.module_selector('app/api.py')
    
    # Built-in parses, WASM checks
    check fastapi_wasm.all_routes_authenticated(api)
}
```

### 2. WASM Pattern Checker (Rust → WASM)

**Location**: New crate `wasm_plugins/fastapi_auth/`

**Responsibility**: Analyze JSON AST for FastAPI authentication patterns

```rust
#[no_mangle]
pub extern "C" fn check_routes_authenticated(ast_json_ptr: i32, ast_json_len: i32) -> (i32, i32) {
    // 1. Read JSON AST from memory
    // 2. Deserialize to Rust structs
    // 3. Traverse AST looking for:
    //    - @app.get/@app.post decorators
    //    - Depends(get_current_user) patterns
    //    - Security() dependencies
    // 4. Return CheckResult
}
```

### 3. Glue Layer (in WasmLibrary)

**Location**: `crates/hielements-core/src/stdlib/wasm.rs`

**Responsibility**: Bridge between Hielements and WASM plugin

```rust
impl Library for WasmLibrary {
    fn check(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult> {
        match function {
            "all_routes_authenticated" => {
                // 1. Extract scope from args
                // 2. For each Python file in scope:
                //    a. Call python.parse_to_ast()
                //    b. Pass AST JSON to WASM
                //    c. Collect results
                // 3. Return aggregated CheckResult
            }
            _ => Err(...)
        }
    }
}
```

## Implementation Plan

### Phase 1: Python Parser Integration (PyO3)

**Goal**: Add libcst parsing capability to built-in Python library

**Tasks**:
1. Add PyO3 dependency to hielements-core
2. Embed Python interpreter
3. Install libcst in embedded Python
4. Add `parse_to_ast()` function to PythonLibrary
5. Define JSON AST format
6. Add tests

**Challenges**:
- PyO3 embedding complexity
- Managing Python dependencies (libcst)
- Cross-platform compatibility

**Alternative**: Use `rustpython-parser` (pure Rust, no Python dependency)

### Phase 2: JSON AST Format Design

**Goal**: Define standard JSON representation of Python CST

**Format Example**:
```json
{
  "type": "Module",
  "body": [
    {
      "type": "FunctionDef",
      "name": "create_payment",
      "decorators": [
        {
          "type": "Decorator",
          "name": "app.post",
          "args": ["/api/payments"]
        }
      ],
      "parameters": [
        {
          "name": "current_user",
          "annotation": "User",
          "default": {
            "type": "Call",
            "func": "Depends",
            "args": ["get_current_user"]
          }
        }
      ]
    }
  ]
}
```

### Phase 3: WASM Plugin Development

**Goal**: Create fastapi_auth.wasm plugin in Rust

**Project Structure**:
```
wasm_plugins/fastapi_auth/
├── Cargo.toml
├── src/
│   ├── lib.rs          # WASM exports
│   ├── ast.rs          # JSON AST types
│   ├── analyzer.rs     # Pattern matching logic
│   └── auth_checker.rs # Authentication detection
├── tests/
│   └── integration_tests.rs
└── README.md
```

**Key Files**:

`src/lib.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ASTModule {
    #[serde(rename = "type")]
    node_type: String,
    body: Vec<ASTNode>,
}

#[no_mangle]
pub extern "C" fn alloc(size: i32) -> *mut u8 {
    // Memory allocation for host-to-WASM data transfer
}

#[no_mangle]
pub extern "C" fn check_routes_authenticated(ptr: i32, len: i32) -> (i32, i32) {
    let ast_json = read_string_from_memory(ptr, len);
    let result = analyze_authentication(&ast_json);
    write_result_to_memory(result)
}
```

`src/analyzer.rs`:
```rust
fn analyze_authentication(ast_json: &str) -> CheckResult {
    let module: ASTModule = serde_json::from_str(ast_json)?;
    
    let mut routes = vec![];
    
    // Traverse AST
    for node in &module.body {
        if let Some(route) = extract_route(node) {
            routes.push(route);
        }
    }
    
    // Check authentication
    let unauth_routes: Vec<_> = routes.iter()
        .filter(|r| !r.has_authentication())
        .collect();
    
    if unauth_routes.is_empty() {
        CheckResult::Pass
    } else {
        CheckResult::Fail(format!(
            "Found {} unauthenticated routes: {:?}",
            unauth_routes.len(),
            unauth_routes
        ))
    }
}
```

### Phase 4: Integration

**Goal**: Wire everything together in WasmLibrary

**Updates to `wasm.rs`**:
```rust
impl Library for WasmLibrary {
    fn check(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult> {
        // Get Python library instance
        let mut python_lib = PythonLibrary::new();
        
        // Extract scope
        let scope = extract_scope_from_args(&args)?;
        
        // Parse all Python files to AST
        let mut all_asts = vec![];
        for file_path in scope.paths {
            let ast_json = python_lib.parse_to_ast(&file_path, workspace)?;
            all_asts.push(ast_json);
        }
        
        // Load WASM module
        let wasm_module = load_wasm_module(&self.config.path)?;
        
        // Call WASM check for each AST
        let results: Vec<CheckResult> = all_asts.iter()
            .map(|ast| call_wasm_check(&wasm_module, function, ast))
            .collect();
        
        // Aggregate results
        aggregate_check_results(results)
    }
}
```

## Pros and Cons

### Advantages

✅ **Security**: Pattern checking logic in WASM sandbox
✅ **Performance**: Near-native WASM execution
✅ **Reliability**: Use proven libcst for parsing
✅ **Maintainability**: Clean separation of concerns
✅ **Extensibility**: Easy to add new patterns
✅ **Distribution**: Single .wasm file for all platforms
✅ **Type Safety**: Rust's type system for pattern logic

### Disadvantages

❌ **Complexity**: Multi-language, multi-component system
❌ **Dependencies**: Requires PyO3 or RustPython
❌ **Size**: Embedded Python interpreter increases binary size
❌ **Development Time**: More components to build and test
❌ **Debugging**: Harder to debug across language boundaries

## Alternative Approaches

### Alternative 1: Pure WASM (tree-sitter)

Use tree-sitter-python in WASM instead of libcst.

**Pros**: No Python dependency, pure Rust
**Cons**: tree-sitter less accurate than libcst for Python

### Alternative 2: External Process + WASM

Keep external Python process for parsing, WASM for checking.

**Pros**: Simpler than embedding Python
**Cons**: Less integrated, requires external dependencies

### Alternative 3: RustPython Parser

Use `rustpython-parser` crate (pure Rust Python parser).

**Pros**: No PyO3, no embedding, pure Rust
**Cons**: May not be as mature/complete as libcst

## Recommended Approach: Alternative 3 (RustPython)

After analysis, I recommend **Alternative 3: RustPython Parser + WASM**

### Why RustPython?

1. **Pure Rust**: No Python embedding needed
2. **Well-maintained**: Active development, good AST support
3. **Simpler**: Fewer dependencies and complexity
4. **Fast**: Native Rust performance
5. **Complete**: Full Python 3.x support

### Updated Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Hielements Interpreter (Rust)                │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  Python Library (built-in) with RustPython                 │ │
│  │                                                            │ │
│  │  fn parse_python_ast(code: &str) -> Result<AST_JSON>     │ │
│  │    ├─> Use rustpython_parser::parse()                    │ │
│  │    ├─> Convert AST to JSON representation                │ │
│  │    └─> Return JSON AST                                   │ │
│  └────────────────────────────────────────────────────────────┘ │
│                            │                                     │
│                            │ JSON AST                            │
│                            ▼                                     │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  WASM Plugin Executor (wasmtime)                          │ │
│  │                                                            │ │
│  │  Loads: fastapi_auth.wasm                                 │ │
│  │  Analyzes: Routes, authentication patterns               │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Implementation Steps

1. **Add RustPython dependency** to hielements-core
   ```toml
   rustpython-parser = "0.3"
   rustpython-ast = "0.3"
   ```

2. **Add AST parsing function** to Python library
   ```rust
   fn parse_to_ast(&self, file_path: &str) -> LibraryResult<Value> {
       use rustpython_parser::parse;
       // Parse and convert to JSON
   }
   ```

3. **Create WASM plugin** for FastAPI pattern checking
   - Pure Rust, compiles to WASM
   - Reads JSON AST
   - Implements authentication detection logic

4. **Wire together** in WasmLibrary

## Development Timeline

- **Week 1**: Add RustPython parsing to Python library
- **Week 2**: Design and implement JSON AST format
- **Week 3**: Create WASM plugin for FastAPI auth checking
- **Week 4**: Integration and testing
- **Week 5**: Documentation and examples

## Success Criteria

1. ✅ Parse Python code using RustPython (built-in)
2. ✅ Convert AST to JSON format
3. ✅ WASM plugin successfully analyzes JSON AST
4. ✅ Detect authenticated routes (Depends, Security, decorators)
5. ✅ Detect unauthenticated routes
6. ✅ Integration with existing Hielements patterns
7. ✅ Tests pass with 100% accuracy
8. ✅ Performance < 100ms per file
9. ✅ Documentation complete
10. ✅ Example projects working

## Conclusion

The **RustPython + WASM hybrid approach** provides:
- Pure Rust implementation (no Python embedding)
- Sandboxed pattern checking (WASM security)
- Fast performance (native parsing + WASM execution)
- Clean architecture (separation of parsing and analysis)

This is the recommended path forward for implementing secure, fast, and maintainable FastAPI authentication checking in Hielements.
