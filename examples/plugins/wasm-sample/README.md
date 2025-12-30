# WASM Sample Plugin

This is a sample Hielements library plugin compiled to WebAssembly.

## Building

To build this plugin, you need to have Rust and the wasm32-unknown-unknown target installed:

```bash
# Install the wasm32 target (if not already installed)
rustup target add wasm32-unknown-unknown

# Build the WASM module
cargo build --target wasm32-unknown-unknown --release
```

The compiled WASM file will be at:
```
target/wasm32-unknown-unknown/release/wasm_sample.wasm
```

## Usage

Add to your `hielements.toml`:

```toml
[libraries]
wasm_sample = { type = "wasm", path = "examples/plugins/wasm-sample/target/wasm32-unknown-unknown/release/wasm_sample.wasm" }
```

Or use auto-detection:

```toml
[libraries]
wasm_sample = { path = "examples/plugins/wasm-sample/target/wasm32-unknown-unknown/release/wasm_sample.wasm" }
```

Then in your `.hie` file:

```hielements
import wasm_sample

element my_component:
    scope src = wasm_sample.simple_selector('src')
    check wasm_sample.always_pass()
    check wasm_sample.check_scope_size(src, 100)
```

## Functions

### Selectors

- `simple_selector(path: string) -> Scope` - Creates a simple folder scope
- `echo_selector(value: any) -> String` - Echoes back the argument

### Checks

- `always_pass() -> Pass` - Always passes
- `always_fail(message: string) -> Fail` - Always fails with the given message
- `check_scope_size(scope: Scope, max: int) -> Pass|Fail` - Checks if scope has <= max files

## Benefits of WASM Plugins

- **Security**: Runs in a sandbox with no file system access by default
- **Performance**: Near-native speed, no IPC overhead
- **Portability**: Single .wasm file works on all platforms
- **Size**: Compiled WASM is typically small (10-50 KB)
- **Languages**: Can be written in Rust, C, C++, AssemblyScript, Go, etc.
