# Hielements

> ⚠️ **Alpha / Experimental** — This project is in early development. The language syntax, APIs, and tooling may change significantly. Use at your own risk and expect breaking changes.

**A language to describe and enforce software architecture.**

Hielements helps you define, document, and enforce the logical structure of your software systems. Unlike traditional architecture documentation that becomes stale, Hielements specifications are formally checked against your actual code—ensuring your architecture stays aligned with reality.

---

## Why Hielements?

Modern software systems are complex. As codebases grow, their actual structure diverges from the original design. Architecture diagrams become outdated, and the "mental model" of how components interact exists only in developers' heads (if at all).

**Hielements solves this by:**

- 📐 **Formalizing architecture** in a declarative language
- ✅ **Enforcing architectural rules** via static checks
- 🔗 **Making relationships explicit** between components
- 🏗️ **Supporting hierarchical composition** for complex systems
- 🎨 **Enabling reusable templates** for consistent architectural patterns
- 🌐 **Working across languages** (Python, Docker, Terraform, and more)
- 🤝 **Enabling human-AI collaboration** through structured specifications

---

## Quick Example

Imagine a microservice with Python code and a Dockerfile. You want to ensure:
1. The service exposes port 8080
2. The Docker container uses the correct Python module as the entry point

**With Hielements:**

```hielements
element orders_service:
    # Define scopes
    scope python_module = python.module_selector('orders')
    scope dockerfile = docker.file_selector('orders_service.dockerfile')
    
    # Define connection points
    connection_point main = python.get_main_module(python_module)
    
    # Enforce rules
    check docker.exposes_port(dockerfile, 8080)
    check docker.entry_point(dockerfile, main)
```

Run `hielements check` and Hielements will verify your architecture against the actual code. If someone changes the Dockerfile or renames the module, the checks will fail—keeping your architecture in sync.

### Reusable Templates

Define architectural patterns once and reuse them across your system:

```hielements
# Define a template for microservices
template microservice:
    element api:
        connection_point rest_endpoint
    element database:
        connection_point connection
    check microservice.api.exposes_rest()

# Implement the template multiple times
element orders_service implements microservice:
    microservice.api.scope = python.module_selector('orders.api')
    microservice.database.scope = postgres.database_selector('orders_db')

element payments_service implements microservice:
    microservice.api.scope = python.module_selector('payments.api')
    microservice.database.scope = postgres.database_selector('payments_db')
```

Templates ensure consistency across similar components and make architectural patterns explicit.

---

## Key Features

### 🧩 Reusable Element Templates

Define architectural patterns once and reuse them across your codebase:

```hielements
template compiler:
    element lexer:
        connection_point tokens: TokenStream
    element parser:
        connection_point ast: AbstractSyntaxTree
    check compiler.lexer.tokens.compatible_with(compiler.parser.input)

element python_compiler implements compiler:
    compiler.lexer.scope = rust.module_selector('pycompiler::lexer')
    compiler.parser.scope = rust.module_selector('pycompiler::parser')
```

Templates ensure consistency across similar components, making architectural patterns explicit and enforceable.

### 🔒 Type-Safe Connection Points

Explicit type annotations enable correct integration across multiple libraries and languages:

```hielements
element api_service:
    # Basic types
    connection_point port: integer = docker.exposed_port(dockerfile)
    connection_point api_url: string = config.get_url()
    connection_point ssl_enabled: boolean = config.get_flag('ssl')
    
    # Custom types for domain-specific interfaces
    connection_point handler: HttpHandler = python.public_functions(module)
    connection_point db_conn: DatabaseConnection = python.class_selector(module, 'Database')
```

Types are optional, maintaining backward compatibility while providing additional safety and documentation.

### 🎯 Cross-Technology Elements

Define elements that span multiple languages and artifacts:

```hielements
element full_stack_feature:
    scope frontend = typescript.module_selector('components/OrderForm')
    scope backend = python.module_selector('api/orders')
    scope database = sql.migration_selector('create_orders_table')
    scope container = docker.file_selector('orders.dockerfile')
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
    connection_point rest_api = python.public_functions(api_module)
    connection_point database = postgres.connection(config)
```

### ✅ Enforceable Rules

Rules are actually checked, not just documented:

```hielements
check docker.exposes_port(dockerfile, 8080)
check python.no_circular_dependencies(module_a, module_b)
check files.matches_pattern(config, '*.yaml')
```

### 🧩 Extensible via Libraries

Built-in support for Python, Docker, and files/folders. Add support for any language by creating Hielements libraries.

Learn how to create custom libraries in the [Usage Guide](USAGE.md#creating-custom-libraries) or [External Libraries Guide](doc/external_libraries.md).

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

### 1. Define Elements

Elements represent logical components with:
- **Scope**: What code/artifacts belong to this element
- **Rules**: Constraints the element must satisfy
- **Connection Points**: APIs, interfaces, or dependencies the element exposes
- **Children**: Sub-elements for hierarchical composition

### 2. Write Rules

Rules use library functions to check properties:

```hielements
check python.function_exists(module, "handle_payment")
check docker.base_image(dockerfile, "python:3.11-slim")
check files.no_files_matching(src, "*.tmp")
```

### 3. Run Checks

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
    scope src = files.folder_selector('src/')
    
    check files.contains(src, 'main.py')
```

Run the check:

```bash
hielements check architecture.hie
```

### Learn More

📖 **[Usage Guide](USAGE.md)** - Comprehensive guide covering:
- Writing Hielements specifications
- Using element templates for reusable patterns
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
- 🔌 [External Libraries Guide](doc/external_libraries.md) - Creating custom libraries
- 🏗️ [Technical Architecture](doc/technical_architecture.md) - Implementation details
- 🔍 [Related Work](doc/related_work.md) - Comparison with similar tools
- 📝 [Summary](doc/summary.md) - High-level overview

---

## Project Status

🚧 **Hielements is in early development.** We are actively building the core interpreter, standard libraries, and tooling.

### Roadmap

- [x] Language design and specification
- [x] Core interpreter implementation (Rust)
- [x] Standard libraries (Python, Docker, files)
- [x] Element templates for reusable patterns
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

**Hielements makes this possible.**

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
