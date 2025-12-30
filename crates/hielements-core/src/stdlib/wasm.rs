//! WebAssembly library support for Hielements.
//!
//! Enables sandboxed, portable libraries through WebAssembly modules.

use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use wasmer::{imports, Instance, Module, Store, Value as WasmValue};

use super::external::{HielementsConfig, LibraryConfigEntry, WasmCapabilitiesConfig};
use super::{CheckResult, Library, LibraryError, LibraryResult, ScopeKind, Value};

/// Capabilities that can be granted to WASM libraries
#[derive(Debug, Clone)]
pub struct WasmCapabilities {
    /// Allow reading files from the workspace
    pub file_read: bool,
    /// Allow writing files to the workspace
    pub file_write: bool,
    /// Allow network access (not yet implemented)
    pub network: bool,
}

impl From<WasmCapabilitiesConfig> for WasmCapabilities {
    fn from(config: WasmCapabilitiesConfig) -> Self {
        Self {
            file_read: config.file_read,
            file_write: config.file_write,
            network: config.network,
        }
    }
}

impl Default for WasmCapabilities {
    fn default() -> Self {
        Self {
            file_read: true,
            file_write: false,
            network: false,
        }
    }
}

/// WASM library configuration
#[derive(Debug, Clone)]
pub struct WasmLibraryConfig {
    pub name: String,
    pub path: String,
    pub capabilities: WasmCapabilities,
}

/// A WASM-based library that runs in a sandboxed environment
pub struct WasmLibrary {
    name: String,
    store: Store,
    instance: Instance,
    /// Capability restrictions for this WASM library
    /// Stored for future use when host functions are implemented.
    /// Currently configured but not enforced due to missing host function integration.
    #[allow(dead_code)]
    capabilities: WasmCapabilities,
    workspace: Arc<Mutex<String>>,
}

impl WasmLibrary {
    /// Load a WASM library from a file
    pub fn load(config: WasmLibraryConfig) -> LibraryResult<Self> {
        let wasm_path = Path::new(&config.path);
        
        if !wasm_path.exists() {
            return Err(LibraryError::new(
                "E600",
                format!("WASM module not found: {}", config.path),
            ));
        }

        // Read WASM file
        let wasm_bytes = std::fs::read(wasm_path).map_err(|e| {
            LibraryError::new("E601", format!("Failed to read WASM module: {}", e))
        })?;

        // Create WASM store and module
        let mut store = Store::default();
        let module = Module::new(&store, wasm_bytes).map_err(|e| {
            LibraryError::new("E602", format!("Failed to compile WASM module: {}", e))
        })?;

        // Workspace for file operations
        let workspace = Arc::new(Mutex::new(String::new()));

        // For now, create a simple import object without host functions
        // TODO: Add host functions for file system access with capabilities
        let import_object = imports! {};

        // Instantiate WASM module
        let instance = Instance::new(&mut store, &module, &import_object).map_err(|e| {
            LibraryError::new("E603", format!("Failed to instantiate WASM module: {}", e))
        })?;

        Ok(Self {
            name: config.name,
            store,
            instance,
            capabilities: config.capabilities,
            workspace,
        })
    }

    /// Call a WASM function
    /// 
    /// Note: Currently uses JSON serialization for simplicity. This introduces some overhead
    /// compared to external processes. Future optimization: use binary protocol or direct
    /// memory passing with WASM Component Model for better performance.
    fn call_wasm_function(
        &mut self,
        function_name: &str,
        function: &str,
        args: Vec<Value>,
        workspace: &str,
    ) -> LibraryResult<Vec<WasmValue>> {
        // Update workspace for host functions
        *self.workspace.lock().unwrap() = workspace.to_string();

        // For now, we use a simple approach: serialize arguments to JSON
        // and pass them as a string pointer to WASM
        let args_json = serde_json::to_string(&SerializableCallParams {
            function: function.to_string(),
            args: args.into_iter().map(value_to_serializable).collect(),
            workspace: workspace.to_string(),
        })
        .map_err(|e| LibraryError::new("E604", format!("Failed to serialize arguments: {}", e)))?;

        // Get the WASM function
        let wasm_fn = self
            .instance
            .exports
            .get_function(function_name)
            .map_err(|e| {
                LibraryError::new(
                    "E605",
                    format!("WASM function '{}' not found: {}", function_name, e),
                )
            })?;

        // Allocate memory for the JSON string in WASM
        let alloc_fn = self
            .instance
            .exports
            .get_function("alloc")
            .map_err(|e| LibraryError::new("E606", format!("WASM alloc function not found: {}", e)))?;

        let json_bytes = args_json.as_bytes();
        let len = json_bytes.len() as i32;

        // Allocate memory
        let ptr = alloc_fn
            .call(&mut self.store, &[WasmValue::I32(len)])
            .map_err(|e| LibraryError::new("E607", format!("Failed to allocate WASM memory: {}", e)))?;

        let ptr_value = match &ptr[0] {
            WasmValue::I32(p) => *p,
            _ => return Err(LibraryError::new("E608", "Invalid pointer from alloc")),
        };

        // Write JSON to WASM memory
        let memory = self
            .instance
            .exports
            .get_memory("memory")
            .map_err(|e| LibraryError::new("E609", format!("WASM memory not found: {}", e)))?;

        let memory_view = memory.view(&self.store);
        // TODO: Optimize using bulk memory operations (e.g., write_slice) for better performance
        for (i, byte) in json_bytes.iter().enumerate() {
            memory_view
                .write_u8(ptr_value as u64 + i as u64, *byte)
                .map_err(|e| LibraryError::new("E610", format!("Failed to write to WASM memory: {}", e)))?;
        }

        // Call the WASM function with pointer and length
        let result = wasm_fn
            .call(&mut self.store, &[WasmValue::I32(ptr_value), WasmValue::I32(len)])
            .map_err(|e| LibraryError::new("E611", format!("WASM function call failed: {}", e)))?;

        Ok(result.to_vec())
    }

