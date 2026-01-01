# Comprehensive Language and Toolchain Review

**Issue:** Perform comprehensive review of the language and toolchain implementation

**Date:** 2026-01-01

## Summary

This document describes the comprehensive review of the Hielements language and toolchain implementation, including:

1. Removal of backward compatibility features (legacy `requires_descendant`, `allows_connection`, etc.)
2. Documentation of identifier scope management
3. Hierarchical split of `hielements.hie` into multiple files
4. Update of documentation and examples

---

## 1. Identifier Scope Management

### Overview

Hielements uses a hierarchical scope management system where identifiers are resolved based on their position in the element hierarchy. See `doc/scope_management.md` for full documentation.

### Key Points

1. **Element-Prefixed Scopes**: Scopes are stored with fully-qualified keys using the element path (e.g., `hielements.core.parser.module`).

2. **Resolution Priority**:
   - First, try exact match in current element's context: `{current_element_path}.{identifier}`
   - Then, try suffix match: any scope ending with `.{identifier}`
   - Finally, check for exact bare identifier match

3. **Template Binding Paths**: Use absolute paths starting with the template name to avoid name clashes.

---

## 2. Removed Legacy Features

The following backward compatibility features were removed:

### Legacy Keywords Removed

| Old Keyword | New Syntax |
|-------------|------------|
| `requires_descendant` | `requires descendant` |
| `allows_connection` | `allows connection to` |
| `forbids_connection` | `forbids connection to` |
| `requires_connection` | `requires connection to` |

### Migration Examples

**Before (legacy):**
```hielements
template dockerized:
    requires_descendant scope dockerfile = docker.file_selector('Dockerfile')
    forbids_connection to external.*
```

**After (unified syntax):**
```hielements
template dockerized:
    requires descendant scope dockerfile = docker.file_selector('Dockerfile')
    forbids connection to external.*
```

---

## 3. Hierarchical Split of hielements.hie

The `hielements.hie` specification is now supported by individual specification files:

### New File Structure

```
hielements.hie              # Main entry point with full specification
specs/
├── core.hie               # Core library (lexer, parser, AST, interpreter)
├── stdlib.hie             # Standard library (files, rust, external, wasm)
├── cli.hie                # CLI module
├── vscode.hie             # VS Code extension
├── documentation.hie      # Documentation structure
├── examples.hie           # Examples validation
└── cicd.hie               # CI/CD configuration
```

### Purpose

The split files allow agents and humans to read only the relevant parts when working on specific areas of the codebase.

---

## 4. Files Changed

### Core Changes

#### Lexer (lexer.rs)
- Removed: `RequiresDescendant`, `AllowsConnection`, `ForbidsConnection`, `RequiresConnection` token kinds
- Updated: Token kind tests

#### Parser (parser.rs)
- Removed: Parsing for legacy keywords
- Removed: `parse_hierarchical_requirement()` and `parse_connection_boundary()` methods
- Removed: Legacy tests
- Updated: Error messages to reflect new syntax only

#### AST (ast.rs)
- Removed: `HierarchicalRequirement`, `HierarchicalRequirementKind` types
- Removed: `ConnectionBoundary`, `ConnectionBoundaryKind` types
- Removed: Legacy fields from `Element` and `Template` structs

#### Interpreter (interpreter.rs)
- Removed: `validate_hierarchical_requirement()` method
- Added: `validate_component_requirement()` method
- Simplified: Validation and execution paths

### Documentation Updates
- Updated: `language_reference.md` - Removed legacy syntax sections
- Created: `scope_management.md` - New documentation on scope handling
- Updated: `examples/hierarchical.hie` - Use new syntax only

### New Files
- `specs/core.hie` - Core library specification
- `specs/stdlib.hie` - Standard library specification
- `specs/cli.hie` - CLI specification
- `specs/vscode.hie` - VS Code extension specification
- `specs/documentation.hie` - Documentation specification
- `specs/examples.hie` - Examples specification
- `specs/cicd.hie` - CI/CD specification

---

## 5. Breaking Changes

This is a **breaking change** for users who depend on the legacy syntax. Migration is straightforward:

| Pattern | Migration |
|---------|-----------|
| `requires_descendant scope` | `requires descendant scope` |
| `requires_descendant check` | `requires descendant check` |
| `requires_descendant element` | `requires descendant element` |
| `requires_descendant implements` | `requires descendant implements` |
| `allows_connection to` | `allows connection to` |
| `forbids_connection to` | `forbids connection to` |
| `requires_connection to` | `requires connection to` |

---

## 6. Testing

All 44 tests pass after the changes:
- Lexer tests: Updated to test unified keywords only
- Parser tests: Updated to test unified syntax only
- All existing functionality preserved

---

## 7. Validation

Both `hielements.hie` and `examples/hierarchical.hie` validate successfully with the CLI:
```bash
cargo run -- check hielements.hie
cargo run -- check examples/hierarchical.hie
```
