# Top-Down Transitivity Feature Design

**Issue:** Evolve element templates for top-down transitivity  
**Date:** 2026-01-01

## Summary

This document describes the design for implementing "top-down transitivity" in Hielements element templates. This feature allows parent elements to prescribe requirements that must be satisfied by one or more of their descendants (children, grandchildren, etc.). Additionally, it introduces connection point boundaries to control which elements descendants can connect to.

## Problem Statement

Currently, Hielements templates define structure that elements must implement with concrete bindings. However, there's no way to express:

1. **Transitive requirements**: A parent element should be able to declare that somewhere in its descendant hierarchy, a specific property must exist. For example:
   - An element is "dockerized" if one of its descendants has a docker scope or check
   - A system is "observable" if one of its descendants exposes metrics

2. **Connection boundaries**: At a top level, control which elements descendants can or cannot connect to. For example:
   - Descendants of "frontend" module can only connect to "api" module
   - Descendants of "secure_zone" cannot connect to "public_zone"

## Requirements

1. **Descendant requirements** (`requires_descendant`): Specify that at least one descendant must satisfy a condition
2. **Connection boundaries** (`allows_connection`, `forbids_connection`): Specify connection constraints that apply to all descendants
3. **Transitive validation**: The interpreter must traverse the element hierarchy to validate these requirements
4. **Template integration**: These features should work with existing element templates

## Design

### 1. New Syntax

#### Transitive Requirements

```hielements
template dockerized:
    ## At least one descendant must have a docker scope
    requires_descendant scope docker = docker.file_selector(*)
    
    ## Or at least one descendant must satisfy this check
    requires_descendant check docker.has_dockerfile()

template observable:
    ## Requires a descendant with prometheus metrics
    requires_descendant element metrics:
        connection_point prometheus_endpoint: MetricsHandler
```

#### Connection Boundaries

```hielements
template isolated_frontend:
    ## Descendants can only connect to api_gateway connection points
    allows_connection to api_gateway.*
    
    ## Descendants cannot connect to database directly
    forbids_connection to database.*

element security_zone:
    ## No descendant can connect to external_network
    forbids_connection to external_network.*
    
    element internal_service:
        # This element inherits the connection constraint
        connection_point api: HttpHandler = service.get_handler()
```

### 2. AST Changes

Add new types to `ast.rs`:

```rust
/// A transitive requirement that must be satisfied by at least one descendant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitiveRequirement {
    /// Kind of requirement
    pub kind: TransitiveRequirementKind,
    /// Source span
    pub span: Span,
}

/// Types of transitive requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitiveRequirementKind {
    /// Requires a descendant with a matching scope
    Scope(ScopeDeclaration),
    /// Requires a descendant with a matching check
    Check(CheckDeclaration),
    /// Requires a descendant element with specific structure
    Element(Element),
}

/// A connection boundary constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionBoundary {
    /// Whether this allows or forbids the connection
    pub kind: ConnectionBoundaryKind,
    /// Target pattern (e.g., "api_gateway.*", "database.connection")
    pub target_pattern: String,
    /// Source span
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionBoundaryKind {
    /// Allows connections only to matching targets
    Allows,
    /// Forbids connections to matching targets
    Forbids,
}
```

### 3. Grammar Additions

```ebnf
(* Transitive requirements *)
transitive_requirement ::= 'requires_descendant' (scope_declaration | check_declaration | element_declaration)

(* Connection boundaries *)
connection_boundary ::= ('allows_connection' | 'forbids_connection') 'to' connection_pattern
connection_pattern  ::= qualified_identifier ('.' '*')?

(* Updated element/template body *)
element_item        ::= scope_declaration
                      | connection_point_declaration
                      | check_declaration
                      | element_declaration
                      | template_binding
                      | transitive_requirement      (* NEW *)
                      | connection_boundary         (* NEW *)
```

### 4. Interpreter Changes

#### Transitive Requirement Validation

The interpreter needs to traverse the element hierarchy to validate transitive requirements:

```rust
fn validate_transitive_requirements(&self, element: &Element) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    for req in &element.transitive_requirements {
        let satisfied = self.find_matching_descendant(element, &req.kind);
        if !satisfied {
            diagnostics.push(Diagnostic::error(
                "E300",
                format!("Transitive requirement not satisfied: no descendant matches {:?}", req.kind)
            ));
        }
    }
    
    diagnostics
}

fn find_matching_descendant(&self, element: &Element, req: &TransitiveRequirementKind) -> bool {
    // Check direct children first
    for child in &element.children {
        if self.matches_requirement(child, req) {
            return true;
        }
        // Recursively check grandchildren
        if self.find_matching_descendant(child, req) {
            return true;
        }
    }
    false
}
```

#### Connection Boundary Enforcement

Connection boundaries are inherited by descendants and checked during validation:

