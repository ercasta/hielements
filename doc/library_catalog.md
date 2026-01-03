# Hielements Library Documentation

This catalog documents all available Hielements libraries, their selectors, and checks.

---

## Table of Contents

- [files](#files)
- [python](#python)
- [rust](#rust)

---

## files

File system operations library for selecting files/folders and checking file existence.

**Version:** 1.0.0

### Selectors

#### `files.file_selector(path: string) -> Scope`

Select a single file by its path relative to the workspace.

**Parameters:**

- `path` (string): Relative path to the file from workspace root

**Example:**

```hielements
scope main = files.file_selector('src/main.rs')
```

#### `files.folder_selector(path: string) -> Scope`

Select a folder and all files within it recursively.

**Parameters:**

- `path` (string): Relative path to the folder from workspace root

**Example:**

```hielements
scope src = files.folder_selector('src')
```

#### `files.glob_selector(pattern: string) -> Scope`

Select files matching a glob pattern.

**Parameters:**

- `pattern` (string): Glob pattern (e.g., '**/*.rs', 'src/*.py')

**Example:**

```hielements
scope rust_files = files.glob_selector('**/*.rs')
```

### Checks

#### `files.exists(scope: Scope, filename: string) -> CheckResult`

Check if a file exists within a scope.

**Parameters:**

- `scope` (Scope): The scope to check within
- `filename` (string): The filename to look for

**Example:**

```hielements
check files.exists(src, 'main.rs')
```

#### `files.contains(scope: Scope, filename: string) -> CheckResult`

Check if a scope contains a file with the given name.

**Parameters:**

- `scope` (Scope): The scope to check
- `filename` (string): The filename to look for

**Example:**

```hielements
check files.contains(docs, 'README.md')
```

#### `files.no_files_matching(scope: Scope, pattern: string) -> CheckResult`

Check that no files match a given pattern within a scope.

**Parameters:**

- `scope` (Scope): The scope to check
- `pattern` (string): Glob pattern that should not match any files

**Example:**

```hielements
check files.no_files_matching(src, '*.bak')
```

#### `files.max_size(scope: Scope, max_bytes: integer) -> CheckResult`

Check that all files in a scope are under a maximum size.

**Parameters:**

- `scope` (Scope): The scope to check
- `max_bytes` (integer): Maximum file size in bytes

**Example:**

```hielements
check files.max_size(assets, 1048576)
```

---

## python

Python language analysis library for selecting and checking Python code constructs.

**Version:** 1.0.0

### Selectors

#### `python.module_selector(module_path: string) -> Scope`

Select a Python module by name (e.g., 'orders' or 'orders.api').

**Parameters:**

- `module_path` (string): Dotted module path

**Example:**

```hielements
scope api = python.module_selector('orders.api')
```

#### `python.function_selector(func_name: string) -> Scope`

Select Python functions by name.

**Parameters:**

- `func_name` (string): Name of the function to find

**Example:**

```hielements
scope handlers = python.function_selector('handle_request')
```

#### `python.class_selector(class_name: string) -> Scope`

Select Python classes by name.

**Parameters:**

- `class_name` (string): Name of the class to find

**Example:**

```hielements
scope models = python.class_selector('UserModel')
```

### Checks

#### `python.imports(scope: Scope, module_name: string) -> CheckResult`

Check that a scope imports a specific module.

**Parameters:**

- `scope` (Scope): The scope to check
- `module_name` (string): Module name to look for

**Example:**

```hielements
check python.imports(api_mod, 'typing')
```

#### `python.no_import(scope: Scope, module_name: string) -> CheckResult`

Check that a scope does NOT import a specific module.

**Parameters:**

- `scope` (Scope): The scope to check
- `module_name` (string): Module name that should not be imported

**Example:**

```hielements
check python.no_import(core_mod, 'django')
```

#### `python.returns_type(scope: Scope, type_name: string) -> CheckResult`

Check that any function in a scope returns a given type.

**Parameters:**

- `scope` (Scope): The scope to check
- `type_name` (string): Return type to look for

**Example:**

```hielements
check python.returns_type(api_mod, 'Response')
```

#### `python.function_returns_type(scope: Scope, func_name: string, type_name: string) -> CheckResult`

Check that a specific function returns a given type.

**Parameters:**

- `scope` (Scope): The scope to check
- `func_name` (string): Function name
- `type_name` (string): Expected return type

**Example:**

```hielements
check python.function_returns_type(api_mod, 'get_user', 'User')
```

#### `python.calls(scope: Scope, target: string) -> CheckResult`

Check that a scope calls a specific identifier (function or object).

**Parameters:**

- `scope` (Scope): The scope to check
- `target` (string): Identifier to look for

**Example:**

```hielements
check python.calls(service_mod, 'logger')
```

#### `python.calls_function(scope: Scope, module_name: string, func_name: string) -> CheckResult`

Check that a scope calls a specific function in a module.

**Parameters:**

- `scope` (Scope): The scope to check
- `module_name` (string): Module name
- `func_name` (string): Function name

**Example:**

```hielements
check python.calls_function(api_mod, 'database', 'connect')
```

#### `python.calls_scope(source_scope: Scope, target_scope: Scope) -> CheckResult`

Check that source scope calls something in target scope.

**Parameters:**

- `source_scope` (Scope): Source scope that should make calls
- `target_scope` (Scope): Target scope that should be called

**Example:**

```hielements
check python.calls_scope(api_mod, utils_mod)
```

---

## rust

Rust language analysis library for selecting and checking Rust code constructs.

**Version:** 1.0.0

### Selectors

#### `rust.crate_selector(crate_name: string) -> Scope`

Select a Rust crate by name from the workspace.

**Parameters:**

- `crate_name` (string): Name of the crate to select

**Example:**

```hielements
scope core = rust.crate_selector('hielements-core')
```

#### `rust.module_selector(module_path: string) -> Scope`

Select a Rust module by path (e.g., 'lexer' or 'stdlib::files').

**Parameters:**

- `module_path` (string): Module path using :: separator

**Example:**

```hielements
scope lexer_mod = rust.module_selector('lexer')
```

#### `rust.struct_selector(struct_name: string) -> Scope`

Select a Rust struct by name.

**Parameters:**

- `struct_name` (string): Name of the struct to find

**Example:**

```hielements
ref token: Token = rust.struct_selector('Token')
```

#### `rust.enum_selector(enum_name: string) -> Scope`

Select a Rust enum by name.

**Parameters:**

- `enum_name` (string): Name of the enum to find

**Example:**

```hielements
ref kind: TokenKind = rust.enum_selector('TokenKind')
```

#### `rust.function_selector(func_name: string) -> Scope`

Select a Rust function by name.

**Parameters:**

- `func_name` (string): Name of the function to find

**Example:**

```hielements
ref parse_fn: Function = rust.function_selector('parse')
```

#### `rust.trait_selector(trait_name: string) -> Scope`

Select a Rust trait by name.

**Parameters:**

- `trait_name` (string): Name of the trait to find

**Example:**

```hielements
ref lib_trait: Trait = rust.trait_selector('Library')
```

#### `rust.impl_selector(type_name: string) -> Scope`

Select Rust impl blocks for a type.

**Parameters:**

- `type_name` (string): Name of the type to find impls for

**Example:**

```hielements
scope impl_blocks = rust.impl_selector('Parser')
```

### Checks

#### `rust.struct_exists(struct_name: string) -> CheckResult`

Check that a struct with the given name exists.

**Parameters:**

- `struct_name` (string): Name of the struct

**Example:**

```hielements
check rust.struct_exists('Parser')
```

#### `rust.enum_exists(enum_name: string) -> CheckResult`

Check that an enum with the given name exists.

**Parameters:**

- `enum_name` (string): Name of the enum

**Example:**

```hielements
check rust.enum_exists('TokenKind')
```

#### `rust.function_exists(func_name: string) -> CheckResult`

Check that a function with the given name exists.

**Parameters:**

- `func_name` (string): Name of the function

**Example:**

```hielements
check rust.function_exists('parse')
```

#### `rust.trait_exists(trait_name: string) -> CheckResult`

Check that a trait with the given name exists.

**Parameters:**

- `trait_name` (string): Name of the trait

**Example:**

```hielements
check rust.trait_exists('Library')
```

#### `rust.impl_exists(type_name: string) -> CheckResult`

Check that an impl block for the given type exists.

**Parameters:**

- `type_name` (string): Name of the type

**Example:**

```hielements
check rust.impl_exists('Parser')
```

#### `rust.implements(type_name: string, trait_name: string) -> CheckResult`

Check that a type implements a specific trait.

**Parameters:**

- `type_name` (string): Name of the implementing type
- `trait_name` (string): Name of the trait

**Example:**

```hielements
check rust.implements('FilesLibrary', 'Library')
```

#### `rust.uses(scope: Scope, module_path: string) -> CheckResult`

Check that a scope uses a specific module.

**Parameters:**

- `scope` (Scope): The scope to check
- `module_path` (string): Module path to look for

**Example:**

```hielements
check rust.uses(parser_mod, 'crate::lexer')
```

#### `rust.has_derive(scope: Scope, derive_name: string) -> CheckResult`

Check that a scope has a specific derive macro.

**Parameters:**

- `scope` (Scope): The scope to check
- `derive_name` (string): Name of the derive (e.g., 'Debug', 'Clone')

**Example:**

```hielements
check rust.has_derive(struct_scope, 'Serialize')
```

#### `rust.has_docs(scope: Scope) -> CheckResult`

Check that a scope has documentation comments.

**Parameters:**

- `scope` (Scope): The scope to check

**Example:**

```hielements
check rust.has_docs(module)
```

#### `rust.has_tests(scope: Scope) -> CheckResult`

Check that a scope has test functions.

**Parameters:**

- `scope` (Scope): The scope to check

**Example:**

```hielements
check rust.has_tests(module)
```

#### `rust.depends_on(scope_a: Scope, scope_b: Scope) -> CheckResult`

Check that scope_a depends on (uses types from) scope_b.

**Parameters:**

- `scope_a` (Scope): Source scope
- `scope_b` (Scope): Target scope that should be a dependency

**Example:**

```hielements
check rust.depends_on(parser_mod, lexer_mod)
```

#### `rust.no_dependency(scope_a: Scope, scope_b: Scope) -> CheckResult`

Check that scope_a does NOT depend on scope_b (architectural boundary).

**Parameters:**

- `scope_a` (Scope): Source scope
- `scope_b` (Scope): Target scope that should NOT be a dependency

**Example:**

```hielements
check rust.no_dependency(core_mod, cli_mod)
```

#### `rust.pipeline_connects(output_scope: Scope, input_scope: Scope) -> CheckResult`

Check that output type from one scope connects to input of another.

**Parameters:**

- `output_scope` (Scope): Scope producing output
- `input_scope` (Scope): Scope consuming input

**Example:**

```hielements
check rust.pipeline_connects(lexer.tokens, parser.input)
```

#### `rust.type_compatible(scope_a: Scope, scope_b: Scope) -> CheckResult`

Check that two scopes have compatible types.

**Parameters:**

- `scope_a` (Scope): First scope
- `scope_b` (Scope): Second scope

**Example:**

```hielements
check rust.type_compatible(producer_type, consumer_type)
```

---

