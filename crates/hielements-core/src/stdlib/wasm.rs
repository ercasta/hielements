//! WebAssembly library support for Hielements.
//!
//! Enables sandboxed user-defined libraries through WebAssembly modules.
//! WASM libraries provide strong isolation and near-native performance.
//!
//! **Note**: Full WASM support is planned but not yet implemented.
//! The infrastructure and configuration support are in place.
//! WASM plugins will use the wasmtime runtime for sandboxed execution.

use std::path::{Path, PathBuf};

use super::{CheckResult, Library, LibraryError, LibraryResult, Value};

/// Configuration for a WASM library.
#[derive(Debug, Clone)]
pub struct WasmLibraryConfig {
    /// Name of the library as it will be referenced in .hie files
    pub name: String,
    /// Path to the WASM file
    pub path: String,
}

/// A WebAssembly library that runs in a sandboxed environment.
///
/// **Note**: This is a placeholder implementation. Full WASM support
/// requires the `wasm` feature to be enabled and wasmtime runtime integration.
pub struct WasmLibrary {
    config: WasmLibraryConfig,
    _workspace: PathBuf,
}

impl WasmLibrary {
    /// Create a new WASM library from configuration.
    ///
    /// Returns an error indicating WASM support is not yet implemented.
    pub fn new(config: WasmLibraryConfig, workspace: &str) -> LibraryResult<Self> {
        // For now, return a library instance that will error on use
        Ok(Self {
            config,
            _workspace: PathBuf::from(workspace),
        })
    }
}

impl Library for WasmLibrary {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn call(&mut self, _function: &str, _args: Vec<Value>, _workspace: &str) -> LibraryResult<Value> {
        Err(LibraryError::new(
            "E600",
            format!(
                "WASM library '{}' support is not yet fully implemented. \
                This feature requires wasmtime runtime integration. \
                Please use external process libraries for now.",
                self.config.name
            ),
        ))
    }

    fn check(&mut self, _function: &str, _args: Vec<Value>, _workspace: &str) -> LibraryResult<CheckResult> {
        Err(LibraryError::new(
            "E601",
            format!(
                "WASM library '{}' support is not yet fully implemented. \
                This feature requires wasmtime runtime integration. \
                Please use external process libraries for now.",
                self.config.name
            ),
        ))
    }
}

/// Load WASM libraries from a configuration file.
///
/// **Note**: Currently returns an empty list as WASM support is not fully implemented.
/// Libraries configured as WASM type will be loaded but will error when used.
pub fn load_wasm_libraries(config_path: &Path, workspace: &str) -> LibraryResult<Vec<WasmLibrary>> {
    use super::external::{HielementsConfig, LibraryType};
    
    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(config_path).map_err(|e| {
        LibraryError::new("E602", format!("Failed to read config file: {}", e))
    })?;

    let config: HielementsConfig = toml::from_str(&content).map_err(|e| {
        LibraryError::new("E603", format!("Failed to parse config file: {}", e))
    })?;

    let mut libraries = Vec::new();
    for (name, entry) in config.libraries {
        // Only load WASM type libraries
        let lib_type = entry.infer_type()?;
        if lib_type == LibraryType::Wasm {
            let path = entry.get_wasm_path()?;
            let lib = WasmLibrary::new(
                WasmLibraryConfig { name, path },
                workspace
            )?;
            libraries.push(lib);
        }
    }

    Ok(libraries)
}

/// Load WASM libraries from a workspace directory.
/// Looks for hielements.toml in the workspace root.
pub fn load_workspace_wasm_libraries(workspace: &str) -> LibraryResult<Vec<WasmLibrary>> {
    let config_path = Path::new(workspace).join("hielements.toml");
    load_wasm_libraries(&config_path, workspace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_config() {
        let config = WasmLibraryConfig {
            name: "test".to_string(),
            path: "test.wasm".to_string(),
        };
        assert_eq!(config.name, "test");
        assert_eq!(config.path, "test.wasm");
    }

    #[test]
    fn test_wasm_library_not_implemented() {
        let config = WasmLibraryConfig {
            name: "test".to_string(),
            path: "test.wasm".to_string(),
        };
        let mut lib = WasmLibrary::new(config, ".").unwrap();
        
        // Should error when trying to use
        let result = lib.call("test_func", vec![], ".");
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not yet fully implemented"));
    }

    // Full integration tests will be added when WASM support is implemented
}