```rust
fn validate_connection_boundaries(&self, element: &Element, inherited: &[ConnectionBoundary]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // Combine parent boundaries with element's own boundaries
    let mut all_boundaries = inherited.to_vec();
    all_boundaries.extend(element.connection_boundaries.clone());
    
    // Check this element's connection points against boundaries
    for cp in &element.connection_points {
        for boundary in &all_boundaries {
            if let Err(e) = self.check_connection_boundary(&cp, boundary) {
                diagnostics.push(e);
            }
        }
    }
    
    // Validate children with accumulated boundaries
    for child in &element.children {
        diagnostics.extend(self.validate_connection_boundaries(child, &all_boundaries));
    }
    
    diagnostics
}
```

### 5. Use Cases

#### Example 1: Dockerized Application

```hielements
template dockerized:
    ## This application must have at least one dockerized component
    requires_descendant scope dockerfile = docker.file_selector(*)
    requires_descendant check docker.has_healthcheck()

element my_app implements dockerized:
    element frontend:
        scope src = files.folder_selector('frontend')
        # Not dockerized
    
    element backend:
        scope src = files.folder_selector('backend')
        scope dockerfile = docker.file_selector('Dockerfile.backend')  # Satisfies requirement!
        check docker.has_healthcheck(dockerfile)  # Satisfies requirement!
```

#### Example 2: Microservices with Connection Boundaries

```hielements
element e_commerce_platform:
    ## Frontend services can only connect to API gateway
    element frontend_zone:
        allows_connection to api_gateway.public_api
        forbids_connection to database.*
        
        element web_app:
            scope src = files.folder_selector('frontend/web')
            # Any connection_points here are checked against boundaries
    
    ## Backend services can access database
    element backend_zone:
        allows_connection to database.*
        
        element order_service:
            scope src = files.folder_selector('backend/orders')
            connection_point db: DatabaseConnection = postgres.connection()  # Allowed!
```

#### Example 3: Security Isolation

```hielements
template secure_processing:
    ## No descendant can connect to external network
    forbids_connection to external.*
    forbids_connection to public_internet.*
    
    ## Must have audit logging somewhere
    requires_descendant element audit:
        check security.has_audit_logging()

element payment_processor implements secure_processing:
    element card_validation:
        scope src = files.folder_selector('src/card')
    
    element audit:  # Satisfies the requires_descendant requirement
        scope src = files.folder_selector('src/audit')
        check security.has_audit_logging()  # Satisfies nested check
```

### 6. Validation Rules

1. **Transitive Requirements**:
   - At least one descendant must satisfy each `requires_descendant`
   - The match can be a direct child or any level of nesting
   - Multiple descendants can satisfy the same requirement

2. **Connection Boundaries**:
   - `allows_connection` creates a whitelist - only listed targets are permitted
   - `forbids_connection` creates a blacklist - listed targets are prohibited
   - Multiple boundaries are combined (allows AND forbids)
   - Boundaries are inherited by all descendants
   - Connection patterns support wildcards (`*`)

3. **Template Integration**:
   - Templates can define transitive requirements
   - Elements implementing templates inherit those requirements
   - Concrete element hierarchies must satisfy all requirements

### 7. Implementation Plan

#### Phase 1: Lexer and Parser
- Add `requires_descendant`, `allows_connection`, `forbids_connection` keywords
- Update parser to handle new syntax
- Update AST with new types

#### Phase 2: Interpreter
- Implement transitive requirement validation
- Implement connection boundary validation
- Add tests for validation logic

#### Phase 3: Integration
- Update hielements.hie self-description
- Create example files
- Update documentation

### 8. Changes to hielements.hie

```hielements
element core:
    # ... existing elements ...
    
    ## Transitive requirement support
    element transitivity:
        scope ast_transitivity = rust.module_selector('ast')
        
        check rust.struct_exists('TransitiveRequirement')
        check rust.enum_exists('TransitiveRequirementKind')
        check rust.struct_exists('ConnectionBoundary')
        check rust.enum_exists('ConnectionBoundaryKind')
        check rust.function_exists('validate_transitive_requirements')
        check rust.function_exists('validate_connection_boundaries')
```

### 9. Testing Strategy

1. **Parser Tests**:
   - Parse `requires_descendant` with scope/check/element
   - Parse `allows_connection` and `forbids_connection`
   - Error cases (malformed syntax)

2. **Validation Tests**:
   - Transitive requirement satisfied at child level
   - Transitive requirement satisfied at grandchild level
   - Transitive requirement not satisfied (error)
   - Connection boundary allows (pass)
   - Connection boundary forbids (fail)
   - Inherited boundary enforcement

3. **Integration Tests**:
   - Full examples with templates and implementations
   - Complex hierarchies with multiple requirements

### 10. Future Extensions

1. **Conditional requirements**: `requires_descendant if condition`
2. **Cardinality**: `requires_descendant exactly(2) element ...`
3. **Named boundaries**: `boundary_group secure: forbids_connection to ...`
4. **Boundary exceptions**: `allows_exception for specific_element`

## Conclusion

The top-down transitivity feature extends Hielements' declarative architecture description capabilities by allowing parent elements to express requirements that must be satisfied somewhere in their descendant hierarchy, and to control how descendants can connect to other parts of the system. This enables more expressive architectural constraints while maintaining the hierarchical nature of the language.
