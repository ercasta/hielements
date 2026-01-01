# Hierarchical Checks Feature Design

**Issue:** Evolve element templates for hierarchical checks  
**Date:** 2026-01-01  
**Updated:** 2026-01-01 (Revised based on clarifications about architectural connections)

## Summary

This document describes the design for implementing "hierarchical checks" in Hielements element templates. This feature allows parent elements to prescribe requirements that must be satisfied by one or more of their descendants (children, grandchildren, etc.). Additionally, it introduces **architectural connection boundaries** to control which elements descendants can have logical dependencies on.

## Key Clarification: Architectural Connections

**Connections in this context refer to logical/architectural dependencies between elements**, such as:
- A Python module importing another module
- A Rust crate depending on another crate
- A service calling another service's API

These are NOT network connections or URLs. The actual verification that code respects these architectural boundaries is the responsibility of language-specific libraries (e.g., `python.no_imports_from()`).

## Problem Statement

Currently, Hielements templates define structure that elements must implement with concrete bindings. However, there's no way to express:

1. **Hierarchical requirements**: A parent element should be able to declare that somewhere in its descendant hierarchy, a specific property must exist. For example:
   - An element is "dockerized" if one of its descendants has a docker scope or check
   - A system is "observable" if one of its descendants exposes metrics

2. **Architectural connection boundaries**: At a top level, control which elements descendants can or cannot have dependencies on. For example:
   - Code in "frontend" module cannot import from "database" module
   - Services in "secure_zone" must not depend on "external" services
   - A service in zone A is required to depend on services in zone B (requires_connection)

## Requirements

1. **Descendant requirements** (`requires_descendant`): Specify that at least one descendant must satisfy a condition
2. **Connection boundaries** (`allows_connection`, `forbids_connection`, `requires_connection`): Specify architectural dependency constraints
3. **Hierarchical validation**: The interpreter must traverse the element hierarchy to validate these requirements
4. **Template integration**: These features should work with existing element templates
5. **Language-agnostic**: Connection semantics are opaque to hielements; actual checking is library-specific
6. **Scope aggregation**: Scopes should expose information that can be aggregated bottom-up for parent elements

## Design Principles

### Hierarchical Dependency Composition

When boundaries apply recursively within the parent/child hierarchy:
- If module A is only allowed to connect to B
- And B is only allowed to connect to C
- Then A→B→C is **allowed** (each hop respects its own boundary)

This allows construction of complex systems where boundaries are applied at each level.

### Language-Specific Verification

The hielements language provides:
1. **Syntax** for declaring connection boundaries
2. **Semantic information** about which scopes belong to which elements
3. **Aggregated scope information** passed to check rules

Libraries are responsible for:
1. **Resolving** what modules/files belong to a scope
2. **Checking** actual imports/dependencies between scopes
3. **Interpreting** wildcards according to language conventions

## Design

### 1. New Syntax

#### Hierarchical Requirements

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

**Architectural connection boundaries** express constraints on import/dependency relationships:

```hielements
template isolated_frontend:
    ## Code in this zone may only import from api_gateway module
    allows_connection to api_gateway.*
    
    ## Code in this zone must NOT import from database modules
    forbids_connection to database.*

element service_integration:
    ## Code in this element MUST import from logging module
    requires_connection to logging.*
    
element security_zone:
    ## No descendant can import from external modules
    forbids_connection to external.*
    
    element internal_service:
        scope src = python.module_selector('internal')
        # This element inherits the connection constraint
        # Libraries will check that 'internal' has no imports from 'external'
```

#### Connection Boundary Semantics

| Keyword | Meaning |
|---------|---------|
| `allows_connection to X` | Code in this scope MAY import/depend on scope X |
| `forbids_connection to X` | Code in this scope MUST NOT import/depend on scope X |
| `requires_connection to X` | Code in this scope MUST import/depend on scope X |

**Note**: Wildcards (`.*`) are interpreted by language-specific libraries. They may be considered violations of strict boundaries or not sufficient for `requires` rules, depending on the library's implementation.

### 2. AST Changes

Add new types to `ast.rs`:

```rust
/// A hierarchical requirement that must be satisfied by at least one descendant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchicalRequirement {
    /// Kind of requirement
    pub kind: HierarchicalRequirementKind,
    /// Source span
    pub span: Span,
}

/// Types of hierarchical requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HierarchicalRequirementKind {
    /// Requires a descendant with a matching scope
    Scope(ScopeDeclaration),
    /// Requires a descendant with a matching check
    Check(CheckDeclaration),
    /// Requires a descendant element with specific structure
    Element(Element),
}

/// A connection boundary constraint for architectural dependencies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionBoundary {
    /// Whether this allows, forbids, or requires the connection
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
    /// Requires connections to matching targets
    Requires,
}
```

### 3. Grammar Additions

