# Hielements

> ⚠️ **Alpha / Experimental** — This project is in early development. The language syntax, APIs, and tooling may change significantly. Use at your own risk and expect breaking changes.

**A language to describe and enforce software architecture.**

Hielements helps you define, document, and enforce the logical structure of your software systems. Unlike traditional architecture documentation that becomes stale, Hielements specifications are formally checked against your actual code—ensuring your architecture stays aligned with reality.

> 📝 **Note**: This documentation describes Hielements V2, which introduces a clearer separation between **prescriptive** (patterns with rules) and **descriptive** (actual implementations) parts of the language. V2 is incompatible with V1.

---

## Why Hielements?

Modern software systems are complex. As codebases grow, their actual structure diverges from the original design. Architecture diagrams become outdated, and the "mental model" of how components interact exists only in developers' heads (if at all).

**Hielements solves this by:**

- 📐 **Formalizing architecture** in a declarative language
- ✅ **Enforcing architectural rules** via static checks
- 🔗 **Making relationships explicit** between components
- 🏗️ **Supporting hierarchical composition** for complex systems
- 🌲 **Providing hierarchical checks** that compose through element hierarchies
- 🎨 **Enabling reusable patterns** for consistent architectural constraints
- 📚 **Providing an executable pattern library** with auto-generated documentation
- 🌐 **Working across languages** (Python, Docker, Terraform, and more)
- 🤝 **Enabling human-AI collaboration** through structured specifications

---

## Prescriptive vs Descriptive

Hielements V2 separates two key concerns:

**🏗️ Prescriptive** — Define the rules and constraints
- **Patterns** (declared with `pattern`) establish architectural blueprints
- **Checks** enforce rules and requirements
- Keywords like `requires`, `forbids`, and `allows` control constraints
- Patterns declare what *should* be true

**📝 Descriptive** — Document what actually exists
- **Elements** describe concrete implementations
- **Scopes** bind to actual code and artifacts
- The `implements` keyword connects elements to patterns
- The `binds` keyword maps implementations to pattern declarations

You can use Hielements **descriptively only** (documenting structure without enforcement) or **prescriptively** (with patterns and checks for enforcement). Mix and match based on your needs.

---

## Quick Example

Imagine a microservice with Python code and a Dockerfile. You want to ensure:
1. The service exposes port 8080
2. The Docker container uses the correct Python module as the entry point

**With Hielements:**

```hielements
element orders_service:
    # Define scopes with language annotations (V2 syntax)
    scope python_module<python> = python.module_selector('orders')
    scope dockerfile<docker> = docker.file_selector('orders_service.dockerfile')
    
    # Define connection points with types
    ref main: PythonModule = python.get_main_module(python_module)
    
    # Enforce rules
    check docker.exposes_port(dockerfile, 8080)
    check docker.entry_point(dockerfile, main)
```

Run `hielements check` and Hielements will verify your architecture against the actual code. If someone changes the Dockerfile or renames the module, the checks will fail—keeping your architecture in sync.

### Reusable Patterns

Define architectural patterns once and reuse them across your system:

```hielements
# Define a pattern for microservices (prescriptive)
pattern microservice {
    element api {
        scope module<python>  # Unbounded scope in pattern
        ref rest_endpoint: RestEndpoint
    }
    element database {
        ref connection: DatabaseConnection
    }
    check microservice.api.exposes_rest()
}

# Implement the pattern multiple times (descriptive + prescriptive)
element orders_service implements microservice {
    # Bind pattern scopes to actual code using V2 syntax
    scope api_mod<python> binds microservice.api.module = python.module_selector('orders.api')
    ref endpoint: RestEndpoint binds microservice.api.rest_endpoint = python.public_functions(api_mod)
    ref db: DatabaseConnection binds microservice.database.connection = postgres.database_selector('orders_db')
}

element payments_service implements microservice {
    scope api_mod<python> binds microservice.api.module = python.module_selector('payments.api')
    ref endpoint: RestEndpoint binds microservice.api.rest_endpoint = python.public_functions(api_mod)
    ref db: DatabaseConnection binds microservice.database.connection = postgres.database_selector('payments_db')
}
```

Patterns ensure consistency across similar components and make architectural constraints explicit.

---

## Key Features

### 🧩 Reusable Patterns

Define architectural patterns once and reuse them across your codebase:

