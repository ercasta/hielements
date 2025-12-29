//! External library support for Hielements.
//!
//! Enables user-defined libraries through external processes that communicate
//! via JSON-RPC over stdio.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};

use serde::{Deserialize, Serialize};

use super::{CheckResult, Library, LibraryError, LibraryResult, Value};

/// Configuration for an external library.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalLibraryConfig {
    /// Name of the library as it will be referenced in .hie files
    pub name: String,
    /// Path to the executable
    pub executable: String,
    /// Optional arguments to pass to the executable
    #[serde(default)]
    pub args: Vec<String>,
}

/// Configuration file structure for hielements.toml
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct HielementsConfig {
    /// External library configurations
    #[serde(default)]
    pub libraries: HashMap<String, ExternalLibraryConfigEntry>,
}

/// Entry in the libraries configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalLibraryConfigEntry {
    /// Path to the executable
    pub executable: String,
    /// Optional arguments to pass to the executable
    #[serde(default)]
    pub args: Vec<String>,
}

/// JSON-RPC request structure
#[derive(Debug, Clone, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    method: String,
    params: serde_json::Value,
    id: u64,
}

/// JSON-RPC response structure
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(default)]
    result: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
    id: u64,
}

/// JSON-RPC error structure
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

/// Library metadata returned by external process
#[derive(Debug, Clone, Deserialize)]
pub struct LibraryMetadata {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub functions: Vec<String>,
    #[serde(default)]
    pub checks: Vec<String>,
}

/// An external library that communicates via JSON-RPC.
pub struct ExternalLibrary {
    config: ExternalLibraryConfig,
    process: Option<Child>,
    request_id: u64,
}

impl ExternalLibrary {
    /// Create a new external library from configuration.
    pub fn new(config: ExternalLibraryConfig) -> Self {
        Self {
            config,
            process: None,
            request_id: 0,
        }
    }

    /// Start the external process if not already running.
    fn ensure_process(&mut self) -> LibraryResult<()> {
        if self.process.is_some() {
            return Ok(());
        }

        let mut cmd = Command::new(&self.config.executable);
        cmd.args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        match cmd.spawn() {
            Ok(child) => {
                self.process = Some(child);
                Ok(())
            }
            Err(e) => Err(LibraryError::new(
                "E500",
                format!(
                    "Failed to start external library '{}': {}",
                    self.config.name, e
                ),
            )),
        }
    }

    /// Send a JSON-RPC request and receive a response.
    fn send_request(&mut self, method: &str, params: serde_json::Value) -> LibraryResult<serde_json::Value> {
        self.ensure_process()?;

        let process = self.process.as_mut().unwrap();
        self.request_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            method: method.to_string(),
            params,
            id: self.request_id,
        };

        // Serialize request
        let request_json = serde_json::to_string(&request).map_err(|e| {
            LibraryError::new("E501", format!("Failed to serialize request: {}", e))
        })?;

        // Write request to stdin
        let stdin = process.stdin.as_mut().ok_or_else(|| {
            LibraryError::new("E502", "Failed to access stdin of external process")
        })?;
        
        writeln!(stdin, "{}", request_json).map_err(|e| {
            LibraryError::new("E503", format!("Failed to write to external process: {}", e))
        })?;
        stdin.flush().map_err(|e| {
            LibraryError::new("E503", format!("Failed to flush to external process: {}", e))
        })?;

