//! WASM library support for Hielements.
//!
//! Enables user-defined libraries through WebAssembly modules that provide
//! sandboxed, portable, and high-performance plugins.

use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use wasmtime::*;

use super::{CheckResult, Library, LibraryError, LibraryResult, Value};

/// Configuration for a WASM library.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WasmLibraryConfig {
    /// Name of the library as it will be referenced in .hie files
    pub name: String,
    /// Path to the WASM file
    pub path: String,
}

/// A WASM library that executes in a sandboxed environment.
pub struct WasmLibrary {
    config: WasmLibraryConfig,
    #[allow(dead_code)] // Keep for future use with WASI features
    engine: Engine,
    module: Module,
    store: Arc<Mutex<Store<()>>>,
}

impl WasmLibrary {
    /// Create a new WASM library from configuration.
    pub fn new(config: WasmLibraryConfig) -> LibraryResult<Self> {
        // Create WASM engine with default configuration
        let engine = Engine::default();
        
        // Load and compile the WASM module
        let module = Module::from_file(&engine, &config.path).map_err(|e| {
            LibraryError::new(
                "E600",
                format!("Failed to load WASM module '{}': {}", config.name, e),
            )
        })?;
        
        // Create store for module execution
        let store = Store::new(&engine, ());
        
        Ok(Self {
            config,
            engine,
            module,
            store: Arc::new(Mutex::new(store)),
        })
    }
    
    /// Call a WASM function with serialized arguments.
    fn call_wasm_function(
        &mut self,
        function_name: &str,
        json_args: &str,
    ) -> LibraryResult<String> {
        let mut store = self.store.lock().map_err(|e| {
            LibraryError::new("E601", format!("Failed to lock WASM store: {}", e))
        })?;
        
        // Create a new instance
        let instance = Instance::new(&mut *store, &self.module, &[]).map_err(|e| {
            LibraryError::new(
                "E602",
                format!("Failed to instantiate WASM module: {}", e),
            )
        })?;
        
        // Get the function export
        let func = instance
            .get_func(&mut *store, function_name)
            .ok_or_else(|| {
                LibraryError::new(
                    "E603",
                    format!("Function '{}' not found in WASM module", function_name),
                )
            })?;
        
        // Allocate memory for input string in WASM
        let alloc_func = instance
            .get_func(&mut *store, "allocate")
            .ok_or_else(|| {
                LibraryError::new("E604", "WASM module must export 'allocate' function")
            })?;
        
        let args_bytes = json_args.as_bytes();
        let args_len = args_bytes.len() as i32;
        
        // Allocate memory in WASM
        let mut results = vec![Val::I32(0)];
        alloc_func
            .call(&mut *store, &[Val::I32(args_len)], &mut results)
            .map_err(|e| {
                LibraryError::new("E605", format!("Failed to allocate WASM memory: {}", e))
            })?;
        
        let ptr = match results[0] {
            Val::I32(p) => p,
            _ => {
                return Err(LibraryError::new(
                    "E606",
                    "allocate function returned non-i32",
                ))
            }
        };
        
        // Write input data to WASM memory
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or_else(|| LibraryError::new("E607", "WASM module must export memory"))?;
        
        memory
            .write(&mut *store, ptr as usize, args_bytes)
            .map_err(|e| {
                LibraryError::new("E608", format!("Failed to write to WASM memory: {}", e))
            })?;
        
        // Call the function
        let mut call_results = vec![Val::I32(0), Val::I32(0)];
        func.call(
            &mut *store,
            &[Val::I32(ptr), Val::I32(args_len)],
            &mut call_results,
        )
        .map_err(|e| {
            LibraryError::new("E609", format!("WASM function call failed: {}", e))
        })?;
        
        // Extract result pointer and length
        let (result_ptr, result_len) = match (&call_results[0], &call_results[1]) {
            (Val::I32(p), Val::I32(l)) => (*p, *l),
            _ => {
                return Err(LibraryError::new(
                    "E610",
                    "WASM function returned invalid types",
                ))
            }
        };
        
        // Read result from WASM memory
        let mut result_bytes = vec![0u8; result_len as usize];
        memory
            .read(&mut *store, result_ptr as usize, &mut result_bytes)
            .map_err(|e| {
                LibraryError::new("E611", format!("Failed to read from WASM memory: {}", e))
            })?;
        
        // Convert to string
        String::from_utf8(result_bytes).map_err(|e| {
            LibraryError::new("E612", format!("Invalid UTF-8 from WASM: {}", e))
        })
    }
    
