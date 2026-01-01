# Named Component Syntax Enhancement

**Issue:** Modify syntax to allow naming architectural components and specifying their type  
**Date:** 2026-01-01

## Summary

This document describes the changes to unify and generalize the syntax for `requires_descendant`, `allows_connection`, `forbids_connection`, and `requires_connection` keywords. The new syntax:

1. Separates keywords (`requires`, `allows`, `forbids`) from modifiers (`descendant`)
2. Allows naming architectural components and specifying their types
3. Confines these features to element templates only (actual elements can only refer to templates)

## Current Syntax

```hielements
# Hierarchical requirements
requires_descendant scope dockerfile = docker.file_selector('Dockerfile')
requires_descendant check docker.has_healthcheck(dockerfile)
requires_descendant element metrics:
    scope module = rust.module_selector('metrics')
requires_descendant implements dockerized

# Connection boundaries
allows_connection to api_gateway.public_api
forbids_connection to external.*
requires_connection to logging.*
```

## New Syntax

```hielements
# Hierarchical requirements (with "descendant" modifier)
requires descendant scope dockerfile = docker.file_selector('Dockerfile')
requires descendant check docker.has_healthcheck(dockerfile)
requires descendant element dock implements dockerized
forbids descendant connection_point prometheus: MetricsHandler

# Immediate children requirements (without "descendant" modifier)
requires element dock implements dockerized
allows element logger

# Connection boundaries (with "connection" keyword)
allows connection to api_gateway.public_api
forbids connection to external.*
requires connection to logging.*
```

## Grammar Changes

```ebnf
(* Updated hierarchical requirements and connection boundaries *)
component_requirement ::= ('requires' | 'allows' | 'forbids') ['descendant'] component_spec

component_spec       ::= scope_spec
                       | check_spec
                       | element_spec
                       | connection_spec

scope_spec           ::= 'scope' identifier '=' expression NEWLINE
check_spec           ::= 'check' expression NEWLINE  
element_spec         ::= 'element' identifier [':' type_name] ['implements' identifier] [':' NEWLINE INDENT element_body DEDENT]
connection_spec      ::= 'connection' ['to'] connection_pattern NEWLINE
connection_point_spec ::= 'connection_point' identifier ':' type_name ['=' expression] NEWLINE
```

## AST Changes

```rust
/// Unified component requirement with optional descendant modifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRequirement {
    /// Whether this is requires, allows, or forbids
    pub action: RequirementAction,
    /// Whether this applies to descendants (true) or immediate children (false)
    pub is_descendant: bool,
    /// The component specification
    pub component: ComponentSpec,
    /// Source span
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequirementAction {
    Requires,
    Allows,
    Forbids,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentSpec {
    /// Scope requirement: scope name = expr
    Scope(ScopeDeclaration),
    /// Check requirement: check expr
    Check(CheckDeclaration),
    /// Element requirement with optional name, type, and implements
    Element {
        name: Identifier,
        type_annotation: Option<TypeAnnotation>,
        implements: Option<Identifier>,
        body: Option<Box<Element>>,
    },
    /// Connection requirement: connection to pattern
    Connection(ConnectionPattern),
    /// Connection point requirement: connection_point name: type = expr
    ConnectionPoint {
        name: Identifier,
        type_annotation: TypeAnnotation,
        expression: Option<Expression>,
    },
}
```

## Lexer Changes

New tokens:
- `Requires` - for the `requires` keyword
- `Allows` - for the `allows` keyword
- `Forbids` - for the `forbids` keyword
- `Descendant` - for the `descendant` modifier
- `Connection` - for the `connection` keyword

Removed tokens:
- `RequiresDescendant` - replaced by `Requires` + `Descendant`
- `AllowsConnection` - replaced by `Allows` + `Connection`
- `ForbidsConnection` - replaced by `Forbids` + `Connection`
- `RequiresConnection` - replaced by `Requires` + `Connection`

## Migration Path

The old syntax (`requires_descendant`, `allows_connection`, etc.) is deprecated but remains supported for backwards compatibility. New code should use the new syntax.

## Template-Only Restriction

These features are confined to element templates. Regular elements can only:
1. Implement templates
2. Use standard scopes, connection points, and checks

This restriction simplifies the language and ensures architectural requirements are defined in reusable templates.

## Examples

### Template with Hierarchical Requirements

```hielements
template dockerized:
    ## At least one descendant must have docker configuration
    requires descendant scope dockerfile = docker.file_selector('Dockerfile')
    requires descendant check docker.has_healthcheck(dockerfile)

template observable:
    ## Requires a named metrics element
    requires descendant element metrics_service implements metrics_provider

template secure_zone:
    ## Forbids any connection to external services
    forbids connection to external.*
```

### Element Implementing Template

```hielements
element my_app implements dockerized, observable:
    scope root = files.folder_selector('.')
    
    element backend:
        scope dockerfile = docker.file_selector('Dockerfile')
        check docker.has_healthcheck(dockerfile)
    
    element metrics_service implements metrics_provider:
        scope module = rust.module_selector('metrics')
```

## Changes to hielements.hie

Update AST section to include new structs:

```hielements
check rust.struct_exists('ComponentRequirement')
check rust.enum_exists('RequirementAction')
check rust.enum_exists('ComponentSpec')
```

## Testing Strategy

1. Parser tests for new syntax
2. Backwards compatibility tests for old syntax
3. Template-only restriction tests
4. Integration tests with example files
