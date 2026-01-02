# Rename "Element Templates" to "Patterns" and Create Pattern Catalog

## Date: 2026-01-02

## Summary
This change renames "element templates" to "patterns" throughout the documentation and codebase,
reflecting that templates are really about conformance to software engineering patterns. Additionally,
a comprehensive pattern catalog is created to serve as both documentation and a test suite for the
language's prescriptive capabilities.

## Changes to hielements.hie

Add reference to the new patterns catalog in the documentation element:
```hielements
## Documentation
element documentation {
    scope docs = files.folder_selector('doc')
    
    ## Pattern catalog - test suite for language prescriptive features
    check files.exists(docs, 'patterns_catalog.md')
    ...
}
```

## Documentation Updates

### Terminology Change: "element templates" → "patterns"

Files to update:
1. `doc/language_reference.md` - Section 8 "Element Templates" → "Patterns"
2. `doc/language_v2.md` - Replace "element templates" references
3. `doc/technical_architecture.md` - Update any template references
4. `USAGE.md` - Section "Using Element Templates" → "Using Patterns"
5. `README.md` - Update references and highlight catalog

### New File: `doc/patterns_catalog.md`

Create an extensive catalog of common software engineering patterns with their hielements implementations:

1. **Structural Patterns**
   - Layered Architecture (N-Tier)
   - Hexagonal Architecture (Ports & Adapters)
   - Clean Architecture
   - Microservices Architecture
   - Module/Package Structure

2. **Behavioral Patterns**
   - Event-Driven Architecture
   - Pipeline/Filter Pattern
   - Observer Pattern (for system architecture)
   - Command Query Responsibility Segregation (CQRS)

3. **Creational Patterns**
   - Factory Pattern (at module level)
   - Builder Pattern (for configuration)
   - Dependency Injection Container

4. **Infrastructure Patterns**
   - Containerized Service
   - Sidecar Pattern
   - Ambassador Pattern
   - Gateway Pattern

5. **Cross-Cutting Patterns**
   - Observability Pattern
   - Resilience Pattern
   - Security Boundary Pattern
   - API Versioning Pattern

Each pattern includes:
- Pattern description
- Common use cases
- Hielements implementation using prescriptive features (pattern, requires, forbids, allows, checks)
- Example bindings

### Update to AGENTS.md

Add requirement that language changes must include catalog review:
```markdown
## Language Evolution

Any change to the Hielements language must also include a review of the patterns catalog 
(`doc/patterns_catalog.md`) to:
1. Update any affected pattern implementations
2. Add new patterns that showcase new language features
3. Ensure catalog coverage drives language evolution
```

## Rationale

The term "pattern" better reflects the intent:
- Templates are really about defining patterns that implementations must conform to
- "Pattern" is a well-understood term in software engineering
- The catalog serves as a test suite ensuring the language can express common patterns
- Coverage of the catalog drives language evolution

## Impact

- Documentation-only change (no code changes to interpreter)
- Improved clarity for users understanding the prescriptive features
- The catalog becomes a reference for language capability testing
