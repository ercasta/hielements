# Hielements

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

---

## Key Features

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
# https://github.com/yourorg/hielements/releases
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

### IDE Support

Install the Hielements extension for VSCode:
- Syntax highlighting
- Real-time error checking
- Go to definition
- Auto-completion

---

## Documentation

- 📖 [Language Reference](doc/language_reference.md) - Complete syntax and semantics
- 🏗️ [Technical Architecture](doc/technical_architecture.md) - Implementation details
- 🔍 [Related Work](doc/related_work.md) - Comparison with similar tools
- 📝 [Summary](doc/summary.md) - High-level overview

---

## Project Status

🚧 **Hielements is in early development.** We are actively building the core interpreter, standard libraries, and tooling.

### Roadmap

- [x] Language design and specification
- [ ] Core interpreter implementation (Rust)
- [ ] Standard libraries (Python, Docker, files)
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