        // Read response from stdout
        let stdout = process.stdout.as_mut().ok_or_else(|| {
            LibraryError::new("E504", "Failed to access stdout of external process")
        })?;
        
        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).map_err(|e| {
            LibraryError::new("E505", format!("Failed to read from external process: {}", e))
        })?;

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&response_line).map_err(|e| {
            LibraryError::new("E506", format!("Failed to parse response: {}. Response was: {}", e, response_line.trim()))
        })?;

        // Check for errors
        if let Some(error) = response.error {
            return Err(LibraryError::new(
                format!("E{}", error.code),
                error.message,
            ));
        }

        response.result.ok_or_else(|| {
            LibraryError::new("E507", "External process returned empty result")
        })
    }

    /// Convert Value to serde_json::Value for serialization.
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

    /// Convert serde_json::Value back to Value.
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

        // Handle tagged values (objects with type keys)
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
                    let values: Result<Vec<_>, _> = items.iter()
                        .map(|item| Self::json_to_value(item.clone()))
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
                                super::ScopeKind::Folder(s.as_str().unwrap_or_default().to_string())
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

                    let paths = scope.get("paths")
                        .and_then(|p| p.as_array())
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect())
                        .unwrap_or_default();

                    let resolved = scope.get("resolved")
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

        Err(LibraryError::new("E508", format!("Cannot convert JSON to Value: {:?}", json)))
    }

    /// Convert JSON to CheckResult.
    fn json_to_check_result(json: serde_json::Value) -> LibraryResult<CheckResult> {
        if let serde_json::Value::Object(obj) = &json {
            if obj.contains_key("Pass") || obj.get("result") == Some(&serde_json::json!("pass")) {
                return Ok(CheckResult::Pass);
            }
            if let Some(msg) = obj.get("Fail") {
                return Ok(CheckResult::Fail(msg.as_str().unwrap_or_default().to_string()));
            }
            if let Some(msg) = obj.get("Error") {
                return Ok(CheckResult::Error(msg.as_str().unwrap_or_default().to_string()));
            }
            if let Some(result) = obj.get("result") {
                match result.as_str() {
                    Some("pass") => return Ok(CheckResult::Pass),
                    Some("fail") => {
                        let msg = obj.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Check failed")
                            .to_string();
                        return Ok(CheckResult::Fail(msg));
                    }
                    Some("error") => {
                        let msg = obj.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Check error")
                            .to_string();
                        return Ok(CheckResult::Error(msg));
                    }
                    _ => {}
                }
            }
        }
        
        // Handle simple string response
        if let serde_json::Value::String(s) = &json {
            match s.to_lowercase().as_str() {
                "pass" | "ok" | "true" => return Ok(CheckResult::Pass),
                "fail" | "false" => return Ok(CheckResult::Fail("Check failed".to_string())),
                _ => return Ok(CheckResult::Fail(s.clone())),
            }
        }

        Err(LibraryError::new("E509", format!("Cannot convert JSON to CheckResult: {:?}", json)))
    }
}

impl Drop for ExternalLibrary {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            // Try to terminate the process gracefully
            let _ = process.kill();
        }
    }
}

impl Library for ExternalLibrary {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn call(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value> {
        let json_args: Vec<_> = args.iter().map(Self::value_to_json).collect();
        let params = serde_json::json!({
            "function": function,
            "args": json_args,
            "workspace": workspace
        });

        let result = self.send_request("library.call", params)?;
        Self::json_to_value(result)
    }

    fn check(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult> {
        let json_args: Vec<_> = args.iter().map(Self::value_to_json).collect();
        let params = serde_json::json!({
            "function": function,
            "args": json_args,
            "workspace": workspace
        });

        let result = self.send_request("library.check", params)?;
        Self::json_to_check_result(result)
    }
}

/// Load external libraries from a configuration file.
pub fn load_external_libraries(config_path: &Path) -> LibraryResult<Vec<ExternalLibrary>> {
    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(config_path).map_err(|e| {
        LibraryError::new("E510", format!("Failed to read config file: {}", e))
    })?;

    let config: HielementsConfig = toml::from_str(&content).map_err(|e| {
        LibraryError::new("E511", format!("Failed to parse config file: {}", e))
    })?;

    let mut libraries = Vec::new();
    for (name, entry) in config.libraries {
        libraries.push(ExternalLibrary::new(ExternalLibraryConfig {
            name,
            executable: entry.executable,
            args: entry.args,
        }));
    }

    Ok(libraries)
}

/// Load external libraries from a workspace directory.
/// Looks for hielements.toml in the workspace root.
pub fn load_workspace_libraries(workspace: &str) -> LibraryResult<Vec<ExternalLibrary>> {
    let config_path = Path::new(workspace).join("hielements.toml");
    load_external_libraries(&config_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let toml_content = r#"
[libraries]
python = { executable = "hielements-python", args = [] }
docker = { executable = "hielements-docker" }
"#;
        let config: HielementsConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.libraries.len(), 2);
        assert!(config.libraries.contains_key("python"));
        assert!(config.libraries.contains_key("docker"));
    }

    #[test]
    fn test_value_to_json_string() {
        let value = Value::String("test".to_string());
        let json = ExternalLibrary::value_to_json(&value);
        assert_eq!(json, serde_json::json!({"String": "test"}));
    }

    #[test]
    fn test_value_to_json_int() {
        let value = Value::Int(42);
        let json = ExternalLibrary::value_to_json(&value);
        assert_eq!(json, serde_json::json!({"Int": 42}));
    }

    #[test]
    fn test_json_to_value_string() {
        let json = serde_json::json!({"String": "hello"});
        let value = ExternalLibrary::json_to_value(json).unwrap();
        match value {
            Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String value"),
        }
    }

    #[test]
    fn test_json_to_check_result_pass() {
        let json = serde_json::json!({"Pass": null});
        let result = ExternalLibrary::json_to_check_result(json).unwrap();
        assert!(result.is_pass());
    }

    #[test]
    fn test_json_to_check_result_fail() {
        let json = serde_json::json!({"Fail": "Something went wrong"});
        let result = ExternalLibrary::json_to_check_result(json).unwrap();
        assert!(result.is_fail());
    }
}
