# Technical Architecture Plan

This document describes the technical architecture for developing Hielements, including technologies used, alternatives considered, and an analysis of pros/cons for the architectural choices specified in the summary.

---

## 1. Overview

Hielements is a language for describing and enforcing software structure. The implementation consists of:

1. **Interpreter** - Core language execution engine (Rust)
2. **Language Server Protocol (LSP)** - IDE integration
3. **VSCode Extension** - Syntax highlighting and LSP client
4. **Standard Libraries** - Language-specific support (Python, Docker, files/folders)

---

## 2. Core Technology Choices

### 2.1 Interpreter Language: Rust

**Choice from Summary:** The interpreter is written in Rust.

#### Pros
| Advantage | Description |
|-----------|-------------|
| **Performance** | Near-C performance for parsing and rule evaluation; critical for large codebases |
| **Memory Safety** | No garbage collector pauses; eliminates entire classes of bugs |
| **Concurrency** | Fearless concurrency enables parallel rule checking |
| **Ecosystem** | Excellent parsing libraries (nom, pest, lalrpop, tree-sitter) |
| **Cross-Platform** | Single codebase compiles to Windows, macOS, Linux |
| **WASM Support** | Can compile to WebAssembly for browser/edge use cases |
| **Reliability** | Strong type system catches errors at compile time |

#### Cons
| Disadvantage | Description |
|--------------|-------------|
| **Learning Curve** | Steeper than Python/JavaScript for contributors |
| **Compilation Time** | Slower build times than interpreted languages |
| **Ecosystem Size** | Smaller than Python/JS for some tooling needs |
| **FFI Complexity** | Calling external tools requires careful interface design |

#### Alternatives Considered

| Alternative | Pros | Cons | Verdict |
|-------------|------|------|---------|
| **Go** | Fast compilation, simple concurrency, good tooling | Less expressive type system, GC pauses, weaker parsing ecosystem | Viable but Rust offers better performance and type safety |
| **Python** | Rapid development, huge ecosystem, easy contributor onboarding | Slow execution, GIL limits concurrency, distribution challenges | Unsuitable for performance-critical interpreter |
| **TypeScript/Node.js** | Large ecosystem, easy LSP development, web integration | Performance limitations, single-threaded, distribution via npm | Could work but performance concerns for large codebases |
| **OCaml/F#** | Excellent for language implementation, pattern matching | Smaller community, less tooling, platform concerns (F#/.NET) | Strong candidate but Rust has broader adoption |
| **C++** | Maximum performance, mature parsing tools | Memory safety issues, complexity, slower iteration | Rust provides similar performance with safety guarantees |

**Recommendation:** Rust is an excellent choice. Consider providing Python bindings (via PyO3) for users who want to script custom rules.

---

### 2.2 Interpreted Language Model

**Choice from Summary:** Hielements is an interpreted language.

#### Pros
| Advantage | Description |
|-----------|-------------|
| **Rapid Iteration** | No compilation step for users writing Hielements specs |
| **Dynamic Loading** | Easy to load and reload specifications |
| **Debugging** | Simpler to implement interactive debugging |
| **Extensibility** | Runtime extension via external tool invocation |

#### Cons
| Disadvantage | Description |
|--------------|-------------|
| **Performance** | Slower than compiled approaches for complex rule evaluation |
| **Type Errors** | Some errors only detected at runtime |
| **Optimization** | Limited optimization opportunities |

#### Alternatives Considered

| Alternative | Pros | Cons | Verdict |
|-------------|------|------|---------|
| **Compiled to Native** | Maximum performance | Slow iteration, complex implementation | Overkill for a DSL |
| **Bytecode VM** | Balance of performance and flexibility | Added complexity | Consider for future optimization |
| **Transpiled to Host Language** | Leverage existing tooling | Lose control over semantics | Limits language design |

**Recommendation:** Interpretation is appropriate. Consider a bytecode VM as a future optimization if performance becomes an issue.

---

### 2.3 Static Analysis Phase

**Requirement:** Even though Hielements is interpreted, static checks for syntax and semantics must be performed before actual execution.

#### Design Principles

1. **Validation Before Execution:** All Hielements specifications undergo syntax and semantic validation before any rule execution
2. **Deferred Execution:** The actual checks (invoking external tools, evaluating rules) are separate from validation
3. **Machine and Human Readable Output:** Check results must be consumable by CI/CD pipelines and readable by developers

