# Element Templates Feature Design

**Issue:** Element templates  
**Date:** 2025-12-30

## Summary

This document analyzes approaches for implementing Element Templates in Hielements. Element Templates allow creating reusable element definitions that truly define the nature of a component. For example, a "compiler" element template would require a "lexer" child element and a "parser" child element.

## Problem Statement

Currently, Hielements allows defining concrete elements with scopes, connection points, and checks. However, there is no way to define reusable "patterns" or "templates" that can be instantiated multiple times with different concrete implementations.

**Example Use Case:**

```hielements
# Without templates - repetitive definitions
element frontend_compiler:
    element lexer:
        scope module = typescript.module_selector('frontend/lexer')
    element parser:
        scope module = typescript.module_selector('frontend/parser')
    check lexer.module.produces(parser.module.input_type)

element backend_compiler:
    element lexer:
        scope module = rust.module_selector('backend/lexer')
    element parser:
        scope module = rust.module_selector('backend/parser')
    check lexer.module.produces(parser.module.input_type)

# With templates - define once, reuse
template compiler:
    element lexer
    element parser
    check lexer.produces(parser.input)

element frontend_compiler implements compiler:
    lexer.scope = typescript.module_selector('frontend/lexer')
    parser.scope = rust.module_selector('frontend/parser')
```

## Requirements

1. **Template Definition:** Ability to define reusable element structures
2. **Template Implementation:** Elements can implement one or more templates
3. **Concrete Scoping:** When implementing, concrete scopes must be specified
4. **Absolute References:** Avoid name clashes between multiple templates via absolute references
5. **User Extensibility:** Templates should be definable in user libraries
6. **Validation:** Template conformance must be validated

## Approach Options

### Option 1: Template Declarations with Implementation Syntax

**Description:** Add a new `template` keyword for defining templates, and an `implements` clause for elements to implement templates.

**Syntax:**

```hielements
# Define a template
template compiler:
    element lexer:
        connection_point tokens
    
    element parser:
        connection_point ast
    
    check compiler.lexer.tokens == compiler.parser.input_type

# Implement the template
element my_compiler implements compiler:
    # Provide concrete scopes for template elements
    compiler.lexer.scope = rust.module_selector('my_lexer')
    compiler.parser.scope = rust.module_selector('my_parser')
    
    # Can add additional elements/checks
    element optimizer:
        scope module = rust.module_selector('optimizer')
```

**How absolute references work:**
- Template properties are prefixed with template name: `compiler.lexer`
- This prevents name clashes when implementing multiple templates
- Example with multiple templates:
```hielements
element service implements microservice, observable:
    microservice.api.scope = python.module_selector('api')
    observable.metrics.scope = python.module_selector('metrics')
```

**Pros:**
- Clear separation between templates and implementations
- Explicit syntax makes intent obvious
- Absolute references naturally prevent name clashes
- Easy to validate template conformance
- Familiar pattern from OOP (interfaces/traits)

**Cons:**
- Adds new keywords and syntax complexity
- Requires significant parser/interpreter changes
- May be complex to resolve nested template references

### Option 2: Parameterized Elements (Generic Elements)

**Description:** Elements can have parameters that must be provided when instantiated.

**Syntax:**

```hielements
# Define a parameterized element
element compiler<lexer_selector, parser_selector>:
    element lexer:
        scope module = lexer_selector
    
    element parser:
        scope module = parser_selector
    
    check lexer.module.produces(parser.module.input)

# Instantiate with concrete parameters
element my_compiler = compiler<
    rust.module_selector('my_lexer'),
    rust.module_selector('my_parser')
>
```

**Pros:**
- More functional/generic programming style
- Explicit parameterization
- Clear where template "holes" are filled

**Cons:**
- Different paradigm from current language design
- Complex to implement type system for parameters
- Less flexible for complex templates
- Name clash prevention not as natural

### Option 3: Mixins/Composition

**Description:** Define reusable element fragments that can be composed into elements.

**Syntax:**

```hielements
# Define a mixin
mixin compiler_structure:
    element lexer
    element parser
    check lexer.produces(parser.input)

# Use the mixin
element my_compiler:
    include compiler_structure
    
    # Provide concrete scopes
    lexer.scope = rust.module_selector('my_lexer')
    parser.scope = rust.module_selector('my_parser')
```

**Pros:**
- Simpler than full template system
- Similar to CSS/SASS mixins
- Easy to understand

**Cons:**
- Implicit merging can be confusing
- Name clashes more likely
- Less structured than templates
- Harder to validate conformance

### Option 4: Template Functions in Libraries

