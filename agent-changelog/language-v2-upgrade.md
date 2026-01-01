# Language V2 Upgrade

## Summary
Upgrade the Hielements project from v1 to v2 of the language specification.

## Key Changes

### 1. Scope Syntax Changes
**Before (v1):**
```hielements
scope src : python = python.module_selector('module')
```

**After (v2):**
```hielements
# In templates - unbounded scope
scope module<rust>

# In elements - bound scope with binds keyword
scope main_module<rust> binds observable.metrics.module = rust.module_selector('payments::api')
```

### 2. Template Structure Changes
- Templates now declare unbounded scopes (no `=` expression)
- Scopes in templates are placeholders to be bound by implementing elements

### 3. Element Binding Changes
- Elements use `binds` keyword to connect scopes to template declarations
- Connection points also use `binds` syntax

### 4. Language Specification
- Angular brackets `<>` used instead of colon syntax
- `scope module<rust>` vs `scope module : rust`

## Files to Change

### Documentation
- `doc/language_reference.md` - Add migration guide, update syntax documentation

### Lexer
- `crates/hielements-core/src/lexer.rs`
  - Add `binds` keyword token
  - Add `<` and `>` bracket tokens

### AST
- `crates/hielements-core/src/ast.rs`
  - Update `ScopeDeclaration` to support unbounded scopes
  - Add binding path for `binds` keyword

### Parser
- `crates/hielements-core/src/parser.rs`
  - Parse new scope syntax with angular brackets
  - Parse optional `binds` clause

### Examples
- Update example files to use v2 syntax

### Self-Description
- `hielements.hie` - Update to use v2 syntax
