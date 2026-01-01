# Hierarchical Checks Refactoring

**Issue:** Refactor "transitive" terminology to "hierarchical" terminology  
**Date:** 2026-01-01  

## Summary

This document tracks the refactoring of the "transitive" terminology to "hierarchical" terminology throughout the codebase. The original implementation used "transitive" to describe checks that compose upwards-downwards through the parent-children element hierarchy, but "hierarchical" is more accurate as it reflects the structural nature of the feature rather than mathematical transitivity.

## Rationale

The term "transitive" typically implies a mathematical relationship where if A→B and B→C, then A→C. However, the feature actually describes requirements and connection boundaries that are inherited/composed through the hierarchical structure of elements (parent to children, children to grandchildren, etc.).

"Hierarchical" better captures:
- The parent-child element structure
- Requirements flowing down from parent to descendants
- Scope aggregation flowing up from children to parents
- Connection boundaries inherited by descendant elements

## Changes Required

### Code Changes
1. **AST Structures** (`ast.rs`)
   - Rename `TransitiveRequirement` → `HierarchicalRequirement`
   - Rename `TransitiveRequirementKind` → `HierarchicalRequirementKind`
   - Update comments and documentation strings

2. **Parser** (`parser.rs`)
   - Rename `parse_transitive_requirement()` → `parse_hierarchical_requirement()`
   - Update variable names: `transitive_requirements` → `hierarchical_requirements`
   - Update test function names

3. **Interpreter** (`interpreter.rs`)
   - Rename `validate_transitive_requirement()` → `validate_hierarchical_requirement()`
   - Update variable names and comments

4. **Lexer** (`lexer.rs`)
   - Note: Keyword `requires_descendant` remains unchanged as it's user-facing syntax
   - Internal token handling updated if needed

### Documentation Changes
1. **Language Reference** (`doc/language_reference.md`)
   - Update section title: "Top-Down Transitivity" → "Hierarchical Checks"
   - Replace "transitive requirement" with "hierarchical requirement" throughout
   - Update examples and explanations

2. **README.md**
   - Add a dedicated section highlighting hierarchical checks as a key feature
   - Update examples to use hierarchical terminology in comments
   - Emphasize the parent-child composition aspect

3. **USAGE.md**
   - Update guide sections to use hierarchical terminology
   - Update examples and best practices

4. **Agent Changelog**
   - Rename `top-down-transitivity.md` → `hierarchical-checks.md`
   - Update content to use hierarchical terminology throughout

### Example Changes
1. **Example File**
   - Rename `examples/transitivity.hie` → `examples/hierarchical.hie`
   - Update comments to use hierarchical terminology

2. **Self-Description** (`hielements.hie`)
   - Update comments referencing transitive structures
   - Change "Top-down transitivity structures" to "Hierarchical check structures"

## Implementation Notes

- User-facing syntax (`requires_descendant`, `allows_connection`, `forbids_connection`, `requires_connection`) remains unchanged
- Only internal naming, documentation, and comments are updated
- This is a pure refactoring with no functional changes
- All tests should pass without modification (only test names updated for clarity)

## Terminology Mapping

| Old Term | New Term |
|----------|----------|
| Transitive requirement | Hierarchical requirement |
| TransitiveRequirement | HierarchicalRequirement |
| TransitiveRequirementKind | HierarchicalRequirementKind |
| transitive_requirements | hierarchical_requirements |
| parse_transitive_requirement | parse_hierarchical_requirement |
| validate_transitive_requirement | validate_hierarchical_requirement |
| Top-Down Transitivity | Hierarchical Checks |
| transitive requirement | hierarchical requirement (in prose) |

## Testing

- Run existing parser tests (renamed to reflect hierarchical terminology)
- Run existing interpreter tests
- Validate examples with `hielements check`
- Ensure documentation renders correctly
- Verify no functional behavior changes

## Benefits

1. **Clearer Communication**: "Hierarchical" immediately conveys the parent-child structure
2. **Reduced Confusion**: Avoids mathematical connotations of "transitive"
3. **Better Alignment**: Matches the architectural hierarchy concept
4. **Improved Documentation**: Easier for users to understand the feature
