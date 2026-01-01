# Hielements Language Reference

This document provides a complete reference for the Hielements language syntax and semantics. Hielements is a declarative language for describing and enforcing software architecture.

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
9. [Imports and Modules](#9-imports-and-modules)
10. [Expressions](#10-expressions)
11. [Built-in Libraries](#11-built-in-libraries)
12. [Comments](#12-comments)
13. [Complete Grammar](#13-complete-grammar)
14. [Examples](#14-examples)

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
| `:` | Block start |
| `(` `)` | Function call, grouping |
| `[` `]` | List literals |

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

### 4.1 Syntax

```
scope_declaration ::= 'scope' identifier '=' selector_expression
```

### 4.2 Scope Selectors

Selectors are library functions that identify parts of your codebase:

```hielements
# File and folder selectors
scope src_folder = files.folder_selector('src/')
scope config_file = files.file_selector('config.yaml')
scope all_python = files.glob_selector('**/*.py')

# Python selectors
scope orders = python.module_selector('orders')
scope handlers = python.package_selector('api.handlers')
scope main_func = python.function_selector('orders', 'main')

# Docker selectors
scope dockerfile = docker.file_selector('Dockerfile')
scope compose = docker.compose_selector('docker-compose.yml')
```

### 4.3 Multiple Scopes

An element can have multiple scopes, representing different aspects:

```hielements
element full_stack_feature:
    scope frontend = typescript.module_selector('components/OrderForm')
    scope backend = python.module_selector('api/orders')
    scope database = sql.migration_selector('migrations/001_orders.sql')
    scope container = docker.file_selector('orders.dockerfile')
```

### 4.4 Scope Composition

Scopes can be combined using set operations (library-dependent):

```hielements
element api_layer:
    scope handlers = python.package_selector('api.handlers')
    scope models = python.package_selector('api.models')
    
    # Combine scopes (hypothetical syntax)
    scope all_api = scopes.union(handlers, models)
```

### 4.5 Scope Semantics

- Scopes are **lazy** - they don't scan the filesystem until needed
- Scopes can **overlap** between elements (a file can belong to multiple elements)
- Scope resolution may **fail** if the target doesn't exist (configurable: error vs warning)
- Scopes provide **source locations** for error reporting

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

### 8.1 Template Declaration

Templates are declared using the `template` keyword and define a structure that elements can implement:

```hielements
template compiler:
    ## Lexer component
    element lexer:
        connection_point tokens
    
    ## Parser component
    element parser:
        connection_point ast
    
    ## Ensure lexer output is compatible with parser input
    check compiler.lexer.tokens.compatible_with(compiler.parser.input)
```

### 8.2 Implementing Templates

Elements implement templates using the `implements` keyword, then provide concrete bindings:

```hielements
element python_compiler implements compiler:
    # Provide concrete scopes for template elements
    compiler.lexer.scope = python.module_selector('compiler.lexer')
    compiler.parser.scope = python.module_selector('compiler.parser')
    
    # Provide concrete connection points
    compiler.lexer.tokens = python.get_tokens(compiler.lexer.scope)
    compiler.parser.ast = python.get_ast(compiler.parser.scope)
    
    # Can add additional elements and checks
    element optimizer:
        scope module = python.module_selector('optimizer')
```

### 8.3 Absolute References

Template properties are referenced using absolute paths prefixed with the template name (e.g., `compiler.lexer`). This prevents name clashes when implementing multiple templates:

```hielements
template microservice:
    element api:
        connection_point rest_endpoint

template observable:
    element api:
        connection_point metrics_endpoint

# No name clash - each 'api' is explicitly qualified
element my_service implements microservice, observable:
    microservice.api.scope = python.module_selector('service.api')
    observable.api.scope = python.module_selector('service.metrics')
    
    # Reference both in checks
    check microservice.api.rest_endpoint.port != observable.api.metrics_endpoint.port
```

### 8.4 Multiple Template Implementation

An element can implement multiple templates:

```hielements
template resilient:
    element circuit_breaker

template secured:
    element authentication

element production_service implements microservice, resilient, secured:
    # Microservice bindings
    microservice.api.scope = python.module_selector('api')
    
    # Resilient bindings
    resilient.circuit_breaker.scope = python.module_selector('resilience')
    
    # Secured bindings
    secured.authentication.scope = python.module_selector('auth')
```

### 8.5 Template Requirements

When implementing a template, all required elements must have their scopes and connection points bound:

```hielements
template web_service:
    element frontend:
        connection_point static_files
    
    element backend:
        connection_point api

# Valid - all required bindings provided
element complete_service implements web_service:
    web_service.frontend.scope = files.folder_selector('frontend/')
    web_service.backend.scope = python.module_selector('backend')
    web_service.frontend.static_files = files.glob_selector('frontend/dist/*')
    web_service.backend.api = python.public_functions(web_service.backend.scope)

# Invalid - missing bindings (would produce validation error)
element incomplete_service implements web_service:
    web_service.frontend.scope = files.folder_selector('frontend/')
    # ERROR: web_service.backend bindings missing
```

### 8.6 Template Checks

Checks defined in templates are automatically included when the template is implemented. The checks use absolute references and are evaluated with the concrete bindings:

```hielements
template microservice:
    element api
    element database
    element container
    
    # Template checks
    check microservice.container.exposes_port(8080)
    check microservice.api.connects_to(microservice.database)

element orders_service implements microservice:
    microservice.api.scope = python.module_selector('orders.api')
    microservice.database.scope = postgres.database_selector('orders_db')
    microservice.container.scope = docker.file_selector('orders.dockerfile')
    
    # The template checks are automatically evaluated:
    # - check orders_service.microservice.container.exposes_port(8080)
    # - check orders_service.microservice.api.connects_to(orders_service.microservice.database)
```

### 8.7 Library-Defined Templates

Templates can be defined in external libraries and imported for use:

```hielements
import architecture_patterns

element my_service implements architecture_patterns.hexagonal:
    # Bind the hexagonal architecture template elements
    hexagonal.domain.scope = python.package_selector('myapp.domain')
    hexagonal.application.scope = python.package_selector('myapp.application')
    hexagonal.adapters.scope = python.package_selector('myapp.adapters')
```

External libraries can provide templates via the library protocol. See the [External Library Plugin Guide](external_libraries.md) for details.

### 8.8 Template Semantics

- Templates define **structure** but not **implementation**
- Elements implementing templates **must provide** all required bindings
- Template checks are **inherited** by implementing elements
- Absolute references **prevent name clashes** between multiple templates
- Templates **cannot be nested** (a template cannot implement another template)
- Template names must be **unique** within their scope

### 8.9 Template-Level Connection Points

Templates can declare connection points at the template level (not just within child elements). These connection points can be used in template checks and must be bound when implementing the template.

**Example:**

```hielements
template microservice:
    element api:
        scope module = rust.module_selector('api')
    
    element container:
        scope dockerfile = files.file_selector('Dockerfile')
    
    ## Template-level connection point
    connection_point port: integer = rust.const_selector('PORT')
    
    ## Template checks can reference the template-level connection point
    check files.exists(container.dockerfile, 'Dockerfile')
    check rust.function_exists(api.module, 'start_server')

## When implementing, bind the template-level connection point
element orders_service implements microservice:
    microservice.api.module = rust.module_selector('orders::api')
    microservice.container.dockerfile = files.file_selector('orders.dockerfile')
    
    ## Bind the template-level port to a specific value
    microservice.port = rust.const_selector('ORDERS_PORT')

element payments_service implements microservice:
    microservice.api.module = rust.module_selector('payments::api')
    microservice.container.dockerfile = files.file_selector('payments.dockerfile')
    
    ## Different service, different port
    microservice.port = rust.const_selector('PAYMENTS_PORT')
```

**Benefits:**
- **Parameterization**: Templates can be parameterized without hardcoding values
- **Reusability**: Same template structure with different concrete values
- **Type Safety**: Connection points have type annotations ensuring correctness
- **Clarity**: Makes template dependencies and parameters explicit

### 8.10 Hierarchical Checks

Hierarchical checks allow parent elements to prescribe requirements that must be satisfied by at least one of their descendants (children, grandchildren, etc.). This is useful for expressing architectural constraints that span multiple levels of the hierarchy.

#### Hierarchical Requirements

The new unified syntax uses `requires`, `allows`, and `forbids` keywords with an optional `descendant` modifier:

```hielements
template dockerized:
    ## At least one descendant must have a docker scope (new syntax)
    requires descendant scope dockerfile = docker.file_selector('Dockerfile')
    
    ## At least one descendant must satisfy this check (new syntax)
    requires descendant check docker.has_healthcheck(dockerfile)

template observable:
    ## At least one descendant must have a metrics element with implements (new syntax)
    requires descendant element metrics_service implements metrics_provider

template production_ready:
    ## At least one descendant must implement the dockerized template (new syntax)
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
        scope dockerfile = docker.file_selector('Dockerfile')  ## Matches!
        check docker.has_healthcheck(dockerfile)               ## Matches!

## Template implementation requirements
element ecommerce_platform implements production_ready:
    ## Frontend - neither dockerized nor observable
    element frontend:
        scope src = files.folder_selector('frontend')
    
    ## Orders - implements dockerized (satisfies first requirement)
    element orders implements dockerized:
        dockerized.container.dockerfile = files.file_selector('orders/Dockerfile')
    
    ## Monitoring - implements observable (satisfies second requirement)
    element monitoring implements observable:
        observable.metrics.module = rust.module_selector('monitoring::metrics')
        observable.metrics.prometheus = rust.function_selector(observable.metrics.module, 'handler')
```

#### Hierarchical Requirement Kinds

| Kind | Syntax | Description |
|------|--------|-------------|
| Scope | `requires descendant scope name = expr` | A descendant must have a matching scope |
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

### 8.11 Connection Boundaries

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

---

## 9. Imports and Modules

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

## 13. Complete Grammar

The following is the complete EBNF grammar for Hielements:

```ebnf
(* Program structure *)
program            ::= import_statement* (template_declaration | element_declaration)+

(* Imports *)
import_statement   ::= 'import' import_path ('as' identifier)?
                     | 'from' import_path 'import' identifier_list
import_path        ::= string_literal | identifier ('.' identifier)*
identifier_list    ::= identifier (',' identifier)*

(* Templates *)
template_declaration ::= doc_comment? 'template' identifier ':' NEWLINE INDENT template_body DEDENT
template_body        ::= template_item+
template_item        ::= scope_declaration
                       | connection_point_declaration
                       | check_declaration
                       | element_declaration
                       | component_requirement

(* Elements *)
(* Note: Elements do NOT support component_requirement - requires/allows/forbids are only in templates *)
element_declaration ::= doc_comment? 'element' identifier template_implementation? ':' NEWLINE INDENT element_body DEDENT
template_implementation ::= 'implements' identifier (',' identifier)*
element_body        ::= element_item+
element_item        ::= scope_declaration
                      | connection_point_declaration
                      | check_declaration
                      | element_declaration
                      | template_binding

(* Template Bindings *)
template_binding    ::= qualified_identifier '=' expression NEWLINE
qualified_identifier ::= identifier ('.' identifier)+

(* Component Requirements - unified syntax - ONLY allowed in templates *)
component_requirement ::= ('requires' | 'allows' | 'forbids') ['descendant'] component_spec
component_spec        ::= scope_declaration
                        | check_declaration
                        | element_spec
                        | connection_spec
                        | connection_point_spec

element_spec          ::= 'element' identifier [':' type_name] ['implements' identifier] [':' NEWLINE INDENT element_body DEDENT]
connection_spec       ::= 'connection' ['to'] connection_pattern NEWLINE
connection_point_spec ::= 'connection_point' identifier ':' type_name ['=' expression] NEWLINE
connection_pattern    ::= identifier ('.' identifier)* ('.' '*')?

(* Declarations *)
scope_declaration           ::= 'scope' identifier '=' expression NEWLINE
connection_point_declaration ::= 'connection_point' identifier ':' identifier '=' expression NEWLINE
check_declaration           ::= 'check' function_call NEWLINE

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

## 14. Examples

### 14.1 Simple Service

```hielements
import python
import docker
import files

## Order Management Service
## Handles order creation, updates, and fulfillment.
element orders_service:
    # Source code scope
    scope python_module = python.module_selector('orders')
    scope tests = files.folder_selector('tests/orders')
    
    # Container scope
    scope dockerfile = docker.file_selector('orders.dockerfile')
    
    # Connection points
    connection_point api = python.public_functions(python_module)
    connection_point main = python.function_selector(python_module, 'main')
    
    # Architectural rules
    check python.function_exists(python_module, 'create_order')
    check python.function_exists(python_module, 'get_order')
    check python.no_circular_imports(python_module)
    
    check docker.exposes_port(dockerfile, 8080)
    check docker.entry_point(dockerfile, main)
    check docker.base_image(dockerfile, 'python:3.11-slim')
```

### 14.2 Microservices Architecture

```hielements
import python
import docker

## E-Commerce Platform
## Main platform containing all microservices.
element ecommerce_platform:
    
    ## Orders Service
    element orders_service:
        scope module = python.module_selector('services.orders')
        scope dockerfile = docker.file_selector('services/orders/Dockerfile')
        
        connection_point api = python.public_functions(module)
        connection_point events = python.class_selector(module, 'OrderEvents')
        
        check docker.exposes_port(dockerfile, 8001)
    
    ## Inventory Service
    element inventory_service:
        scope module = python.module_selector('services.inventory')
        scope dockerfile = docker.file_selector('services/inventory/Dockerfile')
        
        connection_point api = python.public_functions(module)
        
        check docker.exposes_port(dockerfile, 8002)
    
    ## Payments Service
    element payments_service:
        scope module = python.module_selector('services.payments')
        scope dockerfile = docker.file_selector('services/payments/Dockerfile')
        
        connection_point api = python.public_functions(module)
        
        check docker.exposes_port(dockerfile, 8003)
    
    # Cross-service rules
    # Orders can use inventory and payments
    check python.can_import(orders_service.module, inventory_service.api)
    check python.can_import(orders_service.module, payments_service.api)
    
    # Payments should not depend on orders (prevent circular dependency)
    check python.no_dependency(payments_service.module, orders_service.module)
```

### 14.3 Hexagonal Architecture

```hielements
import python
import files

## Application with Hexagonal Architecture
element hexagonal_app:
    
    ## Core Domain
    ## Contains business logic, no external dependencies.
    element domain:
        scope module = python.package_selector('myapp.domain')
        
        connection_point entities = python.class_selector(module, '*Entity')
        connection_point services = python.class_selector(module, '*Service')
        
        # Domain must not import adapters
        check python.no_dependency(module, adapters.module)
    
    ## Application Layer
    ## Use cases and application services.
    element application:
        scope module = python.package_selector('myapp.application')
        
        connection_point use_cases = python.public_functions(module)
        
        # Application can only depend on domain
        check python.imports_only(module, [domain.module])
    
    ## Adapters Layer
    ## External integrations (DB, API, etc.)
    element adapters:
        scope module = python.package_selector('myapp.adapters')
        
        element database_adapter:
            scope module = python.module_selector('myapp.adapters.database')
            connection_point repositories = python.class_selector(module, '*Repository')
        
        element api_adapter:
            scope module = python.module_selector('myapp.adapters.api')
            connection_point routes = python.function_selector(module, 'setup_routes')
        
        # Adapters depend on application and domain
        check python.imports(module, application.module)
        check python.imports(module, domain.module)
```

### 14.4 Infrastructure Validation

```hielements
import docker
import files

## Infrastructure as Code
element infrastructure:
    
    ## Docker Configuration
    element docker_config:
        scope compose = docker.compose_selector('docker-compose.yml')
        scope dockerfile_app = docker.file_selector('Dockerfile')
        scope dockerfile_worker = docker.file_selector('worker.dockerfile')
        
        # All containers should have health checks
        check docker.has_healthcheck(dockerfile_app)
        check docker.has_healthcheck(dockerfile_worker)
        
        # Security: no containers run as root
        check docker.no_root_user(dockerfile_app)
        check docker.no_root_user(dockerfile_worker)
    
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

### 14.5 Testing Requirements

```hielements
import python
import files

## Testing Standards
element testing_standards:
    scope src = files.folder_selector('src/')
    scope tests = files.folder_selector('tests/')
    
    # Mirror structure: tests should mirror src
    element test_coverage:
        scope unit_tests = files.folder_selector('tests/unit/')
        scope integration_tests = files.folder_selector('tests/integration/')
        
        # Required test files
        check files.exists(unit_tests, '__init__.py')
        check files.exists(integration_tests, '__init__.py')
    
    # Each module should have tests
    element orders_tests:
        scope module = python.module_selector('orders')
        scope tests = python.module_selector('tests.test_orders')
        
        check python.module_exists(tests)
        check python.function_exists(tests, 'test_create_order')
```

### 14.6 Element Templates

```hielements
import python
import docker

## Compiler Template
## Defines the structure of a compiler with lexer and parser components.
template compiler:
    ## Lexer - tokenizes source code
    element lexer:
        connection_point tokens
    
    ## Parser - produces abstract syntax tree
    element parser:
        connection_point ast
    
    ## Verify lexer output is compatible with parser input
    check compiler.lexer.tokens.compatible_with(compiler.parser.input)

## Python Compiler Implementation
## Implements the compiler template for Python code.
element python_compiler implements compiler:
    # Bind lexer element to concrete Python module
    compiler.lexer.scope = python.module_selector('pycompiler.lexer')
    compiler.lexer.tokens = python.get_tokens(compiler.lexer.scope)
    
    # Bind parser element to concrete Python module
    compiler.parser.scope = python.module_selector('pycompiler.parser')
    compiler.parser.ast = python.get_ast(compiler.parser.scope)
    
    # Add compiler-specific elements
    element optimizer:
        scope module = python.module_selector('pycompiler.optimizer')
        check python.function_exists(module, 'optimize_ast')

## Microservice Template
## Defines a standard microservice with API, database, and container.
template microservice:
    element api:
        connection_point rest_endpoint
    
    element database:
        connection_point connection
    
    element container:
        connection_point ports
    
    check microservice.container.exposes_port(8080)
    check microservice.api.connects_to(microservice.database)

## Orders Service
## A microservice for managing orders.
element orders_service implements microservice:
    microservice.api.scope = python.module_selector('orders.api')
    microservice.api.rest_endpoint = python.public_functions(microservice.api.scope)
    
    microservice.database.scope = python.module_selector('orders.db')
    microservice.database.connection = python.get_db_connection(microservice.database.scope)
    
    microservice.container.scope = docker.file_selector('orders.dockerfile')
    microservice.container.ports = docker.exposed_ports(microservice.container.scope)

## Multiple Template Implementation
template observable:
    element metrics:
        connection_point prometheus_endpoint

template resilient:
    element circuit_breaker:
        connection_point breaker_config

## Production Service with Multiple Templates
element production_service implements microservice, observable, resilient:
    # Microservice bindings
    microservice.api.scope = python.module_selector('service.api')
    microservice.database.scope = python.module_selector('service.db')
    microservice.container.scope = docker.file_selector('service.dockerfile')
    
    # Observable bindings
    observable.metrics.scope = python.module_selector('service.metrics')
    observable.metrics.prometheus_endpoint = python.function_selector(
        observable.metrics.scope, 
        'metrics_handler'
    )
    
    # Resilient bindings
    resilient.circuit_breaker.scope = python.module_selector('service.resilience')
    resilient.circuit_breaker.breaker_config = python.class_selector(
        resilient.circuit_breaker.scope,
        'CircuitBreakerConfig'
    )
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