#### Validation Phases

| Phase | Description | Checks Performed |
|-------|-------------|------------------|
| **Lexical Analysis** | Tokenization | Invalid characters, malformed tokens |
| **Syntax Validation** | Parse against grammar | Structure errors, missing delimiters |
| **Semantic Analysis** | IR construction and validation | Undefined references, type mismatches, invalid scopes |
| **Dependency Resolution** | External tool availability | Missing tools, invalid tool configurations |

#### Execution Deferral Options

To separate validation from actual execution, several approaches are possible:

| Approach | Description | Pros | Cons |
|----------|-------------|------|------|
| **`--check` / `--validate` flag** | Runs only validation, no execution | Simple, explicit | Separate command invocation |
| **`--dry-run` flag** | Validates and simulates execution without side effects | Shows execution plan | May not catch all runtime issues |
| **Two-phase API** | `validate()` returns validated IR, `execute(ir)` runs it | Programmatic control, composable | More complex API |
| **Validation-only mode (default)** | Default behavior is validate-only, `--execute` to run | Safe by default | Might confuse users expecting execution |

**Recommendation:** Implement a **two-phase architecture** internally:
1. **Phase 1: Validation** - Always runs, produces validated IR or errors
2. **Phase 2: Execution** - Only runs if validation passes and execution is requested

Expose this via CLI as:
- `hielements check <spec>` - Validate only (default, fast feedback)
- `hielements run <spec>` - Validate + Execute
- `hielements run --dry-run <spec>` - Validate + Show execution plan without invoking external tools

#### Check Output Formats

Results must be provided in formats usable by both automated tools and humans:

| Format | Use Case | Description |
|--------|----------|-------------|
| **Human-readable (default)** | Terminal output | Colored, formatted errors with source context |
| **JSON** | CI/CD integration | Structured output for programmatic consumption |
| **SARIF** | GitHub/GitLab integration | Static Analysis Results Interchange Format |
| **JUnit XML** | CI test reporters | Compatible with most CI systems |
| **Checkstyle XML** | Legacy CI integration | Wide tool support |

```bash
# Human-readable output (default)
hielements check spec.hie

# JSON output for scripting
hielements check spec.hie --format json

# SARIF for GitHub code scanning
hielements check spec.hie --format sarif > results.sarif

# Exit codes for CI/CD
# 0 = success, 1 = validation errors, 2 = execution failures
```

**JSON Output Schema Example:**
```json
{
  "version": "1.0",
  "status": "error",
  "errors": [
    {
      "severity": "error",
      "code": "E001",
      "message": "Undefined element 'orders'",
      "file": "architecture.hie",
      "line": 15,
      "column": 10,
      "context": "    uses orders.api"
    }
  ],
  "warnings": [],
  "summary": {
    "total_errors": 1,
    "total_warnings": 0
  }
}
```

**Note:** Output formatting is the responsibility of the CLI tool and CI integration tooling, not the core interpreter library. The interpreter exposes structured diagnostic objects that tools format as needed.

---

### 2.4 External Tool Invocation Model

**Choice from Summary:** Actual scope and rules implementation is implemented using external software; Hielements provides a way to invoke external tools, passing parameters and getting the results.

#### Pros
| Advantage | Description |
|-----------|-------------|
| **Extensibility** | Support any language/technology via external tools |
| **Separation of Concerns** | Core interpreter stays simple; complexity in plugins |
| **Reuse** | Leverage existing static analysis tools |
| **Polyglot** | Tools can be written in any language |

#### Cons
| Disadvantage | Description |
|--------------|-------------|
| **Performance Overhead** | Process spawning and IPC adds latency |
| **Error Handling** | Complex failure modes (tool not found, crashes, timeouts) |
| **Security** | Executing external code has security implications |
| **Distribution** | Users must install external tools |
| **Interface Stability** | External tool APIs may change |

#### Alternatives Considered

| Alternative | Pros | Cons | Verdict |
|-------------|------|------|---------|
| **Plugin DLLs/SOs** | Fast, in-process execution | Language-specific, ABI stability issues, security risks | Complex and limits plugin languages |
| **WASM Plugins** | Sandboxed, portable, fast | Ecosystem maturity, memory constraints | **Implemented (experimental)** - Infrastructure in place for future use |
| **Embedded Scripting (Lua/Rhai)** | Fast, sandboxed, easy integration | Another language to learn, limited ecosystem | Good for simple customizations |
| **gRPC/HTTP Services** | Language-agnostic, networkable | Operational overhead, latency | Overkill for local analysis |

