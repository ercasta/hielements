# Language V2 Upgrade

## Summary
Upgrade the Hielements project from v1 to v2 of the language specification.

## Implementation Completed

### 1. Documentation Updates
- Updated `doc/language_reference.md` with V2 syntax
- Added migration guide (Appendix D) explaining V1 to V2 migration
- Updated grammar section with V2 syntax rules
- Updated all examples in documentation to use V2 syntax

### 2. Lexer Changes (`crates/hielements-core/src/lexer.rs`)
- Added `Binds` keyword token
- Added `LAngle` (`<`) and `RAngle` (`>`) tokens for language annotation
- Added comprehensive tests for new tokens

### 3. AST Changes (`crates/hielements-core/src/ast.rs`)
- Updated `ScopeDeclaration`:
  - Changed `expression` to `Option<Expression>` (supports unbounded scopes)
  - Added `binds: Option<Vec<Identifier>>` for binding path
- Updated `ConnectionPointDeclaration`:
  - Changed `expression` to `Option<Expression>` (supports unbounded connection points)
  - Added `binds: Option<Vec<Identifier>>` for binding path

### 4. Parser Changes (`crates/hielements-core/src/parser.rs`)
- Updated `parse_scope()` to support:
  - Angular brackets for language: `scope module<rust>`
  - Optional `binds` clause: `binds template.element.scope`
  - Optional expression for unbounded scopes
- Updated `parse_connection_point()` to support:
  - Optional `binds` clause
  - Optional expression for unbounded connection points
- Added `parse_qualified_path()` helper for binds paths
- Added `Binds` keyword to identifier context list
- Added 5 new V2 syntax tests

### 5. Interpreter Changes (`crates/hielements-core/src/interpreter.rs`)
- Updated `validate_template()` to handle `Option<Expression>`
- Updated `validate_element()` to handle `Option<Expression>`
- Updated `validate_component_requirement()` to handle `Option<Expression>`
- Updated scope evaluation to handle unbounded scopes

### 6. Example Updates
- Updated `examples/language_example.hie` to V2 syntax
- Created new `examples/v2_syntax_example.hie` demonstrating:
  - Unbounded scopes in templates
  - `binds` keyword for scope binding
  - Angular brackets for language annotation
  - Descriptive-only mode (without templates/binds)

## Key V2 Syntax Changes

### Language Annotation
**Before (V1):**
```hielements
scope src : python = python.module_selector('module')
```

**After (V2):**
```hielements
scope src<python> = python.module_selector('module')
```

### Unbounded Scopes in Templates
**V2 (Templates can have unbounded scopes):**
```hielements
template observable:
    element metrics:
        scope module<rust>  # No '=' expression
        connection_point prometheus: MetricsHandler
```

### Element Bindings with `binds`
**V2 (Elements bind to template scopes):**
```hielements
element my_service implements observable:
    scope metrics_mod<rust> binds observable.metrics.module = rust.module_selector('api')
    connection_point handler: MetricsHandler binds observable.metrics.prometheus = rust.function_selector(metrics_mod, 'handler')
```

## Test Results
- All 66 tests pass
- Self-check (97 checks) passes
- V2 example files validate successfully

## Backward Compatibility
- Colon syntax (`: lang`) still works for backward compatibility
- `implements` and `binds` are optional (descriptive-only mode)
- Expression is optional (unbounded scopes)

