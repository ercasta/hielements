# Connection Point Explicit Typing

## Overview

This document describes the implementation of explicit typing for connection points in Hielements. Connection points **require** type annotations to ensure correct integration across multiple libraries and languages.

## Problem Statement

Connection points previously had no explicit type information, making it difficult to:
1. Validate compatibility between connection points across different elements
2. Ensure type safety when integrating code across multiple languages
3. Document the expected interface contract of connection points
4. Provide better IDE support and error messages

## Solution

Add **mandatory** explicit type annotations to connection points with support for:
- **Basic types**: `string`, `integer`, `float`, `boolean`
- **Custom types**: User-defined type aliases and composite structures

## Syntax

### Basic Type Annotation

```hielements
connection_point <name>: <type> = <expression>
```

Type annotations are **mandatory** for all connection points.

### Examples

```hielements
# Basic types (all mandatory)
connection_point port: integer = docker.exposed_port(dockerfile)
connection_point api_url: string = python.get_api_url(module)
connection_point enabled: boolean = config.get_flag('feature_enabled')
connection_point response_time: float = metrics.get_average_latency()

# Custom type (alias)
connection_point tokens: TokenStream = rust.struct_selector('Token')
connection_point ast: AbstractSyntaxTree = rust.struct_selector('Program')

# Composite type (structure)
connection_point db_config: DbConfig = python.class_selector(module, 'DatabaseConfig')
```

## Type System

### Basic Types

| Type | Description | Example Values |
|------|-------------|----------------|
| `string` | Text data | `"api/v1"`, `"localhost"` |
| `integer` | Whole numbers | `8080`, `443`, `-1` |
| `float` | Decimal numbers | `3.14`, `0.5`, `-2.718` |
| `boolean` | True/false | `true`, `false` |

### Custom Types

Custom types can be:
1. **Type Aliases**: Simple names for basic types
   ```hielements
   # In library or element
   type Port = integer
   type Url = string
   
   connection_point api_port: Port = docker.exposed_port(dockerfile)
   ```

2. **Composite Types**: Structures composed of multiple fields
   ```hielements
   # In library or element
   type ServiceConfig = {
       port: integer,
       host: string,
       ssl_enabled: boolean
   }
   
   connection_point config: ServiceConfig = python.class_selector(module, 'Config')
   ```

## Implementation Details

### AST Changes

Updated `ConnectionPointDeclaration` structure:

```rust
pub struct ConnectionPointDeclaration {
    pub name: Identifier,
    pub type_annotation: TypeAnnotation,  // Mandatory
    pub expression: Expression,
    pub span: Span,
}

pub struct TypeAnnotation {
    pub type_name: Identifier,
    pub span: Span,
}
```

### Lexer Changes

No new tokens required. The `:` token already exists for element declarations.

### Parser Changes

Updated `parse_connection_point` to **require** type annotation:

```rust
fn parse_connection_point(&mut self) -> Result<ConnectionPointDeclaration, Diagnostic> {
    // connection_point <name> : <type> = <expression>
    self.expect(TokenKind::ConnectionPoint)?;
    let name = self.parse_identifier()?;
    
    // Type annotation is mandatory
    self.expect(TokenKind::Colon)?;
    let type_annotation = self.parse_type_annotation()?;
    
    self.expect(TokenKind::Equals)?;
    let expression = self.parse_expression()?;
    // ...
}
```

### Interpreter Changes

Added type validation in the interpreter:

1. **Type Registration**: Types from libraries are registered during import
2. **Type Checking**: Connection point types are validated when elements are instantiated
3. **Type Compatibility**: When connection points are referenced across elements, types are checked for compatibility

### Standard Library Updates

Built-in libraries provide type information for their selectors:

```rust
// In RustLibrary
fn struct_selector(&self, name: &str) -> Value {
    Value::Selector(Selector {
        kind: SelectorKind::RustStruct,
        target: name.to_string(),
        type_info: Some(TypeInfo {
            name: name.to_string(),
            kind: TypeKind::Custom,
        }),
    })
}
```

## Migration Guide

### Type Annotations Required

All connection points **must** have type annotations. This ensures:
- Clear documentation of interfaces
- Type safety across elements
- Better error messages
- IDE support for type checking

Example:

```hielements
element api:
    # Type annotation is mandatory
    connection_point endpoint: HttpHandler = python.function_selector(module, 'handler')
    connection_point port: integer = docker.exposed_port(dockerfile)
```

## Benefits

1. **Type Safety**: Catch type mismatches at specification validation time
2. **Documentation**: Types serve as inline documentation of interfaces
3. **IDE Support**: Better autocomplete and error checking in editors
4. **Cross-Language**: Explicit types enable better integration across language boundaries
5. **Library Development**: Library authors can provide rich type information

## Examples

### Compiler with Typed Connection Points

```hielements
template compiler:
    element lexer:
        connection_point tokens: TokenStream = rust.struct_selector('Token')
    
    element parser:
        connection_point ast: AbstractSyntaxTree = rust.struct_selector('Program')
    
    # Type checking ensures tokens are compatible with parser input
    check compiler.lexer.tokens.compatible_with(compiler.parser.input)
```

### Microservice with Typed Connection Points

```hielements
element orders_service:
    scope api_module = python.module_selector('orders.api')
    scope db_module = python.module_selector('orders.db')
    
    connection_point rest_api: HttpEndpoint = python.public_functions(api_module)
    connection_point database: DbConnection = python.class_selector(db_module, 'Database')
    connection_point port: integer = docker.exposed_port(dockerfile)
    
    # Type checking ensures port is an integer
    check docker.exposes_port(dockerfile, port)
```

### Cross-Language Integration

```hielements
element full_stack_app:
    element frontend:
        scope typescript_module = typescript.module_selector('api-client')
        connection_point api_client: ApiClient = typescript.class_selector(typescript_module, 'OrdersApi')
    
    element backend:
        scope python_module = python.module_selector('api.orders')
        connection_point api_handler: HttpHandler = python.function_selector(python_module, 'handle_orders')
    
    # Type checking ensures frontend client matches backend handler signature
    check api_compatibility(frontend.api_client, backend.api_handler)
```

## Testing Strategy

1. **Parser Tests**: Verify type annotation parsing
2. **Interpreter Tests**: Validate type checking logic
3. **Integration Tests**: Test type compatibility across elements
4. **Backward Compatibility Tests**: Ensure untyped connection points still work
5. **Example Updates**: Update all examples to demonstrate typed connection points

## Future Enhancements

1. **Generic Types**: Support for parameterized types (e.g., `List<string>`)
2. **Union Types**: Allow multiple possible types (e.g., `string | integer`)
3. **Type Inference**: Automatic type inference from library metadata
4. **Type Libraries**: Shared type definitions across specifications
5. **Structural Typing**: Duck-typing style compatibility checking

## Related Work

- Type systems in other specification languages (Alloy, TLA+)
- Interface Definition Languages (IDL, Protocol Buffers, GraphQL)
- Gradual typing systems (TypeScript, Python type hints)

## Conclusion

Explicit typing for connection points provides the foundation for type-safe architectural specifications while maintaining the simplicity and flexibility of Hielements. The gradual typing approach allows incremental adoption without breaking existing specifications.
