# Hielements Language Reference (V2)

This document provides a complete reference for the Hielements V2 language syntax and semantics. Hielements is a declarative language for describing and enforcing software architecture.

**Version**: 2.0 (This version is incompatible with V1 - see [Migration Guide](#appendix-d-migration-guide-from-v1))

## Design Philosophy

Hielements V2 introduces a clearer separation between *descriptive* and *prescriptive* parts of the language:

- **Prescriptive** (templates): Defines the structure, rules, and constraints using `requires`/`forbids`/`allows` keywords and checks
- **Descriptive** (elements): Implements templates and binds to actual code using scopes

It is possible to use only the descriptive part without the prescriptive one; in this case, no enforcement/checks are performed.

---

## Table of Contents

1. [Lexical Structure](#1-lexical-structure)
2. [Program Structure](#2-program-structure)
3. [Elements](#3-elements)
4. [Scopes](#4-scopes)
5. [Connection Points](#5-connection-points)
6. [Rules (Checks)](#6-rules-checks)
7. [Children Elements](#7-children-elements)
8. [Element Templates](#8-element-templates)
9. [Language Declarations](#9-language-declarations)
10. [Imports and Modules](#10-imports-and-modules)
11. [Expressions](#11-expressions)
12. [Built-in Libraries](#12-built-in-libraries)
13. [Comments](#13-comments)
14. [Complete Grammar](#14-complete-grammar)
15. [Examples](#15-examples)
16. [Appendix A: Error Messages](#appendix-a-error-messages)
17. [Appendix B: CLI Reference](#appendix-b-cli-reference)
18. [Appendix C: Best Practices](#appendix-c-best-practices)
19. [Appendix D: Migration Guide from V1](#appendix-d-migration-guide-from-v1)

---

## 1. Lexical Structure

### 1.1 Character Set

Hielements source files are UTF-8 encoded text files. The recommended file extension is `.hie`.

### 1.2 Identifiers

Identifiers name elements, scopes, connection points, and other entities.

```
identifier ::= letter (letter | digit | '_')*
letter     ::= 'a'..'z' | 'A'..'Z' | '_'
digit      ::= '0'..'9'
```

**Valid identifiers:**
```
orders_module
MyService
_internal
api2
```

**Invalid identifiers:**
```
2fast        # Cannot start with digit
my-service   # Hyphens not allowed (use underscores)
```

### 1.3 Keywords

The following are reserved keywords:

| Keyword | Description |
|---------|-------------|
| `element` | Declares an element |
| `template` | Declares an element template |
| `implements` | Declares that an element implements template(s) |
| `binds` | Binds a scope/connection_point to a template declaration (V2) |
| `scope` | Declares a scope selector |
| `connection_point` | Declares a connection point |
| `check` | Declares a rule/check |
| `import` | Imports a library or module |
| `as` | Alias for imports |
| `from` | Selective import |
| `true` | Boolean literal |
| `false` | Boolean literal |
| `requires` | Declares a required component (templates only) |
| `allows` | Declares an allowed component (templates only) |
| `forbids` | Declares a forbidden component (templates only) |
| `descendant` | Modifier for hierarchical requirements (applies to descendants) |
| `connection` | Specifies a connection requirement |
| `to` | Specifies connection target |
| `language` | Declares a language with optional connection checks |
| `connection_check` | Defines a connection verification check for a language |

### 1.4 Literals

#### String Literals

Strings are enclosed in single or double quotes:

```hielements
'single quoted string'
"double quoted string"
```

Escape sequences:
| Sequence | Meaning |
|----------|---------|
| `\\` | Backslash |
| `\'` | Single quote |
| `\"` | Double quote |
| `\n` | Newline |
| `\t` | Tab |

#### Numeric Literals

```hielements
42          # Integer
3.14        # Float
8080        # Port number (integer)
```

#### Boolean Literals

```hielements
true
false
```

### 1.5 Operators and Punctuation

| Symbol | Usage |
|--------|-------|
| `=` | Assignment |
| `.` | Member access |
| `,` | Argument separator |
| `:` | Block start, type annotation |
| `(` `)` | Function call, grouping |
| `[` `]` | List literals |
| `<` `>` | Language specification in scopes (V2) |

---

## 2. Program Structure

A Hielements program (specification) consists of:
1. Optional import statements
2. One or more top-level element declarations

```hielements
# Imports (optional)
import python
import docker

# Top-level elements
element my_service:
    # ... element body
```

### 2.1 File Organization

A typical Hielements project structure:

```
project/
├── architecture.hie        # Main specification
├── modules/
│   ├── backend.hie         # Backend module specs
│   └── frontend.hie        # Frontend module specs
└── hielements.config       # Configuration (optional)
```

---

## 3. Elements

Elements are the fundamental building blocks of Hielements. An element represents a logical component of your software system.

### 3.1 Syntax

```
element_declaration ::= 'element' identifier ':' element_body

element_body ::= (scope_declaration 
                | connection_point_declaration 
                | check_declaration 
                | nested_element)*
```

### 3.2 Basic Element

```hielements
element orders_service:
    scope src = files.folder_selector('src/orders')
    
    check files.contains(src, 'main.py')
```

### 3.3 Element with All Components

```hielements
element payment_gateway:
    # Scopes define what code belongs to this element
    scope python_module = python.module_selector('payments')
    scope config = files.file_selector('config/payments.yaml')
    scope dockerfile = docker.file_selector('payments.dockerfile')
    
    # Connection points expose interfaces to other elements
    connection_point api = python.public_functions(python_module)
    connection_point port = docker.exposed_port(dockerfile)
    
    # Checks enforce rules
    check docker.exposes_port(dockerfile, 8080)
    check python.no_circular_imports(python_module)
    
    # Nested elements for hierarchical structure
    element validation_submodule:
        scope src = python.module_selector('payments.validation')
```

### 3.4 Element Semantics

- Each element defines a **boundary** around a logical component
- Elements can be **nested** to create hierarchies
- Element names must be **unique** within their scope
- Elements are evaluated **lazily** - scopes are resolved when checks execute

---

## 4. Scopes

Scopes define what code, files, or artifacts belong to an element. Scopes are specified using **selectors** from libraries.

### 4.1 Syntax (V2)

In V2, scopes can be either **bound** (in elements) or **unbounded** (in templates). Language is specified using angular brackets:

```
# Bound scope (in elements)
scope_declaration ::= 'scope' identifier ['<' language_name '>'] ['binds' binding_path] '=' selector_expression

# Unbounded scope (in templates)
scope_declaration ::= 'scope' identifier ['<' language_name '>']
```

### 4.2 Scope Selectors

Selectors are library functions that identify parts of your codebase:

```hielements
# File and folder selectors
scope src_folder = files.folder_selector('src/')
scope config_file = files.file_selector('config.yaml')
scope all_python = files.glob_selector('**/*.py')

# Language-specific selectors with V2 angular bracket syntax
scope orders<python> = python.module_selector('orders')
scope backend<rust> = rust.module_selector('backend')

# Docker selectors
scope dockerfile = docker.file_selector('Dockerfile')
scope compose = docker.compose_selector('docker-compose.yml')
```

### 4.3 Scopes with Language Annotations (V2)

Scopes can optionally include a language annotation using **angular brackets** to explicitly declare which programming language the scope belongs to:

```hielements
element my_service:
    # Scope with explicit language annotation (V2 syntax)
    scope src<python> = python.module_selector('my_service')
    scope backend<rust> = rust.module_selector('backend')
    
    # Scope without language annotation (inferred from library)
    scope config = files.file_selector('config.yaml')
```

The language annotation is specified in angular brackets immediately after the scope name (`<language_name>`).

### 4.4 Unbounded Scopes in Templates

In templates, scopes are **unbounded** (declared without a selector expression). They serve as placeholders to be bound by implementing elements:

```hielements
template observable:
    element metrics:
        # Unbounded scope - no '=' expression
        scope module<rust>
        connection_point prometheus: MetricsHandler
```

### 4.5 Binding Scopes with `binds` (V2)

When an element implements a template, it uses the `binds` keyword to connect its scopes to the template's unbounded scopes:

```hielements
element observable_component implements observable:
    # Bind this scope to the template's unbounded scope
    scope main_module<rust> binds observable.metrics.module = rust.module_selector('payments::api')
    
    # Bind a connection point to the template's connection point
    connection_point main_handler: MetricsHandler binds observable.metrics.prometheus = rust.function_selector(main_module, 'handler')
```

The `binds` clause specifies which template scope/connection_point this declaration satisfies.

### 4.6 Multiple Scopes

An element can have multiple scopes, representing different aspects:

```hielements
element full_stack_feature:
    scope frontend<typescript> = typescript.module_selector('components/OrderForm')
    scope backend<python> = python.module_selector('api/orders')
    scope database = sql.migration_selector('migrations/001_orders.sql')
    scope container = docker.file_selector('orders.dockerfile')
```

### 4.7 Scope Composition

Scopes can be combined using set operations (library-dependent):

```hielements
element api_layer:
    scope handlers<python> = python.package_selector('api.handlers')
    scope models<python> = python.package_selector('api.models')
    
    # Combine scopes
    scope all_api<python> = scopes.union(handlers, models)
```

### 4.8 Scope Semantics

- Scopes are **lazy** - they don't scan the filesystem until needed
- Scopes can **overlap** between elements (a file can belong to multiple elements)
- Scope resolution may **fail** if the target doesn't exist (configurable: error vs warning)
- Scopes provide **source locations** for error reporting
- Scopes with **language annotations** enable language-specific connection verification

---

## 5. Connection Points

Connection points expose interfaces, APIs, or dependencies that other elements can reference. They make inter-element relationships explicit and verifiable.

### 5.1 Syntax

```
connection_point_declaration ::= 'connection_point' identifier ':' type_name '=' expression
```

Type annotations are **mandatory** for all connection points.

### 5.2 Basic Connection Points

```hielements
element api_server:
    scope module = python.module_selector('api')
    
    # All connection points must have type annotations
    connection_point rest_api: HttpHandler = python.public_functions(module)
    
    # Expose the main entry point
    connection_point main: Function = python.function_selector(module, 'main')
```

### 5.3 Connection Point Type Annotations

Connection points **must** have explicit type annotations to ensure type safety across libraries and languages:

```hielements
element api_server:
    scope module = python.module_selector('api')
    scope dockerfile = docker.file_selector('Dockerfile')
    
    # Basic type annotations (mandatory)
    connection_point port: integer = docker.exposed_port(dockerfile)
    connection_point api_url: string = python.get_api_url(module)
    connection_point ssl_enabled: boolean = config.get_flag('ssl')
    connection_point timeout: float = config.get_timeout()
    
    # Custom type annotations
    connection_point rest_api: HttpHandler = python.public_functions(module)
    connection_point db_conn: DatabaseConnection = python.class_selector(module, 'Database')
```

#### Basic Types

| Type | Description | Example Values |
|------|-------------|----------------|
| `string` | Text data | `"api/v1"`, `"localhost"` |
| `integer` | Whole numbers | `8080`, `443`, `-1` |
| `float` | Decimal numbers | `3.14`, `0.5`, `-2.718` |
| `boolean` | True/false | `true`, `false` |

#### Custom Types

Custom types are user-defined type names that can represent:
- Type aliases for basic types (e.g., `Port`, `Url`)
- Complex structures from code (e.g., `TokenStream`, `HttpHandler`)
- Library-defined types specific to a domain

```hielements
# Custom type example
template compiler:
    element lexer:
        connection_point tokens: TokenStream = rust.struct_selector('Token')
    
    element parser:
        connection_point ast: AbstractSyntaxTree = rust.struct_selector('Program')
```

### 5.4 Using Connection Points

Connection points are used in checks to verify relationships:

```hielements
element orders_service:
    element api:
        scope module = python.module_selector('orders.api')
        connection_point handlers: HttpHandler = python.public_functions(module)
    
    element docker:
        scope dockerfile = docker.file_selector('orders.dockerfile')
        
        # Verify Docker uses the correct entry point
        check docker.entry_point(dockerfile, api.handlers)
```

### 5.5 Connection Point Types

Different libraries expose different types of connection points:

| Library | Connection Point | Description |
|---------|-----------------|-------------|
| `python` | `public_functions` | All public functions in a module |
| `python` | `exported_classes` | All exported classes |
| `python` | `main_module` | The `__main__` entry point |
| `docker` | `exposed_ports` | Ports exposed by the container |
| `docker` | `volumes` | Mounted volumes |
| `files` | `path` | Filesystem path |

### 5.6 Connection Point Semantics

- Connection points are **computed** from scopes
- They can be **referenced** across element boundaries using dot notation
- Connection points enable **dependency checking** between elements
- They provide **documentation** of element interfaces
- **Type annotations** are mandatory and provide type safety
- **Type checking** occurs at specification validation time (when implemented)

---

## 6. Rules (Checks)

Checks enforce architectural rules. They are the mechanism by which Hielements validates your codebase against specifications.

### 6.1 Syntax

```
check_declaration ::= 'check' function_call
```

### 6.2 Basic Checks

```hielements
element my_service:
    scope dockerfile = docker.file_selector('Dockerfile')
    scope src = python.module_selector('my_service')
    
    # Check that port 8080 is exposed
    check docker.exposes_port(dockerfile, 8080)
    
    # Check that a specific function exists
    check python.function_exists(src, 'handle_request')
    
    # Check for no circular dependencies
    check python.no_circular_imports(src)
```

### 6.3 Check Categories

#### Existence Checks
Verify that something exists:

```hielements
check files.exists(src, 'README.md')
check python.function_exists(module, 'main')
check docker.stage_exists(dockerfile, 'builder')
```

#### Property Checks
Verify properties of artifacts:

```hielements
check docker.base_image(dockerfile, 'python:3.11-slim')
check python.has_docstring(function)
check files.max_size(file, 1048576)  # 1MB max
```

#### Relationship Checks
Verify relationships between components:

```hielements
check docker.entry_point(dockerfile, python_module.main)
check python.imports(module_a, module_b)
check python.no_dependency(module_a, module_b)
```

#### Negative Checks
Verify that something does NOT exist or is NOT true:

```hielements
check files.no_files_matching(src, '*.tmp')
check python.no_circular_imports(module)
check docker.no_root_user(dockerfile)
```

### 6.4 Check Results

Checks produce one of three results:

| Result | Meaning |
|--------|---------|
| **Pass** | The check succeeded |
| **Fail** | The check failed (architectural violation) |
| **Error** | The check could not be evaluated (e.g., file not found) |

### 6.5 Check Semantics

- Checks are evaluated **after** all scopes are resolved
- Checks are **independent** - one failing check doesn't prevent others from running
- Check results include **source locations** for diagnostics
- Checks can be **parallelized** when they have no dependencies

---

## 7. Children Elements

Elements can contain nested (children) elements to create hierarchical structures.

### 7.1 Syntax

Nested elements are declared inside a parent element:

```hielements
element parent:
    element child_a:
        # child_a body
    
    element child_b:
        # child_b body
```

### 7.2 Hierarchical Example

```hielements
element ecommerce_platform:
    
    element orders_service:
        scope module = python.module_selector('services.orders')
        connection_point api = python.public_functions(module)
        
        element orders_db:
            scope migrations = sql.migration_selector('db/orders')
    
    element payments_service:
        scope module = python.module_selector('services.payments')
        connection_point api = python.public_functions(module)
    
    # Cross-service check: orders can call payments
    check python.can_import(orders_service.module, payments_service.module)
```

### 7.3 Referencing Children

Children elements are referenced using dot notation:

```hielements
element system:
    element service_a:
        connection_point api = python.public_functions(module)
    
    element service_b:
        scope module = python.module_selector('service_b')
        
        # Reference sibling's connection point
        check python.imports(module, service_a.api)
```

### 7.4 Scope Inheritance

Children elements inherit the context of their parent but have their own scope:

```hielements
element microservices:
    # Shared configuration for all children
    scope shared_config = files.file_selector('shared/config.yaml')
    
    element service_a:
        scope module = python.module_selector('service_a')
        # Can reference parent's scope
        check files.references(module, shared_config)
```

---

## 8. Element Templates

Element templates allow creating reusable element definitions that define the nature of a component. Templates establish structural patterns that elements can implement with concrete scopes and checks.

In V2, templates define **unbounded** scopes that serve as placeholders, while implementing elements use the **`binds`** keyword to provide concrete bindings.

### 8.1 Template Declaration (V2)

Templates are declared using the `template` keyword and define a structure with **unbounded scopes**:

```hielements
template observable:
    element metrics implements measurable:
        allows language rust
        
        # Unbounded scope - angular brackets specify language
        scope module<rust>
        connection_point prometheus: MetricsHandler
        
        check files.exists(module, 'Cargo.toml')
```

**Key V2 Changes:**
- Scopes in templates are **unbounded** (no `=` expression)
- Language is specified via **angular brackets** (`<rust>`)
- Templates can include `allows`/`requires`/`forbids` constraints

### 8.2 Implementing Templates with `binds` (V2)

Elements implement templates using the `implements` keyword, then use **`binds`** to connect their scopes:

```hielements
element observable_component implements observable:
    # Bind scope to template's unbounded scope
    scope main_module<rust> binds observable.metrics.module = rust.module_selector('payments::api')
    
    # Bind connection point
    connection_point main_handler: MetricsHandler binds observable.metrics.prometheus = rust.function_selector(main_module, 'handler')
```

The `binds` keyword creates an explicit connection between the element's scope and the template's placeholder.

### 8.3 Descriptive-Only Mode

The `implements` and `binds` keywords are **optional**. When omitted, Hielements operates in "descriptive-only" mode without prescriptive enforcement:

```hielements
# Descriptive only - no template implementation
element simple_component:
    scope src<rust> = rust.module_selector('mymodule')
    check rust.function_exists(src, 'main')
```

### 8.4 Absolute References

Template properties are referenced using absolute paths prefixed with the template name (e.g., `observable.metrics`). This prevents name clashes when implementing multiple templates:

```hielements
template microservice:
    element api:
        scope module<rust>
        connection_point rest_endpoint: HttpHandler

template observable:
    element api:
        scope module<rust>
        connection_point metrics_endpoint: MetricsHandler

# No name clash - each 'api' is explicitly qualified
element my_service implements microservice, observable:
    scope api_mod<rust> binds microservice.api.module = rust.module_selector('service::api')
    scope metrics_mod<rust> binds observable.api.module = rust.module_selector('service::metrics')
    
    # Reference both in checks
    check microservice.api.rest_endpoint.port != observable.api.metrics_endpoint.port
```

### 8.5 Multiple Template Implementation

An element can implement multiple templates:

```hielements
template resilient:
    element circuit_breaker:
        scope module<rust>

template secured:
    element authentication:
        scope module<rust>

element production_service implements microservice, resilient, secured:
    # Microservice bindings
    scope api<rust> binds microservice.api.module = rust.module_selector('api')
    
    # Resilient bindings  
    scope resilience<rust> binds resilient.circuit_breaker.module = rust.module_selector('resilience')
    
    # Secured bindings
    scope auth<rust> binds secured.authentication.module = rust.module_selector('auth')
```

### 8.6 Template Requirements

When implementing a template, all unbounded scopes must be bound:

```hielements
template web_service:
    element frontend:
        scope src<typescript>
        connection_point static_files: StaticAssets
    
    element backend:
        scope src<python>
        connection_point api: HttpHandler

# Valid - all required bindings provided
element complete_service implements web_service:
    scope frontend_src<typescript> binds web_service.frontend.src = typescript.module_selector('frontend')
    scope backend_src<python> binds web_service.backend.src = python.module_selector('backend')
    connection_point static: StaticAssets binds web_service.frontend.static_files = files.glob_selector('frontend/dist/*')
    connection_point api: HttpHandler binds web_service.backend.api = python.public_functions(backend_src)

# Invalid - missing bindings (would produce validation error)
element incomplete_service implements web_service:
    scope frontend_src<typescript> binds web_service.frontend.src = typescript.module_selector('frontend')
    # ERROR: web_service.backend bindings missing
```

### 8.7 Template Checks

Checks defined in templates are automatically included when the template is implemented. The checks use absolute references and are evaluated with the concrete bindings:

```hielements
template microservice:
    element api:
        scope module<python>
    element database:
        scope db<postgres>
    element container:
        scope dockerfile
    
    # Template checks
    check microservice.container.exposes_port(8080)
    check microservice.api.connects_to(microservice.database)

element orders_service implements microservice:
    scope api_mod<python> binds microservice.api.module = python.module_selector('orders.api')
    scope db<postgres> binds microservice.database.db = postgres.database_selector('orders_db')
    scope dockerfile binds microservice.container.dockerfile = docker.file_selector('orders.dockerfile')
    
    # The template checks are automatically evaluated with bound scopes
```

### 8.8 Library-Defined Templates

Templates can be defined in external libraries and imported for use:

```hielements
import architecture_patterns

element my_service implements architecture_patterns.hexagonal:
    # Bind the hexagonal architecture template elements
    scope domain<python> binds hexagonal.domain.src = python.package_selector('myapp.domain')
    scope app<python> binds hexagonal.application.src = python.package_selector('myapp.application')
    scope adapters<python> binds hexagonal.adapters.src = python.package_selector('myapp.adapters')
```

External libraries can provide templates via the library protocol. See the [External Library Plugin Guide](external_libraries.md) for details.

### 8.9 Template Semantics (V2)

- Templates define **structure** with **unbounded scopes**
- Elements implementing templates use **`binds`** to connect scopes
- **Angular brackets** specify language (`<rust>`, `<python>`)
- `implements` and `binds` are **optional** (for prescriptive features)
- Template checks are **inherited** by implementing elements
- Absolute references **prevent name clashes** between multiple templates
- Templates **cannot be nested** (a template cannot implement another template)
- Template names must be **unique** within their scope

### 8.10 Template-Level Connection Points

Templates can declare connection points at the template level (not just within child elements). These connection points can be used in template checks and must be bound when implementing the template.

**Example:**

```hielements
template microservice:
    element api:
        scope module<rust>
    
    element container:
        scope dockerfile
    
    ## Template-level unbounded connection point
    connection_point port: integer
    
    ## Template checks can reference the template-level connection point
    check files.exists(container.dockerfile, 'Dockerfile')
    check rust.function_exists(api.module, 'start_server')

## When implementing, bind the template-level connection point
element orders_service implements microservice:
    scope api_mod<rust> binds microservice.api.module = rust.module_selector('orders::api')
    scope dockerfile binds microservice.container.dockerfile = files.file_selector('orders.dockerfile')
    
    ## Bind the template-level port
    connection_point service_port: integer binds microservice.port = rust.const_selector('ORDERS_PORT')

element payments_service implements microservice:
    scope api_mod<rust> binds microservice.api.module = rust.module_selector('payments::api')
    scope dockerfile binds microservice.container.dockerfile = files.file_selector('payments.dockerfile')
    
    ## Different service, different port
    connection_point service_port: integer binds microservice.port = rust.const_selector('PAYMENTS_PORT')
```

**Benefits:**
- **Parameterization**: Templates can be parameterized without hardcoding values
- **Reusability**: Same template structure with different concrete values
- **Type Safety**: Connection points have type annotations ensuring correctness
- **Clarity**: Makes template dependencies and parameters explicit

### 8.11 Hierarchical Checks

Hierarchical checks allow parent elements to prescribe requirements that must be satisfied by at least one of their descendants (children, grandchildren, etc.). This is useful for expressing architectural constraints that span multiple levels of the hierarchy.

#### Hierarchical Requirements

The unified syntax uses `requires`, `allows`, and `forbids` keywords with an optional `descendant` modifier:

```hielements
template dockerized:
    ## At least one descendant must have a docker scope (V2 syntax with unbounded scope)
    requires descendant scope dockerfile
    
    ## At least one descendant must satisfy this check
    requires descendant check docker.has_healthcheck(dockerfile)

template observable:
    ## At least one descendant must have a metrics element with implements
    requires descendant element metrics_service implements metrics_provider

template production_ready:
    ## At least one descendant must implement the dockerized template
    requires descendant implements dockerized
    
    ## At least one descendant must implement the observable template
    requires descendant implements observable
```

#### Satisfying Hierarchical Requirements

When an element implements a template with hierarchical requirements, at least one of its descendants must satisfy each requirement:

```hielements
element my_app implements dockerized:
    ## Frontend - not dockerized
    element frontend:
        scope src = files.folder_selector('frontend')
    
    ## Backend - satisfies the hierarchical requirement!
    element backend:
        scope src = files.folder_selector('backend')
        scope dockerfile binds dockerized.dockerfile = docker.file_selector('Dockerfile')  ## Matches!
        check docker.has_healthcheck(dockerfile)               ## Matches!

## Template implementation requirements
element ecommerce_platform implements production_ready:
    ## Frontend - neither dockerized nor observable
    element frontend:
        scope src = files.folder_selector('frontend')
    
    ## Orders - implements dockerized (satisfies first requirement)
    element orders implements dockerized:
        scope dockerfile binds dockerized.dockerfile = files.file_selector('orders/Dockerfile')
    
    ## Monitoring - implements observable (satisfies second requirement)
    element monitoring implements observable:
        scope metrics_mod<rust> binds observable.metrics.module = rust.module_selector('monitoring::metrics')
        connection_point prometheus: MetricsHandler binds observable.metrics.prometheus = rust.function_selector(metrics_mod, 'handler')
```

#### Hierarchical Requirement Kinds

| Kind | Syntax | Description |
|------|--------|-------------|
| Scope | `requires descendant scope name[<lang>]` | A descendant must have a matching scope |
| Check | `requires descendant check expr` | A descendant must satisfy this check |
| Element | `requires descendant element name [implements template]` | A descendant must have an element with this structure |
| Template Implementation | `requires descendant implements template_name` | A descendant must implement the specified template |
| Connection Point | `requires descendant connection_point name: Type` | A descendant must have a connection point |

#### Immediate vs Descendant Requirements

The `descendant` modifier determines whether the requirement applies to any descendant in the hierarchy or only to immediate children:

```hielements
template microservice:
    ## Immediate child requirement (no descendant modifier)
    requires element api implements api_handler
    
    ## Any descendant requirement (with descendant modifier)
    requires descendant element metrics implements observable
```

### 8.12 Connection Boundaries

Connection boundaries allow specifying constraints on architectural dependencies (imports/dependencies) between elements. These boundaries are inherited by all descendants. **Note**: "Connections" refer to logical/architectural dependencies like module imports, not network connections.

**Important**: Connection boundaries (`allows connection`, `forbids connection`, `requires connection`) are **only allowed in templates**, not in regular elements. Elements can inherit these constraints by implementing templates that define them.

#### Allowing Connections

Use `allows connection to` in templates to whitelist specific connection targets:

```hielements
template frontend_zone:
    ## Code in this zone may only import from api_gateway
    allows connection to api_gateway.public_api

element my_frontend implements frontend_zone:
    element web_app:
        scope src = files.folder_selector('frontend/web')
        ## Any imports from this scope are checked against the boundary
    
    element mobile_api:
        scope src = files.folder_selector('frontend/mobile')
```

#### Forbidding Connections

Use `forbids connection to` in templates to blacklist specific connection targets:

```hielements
template secure_zone:
    ## Code in this zone cannot import from external modules
    forbids connection to external.*
    forbids connection to public_network.*

element internal_service implements secure_zone:
    scope src = files.folder_selector('internal')
    ## This element and all its children inherit the constraint
```

#### Requiring Connections

Use `requires connection to` in templates to mandate that code MUST have a dependency:

```hielements
template service_mesh_zone:
    ## All services in this mesh must import from logging module
    requires connection to logging.*

element service_mesh implements service_mesh_zone:
    element user_service:
        scope src = files.folder_selector('services/user')
        # This service must import from logging.* to satisfy the constraint
```

#### Wildcard Patterns

Connection patterns support wildcards with `.*` to match any sub-path:

```hielements
## In templates:
forbids connection to database.*       ## Matches database.connection, database.pool, etc.
forbids connection to external.*       ## Matches anything under external
allows connection to api.public.*      ## Matches api.public.users, api.public.orders, etc.
```

**Note**: Wildcard interpretation is library-specific. For example, a Python library might expand `logging.*` to all modules in the logging package.

#### Combined Boundaries

Multiple boundaries can be combined - allows create a whitelist, forbids create a blacklist, requires create mandatory dependencies:

```hielements
template secure_service:
    allows connection to api.endpoint
    allows connection to logging.output
    forbids connection to database.*
    forbids connection to external.network
    requires connection to audit.*

element my_secure_service implements secure_service:
    ## Children inherit ALL of these boundaries
    element child_service:
        scope src = files.folder_selector('child')
```

#### Hierarchical Dependency Composition

When A is allowed to connect to B, and B is allowed to connect to C:
- A→B→C is **allowed** (each hop respects its own boundary)
- This enables construction of complex layered architectures

#### Boundary Semantics

- `allows connection` boundaries create a **whitelist** - only listed targets are permitted
- `forbids connection` boundaries create a **blacklist** - listed targets are prohibited
- `requires connection` boundaries create a **mandate** - dependencies MUST exist
- **Connection boundaries are only allowed in templates** - elements inherit them via `implements`
- Boundaries are **inherited** by all descendants within the parent/child hierarchy
- Multiple boundaries are **combined** (allows AND forbids AND requires apply)
- Wildcards (`.*`) match **any path suffix** (library-specific interpretation)
- Actual verification is **language-specific** - libraries check imports/dependencies

### 8.12 Language Constraints

Templates can constrain which programming languages elements may use through `requires`, `allows`, and `forbids` with the `language` keyword:

```hielements
template python_only:
    requires language python
    forbids language rust

template multilingual:
    allows language python
    allows language rust
    allows language java
```

When an element implements a template with language constraints:
- `requires language X` - The element must have at least one scope with language X
- `allows language X` - Only languages X are permitted (whitelist)
- `forbids language X` - Language X is prohibited (blacklist)

---

## 9. Language Declarations

Language declarations define supported languages and their connection verification checks.

### 9.1 Simple Language Declaration

A simple language declaration just registers a language name:

```hielements
language python
language rust
language java
```

### 9.2 Language with Connection Checks

Languages can define connection checks that verify connections between scopes:

```hielements
language python:
    connection_check can_import(source: scope[], target: scope[]):
        python.imports_allowed(source, target)
    
    connection_check no_circular(scopes: scope[]):
        python.no_circular_imports(scopes)

language rust:
    connection_check depends_on(source: scope[], target: scope[]):
        rust.dependency_exists(source, target)
```

### 9.3 Connection Check Semantics

Connection checks:
- Accept `scope[]` parameters representing arrays of scopes
- Return `True` (connection valid) or `False` (connection invalid)
- Are automatically applied recursively along the parent-children hierarchy
- Are language-specific - only applied to scopes with matching language annotation

### 9.4 Connection Verification Process

When verifying element connections:
1. For each language used by an element, gather all scopes of that language
2. For each child element, gather its scopes of the same language
3. Apply all `connection_check` functions defined for that language
4. Recursively verify all descendants

**Example:**

```hielements
language python:
    connection_check can_import(source: scope[], target: scope[]):
        python.imports_allowed(source, target)

element system:
    element frontend:
        scope src : python = python.module_selector('frontend')
    
    element backend:
        scope src : python = python.module_selector('backend')
        
        element api:
            scope src : python = python.module_selector('backend.api')
```

The `can_import` check will verify that:
- `frontend` can import from `backend`
- `backend.api` can import from its parent `backend`
- And so on through the hierarchy

---

## 10. Imports and Modules

Imports bring libraries and other Hielements specifications into scope.

### 8.1 Library Imports

```hielements
# Import entire library
import python
import docker
import files

# Import with alias
import kubernetes as k8s

# Selective import
from python import module_selector, function_exists
```

### 8.2 File Imports

Import other Hielements files:

```hielements
# Import another spec file
import './modules/backend.hie'

# Import with alias
import './shared/common.hie' as common
```

### 8.3 Import Resolution

Import paths are resolved:
1. **Bare imports** (`import python`) - Look up in library registry
2. **Relative paths** (`import './foo.hie'`) - Relative to current file
3. **Absolute paths** (`import '/path/to/foo.hie'`) - Absolute filesystem path

### 8.4 Built-in Libraries

The following libraries are built-in:

| Library | Description |
|---------|-------------|
| `files` | File and folder operations |
| `rust` | Rust code analysis |

### 8.5 External Libraries (Plugins)

Hielements supports user-defined libraries through external processes. External libraries are configured in a `hielements.toml` file in your workspace root.

#### Configuration

Create a `hielements.toml` file:

```toml
[libraries]
mylibrary = { executable = "path/to/my-plugin", args = [] }
python_checks = { executable = "python3", args = ["scripts/python_checks.py"] }
```

Then use in your .hie files:

```hielements
import mylibrary

element mycomponent:
    scope src = mylibrary.custom_selector('src')
    check mylibrary.custom_check(src)
```

#### Protocol

External libraries communicate via JSON-RPC 2.0 over stdin/stdout. See the [External Library Plugin Guide](external_libraries.md) for details on implementing custom plugins.

---

## 10. Expressions

Expressions compute values for scopes, connection points, and check arguments.

### 9.1 Literal Expressions

```hielements
"string literal"
'another string'
42
3.14
true
false
['list', 'of', 'items']
```

### 9.2 Identifier References

```hielements
my_scope                    # Reference a scope
parent.child.connection_pt  # Qualified reference
```

### 9.3 Function Calls

```hielements
python.module_selector('orders')
docker.exposes_port(dockerfile, 8080)
files.glob_selector('**/*.py')
```

### 9.4 Member Access

```hielements
element.connection_point
library.function
parent.child.scope
```

### 9.5 List Expressions

```hielements
check docker.exposes_ports(dockerfile, [80, 443, 8080])
```

---

## 11. Built-in Libraries

### 11.1 `files` Library

The `files` library provides selectors and checks for files and folders.

#### Selectors

| Function | Description |
|----------|-------------|
| `files.file_selector(path)` | Select a specific file |
| `files.folder_selector(path)` | Select a folder |
| `files.glob_selector(pattern)` | Select files matching glob pattern |

#### Checks

| Function | Description |
|----------|-------------|
| `files.exists(scope, filename)` | File exists in scope |
| `files.contains(scope, filename)` | Scope contains file |
| `files.no_files_matching(scope, pattern)` | No files match pattern |
| `files.max_size(file, bytes)` | File size limit |
| `files.matches_pattern(file, pattern)` | File matches pattern |

#### Examples

```hielements
element source_code:
    scope src = files.folder_selector('src/')
    scope tests = files.folder_selector('tests/')
    scope all_py = files.glob_selector('**/*.py')
    
    check files.exists(src, '__init__.py')
    check files.no_files_matching(src, '*.pyc')
    check files.no_files_matching(src, '__pycache__')
```

### 11.2 `python` Library

The `python` library provides analysis for Python code.

#### Selectors

| Function | Description |
|----------|-------------|
| `python.module_selector(name)` | Select Python module by import name |
| `python.package_selector(name)` | Select Python package |
| `python.function_selector(module, name)` | Select specific function |
| `python.class_selector(module, name)` | Select specific class |

#### Connection Point Functions

| Function | Description |
|----------|-------------|
| `python.public_functions(module)` | All public functions |
| `python.exported_classes(module)` | All exported classes |
| `python.get_main_module(module)` | The `__main__` entry |

#### Checks

| Function | Description |
|----------|-------------|
| `python.function_exists(module, name)` | Function exists |
| `python.class_exists(module, name)` | Class exists |
| `python.imports(module_a, module_b)` | A imports B |
| `python.no_circular_imports(module)` | No circular dependencies |
| `python.has_docstring(item)` | Has documentation |
| `python.type_annotated(function)` | Has type annotations |

#### Examples

```hielements
element api_module:
    scope module = python.module_selector('myapp.api')
    scope handlers = python.package_selector('myapp.api.handlers')
    
    connection_point public_api = python.public_functions(module)
    
    check python.function_exists(module, 'create_app')
    check python.no_circular_imports(module)
    check python.has_docstring(public_api)
```

### 11.3 `docker` Library

The `docker` library provides analysis for Dockerfiles.

#### Selectors

| Function | Description |
|----------|-------------|
| `docker.file_selector(path)` | Select a Dockerfile |
| `docker.compose_selector(path)` | Select docker-compose file |
| `docker.stage_selector(file, name)` | Select build stage |

#### Connection Point Functions

| Function | Description |
|----------|-------------|
| `docker.exposed_ports(dockerfile)` | All exposed ports |
| `docker.volumes(dockerfile)` | All volumes |
| `docker.entry_point(dockerfile)` | Container entry point |

#### Checks

| Function | Description |
|----------|-------------|
| `docker.exposes_port(dockerfile, port)` | Port is exposed |
| `docker.base_image(dockerfile, image)` | Uses specific base image |
| `docker.no_root_user(dockerfile)` | Doesn't run as root |
| `docker.stage_exists(dockerfile, name)` | Build stage exists |
| `docker.entry_point(dockerfile, module)` | Entry point matches |
| `docker.has_healthcheck(dockerfile)` | Has HEALTHCHECK instruction |

#### Examples

```hielements
element containerized_service:
    scope dockerfile = docker.file_selector('Dockerfile')
    scope compose = docker.compose_selector('docker-compose.yml')
    
    connection_point ports = docker.exposed_ports(dockerfile)
    
    check docker.exposes_port(dockerfile, 8080)
    check docker.base_image(dockerfile, 'python:3.11-slim')
    check docker.no_root_user(dockerfile)
    check docker.has_healthcheck(dockerfile)
```

---

## 12. Comments

### 12.1 Single-line Comments

```hielements
# This is a comment
element my_service:  # Inline comment
    scope src = files.folder_selector('src/')  # Another comment
```

### 12.2 Multi-line Comments

```hielements
###
This is a multi-line comment.
It can span multiple lines.
###
element my_service:
    scope src = files.folder_selector('src/')
```

### 12.3 Documentation Comments

Documentation comments (doc comments) provide descriptions for elements:

```hielements
## Orders Service
## Handles all order-related operations including creation,
## modification, and fulfillment.
element orders_service:
    scope module = python.module_selector('orders')
```

---

## 14. Complete Grammar (V2)

The following is the complete EBNF grammar for Hielements V2:

```ebnf
(* Program structure *)
program            ::= import_statement* language_declaration* (template_declaration | element_declaration)+

(* Imports *)
import_statement   ::= 'import' import_path ('as' identifier)?
                     | 'from' import_path 'import' identifier_list
import_path        ::= string_literal | identifier ('.' identifier)*
identifier_list    ::= identifier (',' identifier)*

(* Language Declarations *)
language_declaration ::= 'language' identifier NEWLINE
                       | 'language' identifier ':' NEWLINE INDENT connection_check+ DEDENT
connection_check     ::= 'connection_check' identifier '(' parameter_list ')' ':' NEWLINE INDENT expression DEDENT
parameter_list       ::= parameter (',' parameter)*
parameter            ::= identifier ':' 'scope' '[' ']'

(* Templates - with unbounded scopes *)
template_declaration ::= doc_comment? 'template' identifier ':' NEWLINE INDENT template_body DEDENT
template_body        ::= template_item+
template_item        ::= scope_declaration_template
                       | connection_point_declaration_template
                       | check_declaration
                       | element_declaration
                       | component_requirement

(* Scope in templates - can be unbounded (no '=' expression) *)
scope_declaration_template ::= 'scope' identifier language_annotation? NEWLINE
                             | 'scope' identifier language_annotation? '=' expression NEWLINE

(* Connection point in templates - can be unbounded (no '=' expression) *)
connection_point_declaration_template ::= 'connection_point' identifier ':' type_name NEWLINE
                                        | 'connection_point' identifier ':' type_name '=' expression NEWLINE

(* Elements *)
(* Note: Elements do NOT support component_requirement - requires/allows/forbids are only in templates *)
element_declaration ::= doc_comment? 'element' identifier template_implementation? ':' NEWLINE INDENT element_body DEDENT
template_implementation ::= 'implements' identifier (',' identifier)*
element_body        ::= element_item+
element_item        ::= scope_declaration
                      | connection_point_declaration
                      | check_declaration
                      | element_declaration

(* Component Requirements - unified syntax - ONLY allowed in templates *)
component_requirement ::= ('requires' | 'allows' | 'forbids') ['descendant'] component_spec
component_spec        ::= scope_declaration_template
                        | check_declaration
                        | element_spec
                        | connection_spec
                        | connection_point_spec
                        | language_spec

element_spec          ::= 'element' identifier [':' type_name] ['implements' identifier] [':' NEWLINE INDENT element_body DEDENT]
connection_spec       ::= 'connection' ['to'] connection_pattern NEWLINE
connection_point_spec ::= 'connection_point' identifier ':' type_name ['=' expression] NEWLINE
connection_pattern    ::= identifier ('.' identifier)* ('.' '*')?
language_spec         ::= 'language' identifier NEWLINE

(* Declarations - V2 syntax with angular brackets and binds *)
language_annotation  ::= '<' identifier '>'
binds_clause         ::= 'binds' qualified_identifier

scope_declaration           ::= 'scope' identifier language_annotation? binds_clause? '=' expression NEWLINE
connection_point_declaration ::= 'connection_point' identifier ':' type_name binds_clause? '=' expression NEWLINE
check_declaration           ::= 'check' function_call NEWLINE

(* Qualified identifiers for binds references *)
qualified_identifier ::= identifier ('.' identifier)+

(* Type annotations *)
type_name                   ::= identifier  (* Basic types: string, integer, float, boolean; or custom types *)

(* Expressions *)
expression         ::= function_call
                     | member_access
                     | identifier
                     | literal
member_access      ::= expression '.' identifier
function_call      ::= member_access '(' argument_list? ')'
argument_list      ::= expression (',' expression)*

(* Literals *)
literal            ::= string_literal
                     | number_literal
                     | boolean_literal
                     | list_literal
string_literal     ::= '"' character* '"' | "'" character* "'"
number_literal     ::= digit+ ('.' digit+)?
boolean_literal    ::= 'true' | 'false'
list_literal       ::= '[' (expression (',' expression)*)? ']'

(* Comments *)
comment            ::= '#' character* NEWLINE
doc_comment        ::= '##' character* NEWLINE
multiline_comment  ::= '###' character* '###'

(* Lexical elements *)
identifier         ::= letter (letter | digit | '_')*
letter             ::= 'a'..'z' | 'A'..'Z' | '_'
digit              ::= '0'..'9'
character          ::= <any unicode character except newline>
NEWLINE            ::= '\n' | '\r\n'
INDENT             ::= <increase in indentation level>
DEDENT             ::= <decrease in indentation level>
```

---

## 15. Examples

### 15.1 Simple Service (V2)

```hielements
import files
import rust

## Order Management Service
## Handles order creation, updates, and fulfillment.
element orders_service:
    # Source code scope with V2 language annotation
    scope rust_module<rust> = rust.module_selector('orders')
    scope tests = files.folder_selector('tests/orders')
    
    # Container scope
    scope dockerfile = files.file_selector('orders.dockerfile')
    
    # Connection points with type annotations
    connection_point api: HttpHandler = rust.public_functions(rust_module)
    connection_point main: Function = rust.function_selector(rust_module, 'main')
    
    # Architectural rules
    check rust.function_exists(rust_module, 'create_order')
    check rust.function_exists(rust_module, 'get_order')
    check rust.no_circular_imports(rust_module)
    
    check files.exists(dockerfile, 'Dockerfile')
```

### 15.2 Template with Unbounded Scopes (V2)

```hielements
import files
import rust

## Observable Template
## Defines a component that exposes metrics
template observable:
    element metrics:
        allows language rust
        
        # Unbounded scope - will be bound by implementing element
        scope module<rust>
        connection_point prometheus: MetricsHandler
        
        check files.exists(module, 'Cargo.toml')

## Metrics Service implementing the observable template
element metrics_service implements observable:
    # Bind the scope to the template's unbounded scope
    scope metrics_mod<rust> binds observable.metrics.module = rust.module_selector('metrics::api')
    
    # Bind the connection point
    connection_point handler: MetricsHandler binds observable.metrics.prometheus = rust.function_selector(metrics_mod, 'handler')
```

### 15.3 Microservices Architecture (V2)

```hielements
import files
import rust

## Microservice Template
template microservice:
    element api:
        scope module<rust>
        connection_point endpoint: HttpHandler
    
    element container:
        scope dockerfile
    
    check files.exists(container.dockerfile, 'Dockerfile')

## E-Commerce Platform
## Main platform containing all microservices.
element ecommerce_platform:
    
    ## Orders Service
    element orders_service implements microservice:
        scope api_mod<rust> binds microservice.api.module = rust.module_selector('services::orders')
        scope dockerfile binds microservice.container.dockerfile = files.file_selector('services/orders/Dockerfile')
        connection_point api: HttpHandler binds microservice.api.endpoint = rust.public_functions(api_mod)
    
    ## Inventory Service
    element inventory_service implements microservice:
        scope api_mod<rust> binds microservice.api.module = rust.module_selector('services::inventory')
        scope dockerfile binds microservice.container.dockerfile = files.file_selector('services/inventory/Dockerfile')
        connection_point api: HttpHandler binds microservice.api.endpoint = rust.public_functions(api_mod)
    
    ## Payments Service
    element payments_service implements microservice:
        scope api_mod<rust> binds microservice.api.module = rust.module_selector('services::payments')
        scope dockerfile binds microservice.container.dockerfile = files.file_selector('services/payments/Dockerfile')
        connection_point api: HttpHandler binds microservice.api.endpoint = rust.public_functions(api_mod)
    
    # Cross-service rules
    check rust.can_import(orders_service.api_mod, inventory_service.api)
    check rust.can_import(orders_service.api_mod, payments_service.api)
    check rust.no_dependency(payments_service.api_mod, orders_service.api_mod)
```

### 15.4 Hexagonal Architecture

```hielements
import files
import rust

## Application with Hexagonal Architecture
element hexagonal_app:
    
    ## Core Domain
    ## Contains business logic, no external dependencies.
    element domain:
        scope module<rust> = rust.package_selector('myapp::domain')
        
        connection_point entities: Entities = rust.struct_selector(module, '*Entity')
        connection_point services: Services = rust.struct_selector(module, '*Service')
        
        # Domain must not import adapters
        check rust.no_dependency(module, adapters.module)
    
    ## Application Layer
    ## Use cases and application services.
    element application:
        scope module<rust> = rust.package_selector('myapp::application')
        
        connection_point use_cases: UseCases = rust.public_functions(module)
        
        # Application can only depend on domain
        check rust.imports_only(module, [domain.module])
    
    ## Adapters Layer
    ## External integrations (DB, API, etc.)
    element adapters:
        scope module<rust> = rust.package_selector('myapp::adapters')
        
        element database_adapter:
            scope module<rust> = rust.module_selector('myapp::adapters::database')
            connection_point repositories: Repositories = rust.struct_selector(module, '*Repository')
        
        element api_adapter:
            scope module<rust> = rust.module_selector('myapp::adapters::api')
            connection_point routes: Routes = rust.function_selector(module, 'setup_routes')
        
        # Adapters depend on application and domain
        check rust.imports(module, application.module)
        check rust.imports(module, domain.module)
```

### 15.5 Infrastructure Validation

```hielements
import files

## Infrastructure as Code
element infrastructure:
    
    ## Docker Configuration
    element docker_config:
        scope compose = files.file_selector('docker-compose.yml')
        scope dockerfile_app = files.file_selector('Dockerfile')
        scope dockerfile_worker = files.file_selector('worker.dockerfile')
        
        # All containers should have health checks
        check files.exists(dockerfile_app, 'HEALTHCHECK')
        check files.exists(dockerfile_worker, 'HEALTHCHECK')
    
    ## Configuration Files
    element config:
        scope env_example = files.file_selector('.env.example')
        scope config_dir = files.folder_selector('config/')
        
        # Required configuration files
        check files.exists(config_dir, 'production.yaml')
        check files.exists(config_dir, 'development.yaml')
        check files.exists(config_dir, 'testing.yaml')
        
        # No secrets in config files
        check files.no_files_matching(config_dir, '*.secret')
        check files.no_files_matching(config_dir, '*password*')
```

### 15.6 Testing Requirements

```hielements
import files
import rust

## Testing Standards
element testing_standards:
    scope src = files.folder_selector('src/')
    scope tests = files.folder_selector('tests/')
    
    # Mirror structure: tests should mirror src
    element test_coverage:
        scope unit_tests = files.folder_selector('tests/unit/')
        scope integration_tests = files.folder_selector('tests/integration/')
        
        # Required test files
        check files.exists(unit_tests, 'mod.rs')
        check files.exists(integration_tests, 'mod.rs')
    
    # Each module should have tests
    element orders_tests:
        scope module<rust> = rust.module_selector('orders')
        scope tests<rust> = rust.module_selector('tests::test_orders')
        
        check rust.module_exists(tests)
        check rust.function_exists(tests, 'test_create_order')
```

### 15.7 Element Templates (V2)

```hielements
import files
import rust

## Compiler Template with unbounded scopes
## Defines the structure of a compiler with lexer and parser components.
template compiler:
    ## Lexer - tokenizes source code
    element lexer:
        scope module<rust>
        connection_point tokens: TokenStream
    
    ## Parser - produces abstract syntax tree
    element parser:
        scope module<rust>
        connection_point ast: AbstractSyntaxTree
    
    ## Verify lexer output is compatible with parser input
    check compiler.lexer.tokens.compatible_with(compiler.parser.input)

## Rust Compiler Implementation
## Implements the compiler template for Rust code.
element rust_compiler implements compiler:
    # Bind lexer with V2 binds syntax
    scope lexer_mod<rust> binds compiler.lexer.module = rust.module_selector('rustcompiler::lexer')
    connection_point tokens: TokenStream binds compiler.lexer.tokens = rust.function_selector(lexer_mod, 'tokenize')
    
    # Bind parser
    scope parser_mod<rust> binds compiler.parser.module = rust.module_selector('rustcompiler::parser')
    connection_point ast: AbstractSyntaxTree binds compiler.parser.ast = rust.function_selector(parser_mod, 'parse')
    
    # Add compiler-specific elements
    element optimizer:
        scope module<rust> = rust.module_selector('rustcompiler::optimizer')
        check rust.function_exists(module, 'optimize_ast')

## Microservice Template with unbounded scopes
template microservice:
    element api:
        scope module<rust>
        connection_point rest_endpoint: HttpHandler
    
    element database:
        scope module<rust>
        connection_point connection: DbConnection
    
    element container:
        scope dockerfile
        connection_point ports: integer
    
    check microservice.container.exposes_port(8080)
    check microservice.api.connects_to(microservice.database)

## Observable Template
template observable:
    element metrics:
        scope module<rust>
        connection_point prometheus_endpoint: MetricsHandler

## Resilient Template
template resilient:
    element circuit_breaker:
        scope module<rust>
        connection_point breaker_config: BreakerConfig

## Production Service with Multiple Templates (V2 syntax)
element production_service implements microservice, observable, resilient:
    # Microservice bindings with V2 binds syntax
    scope api_mod<rust> binds microservice.api.module = rust.module_selector('service::api')
    scope db_mod<rust> binds microservice.database.module = rust.module_selector('service::db')
    scope dockerfile binds microservice.container.dockerfile = files.file_selector('service.dockerfile')
    
    connection_point api: HttpHandler binds microservice.api.rest_endpoint = rust.public_functions(api_mod)
    connection_point db: DbConnection binds microservice.database.connection = rust.struct_selector(db_mod, 'DbConnection')
    connection_point ports: integer binds microservice.container.ports = rust.const_selector(api_mod, 'PORT')
    
    # Observable bindings
    scope metrics_mod<rust> binds observable.metrics.module = rust.module_selector('service::metrics')
    connection_point prometheus: MetricsHandler binds observable.metrics.prometheus_endpoint = rust.function_selector(metrics_mod, 'metrics_handler')
    
    # Resilient bindings
    scope resilience_mod<rust> binds resilient.circuit_breaker.module = rust.module_selector('service::resilience')
    connection_point breaker: BreakerConfig binds resilient.circuit_breaker.breaker_config = rust.struct_selector(resilience_mod, 'CircuitBreakerConfig')
```

---

## Appendix A: Error Messages

Common error messages and their meanings:

| Error Code | Message | Meaning |
|------------|---------|---------|
| E001 | Undefined element '{name}' | Referenced element doesn't exist |
| E002 | Undefined scope '{name}' | Referenced scope doesn't exist |
| E003 | Undefined connection point '{name}' | Referenced connection point doesn't exist |
| E004 | Library '{name}' not found | Import references unknown library |
| E005 | Duplicate element '{name}' | Element name already defined in scope |
| E006 | Check failed: {message} | Architectural rule violation |
| E007 | Scope resolution failed | Selector couldn't find target |
| E008 | Invalid argument type | Wrong type passed to function |
| E009 | Syntax error | Invalid Hielements syntax |
| E010 | Cyclic element reference | Elements reference each other cyclically |

---

## Appendix B: CLI Reference

### Basic Commands

```bash
# Validate specification syntax (no execution)
hielements check architecture.hie

# Run all checks
hielements run architecture.hie

# Dry run (show what would be checked)
hielements run --dry-run architecture.hie

# Output formats
hielements check --format json architecture.hie
hielements check --format sarif architecture.hie
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (all checks passed) |
| 1 | Check failures (architectural violations) |
| 2 | Errors (syntax errors, missing files, etc.) |

---

## Appendix C: Best Practices

### Naming Conventions

- Use `snake_case` for identifiers
- Use descriptive names that reflect the logical component
- Prefix private/internal elements with `_`

### Organization

- Keep one logical system per file
- Use nested elements for hierarchical structure
- Group related checks together

### Documentation

- Add doc comments (`##`) to all top-level elements
- Document connection points that are used by other elements
- Keep comments up-to-date with code changes

### Performance

- Use specific selectors over broad glob patterns
- Leverage caching in CI/CD pipelines
- Split large specifications into multiple files

---

## Appendix D: Migration Guide from V1

This section helps migrate existing Hielements V1 code to V2 syntax.

### D.1 Overview of Changes

| Feature | V1 Syntax | V2 Syntax |
|---------|-----------|-----------|
| Language annotation | `scope name : lang = expr` | `scope name<lang> = expr` |
| Template scopes | `scope name = expr` | `scope name<lang>` (unbounded) |
| Binding scopes | `template.element.scope = expr` | `scope name<lang> binds template.element.scope = expr` |
| Connection points | `connection_point name: Type = expr` | `connection_point name: Type binds path = expr` (for bindings) |

### D.2 Language Annotation Changes

**V1 (Deprecated):**
```hielements
element my_service:
    scope src : python = python.module_selector('my_service')
    scope backend : rust = rust.module_selector('backend')
```

**V2 (Current):**
```hielements
element my_service:
    scope src<python> = python.module_selector('my_service')
    scope backend<rust> = rust.module_selector('backend')
```

**Migration**: Replace `: language` with `<language>` after the scope name.

### D.3 Template Unbounded Scopes

**V1 (Deprecated):**
```hielements
template compiler:
    element lexer:
        scope module = rust.module_selector('lexer')  # Bound in template
        connection_point tokens: TokenStream = rust.function_selector(module, 'tokenize')
```

**V2 (Current):**
```hielements
template compiler:
    element lexer:
        scope module<rust>  # Unbounded - no '=' expression
        connection_point tokens: TokenStream
```

**Migration**: Remove the `= expression` part from template scopes. They become placeholders.

### D.4 Element Bindings with `binds`

**V1 (Deprecated):**
```hielements
element my_compiler implements compiler:
    compiler.lexer.scope = rust.module_selector('mycompiler::lexer')
    compiler.lexer.tokens = rust.function_selector(compiler.lexer.scope, 'tokenize')
```

**V2 (Current):**
```hielements
element my_compiler implements compiler:
    scope lexer_mod<rust> binds compiler.lexer.module = rust.module_selector('mycompiler::lexer')
    connection_point tokens: TokenStream binds compiler.lexer.tokens = rust.function_selector(lexer_mod, 'tokenize')
```

**Migration**:
1. Change `template.element.scope = expr` to `scope name<lang> binds template.element.scope = expr`
2. Change `template.element.connection_point = expr` to `connection_point name: Type binds template.element.connection_point = expr`

### D.5 Descriptive-Only Mode

V2 supports using the language without templates or bindings. If you're not using prescriptive features, you can write V2 code that looks similar to V1:

```hielements
# V2 descriptive-only (no templates, no binds)
element my_service:
    scope src<rust> = rust.module_selector('my_service')
    connection_point api: HttpHandler = rust.public_functions(src)
    check rust.function_exists(src, 'main')
```

### D.6 Complete Migration Example

**V1 Code:**
```hielements
import python
import docker

template microservice:
    element api:
        scope module = python.module_selector('api')
        connection_point endpoint = python.public_functions(module)
    
    element container:
        scope dockerfile = docker.file_selector('Dockerfile')
    
    check docker.exposes_port(dockerfile, 8080)

element orders implements microservice:
    microservice.api.scope = python.module_selector('orders.api')
    microservice.api.endpoint = python.public_functions(microservice.api.scope)
    microservice.container.dockerfile = docker.file_selector('orders.dockerfile')
```

**V2 Code:**
```hielements
import files
import rust

template microservice:
    element api:
        scope module<rust>  # Unbounded
        connection_point endpoint: HttpHandler
    
    element container:
        scope dockerfile
    
    check files.exists(container.dockerfile, 'Dockerfile')

element orders implements microservice:
    scope api_mod<rust> binds microservice.api.module = rust.module_selector('orders::api')
    connection_point endpoint: HttpHandler binds microservice.api.endpoint = rust.public_functions(api_mod)
    scope dockerfile binds microservice.container.dockerfile = files.file_selector('orders.dockerfile')
```

### D.7 Migration Checklist

- [ ] Update all language annotations from `: lang` to `<lang>`
- [ ] Remove `= expression` from template scopes (make them unbounded)
- [ ] Remove `= expression` from template connection points (make them unbounded)
- [ ] Add `binds template.path` clause to element scopes that bind to templates
- [ ] Add `binds template.path` clause to element connection points that bind to templates
- [ ] Update any references to use the new local scope names
- [ ] Test that all checks pass with the new syntax

### D.8 Backward Compatibility Note

**Hielements V2 is NOT backward compatible with V1.** The V1 syntax is deprecated and no longer supported. All existing V1 code must be migrated to V2 syntax.

The key philosophy changes:
- **Templates are prescriptive** - they define structure without implementation
- **Elements are descriptive** - they bind to actual code
- **`binds` makes connections explicit** - clearer separation of concerns
- **Angular brackets for language** - more consistent with type syntax conventions
