//! Python language library for Hielements.
//!
//! Provides selectors and checks for Python code analysis.
//! Uses regex-based pattern matching for common Python constructs.
//!
//! ## Limitations
//! - Text-based analysis (not AST-based) means patterns in comments or strings will match
//! - For production use, consider using AST-based tools like `ast` module or external analyzers
//! - Word boundaries are enforced to avoid substring false positives (e.g., 'os' won't match 'osmesa')

use std::fs;
use std::path::PathBuf;

use walkdir::WalkDir;

use super::{CheckResult, Library, LibraryError, LibraryResult, Scope, ScopeKind, Value};

/// Directories to exclude when scanning for Python files.
const EXCLUDED_DIRS: &[&str] = &[
    ".git", 
    "node_modules", 
    "__pycache__", 
    ".venv", 
    "venv", 
    ".tox", 
    ".pytest_cache",
    ".mypy_cache",
    "dist",
    "build",
    "*.egg-info",
];

/// Check if a directory name should be excluded from scanning.
fn is_excluded_dir(name: &str) -> bool {
    EXCLUDED_DIRS.iter().any(|&excluded| {
        if excluded.ends_with('*') {
            name.starts_with(&excluded[..excluded.len() - 1])
        } else {
            name == excluded
        }
    })
}

/// Walk a directory and collect all .py files, skipping excluded directories.
fn find_python_files(base_path: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();
    
    let walker = WalkDir::new(base_path)
        .into_iter()
        .filter_entry(|e| {
            // Skip excluded directories entirely (don't descend into them)
            if e.file_type().is_dir() {
                if let Some(name) = e.file_name().to_str() {
                    return !is_excluded_dir(name);
                }
            }
            true
        });
    
    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map(|e| e == "py").unwrap_or(false) {
            files.push(path.to_path_buf());
        }
    }
    
    files
}

/// The Python library.
pub struct PythonLibrary;

impl PythonLibrary {
    pub fn new() -> Self {
        Self
    }

