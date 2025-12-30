//! WASM library support for Hielements.
//!
//! Enables sandboxed user-defined libraries through WebAssembly modules
//! with capability-based security via WASI.
//!
//! Note: This is a minimal implementation. WASI file system access will be
//! added in a future iteration once we stabilize the API usage.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{CheckResult, Library, LibraryError, LibraryResult, Value};

/// Capabilities that can be granted to a WASM plugin.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WasmCapabilities {
    /// File system access level: "none", "read", or "write"
    #[serde(default)]
    pub fs: String,
    
    /// Restrict file access to workspace only
    #[serde(default = "default_true")]
    pub workspace_only: bool,
    
    /// Allow environment variable access
    #[serde(default)]
    pub env_access: bool,
}

impl Default for WasmCapabilities {
    fn default() -> Self {
        Self {
            fs: String::new(),
            workspace_only: true,
            env_access: false,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Configuration for a WASM library.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WasmLibraryConfig {
    /// Name of the library as it will be referenced in .hie files
    pub name: String,
    
    /// Path to the .wasm file
    pub path: String,
    
    /// Capabilities granted to this plugin
    #[serde(default)]
    pub capabilities: WasmCapabilities,
}

/// A WASM library that runs in a sandboxed environment.
/// 
/// Note: This is a placeholder implementation. Full WASM support with WASI
/// will be implemented once the wasmtime API is properly stabilized.
pub struct WasmLibrary {
    config: WasmLibraryConfig,
    _workspace: PathBuf,
}

impl WasmLibrary {
    /// Create a new WASM library from configuration.
    pub fn new(config: WasmLibraryConfig, workspace: &str) -> LibraryResult<Self> {
        // Verify the WASM file exists
        let wasm_path = Path::new(workspace).join(&config.path);
        if !wasm_path.exists() {
            return Err(LibraryError::new(
                "E601",
                format!("WASM module not found: {}", config.path),
            ));
        }

        Ok(Self {
            config,
            _workspace: PathBuf::from(workspace),
        })
    }

    /// Convert Value to JSON for passing to WASM.
    fn value_to_json(value: &Value) -> serde_json::Value {
        match value {
            Value::Null => serde_json::Value::Null,
            Value::Bool(b) => serde_json::Value::Bool(*b),
            Value::Int(i) => serde_json::json!({"Int": i}),
            Value::Float(f) => serde_json::json!({"Float": f}),
            Value::String(s) => serde_json::json!({"String": s}),
            Value::List(items) => {
                let json_items: Vec<_> = items.iter().map(Self::value_to_json).collect();
                serde_json::json!({"List": json_items})
            }
            Value::Scope(scope) => {
                let kind = match &scope.kind {
                    super::ScopeKind::File(s) => serde_json::json!({"File": s}),
                    super::ScopeKind::Folder(s) => serde_json::json!({"Folder": s}),
                    super::ScopeKind::Glob(s) => serde_json::json!({"Glob": s}),
                };
                serde_json::json!({
                    "Scope": {
                        "kind": kind,
                        "paths": scope.paths,
                        "resolved": scope.resolved
                    }
                })
            }
            Value::ConnectionPoint(cp) => {
                serde_json::json!({
                    "ConnectionPoint": {
                        "name": cp.name,
                        "kind": cp.kind,
                        "data": {}
                    }
                })
            }
        }
    }

    /// Convert JSON from WASM back to Value.
    fn json_to_value(json: serde_json::Value) -> LibraryResult<Value> {
        // Handle direct primitives
        match &json {
            serde_json::Value::Null => return Ok(Value::Null),
            serde_json::Value::Bool(b) => return Ok(Value::Bool(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    return Ok(Value::Int(i));
                } else if let Some(f) = n.as_f64() {
                    return Ok(Value::Float(f));
                }
            }
            serde_json::Value::String(s) => return Ok(Value::String(s.clone())),
            _ => {}
        }

        // Handle tagged values
        if let serde_json::Value::Object(obj) = &json {
            if let Some(s) = obj.get("String") {
                return Ok(Value::String(s.as_str().unwrap_or_default().to_string()));
            }
            if let Some(i) = obj.get("Int") {
                return Ok(Value::Int(i.as_i64().unwrap_or(0)));
            }
            if let Some(f) = obj.get("Float") {
                return Ok(Value::Float(f.as_f64().unwrap_or(0.0)));
            }
            if let Some(scope_obj) = obj.get("Scope") {
                if let serde_json::Value::Object(scope) = scope_obj {
                    let kind = if let Some(kind_obj) = scope.get("kind") {
                        if let serde_json::Value::Object(k) = kind_obj {
                            if let Some(s) = k.get("File") {
                                super::ScopeKind::File(s.as_str().unwrap_or_default().to_string())
                            } else if let Some(s) = k.get("Folder") {
                                super::ScopeKind::Folder(
                                    s.as_str().unwrap_or_default().to_string(),
                                )
                            } else if let Some(s) = k.get("Glob") {
                                super::ScopeKind::Glob(s.as_str().unwrap_or_default().to_string())
                            } else {
                                super::ScopeKind::File(String::new())
                            }
                        } else {
                            super::ScopeKind::File(String::new())
                        }
                    } else {
                        super::ScopeKind::File(String::new())
                    };

                    let paths = scope
                        .get("paths")
                        .and_then(|p| p.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();

                    let resolved = scope
                        .get("resolved")
                        .and_then(|r| r.as_bool())
                        .unwrap_or(false);

                    return Ok(Value::Scope(super::Scope {
                        kind,
                        paths,
                        resolved,
                    }));
                }
            }
        }

        Err(LibraryError::new(
            "E615",
            format!("Cannot convert JSON to Value: {:?}", json),
        ))
    }

    /// Convert JSON to CheckResult.
    fn json_to_check_result(json: serde_json::Value) -> LibraryResult<CheckResult> {
        if let serde_json::Value::Object(obj) = &json {
            if obj.contains_key("Pass") {
                return Ok(CheckResult::Pass);
            }
            if let Some(msg) = obj.get("Fail") {
                return Ok(CheckResult::Fail(
                    msg.as_str().unwrap_or_default().to_string(),
                ));
            }
            if let Some(msg) = obj.get("Error") {
                return Ok(CheckResult::Error(
                    msg.as_str().unwrap_or_default().to_string(),
                ));
            }
        }

        Err(LibraryError::new(
            "E616",
            format!("Cannot convert JSON to CheckResult: {:?}", json),
        ))
    }
}

impl Library for WasmLibrary {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn call(
        &mut self,
        _function: &str,
        _args: Vec<Value>,
        _workspace: &str,
    ) -> LibraryResult<Value> {
        // TODO: Implement WASM execution with wasmtime
        // For now, return an error indicating WASM support is not yet fully implemented
        Err(LibraryError::new(
            "E617",
            "WASM library execution is not yet fully implemented. Please use external process libraries for now.",
        ))
    }

    fn check(
        &mut self,
        _function: &str,
        _args: Vec<Value>,
        _workspace: &str,
    ) -> LibraryResult<CheckResult> {
        // TODO: Implement WASM execution with wasmtime
        // For now, return an error indicating WASM support is not yet fully implemented
        Err(LibraryError::new(
            "E619",
            "WASM library execution is not yet fully implemented. Please use external process libraries for now.",
        ))
    }
}

/// Load a WASM library from configuration.
pub fn load_wasm_library(
    config: WasmLibraryConfig,
    workspace: &str,
) -> LibraryResult<WasmLibrary> {
    WasmLibrary::new(config, workspace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_capabilities_default() {
        let caps = WasmCapabilities::default();
        assert_eq!(caps.fs, "");
        assert!(caps.workspace_only);
        assert!(!caps.env_access);
    }

    #[test]
    fn test_value_to_json() {
        let value = Value::String("test".to_string());
        let json = WasmLibrary::value_to_json(&value);
        assert_eq!(json, serde_json::json!({"String": "test"}));
    }

    #[test]
    fn test_json_to_value_string() {
        let json = serde_json::json!({"String": "hello"});
        let value = WasmLibrary::json_to_value(json).unwrap();
        match value {
            Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String value"),
        }
    }

    #[test]
    fn test_json_to_check_result_pass() {
        let json = serde_json::json!({"Pass": null});
        let result = WasmLibrary::json_to_check_result(json).unwrap();
        assert!(result.is_pass());
    }

    #[test]
    fn test_json_to_check_result_fail() {
        let json = serde_json::json!({"Fail": "Something went wrong"});
        let result = WasmLibrary::json_to_check_result(json).unwrap();
        assert!(result.is_fail());
    }
}
