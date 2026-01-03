# Documentation Extraction Feature (Docstrings)

**Issue:** Documentation extraction for Hielements libraries  
**Date:** 2026-01-03

## Summary

This document describes the implementation of an automatic documentation extraction feature for Hielements libraries. This feature allows creating catalogs of patterns, checks, and scope selectors that can be used by both humans (Markdown) and AI agents (JSON).

## Goals

1. **Self-documenting libraries**: Libraries can provide metadata about their available functions, checks, and patterns
2. **Dual-format catalogs**: Generate both human-readable Markdown and agent-readable JSON documentation
3. **Extensibility support**: Works with both built-in and user-defined external libraries
4. **Pattern catalog regeneration**: Use this feature to regenerate the existing patterns catalog

## Design

### Library Documentation Interface

Extend the `Library` trait with documentation capabilities:

```rust
/// Metadata about a library function or check
pub struct FunctionDoc {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDoc>,
    pub return_type: String,
    pub example: Option<String>,
}

pub struct ParameterDoc {
    pub name: String,
    pub param_type: String,
    pub description: String,
}

/// Extended Library trait with documentation support
pub trait Library {
    fn name(&self) -> &str;
    fn call(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value>;
    fn check(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult>;
    
    // New documentation methods with default implementations
    fn description(&self) -> Option<&str> { None }
    fn version(&self) -> Option<&str> { None }
    fn functions(&self) -> Vec<FunctionDoc> { vec![] }
    fn checks(&self) -> Vec<FunctionDoc> { vec![] }
}
```

### JSON-RPC Protocol Extension

For external libraries, add a new `library.doc` method:

```json
// Request
{"jsonrpc": "2.0", "method": "library.doc", "id": 1}

// Response
{
    "jsonrpc": "2.0",
    "result": {
        "name": "mylibrary",
        "description": "Custom library for Python analysis",
        "version": "1.0.0",
        "functions": [
            {
                "name": "module_selector",
                "description": "Select Python modules by path pattern",
                "parameters": [
                    {"name": "path", "type": "string", "description": "Path pattern"}
                ],
                "return_type": "Scope",
                "example": "mylibrary.module_selector('src/')"
            }
        ],
        "checks": [
            {
                "name": "has_init",
                "description": "Check if a Python package has __init__.py",
                "parameters": [
                    {"name": "scope", "type": "Scope", "description": "Package scope"}
                ],
                "return_type": "CheckResult",
                "example": "check mylibrary.has_init(src)"
            }
        ]
    },
    "id": 1
}
```

### CLI Command

Add `hielements doc` command:

```bash
# Generate human-readable documentation
hielements doc --format markdown

# Generate agent-readable documentation  
hielements doc --format json

# Document specific libraries
hielements doc --library files --library rust

# Output to file
hielements doc --format markdown --output catalog.md
```

### Output Formats

**Human-readable (Markdown)**:
```markdown
# Hielements Library Documentation

## files

File system operations library.

### Selectors

#### `files.file_selector(path: string) -> Scope`

Select a single file by path.

**Parameters:**
- `path` (string): Relative path from workspace

**Example:**
```hielements
scope main = files.file_selector('src/main.rs')
```

### Checks

#### `files.exists(scope: Scope, filename: string) -> CheckResult`

Check if a file exists within a scope.

**Parameters:**
- `scope` (Scope): The scope to check
- `filename` (string): The filename to look for

**Example:**
```hielements
check files.exists(src, 'main.rs')
```
```

**Agent-readable (JSON)**:
```json
{
    "version": "1.0",
    "libraries": [
        {
            "name": "files",
            "description": "File system operations library",
            "functions": [...],
            "checks": [...]
        }
    ]
}
```

## Implementation Plan

### Changes to hielements.hie

```hielements
## Standard Library
element stdlib:
    # ... existing content ...
    
    ## Documentation module
    element documentation:
        scope doc_module = files.file_selector('crates/hielements-core/src/doc.rs')
        
        check files.exists(stdlib_src, 'doc.rs')
        check rust.struct_exists('FunctionDoc')
        check rust.struct_exists('ParameterDoc')
        check rust.struct_exists('LibraryDoc')
        check rust.function_exists('generate_markdown')
        check rust.function_exists('generate_json')
```

### Code Changes

1. **Create `doc.rs` module** in `hielements-core/src/`
   - Define `FunctionDoc`, `ParameterDoc`, `LibraryDoc` structs
   - Implement documentation extraction from libraries
   - Implement Markdown and JSON output generators

2. **Extend `Library` trait** in `stdlib/mod.rs`
   - Add `description()`, `version()`, `functions()`, `checks()` methods
   - Provide default implementations returning empty/None

3. **Update built-in libraries** (`files.rs`, `rust.rs`, `python.rs`)
   - Implement documentation methods with actual descriptions
   - Document all available selectors and checks

4. **Update `ExternalLibrary`** in `stdlib/external.rs`
   - Add support for `library.doc` JSON-RPC method
   - Parse and expose documentation from external libraries

5. **Add CLI command** in `hielements-cli/src/main.rs`
   - Add `doc` subcommand with format and output options
   - Implement documentation generation workflow

### Files to Create/Modify

| File | Action | Description |
|------|--------|-------------|
| `crates/hielements-core/src/doc.rs` | Create | Documentation types and generators |
| `crates/hielements-core/src/lib.rs` | Modify | Export doc module |
| `crates/hielements-core/src/stdlib/mod.rs` | Modify | Extend Library trait |
| `crates/hielements-core/src/stdlib/files.rs` | Modify | Add documentation |
| `crates/hielements-core/src/stdlib/rust.rs` | Modify | Add documentation |
| `crates/hielements-core/src/stdlib/python.rs` | Modify | Add documentation |
| `crates/hielements-core/src/stdlib/external.rs` | Modify | Support library.doc |
| `crates/hielements-cli/src/main.rs` | Modify | Add doc command |
| `hielements.hie` | Modify | Add documentation element |
| `doc/external_libraries.md` | Modify | Document library.doc method |

## Testing Strategy

1. **Unit tests** for documentation struct serialization
2. **Integration tests** for built-in library documentation extraction
3. **End-to-end tests** for CLI command execution
4. **Test external library** with documentation support

## Security Considerations

- Documentation is read-only, no security implications
- External libraries only expose metadata, no code execution for docs

## Conclusion

This feature enables automatic documentation generation for Hielements libraries, supporting both human and AI consumption. The design builds on the existing extensibility mechanism and maintains backward compatibility by using default trait implementations.