    /// Convert Value to JSON string for WASM.
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
    
    /// Convert JSON string to Value.
    fn json_to_value(json_str: &str) -> LibraryResult<Value> {
        let json: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
            LibraryError::new("E613", format!("Failed to parse JSON from WASM: {}", e))
        })?;
        
        Self::json_value_to_value(json)
    }
    
    /// Convert serde_json::Value to Value.
    fn json_value_to_value(json: serde_json::Value) -> LibraryResult<Value> {
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
            if let Some(b) = obj.get("Bool") {
                return Ok(Value::Bool(b.as_bool().unwrap_or(false)));
            }
            if let Some(list) = obj.get("List") {
                if let serde_json::Value::Array(items) = list {
                    let values: Result<Vec<_>, _> = items
                        .iter()
                        .map(|item| Self::json_value_to_value(item.clone()))
                        .collect();
                    return Ok(Value::List(values?));
                }
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
            "E614",
            format!("Cannot convert JSON to Value: {:?}", json),
        ))
    }
    
    /// Convert JSON string to CheckResult.
    fn json_to_check_result(json_str: &str) -> LibraryResult<CheckResult> {
        let json: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
            LibraryError::new("E615", format!("Failed to parse JSON from WASM: {}", e))
        })?;
        
        if let serde_json::Value::Object(obj) = &json {
            if obj.contains_key("Pass") || obj.get("result") == Some(&serde_json::json!("pass")) {
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
            if let Some(result) = obj.get("result") {
                match result.as_str() {
                    Some("pass") => return Ok(CheckResult::Pass),
                    Some("fail") => {
                        let msg = obj
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Check failed")
                            .to_string();
                        return Ok(CheckResult::Fail(msg));
                    }
                    Some("error") => {
                        let msg = obj
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Check error")
                            .to_string();
                        return Ok(CheckResult::Error(msg));
                    }
                    _ => {}
                }
            }
        }
        
        Err(LibraryError::new(
            "E616",
            format!("Cannot convert JSON to CheckResult: {}", json_str),
        ))
    }
}

impl Library for WasmLibrary {
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn call(
        &mut self,
        function: &str,
        args: Vec<Value>,
        workspace: &str,
    ) -> LibraryResult<Value> {
        // Serialize arguments to JSON
        let json_args: Vec<_> = args.iter().map(Self::value_to_json).collect();
        let request = serde_json::json!({
            "function": function,
            "args": json_args,
            "workspace": workspace
        });
        
        let request_str = serde_json::to_string(&request).map_err(|e| {
            LibraryError::new("E617", format!("Failed to serialize request: {}", e))
        })?;
        
        // Call WASM function
        let result_str = self.call_wasm_function("library_call", &request_str)?;
        
        // Parse result
        Self::json_to_value(&result_str)
    }
    
    fn check(
        &mut self,
        function: &str,
        args: Vec<Value>,
        workspace: &str,
    ) -> LibraryResult<CheckResult> {
        // Serialize arguments to JSON
        let json_args: Vec<_> = args.iter().map(Self::value_to_json).collect();
        let request = serde_json::json!({
            "function": function,
            "args": json_args,
            "workspace": workspace
        });
        
        let request_str = serde_json::to_string(&request).map_err(|e| {
            LibraryError::new("E618", format!("Failed to serialize request: {}", e))
        })?;
        
        // Call WASM function
        let result_str = self.call_wasm_function("library_check", &request_str)?;
        
        // Parse result
        Self::json_to_check_result(&result_str)
    }
}

/// Load a WASM library from a configuration.
pub fn load_wasm_library(config: WasmLibraryConfig) -> LibraryResult<WasmLibrary> {
    WasmLibrary::new(config)
}

/// Load a WASM library from a file path.
pub fn load_wasm_library_from_path(name: String, path: &Path) -> LibraryResult<WasmLibrary> {
    let config = WasmLibraryConfig {
        name,
        path: path.to_string_lossy().to_string(),
    };
    load_wasm_library(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_value_to_json() {
        let value = Value::String("test".to_string());
        let json = WasmLibrary::value_to_json(&value);
        assert_eq!(json, serde_json::json!({"String": "test"}));
    }
    
    #[test]
    fn test_json_value_to_value_string() {
        let json = serde_json::json!({"String": "hello"});
        let value = WasmLibrary::json_value_to_value(json).unwrap();
        match value {
            Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String value"),
        }
    }
}
