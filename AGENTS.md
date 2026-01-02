# Development instructions

## Change planning and execution
- Read hielements.hie to get and understanding of the system
- Plan for changes by defining what changes are needed to hielements.hie. Track these in a separate file in "agent-changelog" folder (1 file per each request)
- Change hielements.hie accordingly
- Implement the changes
- Run Hielements checks to ensure alignment

## Language Evolution

Any change to the Hielements language must also include a review of the Pattern Catalog (`doc/patterns_catalog.md`) to:
1. **Update affected patterns**: If language syntax or semantics change, update any patterns that use the modified features
2. **Add new patterns**: When adding new language features, add patterns that showcase those capabilities
3. **Verify coverage**: Ensure the catalog adequately exercises the language's prescriptive features

The Pattern Catalog serves as a living test suite for the language. Coverage of this catalog drives language evolution—if a common software engineering pattern cannot be expressed in Hielements, the language should be extended to support it.

### Pattern Catalog Requirements

When modifying the catalog:
- Each pattern must use prescriptive features (`template`, `requires`, `forbids`, `allows`, `check`, `ref`, `uses`)
- Each pattern must include:
  - Clear description and use cases
  - Complete Hielements implementation
  - At least one example binding/implementation
- Patterns should cover diverse software engineering domains (structural, behavioral, infrastructure, etc.)
