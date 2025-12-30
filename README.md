# Hielements

> ⚠️ **Alpha / Experimental** — This project is in early development. The language syntax, APIs, and tooling may change significantly. Use at your own risk and expect breaking changes.

**A language to describe and enforce software architecture.**

Hielements helps you define, document, and enforce the logical structure of your software systems. Unlike traditional architecture documentation that becomes stale, Hielements specifications are formal, verifiable, and kept in sync with the actual codebase....

---

## Why Hielements?

Modern software systems are complex. As codebases grow, their actual structure diverges from the original design. Architecture diagrams become outdated, and the "mental model" of how components interact becomes inconsistent among developers.

**Hielements solves this by:**

- 📐 **Formalizing architecture** in a declarative language
- ✅ **Enforcing architectural rules** via static checks
- 🔗 **Making relationships explicit** between components
- 🏗️ **Supporting hierarchical composition** for complex systems
- 🎨 **Enabling reusable templates** for consistent architectural patterns
- 🌐 **Working across languages** (Python, Docker, Terraform, and more)
- 🤝 **Enabling human-AI collaboration** through structured specifications....

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

Run `hielements check` and Hielements will verify your architecture against the actual code. If someone changes the Dockerfile or renames the module, the checks will fail—keeping your architecture intact.

---

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

Templates ensure consistency across similar components and make architectural patterns explicit....

---

### 🔒 Type-Safe Connection Points

Explicit type annotations are **required** for all connection points, enabling correct integration across multiple libraries and languages. Below are examples of connection points typing added for better interfacing:

```hielements
element api_service:
    # Basic types (mandatory)
    connection_point port: integer = docker.exposed_port(dockerfile)
    connection_point api_url: string = config.get_url()
    connection_point ssl_enabled: boolean = config.get_flag('ssl')

    # Custom types for domain-specific interfaces
    connection_point handler: HttpHandler = python.public_functions(module)
    connection_point db_conn: DatabaseConnection = python.class_selector(module, 'Database')
```

Mandatory types provide safety and serve as inline documentation of interfaces.

---