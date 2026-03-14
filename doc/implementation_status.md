# Hielements Implementation Status

This document tracks the implementation status of each Hielements language feature. It serves as a compliance checklist for language implementation and a reference for users wondering which features are enforced at runtime.

---

## Feature Implementation Status

### Core Language Features

| Feature | Status | Notes |
|---------|--------|-------|
| `element` declarations | ✅ Implemented | Full parsing and evaluation |
| Nested elements (hierarchy) | ✅ Implemented | Arbitrary nesting depth supported |
| `scope` declarations (bound) | ✅ Implemented | With selector expressions |
| `scope` declarations (unbounded, in patterns) | ✅ Implemented | Without `=` expression, placeholder for `binds` |
| Language annotations `<lang>` | ✅ Implemented | Stored in AST, used in connection checks |
| `ref` declarations | ✅ Implemented | V3 preferred keyword (alias for `connection_point`) |
| `connection_point` declarations | ✅ Implemented | V2/backward compat, parsed as `ref` |
| `check` declarations | ✅ Implemented | Evaluated at runtime via library calls |
| `uses` declarations (syntax) | ✅ Parsed | AST node stored, dependency not validated |
| `import` statements | ✅ Implemented | Loads built-in and external libraries |
| Doc comments (`##`) | ✅ Implemented | Stored in AST for tooling |

### Pattern / Template Features

| Feature | Status | Notes |
|---------|--------|-------|
| `pattern` keyword | ✅ Implemented | V3 preferred keyword |
| `template` keyword | ✅ Implemented | V2/backward compat, equivalent to `pattern` |
| `implements` keyword (syntax) | ✅ Parsed | Stored in AST; binding completeness **not enforced** |
| `binds` keyword (syntax) | ✅ Parsed | Stored in AST; path resolution **not enforced** |
| Pattern-level `requires` | ✅ Implemented | Hierarchical requirements evaluated |
| Pattern-level `allows` | ✅ Implemented | Language/connection allow rules evaluated |
| Pattern-level `forbids` | ✅ Implemented | Language/connection forbid rules evaluated |
| `descendant` modifier | ✅ Implemented | Searches entire descendant tree |
| `requires descendant element` | ✅ Implemented | |
| `requires descendant scope` | ✅ Implemented | |
| `requires descendant check` | ✅ Implemented | |
| `requires descendant implements` | ✅ Implemented | |
| Connection boundaries (`allows/forbids/requires connection to`) | ✅ Implemented | Pattern-only constraints |
| Language constraints (`requires/allows/forbids language`) | ✅ Implemented | |

### Syntax Variants

| Feature | Status | Notes |
|---------|--------|-------|
| Indentation-based blocks | ✅ Implemented | V2/V3 compatible |
| Curly bracket blocks `{}` | ✅ Implemented | V3 feature |
| Both syntaxes mixed | ✅ Implemented | Can be used in the same file |

### Built-in Libraries

| Library | Status | Notes |
|---------|--------|-------|
| `files` library | ✅ Implemented | File/folder selectors and checks |
| `rust` library | ✅ Implemented | Rust code analysis (structs, enums, traits, etc.) |
| `python` library | ✅ Implemented | Python module analysis |
| External library plugins (JSON-RPC) | ✅ Implemented | See `doc/external_libraries.md` |
| WASM library plugins | ✅ Implemented | See `doc/wasm_plugins.md` |

### Language Declarations

| Feature | Status | Notes |
|---------|--------|-------|
| `language` declaration | ✅ Implemented | Registers a language name |
| `connection_check` definition | ✅ Implemented | Defines connection verification functions |
| Connection check evaluation | ✅ Implemented | Applied hierarchically to matching scopes |

---

## Known Limitations

### 1. Scope Binding Enforcement (`implements` + `binds`)

**Status**: Not yet implemented

When an element `implements` a pattern, the interpreter does not currently:
- Validate that all unbounded scopes in the pattern are provided via `binds` clauses
- Verify that `binds` paths resolve to valid pattern properties
- Enforce pattern structural requirements on the implementing element

**Impact**: Missing or incorrect `binds` declarations will not produce errors.

**Example of what is NOT checked**:
```hielements
pattern microservice {
    element api {
        scope module<rust>      ## unbounded scope - must be bound
    }
}

element orders implements microservice {
    ## Missing: scope ... binds microservice.api.module = ...
    ## This should be an error but currently passes silently
}
```

**Planned fix**: Build a template registry during interpretation and validate binding completeness.

---

### 2. `uses` Declaration Validation

**Status**: Not yet implemented

`uses` declarations are parsed into the AST but the interpreter does not:
- Validate that the source identifier refers to a known scope in the current element
- Validate that the target identifier resolves to a known element or scope
- Enforce any dependency rules based on `uses` declarations

**Impact**: Invalid `uses` declarations will not produce errors.

**Example**:
```hielements
element parser {
    scope module = rust.module_selector('parser')
    module uses lexer    ## 'lexer' is not checked to exist
}
```

**Planned fix**: Implement scope/element resolution for `uses` targets as part of binding enforcement.

---

### 3. Type Checking for `ref` Declarations

**Status**: Not yet implemented

Type annotations on `ref` declarations are parsed and stored but not enforced:
- Type compatibility is not checked when binding refs
- No type validation occurs between pattern ref types and implementing element ref types

**Impact**: Type mismatches in ref declarations will not produce errors.

**Planned fix**: Implement type annotation validation as part of the binding enforcement feature.

---

### 4. Template/Pattern Registry

**Status**: Not yet implemented

There is no runtime registry of templates/patterns that would allow:
- Verifying that `implements X` refers to a known pattern
- Checking that `binds pattern.element.scope` paths are valid
- Cross-element validation of binding completeness

**Planned fix**: Build template registry during the interpretation phase, before executing checks.

---

## Implementation Roadmap

The following features are planned in priority order:

1. **Template Registry** (prerequisite for items 2-4):
   - Build registry during interpretation
   - Map pattern names → structure (scopes, refs, checks, elements)

2. **Binding Completeness Validation**:
   - When element `implements` pattern, verify all unbounded scopes are bound
   - Verify all unbounded refs are bound
   - Report missing bindings as errors

3. **`binds` Path Resolution**:
   - Validate that `binds pattern.element.scope` paths exist in the pattern
   - Report invalid binding paths as errors

4. **`uses` Declaration Enforcement**:
   - Validate source is a known scope in current element
   - Validate target resolves to a known element or scope

5. **Type Safety for `ref`**:
   - Validate type annotations when binding refs across patterns/elements

---

## Running Implementation Checks

To verify the current implementation status against the codebase:

```bash
# Run the self-description checks (verifies all 121 structural checks)
hielements run hielements.hie --workspace .

# Check a specific .hie file for syntax errors
hielements check your_spec.hie
```

See `hielements.hie` for the canonical self-description that serves as the live compliance test.
