# Python Library Implementation

## Overview
This change adds a Python language library to Hielements, providing selectors and checks for Python code analysis.

## Changes to hielements.hie

### New Element: Python Library

Added a new element under `stdlib` to describe the Python library:

```hielements
element stdlib:
    # ... existing elements ...
    
    ## Python library - Python code analysis
    element python_library:
        scope python_module = files.file_selector('crates/hielements-core/src/stdlib/python.rs')
        
        check rust.struct_exists('PythonLibrary')
        check rust.implements('PythonLibrary', 'Library')
        check rust.has_tests(python_module)
```

## Implementation Details

### File Structure
- **New file**: `crates/hielements-core/src/stdlib/python.rs`
- **Modified**: `crates/hielements-core/src/stdlib/mod.rs` - registered Python library

### Selectors Implemented
1. `module_selector(module_path)` - Select Python module by path (e.g., "orders" or "orders.api")
2. `function_selector(func_name)` - Select function by name
3. `class_selector(class_name)` - Select class by name

### Checks Implemented (First Checks - Per Requirements)

#### 1. Import Checks
- `imports(scope, module_name)` - Check if scope imports a module
- `no_import(scope, module_name)` - Check that scope does NOT import a module

#### 2. Return Type Checks
- `returns_type(scope, type_name)` - Check if any function in scope returns a given type
- `function_returns_type(scope, func_name, type_name)` - Check if a specific function returns a given type

#### 3. Call Checks
- `calls(scope, target)` - Check if scope calls something (function or module)
- `calls_function(scope, module_name, func_name)` - Check if scope calls a specific function in a module
- `calls_scope(source_scope, target_scope)` - Check if source scope calls anything in target scope

### Design Decisions
1. **Pattern matching approach**: Similar to Rust library, uses regex-based pattern matching for Python constructs
2. **Excluded directories**: Excludes common Python directories like `__pycache__`, `.venv`, `.pytest_cache`, etc.
3. **Type annotation support**: Handles both simple and generic type annotations (e.g., `-> User`, `-> List[str]`)
4. **Async function support**: All checks support both regular and async functions

### Testing
- All 10 unit tests pass
- Tests cover all selectors and checks
- Tests use tempfile for isolated test environments

## Usage Examples

```hielements
import python

element api_service:
    scope src<python> = python.module_selector('api.orders')
    scope utils<python> = python.module_selector('utils')
    
    # Import checks
    check python.imports(src, 'typing')
    check python.no_import(src, 'internal_module')
    
    # Return type checks
    check python.returns_type(src, 'Order')
    check python.function_returns_type(src, 'create_order', 'Order')
    
    # Call checks
    check python.calls(src, 'validate')
    check python.calls_function(src, 'logger', 'info')
    check python.calls_scope(src, utils)
```

## Alignment with Hielements Philosophy
- **Extensible**: Follows the Library trait pattern for consistency
- **Language-agnostic**: Python library integrates seamlessly with existing libraries (files, rust)
- **Behavioral checks**: Supports architectural validation through call and import checks
- **Type-aware**: Validates type annotations in Python code
