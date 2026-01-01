# Templates-Only Keywords and Scope Tracking

## Summary

This change restricts `requires`, `allows`, and `forbids` keywords to templates only. These keywords are used to define architectural constraints that elements can inherit by implementing templates.

## Changes Made

### 1. Parser Changes

The parser now rejects `requires`, `allows`, and `forbids` in regular element bodies with a clear error message:
```
error[E012] : 'requires' is only allowed in templates, not in regular elements. 
Define a template with this constraint and have the element implement it.
```

This ensures that architectural constraints are defined declaratively in templates, while elements describe the actual structure and implement those templates.

### 2. Documentation Updates

- Updated `README.md` to use the correct syntax (`requires descendant` instead of `requires_descendant`)
- Updated `doc/language_reference.md`:
  - Clarified that `requires`, `allows`, `forbids` are templates-only
  - Updated all examples to use templates instead of elements for constraints
  - Updated the grammar to remove `component_requirement` from `element_body`
  - Added notes about the templates-only restriction

### 3. Example Updates

- Fixed `examples/advanced_templates.hie` to use correct syntax
- Updated `examples/hierarchical.hie` to use templates for all constraints

## Design Rationale

Templates define **architectural patterns and constraints** that specify how components should be structured and what dependencies they can/cannot have. Elements describe the **actual structure** of the system and implement those patterns.

This separation provides:
1. **Reusability**: Architectural patterns are defined once in templates
2. **Clarity**: Clear distinction between constraints (templates) and structure (elements)
3. **Enforcement**: Elements must explicitly implement templates to inherit constraints

## Scope Tracking for Connection Rules (Future Work)

The `requires connection to`, `allows connection to`, and `forbids connection to` constraints require runtime enforcement by tracking:

1. **Scope association**: Each element's scope defines what code belongs to it
2. **Connection resolution**: Libraries must resolve what modules/dependencies exist
3. **Constraint checking**: Compare actual dependencies against allowed/forbidden patterns

### Current Status

- Parsing and validation of component requirements is implemented
- Actual enforcement during `run` phase is NOT yet implemented
- The interpreter validates the syntax of requirements but doesn't enforce them at runtime

### Implementation Notes for Future Work

To fully enforce connection boundaries:

1. **Build a scope registry**: Track all scopes defined in the element hierarchy
2. **Collect inherited constraints**: Propagate template constraints to implementing elements
3. **During runtime**:
   - For each element with scopes, resolve what modules/files belong to it
   - Use language-specific libraries to determine actual imports/dependencies
   - Check if dependencies satisfy `requires`, `allows`, `forbids` constraints
   - Report violations as check failures

Example pseudo-code:
```rust
fn enforce_connection_boundaries(element: &Element, parent_constraints: &Constraints) {
    // Merge inherited constraints from templates
    let constraints = parent_constraints.merge(&element.template_constraints());
    
    for scope in &element.scopes {
        let resolved_scope = self.resolve_scope(scope);
        let actual_deps = library.get_dependencies(resolved_scope);
        
        // Check forbids
        for pattern in &constraints.forbids {
            if actual_deps.matches(pattern) {
                report_error("Forbidden connection detected");
            }
        }
        
        // Check allows (if any allows exist, only those are permitted)
        if !constraints.allows.is_empty() {
            for dep in &actual_deps {
                if !constraints.allows.any(|p| p.matches(dep)) {
                    report_error("Connection not in allow list");
                }
            }
        }
        
        // Check requires
        for pattern in &constraints.requires {
            if !actual_deps.matches(pattern) {
                report_error("Required connection not found");
            }
        }
    }
    
    // Recursively check children
    for child in &element.children {
        enforce_connection_boundaries(child, &constraints);
    }
}
```

## Testing

All existing tests pass. The changes ensure:
- Templates can use `requires`/`allows`/`forbids`
- Elements cannot use these keywords (clear error message)
- All examples parse and validate correctly
