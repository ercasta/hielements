//! Rust language library for Hielements.
//!
//! Provides selectors and checks for Rust code analysis.
//! Uses simple regex-based parsing for common Rust constructs.

use std::fs;
use std::path::Path;

use glob::glob;
use walkdir::WalkDir;

use super::{CheckResult, Library, LibraryError, LibraryResult, Scope, ScopeKind, Value};

/// The Rust library.
pub struct RustLibrary;

impl RustLibrary {
    pub fn new() -> Self {
        Self
    }

    /// Select a Rust crate by name (looks for Cargo.toml in workspace).
    fn crate_selector(&self, crate_name: &str, workspace: &str) -> LibraryResult<Value> {
        let mut crate_path = None;
        
        // Look for the crate in common locations
        let search_patterns = [
            format!("{}/Cargo.toml", workspace),
            format!("{}/crates/{}/Cargo.toml", workspace, crate_name),
            format!("{}/{}/Cargo.toml", workspace, crate_name),
        ];
        
        for pattern in &search_patterns {
            if let Ok(path) = Path::new(pattern).canonicalize() {
                if path.exists() {
                    // Read Cargo.toml to verify crate name
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains(&format!("name = \"{}\"", crate_name)) {
                            crate_path = Some(path.parent().unwrap().to_path_buf());
                            break;
                        }
                    }
                }
            }
        }
        
