# Scope Management in Hielements

This document describes how identifier scopes are managed and resolved in the Hielements language.

## Overview

Hielements uses a hierarchical scope management system where identifiers are resolved based on their position in the element hierarchy. The interpreter maintains a scope registry that maps qualified identifier paths to their values.

## Scope Storage

### Scope Registration

When elements are processed during interpretation, scopes are registered with their **fully-qualified paths**:

```rust
// During element processing
let scope_name = format!("{}.{}", element_path, scope.name.name);
self.scopes.insert(scope_name, value);
```

For example, in this specification:

```hielements
element hielements:
    element core:
        element parser:
            scope module = rust.module_selector('parser')
```

The scope `module` is stored with the key `hielements.core.parser.module`.

### Scope Value Types

Scopes can hold different value types:

| Type | Description |
|------|-------------|
| `Scope` | File/folder selection result (paths, kind, resolved status) |
| `ConnectionPoint` | Named interface or API endpoint |
| `Value` | Primitive values (string, int, float, bool, list) |

## Scope Resolution Algorithm

The interpreter resolves identifiers using a three-phase lookup:

### Phase 1: Current Element Context

First, try to find the identifier in the current element's context:

```rust
let current_scope_key = format!("{}.{}", self.current_element_path, id.name);
if let Some(value) = self.scopes.get(&current_scope_key) {
    return Ok(value.clone());
}
```

### Phase 2: Suffix Match

If not found, try suffix matching across all scopes:

```rust
let lookup_suffix = format!(".{}", id.name);
for (name, value) in &self.scopes {
    if name.ends_with(&lookup_suffix) || name == &id.name {
        return Ok(value.clone());
    }
}
```

### Phase 3: Error

If no match is found, raise an undefined identifier error.

## Scope Reference Patterns

### Local Reference

Within the same element, scopes can be referenced directly by name:

```hielements
element api:
    scope module = rust.module_selector('api')
    check rust.function_exists(module, 'main')  # 'module' resolves locally
```

### Child Element Reference

Parent elements can reference child element scopes using dot notation:

```hielements
element parent:
    element child:
        scope src = files.folder_selector('src')
    
    check files.exists(child.src, 'main.rs')  # References child's scope
```

### Sibling Element Reference

Sibling elements can also reference each other's scopes:

```hielements
element system:
    element service_a:
        scope module = rust.module_selector('service_a')
        ref api: Handler = rust.function_selector(module, 'handler')
    
    element service_b:
        scope module = rust.module_selector('service_b')
        check rust.depends_on(module, service_a.api)  # Cross-sibling reference
```

## Pattern Scope Binding

### Pattern Placeholders

Patterns (declared with `template`) define scope placeholders that implementations must bind:

```hielements
pattern microservice:
    element api:
        scope module  # Placeholder - no expression
        ref endpoint: Handler
```

### Binding Syntax

Implementations bind concrete values using qualified paths:

```hielements
element my_service implements microservice:
    microservice.api.module = rust.module_selector('api')
    microservice.api.endpoint = rust.function_selector(microservice.api.module, 'handler')
```

### Binding Resolution

Pattern bindings use **absolute paths** starting with the pattern name to:
1. Avoid name clashes when implementing multiple patterns
2. Clearly identify which pattern property is being bound
3. Support nested element references within patterns

## Connection Point Scoping

Connection points follow similar scoping rules but represent interfaces rather than file selections:

```hielements
element api_service:
    scope module = rust.module_selector('api')
    
    # Connection points can reference scopes
    ref handler: HttpHandler = rust.function_selector(module, 'handle_request')
    
    # Or be standalone typed declarations
    ref port: integer = docker.exposed_port(dockerfile)
```

## Validation-Time vs Runtime Resolution

### Validation Phase

During semantic validation:
- Check for undefined references
- Verify import availability
- Validate expression structure

### Runtime Phase

During check execution:
- Fully resolve scope expressions
- Invoke library functions
- Execute checks against actual code

## Scope Visibility Rules

| Context | Can Access |
|---------|------------|
| Element | Own scopes, parent scopes (via qualified path), child scopes (via child.scope) |
| Pattern | Own placeholder scopes, bound scopes in implementations |
| Check | All scopes in scope at the check location |

## Error Messages

When scope resolution fails, the interpreter provides helpful error messages:

```
Error E200: Undefined identifier: module
  --> architecture.hie:15:10
   |
15 |     check rust.function_exists(module, 'main')
   |                                ^^^^^^
   |
   = help: Did you mean to reference a scope? Available scopes: src, config
```

## Best Practices

### 1. Use Descriptive Scope Names

```hielements
# Good
scope python_module = python.module_selector('orders')
scope config_file = files.file_selector('config.yaml')

# Avoid
scope m = python.module_selector('orders')
scope f = files.file_selector('config.yaml')
```

### 2. Qualify Cross-Element References

```hielements
# Good - explicit reference
check rust.depends_on(module, orders_service.api)

# Avoid relying on suffix matching for clarity
```

### 3. Keep Scope Hierarchies Shallow

Deep nesting can make scope resolution confusing. Prefer flatter hierarchies when possible.

### 4. Document Scope Purposes

Use comments to clarify what each scope represents:

```hielements
## Python source code for the orders module
scope python_src = python.module_selector('orders')

## Docker configuration for containerization
scope dockerfile = docker.file_selector('Dockerfile')
```

## Implementation Details

The scope management is implemented in `crates/hielements-core/src/interpreter.rs`. Key components:

- `scopes: HashMap<String, Value>` - The scope registry
- `current_element_path: String` - Tracks the current element context
- `evaluate_expression()` - Handles scope resolution in expressions
- `process_scope()` - Registers new scopes during element processing
