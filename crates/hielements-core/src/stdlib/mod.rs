//! Standard library modules for Hielements.

pub mod files;
pub mod rust;

use std::collections::HashMap;

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
pub trait Library {
    /// Get the library name.
    fn name(&self) -> &str;

    /// Call a function in the library.
    fn call(&self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value>;

    /// Execute a check function.
    fn check(&self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult>;
}

/// Registry of available libraries.
#[derive(Default)]
pub struct LibraryRegistry {
    libraries: HashMap<String, Box<dyn Library>>,
}

impl LibraryRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            libraries: HashMap::new(),
        };
        // Register built-in libraries
        registry.register(Box::new(files::FilesLibrary::new()));
        registry.register(Box::new(rust::RustLibrary::new()));
        registry
    }

    pub fn register(&mut self, library: Box<dyn Library>) {
        self.libraries.insert(library.name().to_string(), library);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Library> {
        self.libraries.get(name).map(|b| b.as_ref())
    }
}
