# Update Keywords: template→pattern, connection_point→ref

## Date: 2026-01-02

## Summary
Updated the Hielements language to use clearer, more intuitive keywords. The `template` keyword has been renamed to `pattern`, and `connection_point` has been renamed to `ref`. Both old keywords remain supported for backward compatibility.

## Rationale

### Why `pattern` instead of `template`?
- The term "pattern" better reflects the intent: defining architectural patterns that implementations conform to
- "Pattern" is a well-understood term in software engineering (design patterns, architectural patterns)
- The patterns catalog (doc/patterns_catalog.md) was already using the `pattern` keyword
- Aligns terminology across documentation and code

### Why `ref` instead of `connection_point`?
- Shorter, clearer, and more concise
- More aligned with common programming terminology (references, ref)
- The V3 syntax examples were already using `ref`
- Reduces verbosity in specifications

## Changes Made

### Compiler/Interpreter Changes

1. **Lexer** (`crates/hielements-core/src/lexer.rs`)
   - Added `Pattern` token kind alongside `Template`
   - Both keywords are now recognized by the lexer

2. **Parser** (`crates/hielements-core/src/parser.rs`)
   - Updated parser to accept both `pattern` and `template` keywords
   - Added support for both keywords in all relevant contexts
   - Error messages updated to mention both keywords

3. **VSCode Extension**
   - Updated syntax highlighting to recognize `pattern` keyword
   - Updated language configuration for both `pattern` and `ref`
   - Added curly bracket support for V3 syntax

### Documentation Updates

Updated all documentation to use the new preferred keywords:

1. **README.md**
   - All examples updated to use `pattern` and `ref`
   - Documentation text updated to reference new keywords
   - Notes added about preferred terminology

2. **USAGE.md**
   - All code examples updated
   - Explanatory text updated

3. **doc/language_reference.md**
   - Keywords table updated to show both old and new keywords
   - Grammar updated to reflect both syntaxes
   - Examples updated throughout

4. **doc/language_v2.md**
   - Updated to use new keywords

5. **doc/scope_management.md**
   - Updated to use `ref` keyword

6. **doc/summary.md**
   - Updated to use new keywords

### Examples Updates

All example files updated to use new keywords:
- `examples/v2_syntax_example.hie`
- `examples/v3_syntax_example.hie`
- `examples/language_example.hie`
- `examples/template_compiler.hie`
- `examples/template_microservice.hie`
- `examples/hierarchical.hie`
- `examples/typed_connection_points.hie`
- `examples/advanced_templates.hie`

### Specs Updates

- `specs/stdlib.hie` - Updated to use `ref` keyword

## Backward Compatibility

Both old keywords (`template` and `connection_point`) remain fully supported:

```hielements
# Old syntax (still works)
template my_pattern:
    element component:
        connection_point output: string

# New syntax (preferred)
pattern my_pattern {
    element component {
        ref output: string
    }
}

# Can mix and match
pattern mixed {
    element comp {
        connection_point legacy: string  # Still works
        ref modern: string               # Preferred
    }
}
```

## Testing

All existing tests pass (88 tests):
- Parser tests verify both keywords work
- Integration tests validate backward compatibility
- Example files validate successfully with both syntaxes

## Migration Guide

### For Users

**Recommended but not required:** Update your specifications to use the new keywords:

```bash
# Automated update (use with caution, review changes)
sed -i 's/^template /pattern /g' your-spec.hie
sed -i 's/connection_point /ref /g' your-spec.hie
```

**No breaking changes:** Existing specifications continue to work without modification.

### For Tool Developers

If you're building tools that parse Hielements specifications:
- Update your lexer/parser to recognize both `pattern` and `template`
- Update your lexer/parser to recognize both `ref` and `connection_point`
- Consider warning users about deprecated keywords (optional)

## Future Considerations

In a future major version (V4+), we may deprecate the old keywords with warnings, but backward compatibility will be maintained for the foreseeable future.

## Related Changes

This change completes the terminology update started in PR #35 which:
- Created the patterns catalog using `pattern` keyword
- Introduced the concept of patterns as architectural blueprints
- Renamed "element templates" to "patterns" in documentation

The keyword update ensures the language syntax matches the updated terminology.