    /// Select a Python module by name (e.g., "orders" or "orders.api").
    fn module_selector(&self, module_path: &str, workspace: &str) -> LibraryResult<Value> {
        // Convert module path to file path patterns
        // "orders.api" -> "orders/api.py" or "orders/api/__init__.py"
        let parts: Vec<&str> = module_path.split('.').collect();
        let last_part = parts.last().unwrap_or(&"");
        
        let scope = Scope::new(ScopeKind::File(module_path.to_string()));
        let mut found_paths = Vec::new();
        
        // Look for matching Python files
        for path in find_python_files(workspace) {
            let path_str = path.to_string_lossy();
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            
            // Check if this file matches the module name
            // Match "module_name.py" or "module_name/__init__.py"
            if file_stem == *last_part || (file_stem == "__init__" && path_str.contains(last_part)) {
                // Verify the full path matches if it's a dotted path
                if parts.len() > 1 {
                    let expected_path = parts.join("/");
                    if path_str.contains(&expected_path) {
                        found_paths.push(path_str.to_string());
                    }
                } else {
                    found_paths.push(path_str.to_string());
                }
            }
        }
        
        // Deduplicate
        found_paths.sort();
        found_paths.dedup();
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Python function by name.
    fn function_selector(&self, func_name: &str, workspace: &str) -> LibraryResult<Value> {
        let scope = Scope::new(ScopeKind::File(format!("fn:{}", func_name)));
        let mut found_paths = Vec::new();
        
        // Use regex for more precise matching with word boundaries
        let pattern = format!(r"\bdef\s+{}[(]|\basync\s+def\s+{}[(]", regex::escape(func_name), regex::escape(func_name));
        let re = regex::Regex::new(&pattern).ok();
        
        for entry in find_python_files(workspace) {
            if let Ok(content) = fs::read_to_string(&entry) {
                if let Some(ref regex) = re {
                    if regex.is_match(&content) {
                        found_paths.push(entry.to_string_lossy().to_string());
                    }
                } else {
                    // Fallback to exact pattern matching
                    if content.contains(&format!("def {}(", func_name))
                        || content.contains(&format!("async def {}(", func_name)) {
                        found_paths.push(entry.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Python class by name.
    fn class_selector(&self, class_name: &str, workspace: &str) -> LibraryResult<Value> {
        let scope = Scope::new(ScopeKind::File(format!("class:{}", class_name)));
        let mut found_paths = Vec::new();
        
        // Use regex for more precise matching with word boundaries
        let pattern = format!(r"\bclass\s+{}[:(]", regex::escape(class_name));
        let re = regex::Regex::new(&pattern).ok();
        
        for entry in find_python_files(workspace) {
            if let Ok(content) = fs::read_to_string(&entry) {
                if let Some(ref regex) = re {
                    if regex.is_match(&content) {
                        found_paths.push(entry.to_string_lossy().to_string());
                    }
                } else {
                    // Fallback to exact pattern matching
                    if content.contains(&format!("class {}:", class_name))
                        || content.contains(&format!("class {}(", class_name)) {
                        found_paths.push(entry.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    // Check functions
    
    /// Check that a scope imports a module (import checks).
    /// Checks for "import module_name" or "from module_name import ..."
    fn check_imports(&self, scope: &Scope, module_name: &str, _workspace: &str) -> CheckResult {
        // Use regex for more precise matching with word boundaries
        let patterns = [
            format!(r"\bimport\s+{}\b", regex::escape(module_name)),
            format!(r"\bfrom\s+{}\s+import\b", regex::escape(module_name)),
            format!(r"\bfrom\s+{}\..*\s+import\b", regex::escape(module_name)),
        ];
        
        for path in &scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                for pattern in &patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if re.is_match(&content) {
                            return CheckResult::Pass;
                        }
                    }
                }
            }
        }
        
        CheckResult::Fail(format!("Scope does not import '{}'", module_name))
    }

    /// Check that a scope does NOT import a module (negative import check).
    fn check_no_import(&self, scope: &Scope, module_name: &str, workspace: &str) -> CheckResult {
        match self.check_imports(scope, module_name, workspace) {
            CheckResult::Pass => CheckResult::Fail(format!("Scope imports '{}' but should not", module_name)),
            CheckResult::Fail(_) => CheckResult::Pass,
            CheckResult::Error(e) => CheckResult::Error(e),
        }
    }

    /// Check that any function in a scope returns a given type.
    /// Looks for "-> TypeName" in function definitions.
    fn check_returns_type(&self, scope: &Scope, type_name: &str, _workspace: &str) -> CheckResult {
        for path in &scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                // Match "-> TypeName:" or "-> TypeName)" in function signatures
                // This handles both regular and async functions
                if content.contains(&format!("-> {}:", type_name))
                    || content.contains(&format!("-> {}):", type_name)) {
                    return CheckResult::Pass;
                }
                
                // Also check for generic types like List[TypeName], Optional[TypeName]
                if content.contains(&format!("-> ") ) && content.contains(type_name) {
                    // Simple heuristic: if "-> " exists and type_name appears after it
                    let lines: Vec<&str> = content.lines().collect();
                    for line in lines {
                        if line.contains("def ") && line.contains("-> ") {
                            let after_arrow = line.split("-> ").nth(1);
                            if let Some(return_part) = after_arrow {
                                if return_part.contains(type_name) {
                                    return CheckResult::Pass;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        CheckResult::Fail(format!("No function returns type '{}'", type_name))
    }

    /// Check that a specific function returns a given type.
    fn check_function_returns_type(&self, scope: &Scope, func_name: &str, type_name: &str, _workspace: &str) -> CheckResult {
        // Use regex for precise function matching
        let func_pattern = format!(r"\b(async\s+)?def\s+{}[(]", regex::escape(func_name));
        let func_re = regex::Regex::new(&func_pattern).ok();
        
        for path in &scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                let lines: Vec<&str> = content.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    // Check if this line defines the function using regex
                    let is_function_def = if let Some(ref re) = func_re {
                        re.is_match(line)
                    } else {
                        line.contains(&format!("def {}(", func_name)) 
                            || line.contains(&format!("async def {}(", func_name))
                    };
                    
                    if is_function_def {
                        // Check if return type is on this line or next lines (for multi-line signatures)
                        let mut check_lines: Vec<&str> = vec![line];
                        for j in (i + 1)..std::cmp::min(i + 5, lines.len()) {
                            check_lines.push(lines[j]);
                            if lines[j].contains(':') {
                                break;
                            }
                        }
                        
                        let signature = check_lines.join(" ");
                        if signature.contains(&format!("-> {}:", type_name))
                            || signature.contains(&format!("-> {}):", type_name))
                            || (signature.contains("-> ") && signature.split("-> ").nth(1).map_or(false, |s: &str| s.contains(type_name))) {
                            return CheckResult::Pass;
                        }
                    }
                }
            }
        }
        
        CheckResult::Fail(format!("Function '{}' does not return type '{}'", func_name, type_name))
    }

    /// Check that a scope calls something from another scope or module.
    /// Looks for function calls like "module.function()" or "object.method()"
    fn check_calls(&self, scope: &Scope, target: &str, _workspace: &str) -> CheckResult {
        // Use regex for more precise matching with word boundaries
        let patterns = [
            format!(r"\b{}[(]", regex::escape(target)),  // Direct call: target(
            format!(r"\b{}\.", regex::escape(target)),   // Module/object access: target.
        ];
        
        for path in &scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                for pattern in &patterns {
                    if let Ok(re) = regex::Regex::new(pattern) {
                        if re.is_match(&content) {
                            return CheckResult::Pass;
                        }
                    }
                }
            }
        }
        
        CheckResult::Fail(format!("Scope does not call '{}'", target))
    }

    /// Check that a scope calls a specific function in another module.
    /// Looks for "module.function()" calls.
    fn check_calls_function(&self, scope: &Scope, module_name: &str, func_name: &str, _workspace: &str) -> CheckResult {
        // Use regex for precise matching with word boundaries
        let pattern = format!(r"\b{}\.\s*{}[(]", regex::escape(module_name), regex::escape(func_name));
        
        for path in &scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(re) = regex::Regex::new(&pattern) {
                    if re.is_match(&content) {
                        return CheckResult::Pass;
                    }
                }
            }
        }
        
        CheckResult::Fail(format!("Scope does not call '{}.{}'", module_name, func_name))
    }

    /// Check that source scope calls something in target scope.
    /// This is a behavioral check between two scopes.
    fn check_calls_scope(&self, source_scope: &Scope, target_scope: &Scope, _workspace: &str) -> CheckResult {
        // Extract identifiers from target scope (functions, classes)
        let mut target_identifiers = Vec::new();
        
        for path in &target_scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                // Extract function names
                for line in content.lines() {
                    if line.contains("def ") {
                        if let Some(func_part) = line.split("def ").nth(1) {
                            if let Some(func_name) = func_part.split('(').next() {
                                target_identifiers.push(func_name.trim().to_string());
                            }
                        }
                    }
                    
                    // Extract class names
                    if line.contains("class ") {
                        if let Some(class_part) = line.split("class ").nth(1) {
                            if let Some(class_name) = class_part.split(&[':', '('][..]).next() {
                                target_identifiers.push(class_name.trim().to_string());
                            }
                        }
                    }
                }
            }
        }
        
        // Check if source scope calls any of the target identifiers
        for source_path in &source_scope.paths {
            if let Ok(source_content) = fs::read_to_string(source_path) {
                for identifier in &target_identifiers {
                    if source_content.contains(&format!("{}(", identifier))
                        || source_content.contains(&format!("{}.", identifier)) {
                        return CheckResult::Pass;
                    }
                }
            }
        }
        
        CheckResult::Fail("Source scope does not call anything in target scope".to_string())
    }
}

impl Library for PythonLibrary {
    fn name(&self) -> &str {
        "python"
    }

    fn call(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value> {
        match function {
            "module_selector" => {
                let path = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E400", "module_selector requires a module path"))?;
                self.module_selector(path, workspace)
            }
            "function_selector" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E401", "function_selector requires a function name"))?;
                self.function_selector(name, workspace)
            }
            "class_selector" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E402", "class_selector requires a class name"))?;
                self.class_selector(name, workspace)
            }
            _ => Err(LibraryError::new("E499", format!("Unknown function: python.{}", function)))
        }
    }

    fn check(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult> {
        match function {
            "imports" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E410", "imports requires a scope"))?;
                let module_name = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E411", "imports requires a module name"))?;
                Ok(self.check_imports(scope, module_name, workspace))
            }
            "no_import" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E412", "no_import requires a scope"))?;
                let module_name = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E413", "no_import requires a module name"))?;
                Ok(self.check_no_import(scope, module_name, workspace))
            }
            "returns_type" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E414", "returns_type requires a scope"))?;
                let type_name = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E415", "returns_type requires a type name"))?;
                Ok(self.check_returns_type(scope, type_name, workspace))
            }
            "function_returns_type" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E416", "function_returns_type requires a scope"))?;
                let func_name = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E417", "function_returns_type requires a function name"))?;
                let type_name = args.get(2)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E418", "function_returns_type requires a type name"))?;
                Ok(self.check_function_returns_type(scope, func_name, type_name, workspace))
            }
            "calls" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E419", "calls requires a scope"))?;
                let target = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E420", "calls requires a target identifier"))?;
                Ok(self.check_calls(scope, target, workspace))
            }
            "calls_function" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E421", "calls_function requires a scope"))?;
                let module_name = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E422", "calls_function requires a module name"))?;
                let func_name = args.get(2)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E423", "calls_function requires a function name"))?;
                Ok(self.check_calls_function(scope, module_name, func_name, workspace))
            }
            "calls_scope" => {
                let source_scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E424", "calls_scope requires a source scope"))?;
                let target_scope = args.get(1)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E425", "calls_scope requires a target scope"))?;
                Ok(self.check_calls_scope(source_scope, target_scope, workspace))
            }
            _ => Err(LibraryError::new("E499", format!("Unknown check: python.{}", function)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_function_selector() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.py"), r#"
def hello_world():
    print("Hello!")

def another_func():
    pass
"#).unwrap();

        let lib = PythonLibrary::new();
        let result = lib.function_selector("hello_world", dir.path().to_str().unwrap()).unwrap();

        if let Value::Scope(scope) = result {
            assert!(!scope.paths.is_empty());
        } else {
            panic!("Expected Scope value");
        }
    }

    #[test]
    fn test_class_selector() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("models.py"), r#"
class MyClass:
    def __init__(self):
        pass

class AnotherClass(BaseClass):
    pass
"#).unwrap();

        let lib = PythonLibrary::new();
        let result = lib.class_selector("MyClass", dir.path().to_str().unwrap()).unwrap();

        if let Value::Scope(scope) = result {
            assert!(!scope.paths.is_empty());
        } else {
            panic!("Expected Scope value");
        }
    }

    #[test]
    fn test_module_selector() {
        let dir = tempdir().unwrap();
        let api_dir = dir.path().join("api");
        fs::create_dir(&api_dir).unwrap();
        fs::write(api_dir.join("__init__.py"), "# API module").unwrap();

        let lib = PythonLibrary::new();
        let result = lib.module_selector("api", dir.path().to_str().unwrap()).unwrap();

        if let Value::Scope(scope) = result {
            assert!(!scope.paths.is_empty());
        } else {
            panic!("Expected Scope value");
        }
    }

    #[test]
    fn test_imports_check() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("main.py");
        fs::write(&test_file, r#"
import os
from typing import List
import requests
"#).unwrap();

        let lib = PythonLibrary::new();
        let scope = Scope::new(ScopeKind::File("main.py".to_string()))
            .with_paths(vec![test_file.to_string_lossy().to_string()]);

        let result = lib.check_imports(&scope, "typing", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_imports(&scope, "requests", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_imports(&scope, "nonexistent", dir.path().to_str().unwrap());
        assert!(result.is_fail());
    }

    #[test]
    fn test_no_import_check() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("main.py");
        fs::write(&test_file, r#"
import os
from typing import List
"#).unwrap();

        let lib = PythonLibrary::new();
        let scope = Scope::new(ScopeKind::File("main.py".to_string()))
            .with_paths(vec![test_file.to_string_lossy().to_string()]);

        let result = lib.check_no_import(&scope, "requests", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_no_import(&scope, "typing", dir.path().to_str().unwrap());
        assert!(result.is_fail());
    }

    #[test]
    fn test_returns_type_check() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("api.py");
        fs::write(&test_file, r#"
def get_user() -> User:
    return User()

def get_list() -> List[str]:
    return []

async def fetch_data() -> Dict[str, Any]:
    return {}
"#).unwrap();

        let lib = PythonLibrary::new();
        let scope = Scope::new(ScopeKind::File("api.py".to_string()))
            .with_paths(vec![test_file.to_string_lossy().to_string()]);

        let result = lib.check_returns_type(&scope, "User", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_returns_type(&scope, "List", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_returns_type(&scope, "Dict", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_returns_type(&scope, "NonExistent", dir.path().to_str().unwrap());
        assert!(result.is_fail());
    }

    #[test]
    fn test_function_returns_type_check() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("service.py");
        fs::write(&test_file, r#"
def process_data() -> Result:
    return Result()

def calculate() -> int:
    return 42

async def fetch() -> Response:
    return Response()
"#).unwrap();

        let lib = PythonLibrary::new();
        let scope = Scope::new(ScopeKind::File("service.py".to_string()))
            .with_paths(vec![test_file.to_string_lossy().to_string()]);

        let result = lib.check_function_returns_type(&scope, "process_data", "Result", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_function_returns_type(&scope, "calculate", "int", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_function_returns_type(&scope, "fetch", "Response", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_function_returns_type(&scope, "process_data", "WrongType", dir.path().to_str().unwrap());
        assert!(result.is_fail());
    }

    #[test]
    fn test_calls_check() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("main.py");
        fs::write(&test_file, r#"
import logger

def main():
    logger.info("Starting")
    process_data()
    result = calculate(10)
"#).unwrap();

        let lib = PythonLibrary::new();
        let scope = Scope::new(ScopeKind::File("main.py".to_string()))
            .with_paths(vec![test_file.to_string_lossy().to_string()]);

        let result = lib.check_calls(&scope, "logger", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_calls(&scope, "process_data", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_calls(&scope, "calculate", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_calls(&scope, "nonexistent", dir.path().to_str().unwrap());
        assert!(result.is_fail());
    }

    #[test]
    fn test_calls_function_check() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("app.py");
        fs::write(&test_file, r#"
import database

def run():
    database.connect()
    user = database.query("SELECT * FROM users")
"#).unwrap();

        let lib = PythonLibrary::new();
        let scope = Scope::new(ScopeKind::File("app.py".to_string()))
            .with_paths(vec![test_file.to_string_lossy().to_string()]);

        let result = lib.check_calls_function(&scope, "database", "connect", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_calls_function(&scope, "database", "query", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_calls_function(&scope, "database", "disconnect", dir.path().to_str().unwrap());
        assert!(result.is_fail());
    }

    #[test]
    fn test_calls_scope_check() {
        let dir = tempdir().unwrap();
        
        // Create target scope with functions
        let target_file = dir.path().join("utils.py");
        fs::write(&target_file, r#"
def helper_function():
    pass

class HelperClass:
    def method(self):
        pass
"#).unwrap();

        // Create source scope that calls target
        let source_file = dir.path().join("main.py");
        fs::write(&source_file, r#"
from utils import helper_function, HelperClass

def main():
    helper_function()
    obj = HelperClass()
"#).unwrap();

        let lib = PythonLibrary::new();
        let target_scope = Scope::new(ScopeKind::File("utils.py".to_string()))
            .with_paths(vec![target_file.to_string_lossy().to_string()]);
        let source_scope = Scope::new(ScopeKind::File("main.py".to_string()))
            .with_paths(vec![source_file.to_string_lossy().to_string()]);

        let result = lib.check_calls_scope(&source_scope, &target_scope, dir.path().to_str().unwrap());
        assert!(result.is_pass());
    }
}