```ebnf
(* Hierarchical requirements *)
hierarchical_requirement ::= 'requires_descendant' (scope_declaration | check_declaration | element_declaration)

(* Connection boundaries - now includes requires_connection *)
connection_boundary ::= ('allows_connection' | 'forbids_connection' | 'requires_connection') 'to' connection_pattern
connection_pattern  ::= qualified_identifier ('.' '*')?

(* Updated element/template body *)
element_item        ::= scope_declaration
                      | connection_point_declaration
                      | check_declaration
                      | element_declaration
                      | template_binding
                      | hierarchical_requirement      (* NEW *)
                      | connection_boundary         (* NEW *)
```

### 4. Interpreter Changes

#### Hierarchical Requirement Validation

The interpreter needs to traverse the element hierarchy to validate hierarchical requirements:

```rust
fn validate_hierarchical_requirements(&self, element: &Element) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    for req in &element.hierarchical_requirements {
        let satisfied = self.find_matching_descendant(element, &req.kind);
        if !satisfied {
            diagnostics.push(Diagnostic::error(
                "E300",
                format!("Hierarchical requirement not satisfied: no descendant matches {:?}", req.kind)
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

1. **Hierarchical Requirements**:
   - At least one descendant must satisfy each `requires_descendant`
   - The match can be a direct child or any level of nesting
   - Multiple descendants can satisfy the same requirement

2. **Connection Boundaries**:
   - `allows_connection` creates a whitelist - only listed targets are permitted
   - `forbids_connection` creates a blacklist - listed targets are prohibited
   - `requires_connection` mandates that a dependency MUST exist
   - Multiple boundaries are combined (allows AND forbids AND requires)
   - Boundaries are inherited by all descendants within the parent/child hierarchy
   - Connection patterns support wildcards (`*`) - interpretation is library-specific

3. **Hierarchical Dependency Composition**:
   - If A is only allowed to connect to B, and B is only allowed to connect to C
   - Then A→B→C is allowed (each hop respects its own boundary)
   - This enables construction of complex layered architectures

4. **Template Integration**:
   - Templates can define hierarchical requirements
   - Elements implementing templates inherit those requirements
   - Concrete element hierarchies must satisfy all requirements

5. **Language-Specific Verification**:
   - Hielements declares boundaries; libraries verify them
   - Scope information is aggregated bottom-up to parent elements
   - Libraries receive aggregated scope info to perform actual import/dependency checks

### 7. Implementation Plan

#### Phase 1: Lexer and Parser
- Add `requires_descendant`, `allows_connection`, `forbids_connection`, `requires_connection` keywords
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
    
    ## Hierarchical requirement support
    element hierarchical_checks:
        scope ast_hierarchical = rust.module_selector('ast')
        
        check rust.struct_exists('HierarchicalRequirement')
        check rust.enum_exists('HierarchicalRequirementKind')
        check rust.struct_exists('ConnectionBoundary')
        check rust.enum_exists('ConnectionBoundaryKind')
        check rust.function_exists('validate_hierarchical_requirements')
        check rust.function_exists('validate_connection_boundaries')
```

### 9. Testing Strategy

1. **Parser Tests**:
   - Parse `requires_descendant` with scope/check/element
   - Parse `allows_connection`, `forbids_connection`, and `requires_connection`
   - Error cases (malformed syntax)

2. **Validation Tests**:
   - Hierarchical requirement satisfied at child level
   - Hierarchical requirement satisfied at grandchild level
   - Hierarchical requirement not satisfied (error)
   - Connection boundary allows (pass)
   - Connection boundary forbids (fail)
   - Connection boundary requires (check)
   - Inherited boundary enforcement

3. **Integration Tests**:
   - Full examples with templates and implementations
   - Complex hierarchies with multiple requirements

### 10. Scope Aggregation for Libraries

For connection boundary checking to work effectively, scopes must expose information that can be:
1. **Queried** - "Which files/modules belong to this scope?"
2. **Aggregated** - "Combine all child scopes into parent scope info"
3. **Passed to checks** - Libraries receive scope membership info to verify imports

Example flow:
```
element frontend:
    scope src = python.module_selector('frontend')
    forbids_connection to database.*
    
    element ui:
        scope src = python.module_selector('frontend.ui')
        # Inherits forbids_connection to database.*
```

When checking `frontend`, the library receives:
- Frontend's scope: `frontend/` and `frontend/ui/`
- Forbidden target pattern: `database.*`
- Library resolves `database.*` to actual modules and checks imports

### 11. Future Extensions

1. **Conditional requirements**: `requires_descendant if condition`
2. **Cardinality**: `requires_descendant exactly(2) element ...`
3. **Named boundaries**: `boundary_group secure: forbids_connection to ...`
4. **Boundary exceptions**: `allows_exception for specific_element`

## Conclusion

The hierarchical checks feature extends Hielements' declarative architecture description capabilities by allowing parent elements to express requirements that must be satisfied somewhere in their descendant hierarchy, and to control architectural dependencies (imports/dependencies) between elements. The actual verification is delegated to language-specific libraries, keeping hielements language-agnostic while enabling powerful architectural constraints.