**Current Status**: 
- **External tool invocation** (JSON-RPC over stdio) is the primary plugin mechanism
- **WASM plugins** infrastructure has been implemented with capability-based security
- Both mechanisms are supported, allowing users to choose based on their needs:
  - External plugins for flexibility and existing tool integration
  - WASM plugins (when fully enabled) for enhanced security and performance

**WASM Plugin Implementation:**
1. Infrastructure components: ✅ Complete
   - WasmLibrary implementing Library trait
   - WasmCapabilities for fine-grained security
   - Configuration support in hielements.toml
   - Type conversions and serialization
2. Runtime execution: ⏳ Planned for future release
   - Wasmtime integration pending API stabilization
   - WASI file system access for workspace reads
3. Benefits when enabled:
   - Capability-based sandboxing
   - Near-native performance
   - Platform-independent binaries

---

### 2.5 Intermediate Representation (IR)

**Choice from Summary:** To allow a full-fledged implementation of the Language Server Protocol, consider using an intermediate representation within the interpreter.

#### Pros
| Advantage | Description |
|-----------|-------------|
| **LSP Support** | Enables go-to-definition, hover, completion |
| **Incremental Processing** | Recompute only changed portions |
| **Optimization** | IR can be optimized before evaluation |
| **Analysis** | Enables static analysis of Hielements specs |
| **Debugging** | Better error messages and source mapping |

#### Cons
| Disadvantage | Description |
|--------------|-------------|
| **Complexity** | Additional layer to implement and maintain |
| **Memory** | IR consumes additional memory |
| **Development Time** | Longer initial implementation |

#### Recommended IR Design

```
┌─────────────────────────────────────────────────────────────────┐
│                        Source Text                               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Concrete Syntax Tree (CST)                    │
│         (Preserves whitespace, comments for formatting)          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Abstract Syntax Tree (AST)                     │
│              (Semantic structure, source locations)              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    High-Level IR (HIR)                           │
│      (Resolved names, type info, desugared constructs)           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Evaluation                                │
└─────────────────────────────────────────────────────────────────┘
```

**Recommendation:** Implement at minimum AST with source locations. HIR is valuable for LSP features. Consider using `salsa` crate for incremental computation.

---

## 3. Tooling Architecture

### 3.1 Language Server Protocol (LSP)

#### Architecture Options

| Option | Description | Pros | Cons |
|--------|-------------|------|------|
| **Embedded in Interpreter** | LSP server built into main binary | Single distribution, shared code | Larger binary, coupling |
| **Separate Process** | LSP as separate binary communicating with interpreter | Clean separation, independent updates | IPC overhead, two binaries |
| **Library Mode** | Interpreter as library used by LSP server binary | Flexible, testable | API design effort |

**Recommendation:** Library mode with the interpreter as a Rust library. The LSP **server binary** links against it. This enables:
- Unit testing of interpreter logic
- Reuse in other tools (CI runners, etc.)
- Clean API boundaries

#### VSCode Integration Architecture

**Important Clarification:** VSCode does **not** require a REST API for LSP integration. The LSP protocol uses **stdio** (standard input/output) or **socket** communication, not HTTP/REST.

```
┌─────────────────────────────────────────────────────────────────┐
│                        VSCode                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Hielements Extension (TypeScript)            │   │
│  │  - Uses vscode-languageclient library                     │   │
│  │  - Spawns LSP server as child process                     │   │
│  │  - Communicates via stdio (JSON-RPC)                      │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │ stdio (JSON-RPC)
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Hielements LSP Server Binary                    │
│  - Standalone executable (hielements-lsp or hielements --lsp)   │
│  - Built with tower-lsp crate                                   │
│  - Links against interpreter library                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Interpreter Library (Rust)                      │
│  - Parsing, validation, IR construction                          │
│  - Exposes API for diagnostics, symbols, completions            │
└─────────────────────────────────────────────────────────────────┘
```