**Description:** Templates are functions in libraries that generate element structures.

**Syntax:**

```hielements
import templates

# Call a template function from a library
element my_compiler = templates.compiler(
    lexer: rust.module_selector('my_lexer'),
    parser: rust.module_selector('my_parser')
)
```

**Pros:**
- No new language syntax needed
- Templates can be distributed in libraries
- Fully extensible by users
- Can use full power of host language

**Cons:**
- Less declarative
- Harder to type-check and validate
- Generated structures less transparent
- Debugging more difficult

### Option 5: Hybrid Approach (Recommended)

**Description:** Combine Option 1 (template declarations) with Option 4 (library support) for maximum flexibility.

**Phase 1: Built-in Templates**
```hielements
# Built-in template syntax
template compiler:
    element lexer:
        connection_point tokens
    element parser:
        connection_point ast

element my_compiler implements compiler:
    compiler.lexer.scope = rust.module_selector('lexer')
    compiler.parser.scope = rust.module_selector('parser')
```

**Phase 2: Library-Defined Templates**
```hielements
import templates_lib

# Library can export templates
element my_service implements templates_lib.microservice:
    microservice.api.scope = python.module_selector('api')
```

**Pros:**
- Built-in syntax for common patterns
- Extensible via libraries
- Progressive complexity
- Best of both worlds

**Cons:**
- Two ways to define templates (could be confusing)
- More implementation complexity

## Recommendation: Option 5 (Hybrid Approach)

The hybrid approach is recommended because:

1. **Progressive Enhancement:** Start with built-in template syntax (simpler), add library support later
2. **Extensibility:** Users can create templates in libraries without modifying core
3. **Clear Semantics:** Built-in syntax makes template intent explicit
4. **Familiar Pattern:** Similar to interfaces/traits in mainstream languages
5. **Absolute References:** Natural solution to name clashes

## Implementation Plan

### Phase 1: Core Template Syntax

#### 1. Language Changes

**New Keywords:**
- `template` - Declare a template
- `implements` - Declare that an element implements one or more templates

**AST Changes:**
```rust
// Add to ast.rs
pub struct Template {
    pub doc_comment: Option<String>,
    pub name: Identifier,
    pub elements: Vec<Element>,
    pub scopes: Vec<ScopeDeclaration>,
    pub connection_points: Vec<ConnectionPointDeclaration>,
    pub checks: Vec<CheckDeclaration>,
    pub span: Span,
}

pub struct TemplateImplementation {
    pub template_name: Identifier,
    pub bindings: Vec<TemplateBinding>,
    pub span: Span,
}

pub struct TemplateBinding {
    pub path: Vec<Identifier>,  // e.g., ["template_name", "element_name", "scope"]
    pub expression: Expression,
    pub span: Span,
}

// Modify Element to support templates
pub struct Element {
    // ... existing fields
    pub implements: Vec<TemplateImplementation>,
    // ... rest of fields
}

// Modify Program to include templates
pub struct Program {
    pub imports: Vec<ImportStatement>,
    pub templates: Vec<Template>,  // NEW
    pub elements: Vec<Element>,
    pub span: Span,
}
```

#### 2. Parser Changes

- Add parsing for `template` declarations
- Add parsing for `implements` clause in elements
- Add parsing for template property bindings (e.g., `compiler.lexer.scope = ...`)

#### 3. Interpreter Changes

- Validate template structure
- Validate that implementing elements provide all required bindings
- Resolve template references during element instantiation
- Support absolute property references with template name prefix

### Phase 2: Library-Defined Templates

#### 1. Extend Library Trait

```rust
pub trait Library {
    fn name(&self) -> &str;
    fn call(&self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value>;
    fn check(&self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult>;
    
    // NEW: Support for library-defined templates
    fn get_template(&self, name: &str) -> LibraryResult<Option<Template>>;
    fn list_templates(&self) -> Vec<String>;
}
```

#### 2. Template Protocol for External Libraries

Extend JSON-RPC protocol:
```json
// Get template request
{"jsonrpc": "2.0", "method": "library.get_template", "params": {"name": "microservice"}, "id": 1}

// Response
{
  "jsonrpc": "2.0", 
  "result": {
    "name": "microservice",
    "elements": [
      {"name": "api", "required": true},
      {"name": "database", "required": false}
    ],
    "checks": ["api.exposes_rest", "database.migrations_exist"]
  }, 
  "id": 1
}
```

## Name Clash Prevention

The absolute reference system prevents name clashes:

