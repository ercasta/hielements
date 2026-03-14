# Documentation Restructuring and Implementation Status

**Date**: 2026-03-14
**Context**: Restructure language documentation for multiple audiences (internals, tutorials, compliance); document implementation errors/gaps.

## Summary

This change improves the language documentation by:

1. **Fixing structural issues in `language_reference.md`**
2. **Creating `doc/implementation_status.md`** - a compliance checklist for language implementation
3. **Updating `hielements.hie`** to enforce presence of `implementation_status.md`

---

## 1. Changes to `doc/language_reference.md`

### Issues Fixed

#### Section Numbering
The document had several numbering problems accumulated from incremental additions:
- Section "5a" (Uses Declarations) was not in the Table of Contents and used non-standard naming
- Two sections were both numbered "10" (Imports and Modules, and Expressions)
- Subsections in the Imports section used wrong prefix ("8.1" instead of matching the section number)
- Subsections in the Expressions section used wrong prefix ("9.1" instead of matching section)
- Section 13 was skipped (jumped from 12 to 14) in the body, though the TOC showed 13
- Section 8 (Patterns) had two subsections both labeled "8.12" (Connection Boundaries and Language Constraints)

#### Stale Notes
- Removed the editorial note "THIS HAS TO BE REMOVED" from the `template` keyword row in the keywords table. The note was an internal developer reminder left in the published documentation.

#### Inaccurate Labels
- The grammar section title said "V2" in a document titled "V3"
- Several section names in the Examples and Patterns sections still had "(V2)" labels

#### Patterns Section Description
- Updated the opening paragraph to use the preferred `pattern` keyword instead of `template`

### New Section Structure

| Old Number | New Number | Section |
|-----------|-----------|---------|
| (not in TOC) 5a | 6 | Uses Declarations (V3) |
| 6 | 7 | Rules (Checks) |
| 7 | 8 | Children Elements |
| 8 | 9 | Patterns |
| 9 | 10 | Language Declarations |
| 10 (first) | 11 | Imports and Modules |
| 10 (second) | 12 | Expressions |
| 11 (body) / 12 (TOC) | 13 | Built-in Libraries |
| 12 (body) / 13 (TOC) | 14 | Comments |
| 14 | 15 | Complete Grammar (V3) |
| 15 | 16 | Examples |

### Appendix D Updates
- Renamed: "Migration Guide from V1" → "Migration Guide (V1/V2 → V3)"
- Added **section D.9**: V2 → V3 migration guide documenting the new V3 features:
  - Curly bracket syntax
  - `ref` keyword (replacing `connection_point`)
  - `uses` declarations
  - `pattern` keyword (replacing `template`)

---

## 2. New File: `doc/implementation_status.md`

Created a comprehensive implementation status document serving the "compliance checks for language implementation" purpose. It includes:

- **Feature Implementation Status**: Table of all language features with ✅/❌ status
- **Known Limitations**: Detailed descriptions of 4 not-yet-implemented features:
  1. Scope binding enforcement (`implements` + `binds` completeness not checked)
  2. `uses` declaration validation (parsed but not enforced)
  3. Type checking for `ref` declarations (parsed but not enforced)
  4. Template/Pattern registry (not yet built)
- **Implementation Roadmap**: Priority-ordered list of planned features
- **Running Checks**: How to use `hielements run` for compliance verification

---

## 3. Updates to `hielements.hie`

Added a new check in the `documentation` element:
```hielements
## Implementation status - tracks which language features are implemented
check files.exists(docs, 'implementation_status.md')
```

This ensures the implementation status document is always present.

---

## 4. Verification

Running `hielements run hielements.hie --workspace .` shows:
- **122 total checks** (was 121)
- **122 passed, 0 failed, 0 errors**

The new check for `implementation_status.md` passes successfully.

---

## 5. Implementation Errors Found During Check

No implementation errors were found during `hielements check` — all 122 checks pass.

However, the following **known limitations** were identified through code analysis (not via runtime errors, since these features are not yet enforced):

| Issue | Impact | Reference |
|-------|--------|-----------|
| `implements` binding completeness not checked | Missing bindings silently accepted | `doc/implementation_status.md` section 1 |
| `uses` declarations not validated | Invalid uses silently accepted | `doc/implementation_status.md` section 2 |
| `ref` type annotations not enforced | Type mismatches silently accepted | `doc/implementation_status.md` section 3 |
| Template/pattern registry missing | `binds` paths not validated | `doc/implementation_status.md` section 4 |

These are pre-existing issues documented in `agent-changelog/consistency-restoration-analysis.md` (2026-03-08). They are documented in `implementation_status.md` for future implementation.
