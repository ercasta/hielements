# Pattern Library Generation

## Objective
Convert the current pattern catalogue from markdown to a library format, and enable automatic regeneration of the pattern catalogue documentation.

## Current State
- Pattern catalogue exists in `doc/patterns_catalog.md`
- Contains extensive pattern definitions in markdown format with embedded Hielements code
- Documentation is manually maintained

## Proposed Changes

### 1. Create Patterns Library Directory
- Create `patterns/` directory at root level
- Store individual pattern definitions as `.hie` files
- Organize by category (structural, behavioral, creational, infrastructure, etc.)

### 2. Extract Patterns from Markdown
- Convert each pattern from markdown to a standalone `.hie` file
- Maintain the pattern definitions with their checks and requirements
- Add documentation comments to the `.hie` files

### 3. Create Pattern Documentation Generator
- Extend the existing `hielements doc` command or create a new subcommand
- Generate markdown documentation from `.hie` pattern files
- Include pattern descriptions, use cases, and examples

### 4. Update Documentation
- Regenerate `doc/patterns_catalog.md` from pattern library
- Update README.md to highlight automatic catalogue generation
- Add instructions for using and contributing to the pattern library

## Implementation Plan

1. Create `patterns/` directory structure
2. Extract patterns from `patterns_catalog.md` and convert to `.hie` files
3. Implement pattern documentation generation
4. Regenerate pattern catalogue
5. Update README with new feature highlight
6. Test the complete workflow

## Benefits
- Patterns become executable and testable
- Automatic documentation generation ensures consistency
- Easier to maintain and extend patterns
- Patterns can be imported and reused across projects