```hielements
pattern compiler {
    element lexer {
        scope module<rust>  # Unbounded scope (V2)
        ref tokens: TokenStream
    }
    element parser {
        scope module<rust>  # Unbounded scope (V2)
        ref ast: AbstractSyntaxTree
    }
    check compiler.lexer.tokens.compatible_with(compiler.parser.input)
}

element python_compiler implements compiler {
    # Bind pattern scopes using V2 binds keyword
    scope lexer_mod<rust> binds compiler.lexer.module = rust.module_selector('pycompiler::lexer')
    ref lexer_tokens: TokenStream binds compiler.lexer.tokens = rust.function_selector(lexer_mod, 'tokenize')
    
    scope parser_mod<rust> binds compiler.parser.module = rust.module_selector('pycompiler::parser')
    ref parser_ast: AbstractSyntaxTree binds compiler.parser.ast = rust.function_selector(parser_mod, 'parse')
}
```

Patterns ensure consistency across similar components, making architectural constraints explicit and enforceable.

> 📚 **See the [Pattern Catalog](doc/patterns_catalog.md)** for an extensive collection of common software engineering patterns with their Hielements implementations.

#### 🆕 Executable Pattern Library

Hielements includes a comprehensive **pattern library** (`patterns/` directory) with reusable architectural blueprints:
- All patterns are stored as executable `.hie` files, not just documentation
- Patterns can be imported and implemented in your projects
- **Automatic catalog generation** keeps documentation in sync with patterns
- Covers structural, behavioral, infrastructure, and cross-cutting concerns

This approach ensures patterns are living artifacts that can be validated, tested, and evolved alongside your code.

### 🔒 Type-Safe Connection Points

Explicit type annotations are **required** for all connection points, enabling correct integration across multiple libraries and languages. Below are examples of connection points typing added for better interfacing:

```hielements
element api_service:
    # Basic types (mandatory)
    ref port: integer = docker.exposed_port(dockerfile)
    ref api_url: string = config.get_url()
    ref ssl_enabled: boolean = config.get_flag('ssl')
    
    # Custom types for domain-specific interfaces
    ref handler: HttpHandler = python.public_functions(module)
    ref db_conn: DatabaseConnection = python.class_selector(module, 'Database')
```

Mandatory types provide safety and serve as inline documentation of interfaces.

### 🌲 Hierarchical Checks

Define requirements that must be satisfied somewhere in your element hierarchy, enabling flexible yet enforceable architectural constraints:

```hielements
pattern dockerized {
    ## At least one descendant must have a docker configuration
    requires descendant scope dockerfile<docker>
    requires descendant check docker.has_healthcheck(dockerfile)
}

element my_app implements dockerized {
    element frontend {
        scope src<files> = files.folder_selector('frontend')
        # Not dockerized - that's OK
    }
    
    element backend {
        scope dockerfile<docker> binds dockerized.dockerfile = docker.file_selector('Dockerfile.backend')
        check docker.has_healthcheck(dockerfile)
        # This satisfies the hierarchical requirement!
    }
}
```

Hierarchical checks also support **connection boundaries** to control architectural dependencies:

```hielements
pattern frontend_zone {
    ## Code in this zone may only import from API gateway
    allows connection to api_gateway.public_api
    forbids connection to database.*
}

element my_frontend implements frontend_zone {
    element web_app {
        scope src<javascript> = files.folder_selector('frontend/web')
        # Inherits connection boundaries - cannot access database
    }
}
```

Benefits:
- **Flexible enforcement**: Requirements can be satisfied by any descendant
- **Architectural boundaries**: Control dependencies between system layers
- **Composable constraints**: Boundaries inherit through element hierarchy

### 🎯 Cross-Technology Elements

Define elements that span multiple languages and artifacts:

```hielements
element full_stack_feature:
    scope frontend<typescript> = typescript.module_selector('components/OrderForm')
    scope backend<python> = python.module_selector('api/orders')
    scope database<sql> = sql.migration_selector('create_orders_table')
    scope container<docker> = docker.file_selector('orders.dockerfile')
```

### 🏗️ Hierarchical Composition

Build complex systems from smaller, well-defined elements:

```hielements
element payment_system:
    element payment_gateway
    element fraud_detection
    element transaction_log
    
    check payment_gateway.exposes_api(payment_gateway.api, fraud_detection)
```

### 🔗 Explicit Connection Points

Make inter-component relationships visible and verifiable:

```hielements
element api_server:
    ref rest_api = python.public_functions(api_module)
    ref database = postgres.connection(config)
```

### ✅ Enforceable Rules

Rules are actually checked, not just documented:

```hielements
check docker.exposes_port(dockerfile, 8080)
check python.no_circular_dependencies(module_a, module_b)
check files.matches_pattern(config, '*.yaml')
```

### 🧩 Extensible via Libraries

Built-in support for Python, Docker, and files/folders. Extend with custom libraries using:

- **External Process Plugins**: Write plugins in any language (Python, JS, Go, etc.) via JSON-RPC
- **WASM Plugins**: Sandboxed, near-native performance for security-critical use cases (infrastructure ready, runtime integration in progress)

The hybrid approach balances **flexibility** (external tools when needed) with **security** (WASM sandboxing for untrusted code).

