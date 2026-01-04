# Hielements Usage Guide

This guide walks you through using Hielements to describe, document, and enforce the architecture of your software systems.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Writing Your First Hielements Spec](#writing-your-first-hielements-spec)
3. [Using Patterns](#using-patterns)
4. [Creating Custom Libraries](#creating-custom-libraries)
5. [Best Practices](#best-practices)
6. [Integration with CI/CD](#integration-with-cicd)
7. [IDE Integration](#ide-integration)

---

## Getting Started

### Installation

Hielements is not yet published on cargo. Compile from source:

```bash
# Prerequisites: Install Rust toolchain (https://rustup.rs)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/ercasta/hielements.git
cd hielements
cargo build --release
cargo install --path crates/hielements-cli
```

Verify the installation:

```bash
hielements --version
```

### Initialize Your Project

Create initial configuration for your project:

```bash
hielements init my_project
```

This generates:
- `my_project.hie` - Initial specification with a root element
- `hielements.toml` - Configuration for custom libraries
- `USAGE_GUIDE.md` - Quick reference guide

### Your First Check

Create a file named `architecture.hie` in your project root:

```hielements
import files

element my_project:
    scope root = files.folder_selector('.')
    
    check files.exists(root, 'README.md')
    check files.exists(root, 'LICENSE')
```

Run the check:

```bash
hielements check architecture.hie
```

Hielements will verify that your project has both `README.md` and `LICENSE` files.

---

## Writing Your First Hielements Spec

### Basic Structure

A Hielements specification consists of:
- **Imports**: Libraries providing selectors and checks
- **Elements**: Logical components of your system
- **Scopes**: What code/artifacts belong to an element
- **Connection Points**: APIs or interfaces the element exposes
- **Checks**: Rules that must be satisfied

### Example: Web Service Architecture

```hielements
import files
import python

element web_service:
    ## Define scopes - what code belongs to this element
    scope api_module = python.module_selector('src/api')
    scope database_module = python.module_selector('src/database')
    scope config = files.file_selector('config.yaml')
    
    ## Define connection points - what this element exposes
    ref rest_api = python.public_functions(api_module)
    ref db_connection = python.class_selector(database_module, 'Database')
    
    ## Define checks - rules that must be satisfied
    check files.exists(config)
    check python.has_tests(api_module)
    check python.no_circular_imports(api_module)
    check python.function_exists(api_module, 'health_check')
```

### Hierarchical Elements

Build complex systems from smaller elements:

```hielements
import python
import files

element ecommerce_system:
    ## Order management service
    element orders_service:
        scope module = python.module_selector('services/orders')
        ref api = python.public_functions(module)
        
        check python.has_tests(module)
        check python.has_docstrings(module)
    
    ## Payment processing service
    element payments_service:
        scope module = python.module_selector('services/payments')
        ref api = python.public_functions(module)
        
        check python.has_tests(module)
        check python.function_exists(module, 'process_payment')
    
    ## Shared database schema
    element database:
        scope migrations = files.folder_selector('db/migrations')
        
        check files.contains(migrations, '001_create_orders.sql')
        check files.contains(migrations, '002_create_payments.sql')
    
    ## Cross-service checks
    check python.can_import(orders_service.module, payments_service.api)
```

### Working with Multiple Technologies

Hielements works across different languages and technologies:

```hielements
import python
import docker
import files

element containerized_service:
    ## Python application
    scope python_src = python.module_selector('app')
    ref main = python.get_main_module(python_src)
    
    ## Docker configuration
    scope dockerfile = docker.file_selector('Dockerfile')
    
    ## Configuration files
    scope config = files.file_selector('config.yaml')
    
    ## Cross-technology checks
    check docker.base_image(dockerfile, 'python:3.11-slim')
    check docker.exposes_port(dockerfile, 8080)
    check docker.entry_point(dockerfile, main)
    check python.has_tests(python_src)
```

---

## Using Patterns

Patterns are reusable architectural blueprints that can be instantiated with concrete implementations. Patterns (declared with the `template` keyword) help enforce consistent structure across similar components.

> 📚 **See the [Pattern Catalog](doc/patterns_catalog.md)** for an extensive collection of common software engineering patterns with their Hielements implementations.

### What Are Patterns?

Patterns define the **structure** and **requirements** that elements must satisfy. Think of them as architectural blueprints or interfaces that elements implement.

### Defining a Pattern

Use the `template` keyword to define reusable patterns:

```hielements
import rust

pattern compiler:
    ## Required: Lexer component
    element lexer:
        ref tokens
    
    ## Required: Parser component
    element parser:
        ref ast
    
    ## Structural check: lexer output compatible with parser input
    check compiler.lexer.tokens.compatible_with(compiler.parser.input)
```

### Implementing a Pattern

Use the `implements` keyword to create concrete implementations:

```hielements
element python_compiler implements compiler:
    ## Bind lexer to concrete Rust module
    compiler.lexer.scope = rust.module_selector('pycompiler::lexer')
    compiler.lexer.tokens = rust.function_selector(compiler.lexer.scope, 'tokenize')
    
    ## Bind parser to concrete Rust module
    compiler.parser.scope = rust.module_selector('pycompiler::parser')
    compiler.parser.ast = rust.function_selector(compiler.parser.scope, 'parse')
    
    ## Add implementation-specific elements
    element optimizer:
        scope module = rust.module_selector('pycompiler::optimizer')
        check rust.function_exists(module, 'optimize_ast')
```

### Multiple Pattern Implementation

Elements can implement multiple patterns:

```hielements
pattern microservice:
    element api:
        ref rest_endpoint
    element database:
        ref connection

pattern observable:
    element metrics:
        ref prometheus_endpoint
    element logging:
        ref log_output

## Implement both patterns
element orders_service implements microservice, observable:
    ## Microservice bindings
    microservice.api.scope = python.module_selector('orders.api')
    microservice.database.scope = postgres.database_selector('orders_db')
    
    ## Observable bindings
    observable.metrics.scope = python.module_selector('orders.metrics')
    observable.logging.scope = python.module_selector('orders.logging')
    
    ## Cross-pattern checks
    check microservice.api.exposes_rest()
    check observable.metrics.prometheus_endpoint.is_available()
```

### Benefits of Patterns

1. **Consistency**: Ensure similar components follow the same structure
2. **Reusability**: Define architectural patterns once, use everywhere
3. **Evolution**: Update the pattern to update all implementations
4. **Documentation**: Patterns serve as architectural documentation
5. **Validation**: Automatically verify implementations conform to patterns

### When to Use Patterns

Use patterns when:
- You have multiple components with similar structure (e.g., multiple microservices)
- You want to enforce architectural constraints (e.g., hexagonal architecture)
- You need consistent structure across teams or projects
- You're building a framework or platform with expected patterns

Don't use patterns when:
- Components are truly unique with no shared structure
- The pattern is used only once
- The abstraction would be more complex than the concrete code

---

## Creating Custom Libraries

Hielements is extensible via custom libraries. Libraries provide selectors (to identify code) and checks (to verify properties).

### When to Create a Custom Library

Create a custom library when you need to:
- Support a new programming language or technology
- Add domain-specific checks for your organization
- Integrate with existing static analysis tools
- Extend Hielements with custom functionality

### Library Types

1. **Built-in Libraries**: Rust libraries compiled into Hielements (files, rust)
2. **External Libraries**: Standalone programs communicating via JSON-RPC

For most use cases, external libraries are recommended as they:
- Can be written in any language (Python, JavaScript, Go, etc.)
- Don't require modifying Hielements source code
- Can leverage existing tools and libraries
- Run in isolated processes for security

### Creating an External Library

External libraries are programs that communicate with Hielements via JSON-RPC over stdin/stdout.

#### Quick Start

1. **Configure the library** in `hielements.toml`:

```toml
[libraries]
mylib = { executable = "python3", args = ["scripts/mylib.py"] }
```

2. **Implement the protocol** in your chosen language:

```python
#!/usr/bin/env python3
import json
import sys

def handle_call(function, args, workspace):
    """Handle selector function calls."""
    if function == "my_selector":
        # Your selector logic here
        return {
            "Scope": {
                "kind": {"Folder": "src/"},
                "paths": ["/path/to/file1.py", "/path/to/file2.py"],
                "resolved": True
            }
        }
    raise ValueError(f"Unknown function: {function}")

def handle_check(function, args, workspace):
    """Handle check function calls."""
    if function == "my_check":
        # Your check logic here
        return {"Pass": None}  # or {"Fail": "reason"} or {"Error": "message"}
    raise ValueError(f"Unknown check: {function}")

def main():
    for line in sys.stdin:
        request = json.loads(line.strip())
        method = request.get("method")
        params = request.get("params", {})
        
        if method == "library.call":
            result = handle_call(
                params.get("function"),
                params.get("args", []),
                params.get("workspace", ".")
            )
        elif method == "library.check":
            result = handle_check(
                params.get("function"),
                params.get("args", []),
                params.get("workspace", ".")
            )
        
        response = {"jsonrpc": "2.0", "result": result, "id": request.get("id")}
        print(json.dumps(response), flush=True)

if __name__ == "__main__":
    main()
```

3. **Use the library** in your Hielements specs:

```hielements
import mylib

element my_component:
    scope src = mylib.my_selector('src/')
    check mylib.my_check(src)
```

### Detailed Documentation

For complete documentation on creating custom libraries, including:
- Full protocol specification
- Value type serialization
- Error handling
- Best practices
- Troubleshooting

See [External Library Plugin Guide](doc/external_libraries.md).

### Library Development Workflow

1. **Design**: Define what selectors and checks your library will provide
2. **Implement**: Write the library following the JSON-RPC protocol
3. **Test**: Test manually with echo commands and unit tests
4. **Configure**: Add to `hielements.toml`
5. **Use**: Import and use in your `.hie` files
6. **Iterate**: Refine based on usage

### Sharing and Distributing Libraries

Once you've created a custom library, you can share it with others in several ways:

#### 1. Distribute as Source Code

The simplest approach for external process plugins written in interpreted languages:

```bash
# Share your Python plugin
my-hielements-library/
├── README.md              # Usage instructions
├── mylibrary.py          # Plugin implementation
├── requirements.txt      # Python dependencies (if any)
└── hielements.toml.example  # Example configuration
```

**Usage for consumers:**
```toml
# Add to their hielements.toml
[libraries]
mylibrary = { executable = "python3", args = ["path/to/mylibrary.py"] }
```

**Best for:**
- Quick prototyping and iteration
- Python, JavaScript, or other scripting languages
- Internal team sharing

#### 2. Package as Executable Binary

Compile your plugin to a native executable for easy distribution:

```bash
# For Go plugins
go build -o mylibrary-plugin ./cmd/plugin

# For Rust plugins
cargo build --release
cp target/release/mylibrary-plugin ./dist/

# For Python plugins using PyInstaller
pyinstaller --onefile mylibrary.py
```

**Distribution:**
- Provide platform-specific binaries (Linux, macOS, Windows)
- Users download and reference the executable path

```toml
[libraries]
mylibrary = { executable = "./bin/mylibrary-plugin" }
```

**Best for:**
- Production use
- Performance-critical plugins
- Compiled languages (Go, Rust, C++)
- External distribution

#### 3. Publish to Package Registries

Leverage existing package ecosystems:

**Python (PyPI):**
```bash
# Package structure
my-hielements-lib/
├── setup.py
├── mylibrary/
│   ├── __init__.py
│   └── plugin.py
└── README.md

# Publish
python setup.py sdist bdist_wheel
twine upload dist/*

# Users install
pip install mylibrary-hielements
```

**npm (for Node.js plugins):**
```bash
# Publish
npm publish

# Users install
npm install -g mylibrary-hielements
```

**Configuration:**
```toml
[libraries]
# Python package installed via pip
mylibrary = { executable = "python3", args = ["-m", "mylibrary.plugin"] }

# Node.js package installed via npm
jslibrary = { executable = "npx", args = ["jslibrary-hielements"] }
```

**Best for:**
- Public open-source libraries
- Community contributions
- Automatic dependency management

#### 4. Distribute as WASM Module (Future)

**Note**: WASM plugin infrastructure is ready, but runtime integration is in progress.

When fully available, WASM provides the best distribution experience:

```bash
# Build Rust plugin to WASM
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/mylibrary.wasm ./dist/
```

**Distribution:**
- Single `.wasm` file works on all platforms (Linux, macOS, Windows)
- No dependencies to install
- Strong security sandboxing

```toml
[libraries]
mylibrary = { path = "mylibrary.wasm" }
```

**Best for:**
- Cross-platform distribution (when available)
- Security-sensitive plugins
- Performance-critical operations
- Easy deployment (single file)

See [WASM Plugins Guide](doc/wasm_plugins.md) for current status and roadmap.

#### 5. Share via Git Repository

Host your library in a Git repository:

```bash
# Library repository structure
my-hielements-library/
├── README.md
├── src/
│   └── plugin.py (or plugin.go, plugin.rs, etc.)
├── tests/
│   └── test_plugin.py
├── hielements.toml.example
└── LICENSE
```

**Users can:**
- Clone the repository
- Add as a git submodule
- Reference from their project

```bash
# Clone to project
git clone https://github.com/username/my-hielements-library libs/mylibrary

# Or add as submodule
git submodule add https://github.com/username/my-hielements-library libs/mylibrary
```

```toml
[libraries]
mylibrary = { executable = "python3", args = ["libs/mylibrary/src/plugin.py"] }
```

**Best for:**
- Open-source projects
- Version control
- Collaborative development
- Private organization libraries

### Library Documentation Best Practices

When sharing libraries, include:

1. **README with clear usage instructions**
   - Installation steps
   - Configuration examples
   - Available functions and checks
   - Example `.hie` usage

2. **Configuration template**
   - Provide `hielements.toml.example`
   - Show all configuration options
   - Include security considerations

3. **Function reference**
   - Document all selector functions
   - Document all check functions
   - Parameter types and return values
   - Example usage for each function

4. **Examples**
   - Provide sample `.hie` files
   - Show common use cases
   - Include test cases

5. **Version compatibility**
   - Specify Hielements version requirements
   - Document breaking changes
   - Maintain a changelog

### Example Library Repository

```
mylibrary-hielements/
├── README.md                    # Main documentation
├── CHANGELOG.md                 # Version history
├── LICENSE                      # License information
├── src/
│   └── mylibrary_plugin.py     # Plugin implementation
├── tests/
│   ├── test_plugin.py          # Unit tests
│   └── fixtures/               # Test fixtures
├── examples/
│   ├── basic_usage.hie         # Simple example
│   ├── advanced_usage.hie      # Advanced features
│   └── hielements.toml         # Example configuration
├── docs/
│   ├── installation.md         # Installation guide
│   ├── api.md                  # API reference
│   └── troubleshooting.md      # Common issues
└── requirements.txt            # Dependencies (if Python)
```

---

## Best Practices

### Organizing Hielements Specs

#### Single File vs. Multiple Files

**Single File** (`architecture.hie`):
- Good for small projects
- Easier to understand at a glance
- Simple to maintain

**Multiple Files** (use imports):
- Better for large projects
- Organize by domain or layer
- Easier to collaborate

Example multi-file structure:
```
architecture/
  ├── main.hie          # Top-level system description
  ├── services.hie      # Microservices definitions
  ├── infrastructure.hie # Infrastructure elements
  └── templates.hie     # Reusable templates
```

### Naming Conventions

- **Elements**: Use descriptive names matching your domain (`orders_service`, `payment_gateway`)
- **Scopes**: Name after what they select (`api_module`, `config_file`, `dockerfile`)
- **Connection Points**: Name after what they expose (`rest_api`, `database_connection`, `event_queue`)
- **Templates**: Use generic pattern names (`microservice`, `compiler`, `hexagonal_architecture`)

### Writing Effective Checks

#### Do: Write Specific, Actionable Checks

```hielements
# Good - specific and actionable
check docker.exposes_port(dockerfile, 8080)
check python.function_exists(module, 'health_check')
check files.max_size(config, 1048576)  # 1MB
```

#### Don't: Write Vague or Unverifiable Checks

```hielements
# Bad - too vague
check "service is good"
check "follows best practices"
```

### Incremental Adoption

Start small and grow:

1. **Phase 1**: Document existing structure
   ```hielements
   element my_service:
       scope src = files.folder_selector('src/')
       check files.exists(src, 'main.py')
   ```

2. **Phase 2**: Add basic checks
   ```hielements
   check python.has_tests(src)
   check python.no_syntax_errors(src)
   ```

3. **Phase 3**: Enforce relationships
   ```hielements
   check python.no_circular_imports(module_a, module_b)
   check docker.entry_point(dockerfile, python_main)
   ```

4. **Phase 4**: Use templates for consistency
   ```hielements
   element service_a implements microservice:
       # Ensures consistent structure
   ```

### Documentation with Comments

Use comments to explain architectural decisions:

```hielements
## Orders Service
## Handles order creation, updates, and fulfillment.
## Must remain independent from payment processing for PCI compliance.
element orders_service:
    scope module = python.module_selector('services.orders')
    
    ## Ensure no direct dependency on payments
    ## (must use event bus instead)
    check python.no_dependency(module, payments_service.module)
```

---

## Integration with CI/CD

### GitHub Actions

Add architecture checks to your CI pipeline:

```yaml
# .github/workflows/architecture.yml
name: Architecture Checks

on:
  pull_request:
  push:
    branches: [main]

jobs:
  architecture:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Hielements
        run: cargo install hielements
      
      - name: Check Architecture
        run: hielements check architecture.hie
```

### GitLab CI

```yaml
# .gitlab-ci.yml
architecture:
  stage: test
  image: rust:latest
  script:
    - cargo install hielements
    - hielements check architecture.hie
  rules:
    - if: $CI_PIPELINE_SOURCE == 'merge_request_event'
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH
```

### Pre-commit Hooks

Run checks before committing:

```bash
# .git/hooks/pre-commit
#!/bin/bash
hielements check architecture.hie
if [ $? -ne 0 ]; then
    echo "Architecture checks failed. Please fix violations before committing."
    exit 1
fi
```

---

## IDE Integration

### VS Code Extension

Install the Hielements extension for VS Code:

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Hielements"
4. Click Install

**Features:**
- Syntax highlighting
- Real-time error checking
- Go to definition
- Auto-completion
- Inline documentation

### Language Server Protocol

Hielements implements the Language Server Protocol (LSP), enabling integration with any LSP-compatible editor:

- VS Code
- Vim/Neovim (via coc.nvim or built-in LSP)
- Emacs (via lsp-mode)
- Sublime Text
- Atom

---

## Advanced Topics

### Using Patterns at Scale

For large organizations, consider:

1. **Pattern Libraries**: Create organization-wide pattern libraries
2. **Pattern Governance**: Establish processes for creating and updating patterns
3. **Pattern Documentation**: Document each pattern's purpose and usage
4. **Pattern Versioning**: Version patterns independently of implementations

### Custom Library Development

For complex libraries:

1. **Caching**: Cache analysis results for performance
2. **Incremental Analysis**: Only re-analyze changed files
3. **External Tool Integration**: Leverage existing static analysis tools
4. **Error Handling**: Provide clear, actionable error messages

### Multi-Repository Architecture

For microservices or monorepos:

```hielements
## In repository A
element service_a:
    scope module = python.module_selector('src/')
    ref api = python.public_functions(module)

## In repository B
import service_a from "https://git.example.com/service_a/architecture.hie"

element service_b:
    scope module = python.module_selector('src/')
    check can_communicate(module, service_a.api)
```

---

## Getting Help

- 📖 [Language Reference](doc/language_reference.md) - Complete syntax reference
- 📚 [Pattern Catalog](doc/patterns_catalog.md) - Software engineering patterns
- 🔌 [External Libraries Guide](doc/external_libraries.md) - Creating custom libraries
- 🏗️ [Technical Architecture](doc/technical_architecture.md) - Implementation details
- 💬 [Discussions](https://github.com/ercasta/hielements/discussions) - Ask questions
- 🐛 [Issue Tracker](https://github.com/ercasta/hielements/issues) - Report bugs

---

## Next Steps

1. **Explore Examples**: Check out the [`examples/`](examples/) directory
2. **Read the Language Reference**: Dive deep into [language_reference.md](doc/language_reference.md)
3. **Browse the Pattern Catalog**: See common patterns in [patterns_catalog.md](doc/patterns_catalog.md)
4. **Create Your First Spec**: Start with a simple element describing part of your system
5. **Add Checks**: Gradually add checks to enforce your architecture
6. **Use Patterns**: Abstract common constraints into reusable patterns
7. **Integrate CI/CD**: Add architecture checks to your pipeline
8. **Extend with Libraries**: Create custom libraries for your needs

Happy architecting with Hielements! 🏗️
