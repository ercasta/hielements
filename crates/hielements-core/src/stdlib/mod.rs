//! Standard library modules for Hielements.

pub mod external;
pub mod files;
pub mod rust;
pub mod wasm;

use std::collections::HashMap;
use std::path::Path;

pub use external::{ExternalLibrary, ExternalLibraryConfig, load_external_libraries, load_workspace_libraries};
pub use wasm::{WasmLibrary, WasmLibraryConfig, WasmCapabilities, load_wasm_library};

/// Result type for library function calls.
pub type LibraryResult<T> = Result<T, LibraryError>;

/// Error type for library operations.
#[derive(Debug, Clone)]
pub struct LibraryError {
    pub message: String,
    pub code: String,
}

impl LibraryError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

/// A value that can be passed to/from library functions.
#[derive(Debug, Clone)]
pub enum Value {
    /// Null/none value
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// List of values
    List(Vec<Value>),
    /// A scope (set of files/paths)
    Scope(Scope),
    /// A connection point
    ConnectionPoint(ConnectionPoint),
}

impl Value {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            Value::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_scope(&self) -> Option<&Scope> {
        match self {
            Value::Scope(s) => Some(s),
            _ => None,
        }
    }
}

/// A scope representing a set of files/paths.
#[derive(Debug, Clone)]
pub struct Scope {
    /// Type of scope (file, folder, glob, etc.)
    pub kind: ScopeKind,
    /// Matched paths
    pub paths: Vec<String>,
    /// Whether the scope has been resolved
    pub resolved: bool,
}

impl Scope {
    pub fn new(kind: ScopeKind) -> Self {
        Self {
            kind,
            paths: Vec::new(),
            resolved: false,
        }
    }

    pub fn with_paths(mut self, paths: Vec<String>) -> Self {
        self.paths = paths;
        self.resolved = true;
        self
    }
}

/// Type of scope selector.
#[derive(Debug, Clone)]
pub enum ScopeKind {
    File(String),
    Folder(String),
    Glob(String),
}

/// A connection point.
#[derive(Debug, Clone)]
pub struct ConnectionPoint {
    pub name: String,
    pub kind: String,
    pub data: HashMap<String, Value>,
}

/// Check result.
#[derive(Debug, Clone)]
pub enum CheckResult {
    /// Check passed
    Pass,
    /// Check failed with message
    Fail(String),
    /// Check could not be evaluated
    Error(String),
}

impl CheckResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, CheckResult::Pass)
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, CheckResult::Fail(_))
    }
}

/// Trait for library modules.
/// 
/// Libraries provide selectors (via `call`) and checks (via `check`).
/// Both methods take `&mut self` to support libraries that need to manage state,
/// such as external process libraries that maintain a subprocess connection.
pub trait Library {
    /// Get the library name.
    fn name(&self) -> &str;

    /// Call a function in the library (typically a selector).
    fn call(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value>;

    /// Execute a check function.
    fn check(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult>;
}

/// Registry of available libraries.
#[derive(Default)]
pub struct LibraryRegistry {
    libraries: HashMap<String, Box<dyn Library>>,
}

impl LibraryRegistry {
    /// Create a new registry with built-in libraries.
    pub fn new() -> Self {
        let mut registry = Self {
            libraries: HashMap::new(),
        };
        // Register built-in libraries
        registry.register(Box::new(files::FilesLibrary::new()));
        registry.register(Box::new(rust::RustLibrary::new()));
        registry
    }

    /// Create a new registry and load external libraries from a workspace.
    pub fn with_workspace(workspace: &str) -> Self {
        let mut registry = Self::new();
        registry.load_from_workspace(workspace);
        registry
    }

    /// Register a library.
    pub fn register(&mut self, library: Box<dyn Library>) {
        self.libraries.insert(library.name().to_string(), library);
    }

    /// Get an immutable reference to a library.
    pub fn get(&self, name: &str) -> Option<&dyn Library> {
        self.libraries.get(name).map(|b| b.as_ref())
    }

    /// Get a mutable reference to a library.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn Library>> {
        self.libraries.get_mut(name)
    }

    /// Load external libraries from a workspace configuration file.
    /// 
    /// Looks for `hielements.toml` in the workspace root and loads both
    /// external process libraries and WASM libraries.
    pub fn load_from_workspace(&mut self, workspace: &str) {
        let config_path = Path::new(workspace).join("hielements.toml");
        if config_path.exists() {
            // Load using the unified loader
            if let Ok(libs) = load_all_libraries(&config_path, workspace) {
                for lib in libs {
                    self.register(lib);
                }
            }
        }
    }

    /// Check if a library is registered.
    pub fn has(&self, name: &str) -> bool {
        self.libraries.contains_key(name)
    }

    /// Get the names of all registered libraries.
    pub fn names(&self) -> Vec<&str> {
        self.libraries.keys().map(|s| s.as_str()).collect()
    }
}

/// Load all libraries (external and WASM) from a configuration file.
pub fn load_all_libraries(config_path: &Path, workspace: &str) -> LibraryResult<Vec<Box<dyn Library>>> {
    if !config_path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(config_path).map_err(|e| {
        LibraryError::new("E700", format!("Failed to read config file: {}", e))
    })?;

    let config: external::HielementsConfig = toml::from_str(&content).map_err(|e| {
        LibraryError::new("E701", format!("Failed to parse config file: {}", e))
    })?;

    let mut libraries: Vec<Box<dyn Library>> = Vec::new();
    
    for (name, entry) in config.libraries {
        match entry {
            external::LibraryConfigEntry::External { executable, args, .. } => {
                // Load as external process library
                libraries.push(Box::new(ExternalLibrary::new(ExternalLibraryConfig {
                    name,
                    executable,
                    args,
                })));
            }
            external::LibraryConfigEntry::Wasm { path, capabilities, .. } => {
                // Load as WASM library
                match wasm::load_wasm_library(
                    wasm::WasmLibraryConfig {
                        name,
                        path,
                        capabilities,
                    },
                    workspace,
                ) {
                    Ok(lib) => libraries.push(Box::new(lib)),
                    Err(e) => {
                        // Log error but continue loading other libraries
                        eprintln!("Warning: Failed to load WASM library: {}", e.message);
                    }
                }
            }
        }
    }

    Ok(libraries)
}
