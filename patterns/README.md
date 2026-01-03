# Hielements Pattern Library

This directory contains a curated collection of software engineering patterns implemented in Hielements. These patterns demonstrate best practices and common architectural approaches that can be reused across projects.

## Directory Structure

- `structural/` - Structural patterns (layered architecture, hexagonal, microservices, etc.)
- `behavioral/` - Behavioral patterns (event-driven, pipeline, CQRS, saga, etc.)
- `creational/` - Creational patterns (factory, builder, dependency injection, etc.)
- `infrastructure/` - Infrastructure patterns (containerized service, sidecar, API gateway, etc.)
- `cross-cutting/` - Cross-cutting concerns (observability, resilience, security, configuration, etc.)
- `testing/` - Testing patterns (test pyramid, contract testing, etc.)
- `compiler/` - Compiler and interpreter patterns (compiler pipeline, visitor, etc.)

## Using Patterns

Patterns can be imported and implemented in your Hielements specifications:

```hielements
import patterns.structural.layered_architecture

element my_app implements layered_architecture {
    scope presentation_mod<rust> binds layered_architecture.presentation.module = rust.module_selector('app::api')
    # ... more bindings
}
```

## Pattern Documentation

The pattern catalogue documentation is automatically generated from these pattern files using the `hielements doc` command. See `doc/patterns_catalog.md` for the generated documentation.

## Contributing Patterns

When adding new patterns:

1. Create a `.hie` file in the appropriate category directory
2. Include a description comment block at the top with:
   - Pattern name
   - Description of the pattern
   - Use cases where the pattern applies
3. Implement the pattern using prescriptive features (`pattern`, `requires`, `forbids`, `allows`, `check`, `ref`, `uses`)
4. Provide at least one example implementation
5. Regenerate the documentation: `hielements doc-patterns`

## Pattern Guidelines

- **DO** use patterns when you have multiple components with similar structure
- **DO** use patterns to enforce architectural decisions across teams
- **DO** use patterns as documentation for expected component structure
- **DON'T** use patterns for truly unique one-off components
- **DON'T** over-engineer with patterns when a simple element suffices

## Pattern Composition

Patterns can be composed through multiple `implements`:

```hielements
element production_service implements microservice, observability, resilience {
    ## Bindings for each pattern
    # ...
}
```