    /// Read result string from WASM memory
    fn read_wasm_string(&self, ptr: i32, len: i32) -> LibraryResult<String> {
        let memory = self
            .instance
            .exports
            .get_memory("memory")
            .map_err(|e| LibraryError::new("E612", format!("WASM memory not found: {}", e)))?;

        let memory_view = memory.view(&self.store);
        let mut bytes = vec![0u8; len as usize];
        
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = memory_view
                .read_u8(ptr as u64 + i as u64)
                .map_err(|e| LibraryError::new("E613", format!("Failed to read from WASM memory: {}", e)))?;
        }

        String::from_utf8(bytes)
            .map_err(|e| LibraryError::new("E614", format!("Invalid UTF-8 from WASM: {}", e)))
    }
}

impl Library for WasmLibrary {
    fn name(&self) -> &str {
        &self.name
    }

    fn call(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value> {
        let result = self.call_wasm_function("library_call", function, args, workspace)?;

        // Parse result (ptr, len) and read JSON string
        if result.len() != 2 {
            return Err(LibraryError::new("E615", "Invalid result from WASM function"));
        }

        let (ptr, len) = match (&result[0], &result[1]) {
            (WasmValue::I32(p), WasmValue::I32(l)) => (*p, *l),
            _ => return Err(LibraryError::new("E616", "Invalid result types from WASM")),
        };

        let json_str = self.read_wasm_string(ptr, len)?;
        let serializable: SerializableValue = serde_json::from_str(&json_str)
            .map_err(|e| LibraryError::new("E617", format!("Failed to parse WASM result: {}", e)))?;

        serializable_to_value(serializable)
    }

    fn check(
        &mut self,
        function: &str,
        args: Vec<Value>,
        workspace: &str,
    ) -> LibraryResult<CheckResult> {
        let result = self.call_wasm_function("library_check", function, args, workspace)?;

        // Parse result (ptr, len) and read JSON string
        if result.len() != 2 {
            return Err(LibraryError::new("E618", "Invalid result from WASM function"));
        }

        let (ptr, len) = match (&result[0], &result[1]) {
            (WasmValue::I32(p), WasmValue::I32(l)) => (*p, *l),
            _ => return Err(LibraryError::new("E619", "Invalid result types from WASM")),
        };

        let json_str = self.read_wasm_string(ptr, len)?;
        let result: SerializableCheckResult = serde_json::from_str(&json_str)
            .map_err(|e| LibraryError::new("E620", format!("Failed to parse WASM check result: {}", e)))?;

        Ok(result.into())
    }
}

/// Load WASM libraries from configuration file
pub fn load_wasm_libraries(config_path: &Path) -> LibraryResult<Vec<WasmLibrary>> {
    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(config_path).map_err(|e| {
        LibraryError::new("E621", format!("Failed to read config file: {}", e))
    })?;

    let config: HielementsConfig = toml::from_str(&content).map_err(|e| {
        LibraryError::new("E622", format!("Failed to parse config file: {}", e))
    })?;

    let mut libraries = Vec::new();
    for (name, entry) in config.libraries {
        // Only load WASM libraries
        if let LibraryConfigEntry::Wasm { path, capabilities, .. } = entry {
            let wasm_config = WasmLibraryConfig {
                name,
                path,
                capabilities: capabilities.into(),
            };
            libraries.push(WasmLibrary::load(wasm_config)?);
        }
    }

    Ok(libraries)
}

/// Load WASM libraries from a workspace directory
pub fn load_workspace_wasm_libraries(workspace: &str) -> LibraryResult<Vec<WasmLibrary>> {
    let config_path = Path::new(workspace).join("hielements.toml");
    load_wasm_libraries(&config_path)
}

// TODO: Implement host functions for capability-based file system access
// This will require:
// 1. Properly typed host functions using wasmer's HostFunction trait
// 2. Memory management for passing strings between WASM and host
// 3. Error handling and security checks based on capabilities
//
// Example structure:
// #[derive(Clone)]
// struct HostEnv {
//     workspace: Arc<Mutex<String>>,
//     capabilities: WasmCapabilities,
// }
//
// fn host_read_file(env: FunctionEnv<HostEnv>, path_ptr: i32, path_len: i32) -> i32 {
//     // Read file from workspace with capability check
// }

// Serialization types for WASM communication

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableCallParams {
    function: String,
    args: Vec<SerializableValue>,
    workspace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum SerializableValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<SerializableValue>),
    Scope {
        kind: SerializableScopeKind,
        paths: Vec<String>,
        resolved: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SerializableScopeKind {
    File(String),
    Folder(String),
    Glob(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum SerializableCheckResult {
    Pass,
    Fail { message: String },
    Error { message: String },
}

impl From<SerializableCheckResult> for CheckResult {
    fn from(result: SerializableCheckResult) -> Self {
        match result {
            SerializableCheckResult::Pass => CheckResult::Pass,
            SerializableCheckResult::Fail { message } => CheckResult::Fail(message),
            SerializableCheckResult::Error { message } => CheckResult::Error(message),
        }
    }
}

fn value_to_serializable(value: Value) -> SerializableValue {
    match value {
        Value::Null => SerializableValue::Null,
        Value::Bool(b) => SerializableValue::Bool(b),
        Value::Int(i) => SerializableValue::Int(i),
        Value::Float(f) => SerializableValue::Float(f),
        Value::String(s) => SerializableValue::String(s),
        Value::List(items) => {
            SerializableValue::List(items.into_iter().map(value_to_serializable).collect())
        }
        Value::Scope(scope) => {
            let kind = match scope.kind {
                ScopeKind::File(s) => SerializableScopeKind::File(s),
                ScopeKind::Folder(s) => SerializableScopeKind::Folder(s),
                ScopeKind::Glob(s) => SerializableScopeKind::Glob(s),
            };
            SerializableValue::Scope {
                kind,
                paths: scope.paths,
                resolved: scope.resolved,
            }
        }
        Value::ConnectionPoint(_) => SerializableValue::Null, // Can't serialize connection points
    }
}

fn serializable_to_value(serializable: SerializableValue) -> LibraryResult<Value> {
    match serializable {
        SerializableValue::Null => Ok(Value::Null),
        SerializableValue::Bool(b) => Ok(Value::Bool(b)),
        SerializableValue::Int(i) => Ok(Value::Int(i)),
        SerializableValue::Float(f) => Ok(Value::Float(f)),
        SerializableValue::String(s) => Ok(Value::String(s)),
        SerializableValue::List(items) => {
            let values: Result<Vec<_>, _> =
                items.into_iter().map(serializable_to_value).collect();
            Ok(Value::List(values?))
        }
        SerializableValue::Scope { kind, paths, resolved } => {
            let scope_kind = match kind {
                SerializableScopeKind::File(s) => ScopeKind::File(s),
                SerializableScopeKind::Folder(s) => ScopeKind::Folder(s),
                SerializableScopeKind::Glob(s) => ScopeKind::Glob(s),
            };
            Ok(Value::Scope(super::Scope {
                kind: scope_kind,
                paths,
                resolved,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_capabilities_default() {
        let caps = WasmCapabilities::default();
        assert!(caps.file_read);
        assert!(!caps.file_write);
        assert!(!caps.network);
    }

    #[test]
    fn test_value_serialization() {
        let value = Value::String("test".to_string());
        let serializable = value_to_serializable(value);
        match serializable {
            SerializableValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_value_deserialization() {
        let serializable = SerializableValue::Int(42);
        let value = serializable_to_value(serializable).unwrap();
        match value {
            Value::Int(i) => assert_eq!(i, 42),
            _ => panic!("Expected Int"),
        }
    }
}
