# Consistency Restoration Analysis

**Date**: 2026-03-08
**Context**: Resume development after a pause. Analyze inconsistencies and restore consistency before implementing scope binding enforcement.

## Analysis Summary

After thorough analysis of the codebase, the following inconsistencies and incomplete features were identified:

### 1. MCP Server vs CLI Focus

**Finding**: The workspace includes `hielements-mcp` as a member crate, but the project direction is to be used as a CLI tool, not an MCP server.

**Changes**:
- Remove `hielements-mcp` from workspace `members` list (code stays, just excluded from default build)
- Update `hielements.hie` to remove implicit MCP dependency
- Update README.md and USAGE.md to de-emphasize MCP and focus on CLI

### 2. Version Naming Inconsistency (V2 vs V3)

**Finding**: README.md says "V2" in multiple places, but the language reference says "V3", and the code already supports V3 features (curly brackets, `ref` keyword, `uses` declarations). The V3 features are fully implemented and tested.

**Changes**:
- Update README.md version references from V2 to V3
- Ensure consistent V3 naming throughout documentation

### 3. Deprecated `connection_point` keyword

**Finding**: The lexer still has `ConnectionPoint` token kind, and the AST has a `ConnectionPointDeclaration` type alias pointing to `RefDeclaration`. The `ref` keyword is the V3 replacement but old references remain.

**Status**: Kept for backward compatibility. The lexer accepting both is intentional. No changes needed for consistency — just documentation clarity.

### 4. `hielements.hie` Self-Description Gaps

**Finding**: 
- `hielements.hie` header mentions spec files in `specs/` but never imports them
- No description of the MCP crate (despite being a workspace member)  
- After MCP removal from workspace, needs updated to reflect CLI-only focus

**Changes**:
- Update `hielements.hie` to remove references to MCP
- Clean up comments about spec file organization

### 5. Scope Binding Enforcement (Not Yet Implemented)

**Finding**: The parser correctly parses `binds` syntax and template `implements` clauses, but the interpreter does NOT enforce:
- Whether implementing elements actually bind all required template scopes
- Whether `binds` paths resolve to valid template properties
- Whether unbounded scopes in templates get properly bound

**Status**: This is the next feature to implement. Current analysis documents the gap.

### 6. `uses` Declarations Not Enforced

**Finding**: `uses` declarations are parsed into the AST but the interpreter has only a comment: "Uses declarations are validated by their target path / Full validation would require resolving the target element/scope."

**Status**: Related to scope binding enforcement. Will be addressed as part of that feature.

### 7. Template Implementation Validation is Stub

**Finding**: In `validate_element()`, template implementations are acknowledged but not validated:
```rust
// Could add validation that template exists, but that requires
// building a template registry first
let _ = template_impl; // Acknowledge the field
```

**Status**: Prerequisite for scope binding enforcement. Will be addressed as part of that feature.

## Changes Made in This PR

| Area | Change | Rationale |
|------|--------|-----------|
| `Cargo.toml` | Remove `hielements-mcp` from workspace members | CLI-focused, MCP is optional/separate |
| `hielements.hie` | Remove MCP references, update structure | Accurately describe CLI-focused project |
| `README.md` | Fix V2→V3 references, de-emphasize MCP | Consistency with actual implementation |
| `USAGE.md` | Update MCP section, clarify CLI focus | Consistency with project direction |

## Scope Binding Enforcement Preparation

The following components need to be built for scope binding enforcement:

1. **Template Registry**: Build a registry of templates during interpretation that maps template names to their structure (scopes, refs, checks, elements).

2. **Binding Validation**: When an element `implements` a template, validate that:
   - The template exists in the registry
   - All required (unbounded) scopes are bound via `binds` or direct binding
   - All required (unbounded) refs are bound
   - Binding paths resolve to valid template properties

3. **Uses Declaration Resolution**: Validate that `uses` targets resolve to:
   - Known elements (by name in scope)
   - Known scopes (by qualified path)

4. **Scope Visibility Enforcement**: Enforce the scope visibility rules documented in `doc/scope_management.md`:
   - Elements can access own scopes and parent/child scopes via qualified paths
   - Cross-sibling access requires qualified path notation