Learn how to create and share custom libraries in the [Usage Guide](USAGE.md#creating-custom-libraries) or [External Libraries Guide](doc/external_libraries.md).

---

## Use Cases

### 🆕 Greenfield Development

Define your architecture upfront and use Hielements as **design guardrails**:
1. Describe system structure in Hielements
2. Write implementation code
3. Run checks to ensure alignment
4. Agents can use specifications to generate code

### 🏭 Brownfield/Legacy Systems

Reverse-engineer and enforce architecture in existing codebases:
1. Analyze code (manually or with agents) to create initial Hielements specs
2. Refine and formalize the architecture
3. Enforce rules to prevent degradation
4. Use specifications as the source of truth for refactoring

### 🔄 Continuous Architecture Compliance

Integrate Hielements checks into CI/CD:
```yaml
# .github/workflows/architecture.yml
- name: Check Architecture
  run: hielements check
```

Reject PRs that violate architectural rules.

---

## How It Works

### 1. Define Elements (Descriptive)

Elements represent logical components with:
- **Scope**: What code/artifacts belong to this element (with V2 language annotations like `<rust>`)
- **Connection Points**: APIs, interfaces, or dependencies the element exposes
- **Children**: Sub-elements for hierarchical composition

### 2. Define Patterns (Prescriptive - Optional)

Patterns establish architectural blueprints with:
- **Unbounded scopes**: Declared without implementation (`scope module<rust>`)
- **Rules**: Constraints that implementations must satisfy
- **Requirements**: Using `requires`, `forbids`, and `allows` keywords
- **Checks**: Verifiable properties

### 3. Bind Implementations to Patterns

Use the `implements` and `binds` keywords to connect:
```hielements
element my_service implements observable:
    scope metrics_mod<rust> binds observable.metrics.module = rust.module_selector('api')
```

### 4. Write Rules

Rules use library functions to check properties:

```hielements
check python.function_exists(module, "handle_payment")
check docker.base_image(dockerfile, "python:3.11-slim")
check files.no_files_matching(src, "*.tmp")
```

### 5. Run Checks

```bash
hielements check
```

Hielements evaluates all rules against your actual codebase and reports violations.

---

## Architecture

- **Interpreter**: Written in Rust for performance and reliability
- **Extensible**: Language support via pluggable libraries
- **Language Server Protocol**: Full IDE integration (VSCode, with more coming)
- **External Tools**: Libraries can invoke existing static analysis tools

```
┌────────────────────────────────────────────────────────┐
│                 Hielements Spec (.hie)                  │
└────────────────────────────────────────────────────────┘
                           │
                           ▼
                  ┌─────────────────┐
                  │   Interpreter   │
                  │     (Rust)      │
                  └─────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
  ┌──────────┐      ┌──────────┐      ┌──────────┐
  │  Python  │      │  Docker  │      │  Custom  │
  │ Library  │      │ Library  │      │ Library  │
  └──────────┘      └──────────┘      └──────────┘
        │                  │                  │
        └──────────────────┼──────────────────┘
                           ▼
                  ┌─────────────────┐
                  │  Your Codebase  │
                  └─────────────────┘
```

---

## Getting Started

### Installation

```bash
# Install via cargo (Rust package manager)
cargo install hielements

# Or download binary from releases
# https://github.com/ercasta/hielements/releases
```

### Your First Hielements Spec

Create a file `architecture.hie`:

```hielements
element my_service:
    scope src<files> = files.folder_selector('src/')
    
    check files.contains(src, 'main.py')
```

Run the check:

```bash
hielements check architecture.hie
```

### Learn More

📖 **[Usage Guide](USAGE.md)** - Comprehensive guide covering:
- Writing Hielements specifications
- Using patterns for reusable architectural blueprints
- Creating custom libraries
- Best practices and CI/CD integration

### IDE Support

Install the Hielements extension for VSCode:
- Syntax highlighting
- Real-time error checking
- Go to definition
- Auto-completion

---

## Documentation

- 📘 **[Usage Guide](USAGE.md)** - Complete guide to using Hielements
- 📖 [Language Reference](doc/language_reference.md) - Complete syntax and semantics
- 📚 **[Pattern Catalog](doc/patterns_catalog.md)** - Extensive collection of software engineering patterns
- 🔌 [External Libraries Guide](doc/external_libraries.md) - Creating custom libraries
- 📖 **[Library Catalog](doc/library_catalog.md)** - Auto-generated documentation for all built-in libraries
- 🏗️ [Technical Architecture](doc/technical_architecture.md) - Implementation details
- 🔍 [Related Work](doc/related_work.md) - Comparison with similar tools
- 📝 [Summary](doc/summary.md) - High-level overview

### Generating Documentation

#### Library Documentation

Use the `hielements doc` command to generate documentation for all available libraries (including custom ones):

```bash
# Generate markdown documentation
hielements doc --format markdown --output doc/library_catalog.md

# Generate JSON catalog for AI agents
hielements doc --format json --output doc/library_catalog.json

# Filter to specific libraries
hielements doc --library files,rust
```

#### Pattern Catalog

Hielements includes an extensive **pattern library** with reusable architectural patterns stored as executable `.hie` files. The pattern catalog is **automatically generated** from these pattern definitions:

```bash
# Generate the pattern catalog
python3 scripts/generate_pattern_catalog.py
```

The pattern library (`patterns/` directory) contains:
- **Structural patterns**: Layered architecture, hexagonal, microservices, clean architecture
- **Behavioral patterns**: Event-driven, pipeline, CQRS, saga
- **Infrastructure patterns**: Containerized services, sidecar, API gateway
- **Cross-cutting patterns**: Observability, resilience, security
- **Testing patterns**: Test pyramid, contract testing
- **Compiler patterns**: Compiler pipeline, visitor

These patterns are not just documentation—they're executable Hielements specifications that you can import and implement in your own projects. The automatic catalog generation ensures patterns stay synchronized with their implementations.

---

## Project Status

🚧 **Hielements is in early development.** We are actively building the core interpreter, standard libraries, and tooling.

### Roadmap

- [x] Language design and specification
- [x] Core interpreter implementation (Rust)
- [x] Standard libraries (Python, Docker, files)
- [x] Patterns (formerly "element templates") for reusable architectural constraints
- [ ] VSCode extension
- [ ] Language Server Protocol
- [ ] CI/CD integration templates
- [ ] Additional language libraries (JavaScript, Go, Terraform)

---

## Contributing

We welcome contributions! Whether you're interested in:
- Core interpreter development (Rust)
- Language library development (any language)
- Documentation and examples
- IDE extensions
- Testing and feedback

Check out our [Contributing Guide](CONTRIBUTING.md) (coming soon).

---

## Philosophy

**Architecture should be:**
- **Explicit**: Not hidden in code or developers' minds
- **Enforced**: Checked automatically, not just documented
- **Evolvable**: Easy to update as systems change
- **Multi-level**: From high-level system design to low-level module structure
- **Flexible**: Support both description (documenting what exists) and prescription (enforcing what should be)

**Hielements V2 makes this possible** through:
- **Descriptive mode**: Document your architecture without enforcement
- **Prescriptive mode**: Use patterns and checks to enforce architectural rules
- **Hybrid approach**: Mix both modes as needed for different parts of your system

---

## Examples

More examples can be found in the [`examples/`](examples/) directory:
- Microservices architecture
- Layered application (hexagonal architecture)
- Multi-language monorepo
- Infrastructure as Code validation

---

## FAQ

### Is this a replacement for my programming language?
No. Hielements complements your existing languages by adding a layer of architectural specification and enforcement.

### Does Hielements run at compile time or runtime?
Hielements checks are static analysis—they run before your code executes, typically in your CI/CD pipeline.

### What languages are supported?
Built-in support for Python, Docker, and file/folder structures. Additional languages can be added via libraries.

### Can I use Hielements with existing codebases?
Yes! Hielements works with both greenfield and brownfield projects.

### How is this different from linters?
Linters check code quality and style within a single file or module. Hielements checks architectural rules across your entire system, including relationships between components.

### What's the difference between prescriptive and descriptive modes?
**Descriptive mode** lets you document your architecture without enforcement—useful for understanding existing systems or when you need flexibility. **Prescriptive mode** uses patterns (declared with `pattern`), `requires`/`forbids`/`allows` keywords, and checks to enforce architectural rules. You can mix both modes: describe some parts of your system while prescribing rules for others.

---

## License

[MIT License](LICENSE)

---

## Community

- 💬 [Discussions](https://github.com/yourorg/hielements/discussions)
- 🐛 [Issue Tracker](https://github.com/yourorg/hielements/issues)
- 📧 Email: hielements@example.com

---

**Build software that stays true to its design. Start with Hielements.**

---

## Self-Describing Architecture

Hielements literally documents and checks itself — how cool is that?! The repository is driven by a living specification written in `hielements.hie`, and we continuously validate that spec during AI-assisted coding sessions (and in CI) so architectural drift gets caught early.

Peek at the live self-description: [hielements.hie](hielements.hie)

```hielements
# Live excerpt from hielements.hie
element hielements_repo:
    # sanity checks that run as part of validation
    check files.exists('README.md')
    check struct_exists('Interpreter')
```

Want to see it in action? Run:

```bash
hielements check hielements.hie
```

We use this feedback loop to keep the code, docs, and architecture in sync — and it makes AI-assisted development far more reliable and trustworthy!