**Key Points:**
- The interpreter is a **Rust library** (crate) that exposes APIs for parsing, validation, and analysis
- The LSP server is a **standalone binary** that uses the interpreter library and implements the LSP protocol
- VSCode extension **spawns this binary** and communicates via stdio using JSON-RPC (standard LSP protocol)
- No REST API or HTTP server is needed—LSP uses stdio by default
- The `vscode-languageclient` npm package handles all protocol details on the VSCode side

**Distribution Options:**
| Option | Description | Pros | Cons |
|--------|-------------|------|------|
| **Bundled binary** | LSP binary included in VSCode extension | Zero user setup | Larger extension, platform-specific builds |
| **Separate install** | User installs LSP server independently | Smaller extension, shared binary | Extra setup step |
| **Auto-download** | Extension downloads correct binary on install | Best UX, small extension | Requires hosted binaries, network dependency |

**Recommendation:** Start with bundled binaries for each platform (Windows, macOS, Linux). Use the extension's `activate` function to locate and spawn the appropriate binary.

#### LSP Features Priority

| Priority | Feature | Complexity | Value |
|----------|---------|------------|-------|
| P0 | Syntax highlighting (via TextMate grammar) | Low | High |
| P0 | Diagnostics (errors, warnings) | Medium | High |
| P1 | Go to Definition | Medium | High |
| P1 | Hover (type info, docs) | Medium | Medium |
| P2 | Completion | High | High |
| P2 | Find References | Medium | Medium |
| P3 | Rename | High | Medium |
| P3 | Code Actions (quick fixes) | High | Medium |

---

### 3.2 VSCode Extension

#### Technology Choices

| Component | Recommended Technology | Alternatives |
|-----------|----------------------|--------------|
| Extension Language | TypeScript | JavaScript (less type safety) |
| LSP Client | vscode-languageclient | Custom implementation (unnecessary) |
| Syntax Highlighting | TextMate grammar (.tmLanguage.json) | Semantic tokens (more complex) |
| Packaging | vsce | — |
| Testing | VSCode Extension Test Framework | Mocha directly |

#### Extension Scope

The VSCode extension should be thin, primarily:
1. Launching and managing the LSP server
2. Providing TextMate grammar for syntax highlighting
3. Registering commands (run checks, show element tree)
4. Configuration UI for Hielements settings

---

## 4. Standard Libraries Architecture

### 4.1 Library Design Principles

1. **Declarative Interface:** Libraries expose selectors and checkers with clear contracts
2. **Lazy Evaluation:** Selectors should not eagerly scan; defer work until needed
3. **Caching:** Libraries should cache expensive operations (AST parsing, etc.)
4. **Error Reporting:** Rich error messages with source locations
5. **Composability:** Selectors can be combined and filtered

### 4.2 Built-in Libraries

| Library | Scope | Implementation Strategy |
|---------|-------|------------------------|
| **files** | File and folder matching | Native Rust (glob patterns, regex) |
| **python** | Python modules, functions, classes | External: tree-sitter-python or rope/jedi |
| **docker** | Dockerfile analysis | External: dockerfile-parser or native |
| **generic** | Language-agnostic AST queries | External: tree-sitter grammars |

### 4.3 External Tool Protocol

Standardize communication with external tools:

```json
{
  "jsonrpc": "2.0",
  "method": "selector/evaluate",
  "params": {
    "selector": "python.module",
    "args": {"name": "orders"},
    "workspace": "/path/to/project"
  },
  "id": 1
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "matches": [
      {"file": "src/orders/__init__.py", "span": {"start": 0, "end": 100}}
    ]
  },
  "id": 1
}
```

**Recommendation:** Use JSON-RPC 2.0 over stdio (like LSP itself). This provides:
- Well-defined protocol with error handling
- Streaming support for large results
- Language-agnostic implementation

---

## 5. Potential Improvements and Future Directions

### 5.1 Performance Optimizations

| Improvement | Description | Effort | Impact |
|-------------|-------------|--------|--------|
| **Parallel Rule Evaluation** | Evaluate independent rules concurrently | Medium | High |
| **Incremental Checking** | Only re-check affected elements on file change | High | High |
| **Result Caching** | Cache selector results across runs | Medium | Medium |
| **Bytecode Compilation** | Compile to bytecode for faster interpretation | High | Medium |
| **WASM Plugins** | Replace external tools with WASM for hot paths | High | High |