```hielements
template compiler:
    element parser

template interpreter:
    element parser

element my_vm implements compiler, interpreter:
    # No ambiguity - each parser is explicitly qualified
    compiler.parser.scope = rust.module_selector('compile_parser')
    interpreter.parser.scope = rust.module_selector('interp_parser')
    
    # Checks can reference both
    check compiler.parser.ast_output == interpreter.parser.ast_input
```

## Validation Rules

1. **Template Completeness:** All template elements must have bindings provided
2. **Type Consistency:** Bindings must match expected types (scope, connection_point, etc.)
3. **Reference Resolution:** All template references must resolve to valid template elements
4. **No Circular Dependencies:** Templates cannot implement themselves (directly or transitively)
5. **Unique Template Names:** Template names must be unique in a scope

## Examples

### Example 1: Compiler Template

```hielements
template compiler:
    ## Lexer component
    element lexer:
        connection_point tokens
    
    ## Parser component  
    element parser:
        connection_point ast
    
    ## Ensure lexer produces tokens that parser can consume
    check compiler.lexer.tokens.compatible_with(compiler.parser.input)

element python_compiler implements compiler:
    compiler.lexer.scope = python.module_selector('compiler.lexer')
    compiler.parser.scope = python.module_selector('compiler.parser')
    compiler.lexer.tokens = python.get_tokens(compiler.lexer.scope)
    compiler.parser.ast = python.get_ast(compiler.parser.scope)
```

### Example 2: Microservice Template

```hielements
template microservice:
    element api:
        connection_point rest_endpoint
    
    element database:
        connection_point connection
    
    element container:
        connection_point ports
    
    check microservice.api.exposes_rest()
    check microservice.container.ports.includes(microservice.api.port)

element orders_service implements microservice:
    microservice.api.scope = python.module_selector('orders.api')
    microservice.database.scope = postgres.database_selector('orders_db')
    microservice.container.scope = docker.file_selector('orders.dockerfile')
```

### Example 3: Multiple Template Implementation

```hielements
template observable:
    element metrics:
        connection_point prometheus_endpoint

template resilient:
    element circuit_breaker:
        connection_point breaker_config

element robust_service implements microservice, observable, resilient:
    # Microservice bindings
    microservice.api.scope = python.module_selector('service.api')
    microservice.database.scope = postgres.database_selector('service_db')
    
    # Observable bindings
    observable.metrics.scope = python.module_selector('service.metrics')
    
    # Resilient bindings
    resilient.circuit_breaker.scope = python.module_selector('service.resilience')
```

## Migration Path

1. **Phase 1:** Implement core template syntax (this phase)
2. **Phase 2:** Add library template support
3. **Phase 3:** Build standard template library (microservice, compiler, hexagonal, etc.)
4. **Phase 4:** Community templates via external libraries

## Changes to hielements.hie

Add new elements documenting the template feature:

```hielements
element core:
    # ... existing elements ...
    
    ## Template System
    element templates:
        scope template_ast = rust.module_selector('templates')
        
        check rust.struct_exists('Template')
        check rust.struct_exists('TemplateImplementation')
        check rust.struct_exists('TemplateBinding')
        check rust.function_exists('resolve_template')
        check rust.function_exists('validate_template_implementation')
```

## Testing Strategy

1. **Parser Tests:**
   - Template declaration parsing
   - Template implementation parsing
   - Absolute reference parsing
   - Error cases (invalid syntax)

2. **Semantic Tests:**
   - Template validation
   - Implementation validation
   - Reference resolution
   - Name clash detection

3. **Integration Tests:**
   - Complete template definition and usage
   - Multiple template implementation
   - Nested template elements
   - Template checks execution

4. **Example Files:**
   - `examples/template_compiler.hie`
   - `examples/template_microservice.hie`
   - `examples/template_multiple.hie`

## Security Considerations

1. **Template Injection:** Validate that template definitions cannot inject malicious code
2. **Circular References:** Prevent infinite loops in template resolution
3. **Resource Limits:** Limit template expansion depth and complexity
4. **Library Templates:** Same security model as external libraries

## Documentation Updates

1. **Language Reference:**
   - New section: "Element Templates"
   - Update grammar with template syntax
   - Add template examples

2. **Examples:**
   - Create template examples demonstrating patterns
   - Show multiple implementation scenarios

3. **Best Practices:**
   - When to use templates vs regular elements
   - Naming conventions for templates
   - Template composition strategies

## Conclusion

The hybrid template approach provides a powerful mechanism for code reuse in Hielements while maintaining the declarative nature of the language. The absolute reference system elegantly solves name clashing, and the phased implementation allows for progressive enhancement of the feature.