        // If not found by name match, try pattern matching
        if crate_path.is_none() {
            let glob_pattern = format!("{}/crates/*/Cargo.toml", workspace);
            if let Ok(entries) = glob(&glob_pattern) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Ok(content) = fs::read_to_string(&entry) {
                        if content.contains(&format!("name = \"{}\"", crate_name)) {
                            crate_path = Some(entry.parent().unwrap().to_path_buf());
                            break;
                        }
                    }
                }
            }
        }
        
        let scope = Scope::new(ScopeKind::Folder(crate_name.to_string()));
        
        if let Some(path) = crate_path {
            // Collect all .rs files in the crate
            let mut paths = Vec::new();
            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.extension().map(|e| e == "rs").unwrap_or(false) {
                    paths.push(p.to_string_lossy().to_string());
                }
            }
            Ok(Value::Scope(scope.with_paths(paths)))
        } else {
            Ok(Value::Scope(scope.with_paths(vec![])))
        }
    }

    /// Select a Rust module by path (e.g., "lexer" or "stdlib::files").
    fn module_selector(&self, module_path: &str, workspace: &str) -> LibraryResult<Value> {
        // Convert module path to file path patterns
        let parts: Vec<&str> = module_path.split("::").collect();
        let mut possible_paths = Vec::new();
        
        // Try various patterns for finding the module
        let last_part = parts.last().unwrap_or(&"");
        possible_paths.push(format!("{}/**/{}.rs", workspace, last_part));
        possible_paths.push(format!("{}/**/{}/mod.rs", workspace, last_part));
        
        if parts.len() > 1 {
            let path_str = parts.join("/");
            possible_paths.push(format!("{}/**/{}.rs", workspace, path_str));
            possible_paths.push(format!("{}/**/{}/mod.rs", workspace, path_str));
        }
        
        let scope = Scope::new(ScopeKind::File(module_path.to_string()));
        let mut found_paths = Vec::new();
        
        for pattern in &possible_paths {
            if let Ok(entries) = glob(pattern) {
                for entry in entries.filter_map(|e| e.ok()) {
                    found_paths.push(entry.to_string_lossy().to_string());
                }
            }
        }
        
        // Deduplicate
        found_paths.sort();
        found_paths.dedup();
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Rust struct by name.
    fn struct_selector(&self, struct_name: &str, workspace: &str) -> LibraryResult<Value> {
        let pattern = format!("{}/**/*.rs", workspace);
        let scope = Scope::new(ScopeKind::File(format!("struct:{}", struct_name)));
        let mut found_paths = Vec::new();
        
        // Match struct Name followed by space, <, {, (, or ;
        let struct_pattern = format!(r"(pub\s+)?struct\s+{}(\s*[<{{(;]|\s)", struct_name);
        let re = regex::Regex::new(&struct_pattern).ok();
        
        if let Ok(entries) = glob(&pattern) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(content) = fs::read_to_string(&entry) {
                    if let Some(ref re) = re {
                        if re.is_match(&content) {
                            found_paths.push(entry.to_string_lossy().to_string());
                        }
                    } else {
                        // Fallback to simple string matching
                        if content.contains(&format!("struct {}", struct_name)) {
                            found_paths.push(entry.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Rust enum by name.
    fn enum_selector(&self, enum_name: &str, workspace: &str) -> LibraryResult<Value> {
        let pattern = format!("{}/**/*.rs", workspace);
        let scope = Scope::new(ScopeKind::File(format!("enum:{}", enum_name)));
        let mut found_paths = Vec::new();
        
        if let Ok(entries) = glob(&pattern) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(content) = fs::read_to_string(&entry) {
                    if content.contains(&format!("enum {}", enum_name)) 
                        || content.contains(&format!("enum {} ", enum_name))
                        || content.contains(&format!("pub enum {}", enum_name)) {
                        found_paths.push(entry.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Rust function/method by name.
    fn function_selector(&self, func_name: &str, workspace: &str) -> LibraryResult<Value> {
        let pattern = format!("{}/**/*.rs", workspace);
        let scope = Scope::new(ScopeKind::File(format!("fn:{}", func_name)));
        let mut found_paths = Vec::new();
        
        if let Ok(entries) = glob(&pattern) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(content) = fs::read_to_string(&entry) {
                    if content.contains(&format!("fn {}", func_name)) 
                        || content.contains(&format!("fn {}(", func_name))
                        || content.contains(&format!("fn {}<", func_name)) {
                        found_paths.push(entry.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Rust trait by name.
    fn trait_selector(&self, trait_name: &str, workspace: &str) -> LibraryResult<Value> {
        let pattern = format!("{}/**/*.rs", workspace);
        let scope = Scope::new(ScopeKind::File(format!("trait:{}", trait_name)));
        let mut found_paths = Vec::new();
        
        if let Ok(entries) = glob(&pattern) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(content) = fs::read_to_string(&entry) {
                    if content.contains(&format!("trait {}", trait_name))
                        || content.contains(&format!("pub trait {}", trait_name)) {
                        found_paths.push(entry.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Rust impl block.
    fn impl_selector(&self, type_name: &str, workspace: &str) -> LibraryResult<Value> {
        let pattern = format!("{}/**/*.rs", workspace);
        let scope = Scope::new(ScopeKind::File(format!("impl:{}", type_name)));
        let mut found_paths = Vec::new();
        
        if let Ok(entries) = glob(&pattern) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(content) = fs::read_to_string(&entry) {
                    if content.contains(&format!("impl {}", type_name))
                        || content.contains(&format!("impl<") ) && content.contains(&format!("> {}", type_name)) {
                        found_paths.push(entry.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    // Check functions

    /// Check that a struct exists.
    fn check_struct_exists(&self, struct_name: &str, workspace: &str) -> CheckResult {
        if let Ok(Value::Scope(scope)) = self.struct_selector(struct_name, workspace) {
            if !scope.paths.is_empty() {
                CheckResult::Pass
            } else {
                CheckResult::Fail(format!("Struct '{}' not found", struct_name))
            }
        } else {
            CheckResult::Error("Failed to search for struct".to_string())
        }
    }

    /// Check that an enum exists.
    fn check_enum_exists(&self, enum_name: &str, workspace: &str) -> CheckResult {
        if let Ok(Value::Scope(scope)) = self.enum_selector(enum_name, workspace) {
            if !scope.paths.is_empty() {
                CheckResult::Pass
            } else {
                CheckResult::Fail(format!("Enum '{}' not found", enum_name))
            }
        } else {
            CheckResult::Error("Failed to search for enum".to_string())
        }
    }

    /// Check that a function exists.
    fn check_function_exists(&self, func_name: &str, workspace: &str) -> CheckResult {
        if let Ok(Value::Scope(scope)) = self.function_selector(func_name, workspace) {
            if !scope.paths.is_empty() {
                CheckResult::Pass
            } else {
                CheckResult::Fail(format!("Function '{}' not found", func_name))
            }
        } else {
            CheckResult::Error("Failed to search for function".to_string())
        }
    }

    /// Check that a trait exists.
    fn check_trait_exists(&self, trait_name: &str, workspace: &str) -> CheckResult {
        if let Ok(Value::Scope(scope)) = self.trait_selector(trait_name, workspace) {
            if !scope.paths.is_empty() {
                CheckResult::Pass
            } else {
                CheckResult::Fail(format!("Trait '{}' not found", trait_name))
            }
        } else {
            CheckResult::Error("Failed to search for trait".to_string())
        }
    }

    /// Check that an impl exists.
    fn check_impl_exists(&self, type_name: &str, workspace: &str) -> CheckResult {
        if let Ok(Value::Scope(scope)) = self.impl_selector(type_name, workspace) {
            if !scope.paths.is_empty() {
                CheckResult::Pass
            } else {
                CheckResult::Fail(format!("Impl for '{}' not found", type_name))
            }
        } else {
            CheckResult::Error("Failed to search for impl".to_string())
        }
    }

    /// Check that a type implements a trait.
    fn check_implements(&self, type_name: &str, trait_name: &str, workspace: &str) -> CheckResult {
        let pattern = format!("{}/**/*.rs", workspace);
        
        if let Ok(entries) = glob(&pattern) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(content) = fs::read_to_string(&entry) {
                    // Look for impl Trait for Type patterns
                    if content.contains(&format!("impl {} for {}", trait_name, type_name))
                        || content.contains(&format!("impl<") ) 
                            && content.contains(&format!("> {} for {}", trait_name, type_name)) {
                        return CheckResult::Pass;
                    }
                }
            }
        }
        
        CheckResult::Fail(format!("'{}' does not implement '{}'", type_name, trait_name))
    }

    /// Check that a module uses a specific crate/module.
    fn check_uses(&self, scope: &Scope, module_path: &str, _workspace: &str) -> CheckResult {
        for path in &scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                if content.contains(&format!("use {}", module_path))
                    || content.contains(&format!("use {}::", module_path))
                    || content.contains(&format!("{}::", module_path)) {
                    return CheckResult::Pass;
                }
            }
        }
        
        CheckResult::Fail(format!("Module does not use '{}'", module_path))
    }

    /// Check that scope has a specific derive macro.
    fn check_has_derive(&self, scope: &Scope, derive_name: &str, _workspace: &str) -> CheckResult {
        for path in &scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                if content.contains(&format!("#[derive(") ) && content.contains(derive_name) {
                    return CheckResult::Pass;
                }
            }
        }
        
        CheckResult::Fail(format!("No #[derive({})] found", derive_name))
    }

    /// Check that a module has doc comments.
    fn check_has_docs(&self, scope: &Scope, _workspace: &str) -> CheckResult {
        for path in &scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                // Check for //! module docs or /// item docs
                if content.contains("//!") || content.contains("///") {
                    return CheckResult::Pass;
                }
            }
        }
        
        CheckResult::Fail("No documentation comments found".to_string())
    }

    /// Check that there are tests in the scope.
    fn check_has_tests(&self, scope: &Scope, _workspace: &str) -> CheckResult {
        for path in &scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                if content.contains("#[test]") || content.contains("#[cfg(test)]") {
                    return CheckResult::Pass;
                }
            }
        }
        
        CheckResult::Fail("No tests found".to_string())
    }
}

impl Library for RustLibrary {
    fn name(&self) -> &str {
        "rust"
    }

    fn call(&self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value> {
        match function {
            "crate_selector" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E300", "crate_selector requires a crate name"))?;
                self.crate_selector(name, workspace)
            }
            "module_selector" => {
                let path = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E301", "module_selector requires a module path"))?;
                self.module_selector(path, workspace)
            }
            "struct_selector" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E302", "struct_selector requires a struct name"))?;
                self.struct_selector(name, workspace)
            }
            "enum_selector" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E303", "enum_selector requires an enum name"))?;
                self.enum_selector(name, workspace)
            }
            "function_selector" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E304", "function_selector requires a function name"))?;
                self.function_selector(name, workspace)
            }
            "trait_selector" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E305", "trait_selector requires a trait name"))?;
                self.trait_selector(name, workspace)
            }
            "impl_selector" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E306", "impl_selector requires a type name"))?;
                self.impl_selector(name, workspace)
            }
            _ => Err(LibraryError::new("E399", format!("Unknown function: rust.{}", function)))
        }
    }

    fn check(&self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult> {
        match function {
            "struct_exists" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E310", "struct_exists requires a struct name"))?;
                Ok(self.check_struct_exists(name, workspace))
            }
            "enum_exists" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E311", "enum_exists requires an enum name"))?;
                Ok(self.check_enum_exists(name, workspace))
            }
            "function_exists" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E312", "function_exists requires a function name"))?;
                Ok(self.check_function_exists(name, workspace))
            }
            "trait_exists" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E313", "trait_exists requires a trait name"))?;
                Ok(self.check_trait_exists(name, workspace))
            }
            "impl_exists" => {
                let name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E314", "impl_exists requires a type name"))?;
                Ok(self.check_impl_exists(name, workspace))
            }
            "implements" => {
                let type_name = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E315", "implements requires a type name"))?;
                let trait_name = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E316", "implements requires a trait name"))?;
                Ok(self.check_implements(type_name, trait_name, workspace))
            }
            "uses" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E317", "uses requires a scope"))?;
                let module_path = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E318", "uses requires a module path"))?;
                Ok(self.check_uses(scope, module_path, workspace))
            }
            "has_derive" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E319", "has_derive requires a scope"))?;
                let derive_name = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E320", "has_derive requires a derive name"))?;
                Ok(self.check_has_derive(scope, derive_name, workspace))
            }
            "has_docs" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E321", "has_docs requires a scope"))?;
                Ok(self.check_has_docs(scope, workspace))
            }
            "has_tests" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E322", "has_tests requires a scope"))?;
                Ok(self.check_has_tests(scope, workspace))
            }
            _ => Err(LibraryError::new("E399", format!("Unknown check: rust.{}", function)))
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
        let src_dir = dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), r#"
pub fn hello_world() {
    println!("Hello!");
}

fn private_func() {}
"#).unwrap();

        let lib = RustLibrary::new();
        let result = lib.function_selector("hello_world", dir.path().to_str().unwrap()).unwrap();

        if let Value::Scope(scope) = result {
            assert!(!scope.paths.is_empty());
        } else {
            panic!("Expected Scope value");
        }
    }

    #[test]
    fn test_struct_selector() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), r#"
pub struct MyStruct {
    field: i32,
}

struct PrivateStruct;
"#).unwrap();

        let lib = RustLibrary::new();
        let result = lib.struct_selector("MyStruct", dir.path().to_str().unwrap()).unwrap();

        if let Value::Scope(scope) = result {
            assert!(!scope.paths.is_empty());
        } else {
            panic!("Expected Scope value");
        }
    }

    #[test]
    fn test_check_has_tests() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), r#"
pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
"#).unwrap();

        let lib = RustLibrary::new();
        let scope = Scope::new(ScopeKind::File("lib.rs".to_string()))
            .with_paths(vec![src_dir.join("lib.rs").to_string_lossy().to_string()]);
        
        let result = lib.check_has_tests(&scope, dir.path().to_str().unwrap());
        assert!(result.is_pass());
    }
}