### 5.2 Language Enhancements

| Enhancement | Description | Effort | Impact |
|-------------|-------------|--------|--------|
| **Type System** | Add optional types for elements and connection points | High | Medium |
| **Generics** | Parameterized elements for reuse | Medium | Medium |
| **Pattern Matching** | Richer matching syntax for scopes | Medium | High |
| **Imports** | Module system for organizing Hielements specs | Medium | High |
| **Inheritance** | Element inheritance for shared rules | Medium | Medium |

### 5.3 Tooling Enhancements

| Enhancement | Description | Effort | Impact |
|-------------|-------------|--------|--------|
| **Visualization** | Generate diagrams from Hielements specs | Medium | High |
| **CI Integration** | GitHub Actions, GitLab CI templates | Low | High |
| **Auto-Discovery** | Generate Hielements specs from existing code | High | High |
| **IDE Integrations** | JetBrains, Neovim, Emacs support | Medium | Medium |
| **Web Playground** | Browser-based Hielements editor (via WASM) | Medium | Medium |

### 5.4 Ecosystem Improvements

| Improvement | Description | Effort | Impact |
|-------------|-------------|--------|--------|
| **Package Registry** | Central registry for Hielements libraries | High | High |
| **Library Generator** | Scaffolding for new language libraries | Low | Medium |
| **Documentation Generator** | Generate docs from Hielements specs | Medium | Medium |
| **Test Framework** | Testing utilities for Hielements specs | Medium | Medium |

---

## 6. Risk Analysis

### 6.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **External tool reliability** | Medium | High | Comprehensive error handling, fallback mechanisms, tool health checks |
| **Performance with large codebases** | Medium | High | Early benchmarking, incremental processing, caching |
| **LSP complexity** | Medium | Medium | Incremental feature rollout, leverage tower-lsp crate |
| **Cross-platform issues** | Low | Medium | CI testing on all platforms, avoid platform-specific code |

### 6.2 Adoption Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **Learning curve** | High | Medium | Excellent documentation, examples, tutorials |
| **Ecosystem fragmentation** | Medium | Medium | Strong standard library, contribution guidelines |
| **Competition from existing tools** | Medium | Medium | Focus on unique value props (cross-technology, hierarchical) |

---

## 7. Recommended Development Phases

### Phase 1: Foundation (MVP)
- [ ] Core interpreter with basic syntax
- [ ] File/folder library (native Rust)
- [ ] Basic CLI for running checks
- [ ] TextMate grammar for VSCode

### Phase 2: Language Support
- [ ] Python library (via tree-sitter)
- [ ] Docker library
- [ ] External tool protocol (JSON-RPC)
- [ ] Basic LSP (diagnostics, hover)

### Phase 3: Polish
- [ ] Full LSP features (completion, go-to-definition)
- [ ] VSCode extension with configuration UI
- [ ] Documentation and examples
- [ ] CI/CD templates

### Phase 4: Scale
- [ ] Incremental checking
- [ ] Parallel evaluation
- [ ] Additional language libraries
- [ ] Visualization tools

---

## 8. Technology Stack Summary

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Interpreter | Rust | Performance, safety, ecosystem |
| Parsing | pest or tree-sitter | Well-tested, good error recovery |
| LSP Server | tower-lsp (Rust) | Async, well-maintained |
| VSCode Extension | TypeScript | Standard for VSCode |
| External Protocol | JSON-RPC 2.0 over stdio | Language-agnostic, LSP-compatible |
| Python Analysis | tree-sitter-python | Fast, incremental parsing |
| Docker Analysis | dockerfile-parser crate | Native Rust, no external deps |
| Incremental Computation | salsa | Proven in rust-analyzer |
| Testing | Rust built-in + insta | Snapshot testing for parser output |
| Documentation | mdBook | Rust ecosystem standard |

---

## 9. Conclusion

The architectural choices in the summary are well-founded. Rust provides the right balance of performance and safety for an interpreter. The interpreted model with external tool invocation enables extensibility while keeping the core simple.

Key recommendations:
1. **Invest in IR early** for LSP support
2. **Standardize the external tool protocol** using JSON-RPC
3. **Consider WASM plugins** as a future performance optimization
4. **Prioritize incremental processing** for large codebase support
5. **Build Python bindings** to lower the contribution barrier
