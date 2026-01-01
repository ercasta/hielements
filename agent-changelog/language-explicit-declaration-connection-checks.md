# Language Explicit Declaration and Connection Checks

**Issue:** Implement explicit language declarations and connection verification checks  
**Date:** 2026-01-01

## Summary

This document describes the implementation of:
1. Explicit `language` keyword for declaring supported languages
2. Language annotations for scopes (e.g., `scope x : python = ...`)
3. Language constraints in templates (`requires`, `allows`, `forbids` for language)
4. `connection_check` keyword for defining language-specific connection verification

## Problem Statement

Currently, element connections are assumed to be verifiable across the entire system without explicit language awareness. This creates several issues:

1. **No explicit language association**: Scopes don't explicitly declare which programming language they target
2. **No language-specific verification**: Cannot define connection verification that is specific to a language (e.g., Python imports vs Rust dependencies)
3. **No language constraints**: Templates cannot specify which languages elements must or must not use

## Design

### 1. Language Declaration

A new `language` keyword allows declaring supported languages at the top level or within elements:

```hielements
language python
language rust

element my_service:
    # This element uses both languages
    language python
    language rust
```

### 2. Language-Typed Scopes

Scopes must explicitly declare their language:

```hielements
element my_service:
    # Scope with explicit language type
    scope src : python = python.module_selector('my_service')
    scope crate : rust = rust.crate_selector('my_service')
```

The syntax is: `scope <name> : <language> = <expression>`

### 3. Language Constraints in Templates

Templates can use `requires`, `allows`, `forbids` to constrain which languages elements can use:

```hielements
template python_only:
    requires language python
    forbids language rust

template multilingual:
    allows language python
    allows language rust
```

### 4. Connection Checks

The `connection_check` keyword defines language-specific verification functions:

```hielements
language python:
    connection_check import_exists(source: scope[], target: scope[]):
        # Returns True if source can import from target
        return python.can_import(source, target)
    
    connection_check no_circular_imports(scopes: scope[]):
        return python.no_circular_imports(scopes)
```

Connection checks:
- Accept `scope[]` arguments representing unions of scopes
- Return True (connection valid) or False (connection invalid)
- Are automatically applied recursively along the parent-children hierarchy

### 5. Automatic Recursive Verification

When verifying element connections:
1. For each language used by an element, gather all scopes of that language
2. For each child element, gather its scopes of the same language
3. Apply all `connection_check` functions defined for that language
4. Recursively verify all descendants

## AST Changes

### New Types

```rust
/// A language declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDeclaration {
    /// Language name
    pub name: Identifier,
    /// Connection check definitions for this language
    pub connection_checks: Vec<ConnectionCheckDeclaration>,
    /// Source span
    pub span: Span,
}

/// A connection check declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionCheckDeclaration {
    /// Check name
    pub name: Identifier,
    /// Parameters (all are scope[])
    pub parameters: Vec<ConnectionCheckParameter>,
    /// Expression body
    pub body: Expression,
    /// Source span
    pub span: Span,
}

/// A connection check parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionCheckParameter {
    /// Parameter name
    pub name: Identifier,
    /// Source span
    pub span: Span,
}
```

### Updated ScopeDeclaration

```rust
pub struct ScopeDeclaration {
    /// Scope name
    pub name: Identifier,
    /// Optional language type annotation (e.g., `scope x : python = ...`)
    pub language: Option<Identifier>,
    /// Scope expression (selector)
    pub expression: Expression,
    /// Source span
    pub span: Span,
}
```

### Updated ComponentSpec

```rust
pub enum ComponentSpec {
    // ... existing variants ...
    /// Language requirement: `language <name>`
    Language(Identifier),
}
```

## Grammar Additions

```ebnf
(* Language declaration at top level *)
language_declaration ::= 'language' identifier ':' NEWLINE INDENT connection_check* DEDENT
                       | 'language' identifier NEWLINE

(* Simple language reference *)
language_reference ::= 'language' identifier

(* Updated scope declaration with language annotation *)
scope_declaration ::= 'scope' identifier ':' identifier '=' expression NEWLINE

(* Connection check declaration *)
connection_check ::= 'connection_check' identifier '(' parameter_list ')' ':' NEWLINE INDENT expression DEDENT

parameter_list ::= parameter (',' parameter)*
parameter ::= identifier ':' 'scope' '[' ']'

(* Language constraint in component requirements *)
component_spec ::= ... | language_reference
```

## Implementation Plan

### Phase 1: Lexer Updates
- Add `language` keyword
- Add `connection_check` keyword

### Phase 2: AST Updates  
- Add `LanguageDeclaration` struct
- Add `ConnectionCheckDeclaration` struct
- Update `ScopeDeclaration` to include language
- Update `ComponentSpec` to include `Language` variant

### Phase 3: Parser Updates
- Parse `language` declarations at top level
- Parse language-typed scopes
- Parse `connection_check` definitions
- Parse language constraints in templates

### Phase 4: Interpreter Updates
- Track language associations for scopes
- Validate language constraints
- Execute connection checks recursively

### Phase 5: Documentation
- Update language reference
- Update hielements.hie
- Add examples

## Changes to hielements.hie

```hielements
element core:
    element lexer:
        scope module = rust.module_selector('lexer')
        # Add check for new language keyword
        check rust.enum_variant_exists('TokenKind', 'Language')
        check rust.enum_variant_exists('TokenKind', 'ConnectionCheck')
    
    element ast:
        scope module = rust.module_selector('ast')
        # Add checks for new types
        check rust.struct_exists('LanguageDeclaration')
        check rust.struct_exists('ConnectionCheckDeclaration')
        check rust.struct_exists('ConnectionCheckParameter')
```

## Examples

### Example 1: Python-only Service

```hielements
language python

template python_service:
    requires language python
    forbids language rust

element my_api implements python_service:
    scope src : python = python.module_selector('my_api')
    check python.has_docstrings(src)
```

### Example 2: Multi-language System with Connection Checks

```hielements
language python:
    connection_check can_import(source: scope[], target: scope[]):
        return python.imports_allowed(source, target)

language rust:
    connection_check depends_on(source: scope[], target: scope[]):
        return rust.dependency_exists(source, target)

element system:
    element python_frontend:
        scope src : python = python.module_selector('frontend')
    
    element rust_backend:
        scope src : rust = rust.module_selector('backend')
```

## Testing Strategy

1. **Lexer Tests**: Verify new keywords tokenize correctly
2. **Parser Tests**: 
   - Parse language declarations with and without body
   - Parse language-typed scopes
   - Parse connection checks
   - Parse language constraints in templates
3. **Interpreter Tests**:
   - Validate language constraints are enforced
   - Execute connection checks
   - Verify recursive verification

## Future Extensions

1. **Language inheritance**: Templates can inherit language constraints
2. **Language-specific stdlib**: Different libraries per language
3. **Cross-language connection checks**: Verify connections across language boundaries
