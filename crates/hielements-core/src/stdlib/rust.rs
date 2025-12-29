//! Rust language library for Hielements.
//!
//! Provides selectors and checks for Rust code analysis.
//! Uses simple regex-based parsing for common Rust constructs.

use std::fs;
use std::path::{Path, PathBuf};

use glob::glob;
use walkdir::WalkDir;

use super::{CheckResult, Library, LibraryError, LibraryResult, Scope, ScopeKind, Value};

/// Directories to exclude when scanning for Rust files.
const EXCLUDED_DIRS: &[&str] = &["target", ".git", "node_modules", ".cargo", "vendor"];

/// Check if a directory name should be excluded from scanning.
fn is_excluded_dir(name: &str) -> bool {
    EXCLUDED_DIRS.contains(&name)
}

/// Walk a directory and collect all .rs files, skipping excluded directories.
/// This is much more efficient than glob because it skips entire directory trees.
fn find_rust_files(base_path: &str) -> Vec<PathBuf> {
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
        if path.extension().map(|e| e == "rs").unwrap_or(false) {
            files.push(path.to_path_buf());
        }
    }
    
    files
}

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
        let last_part = parts.last().unwrap_or(&"");
        
        let scope = Scope::new(ScopeKind::File(module_path.to_string()));
        let mut found_paths = Vec::new();
        
        // Use efficient WalkDir-based search
        for path in find_rust_files(workspace) {
            let path_str = path.to_string_lossy();
            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            
            // Check if this file matches the module name
            if file_stem == *last_part || (file_stem == "mod" && path_str.contains(last_part)) {
                found_paths.push(path_str.to_string());
            }
        }
        
        // Deduplicate
        found_paths.sort();
        found_paths.dedup();
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Rust struct by name.
    fn struct_selector(&self, struct_name: &str, workspace: &str) -> LibraryResult<Value> {
        let scope = Scope::new(ScopeKind::File(format!("struct:{}", struct_name)));
        let mut found_paths = Vec::new();
        
        // Match struct Name followed by space, <, {, (, or ;
        let struct_pattern = format!(r"(pub\s+)?struct\s+{}(\s*[<{{(;]|\s)", struct_name);
        let re = regex::Regex::new(&struct_pattern).ok();
        
        for entry in find_rust_files(workspace) {
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
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Rust enum by name.
    fn enum_selector(&self, enum_name: &str, workspace: &str) -> LibraryResult<Value> {
        let scope = Scope::new(ScopeKind::File(format!("enum:{}", enum_name)));
        let mut found_paths = Vec::new();
        
        for entry in find_rust_files(workspace) {
            if let Ok(content) = fs::read_to_string(&entry) {
                if content.contains(&format!("enum {}", enum_name)) 
                    || content.contains(&format!("enum {} ", enum_name))
                    || content.contains(&format!("pub enum {}", enum_name)) {
                    found_paths.push(entry.to_string_lossy().to_string());
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Rust function/method by name.
    fn function_selector(&self, func_name: &str, workspace: &str) -> LibraryResult<Value> {
        let scope = Scope::new(ScopeKind::File(format!("fn:{}", func_name)));
        let mut found_paths = Vec::new();
        
        for entry in find_rust_files(workspace) {
            if let Ok(content) = fs::read_to_string(&entry) {
                if content.contains(&format!("fn {}", func_name)) 
                    || content.contains(&format!("fn {}(", func_name))
                    || content.contains(&format!("fn {}<", func_name)) {
                    found_paths.push(entry.to_string_lossy().to_string());
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Rust trait by name.
    fn trait_selector(&self, trait_name: &str, workspace: &str) -> LibraryResult<Value> {
        let scope = Scope::new(ScopeKind::File(format!("trait:{}", trait_name)));
        let mut found_paths = Vec::new();
        
        for entry in find_rust_files(workspace) {
            if let Ok(content) = fs::read_to_string(&entry) {
                if content.contains(&format!("trait {}", trait_name))
                    || content.contains(&format!("pub trait {}", trait_name)) {
                    found_paths.push(entry.to_string_lossy().to_string());
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(found_paths)))
    }

    /// Select a Rust impl block.
    fn impl_selector(&self, type_name: &str, workspace: &str) -> LibraryResult<Value> {
        let scope = Scope::new(ScopeKind::File(format!("impl:{}", type_name)));
        let mut found_paths = Vec::new();
        
        for entry in find_rust_files(workspace) {
            if let Ok(content) = fs::read_to_string(&entry) {
                if content.contains(&format!("impl {}", type_name))
                    || content.contains(&format!("impl<") ) && content.contains(&format!("> {}", type_name)) {
                    found_paths.push(entry.to_string_lossy().to_string());
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
        for entry in find_rust_files(workspace) {
            if let Ok(content) = fs::read_to_string(&entry) {
                // Look for impl Trait for Type patterns
                if content.contains(&format!("impl {} for {}", trait_name, type_name))
                    || content.contains(&format!("impl<") ) 
                        && content.contains(&format!("> {} for {}", trait_name, type_name)) {
                    return CheckResult::Pass;
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

    // ========== Behavioral / Dependency Checks ==========

    /// Extract module name from a file path (e.g., "lexer" from ".../lexer.rs").
    fn extract_module_name(path: &str) -> Option<String> {
        let path = Path::new(path);
        let stem = path.file_stem()?.to_str()?;
        if stem == "mod" {
            // For mod.rs, get the parent folder name
            path.parent()?.file_name()?.to_str().map(|s| s.to_string())
        } else {
            Some(stem.to_string())
        }
    }

    /// Check if scope_a depends on (uses types/modules from) scope_b.
    /// This checks for `use` statements, `mod` declarations, and direct type references.
    fn check_depends_on(&self, scope_a: &Scope, scope_b: &Scope, _workspace: &str) -> CheckResult {
        // Get module names from scope_b
        let mut target_modules: Vec<String> = Vec::new();
        for path in &scope_b.paths {
            if let Some(module_name) = Self::extract_module_name(path) {
                target_modules.push(module_name);
            }
        }
        
        // Also check the scope kind for hints
        if let ScopeKind::File(ref name) = scope_b.kind {
            // Extract last part of module path (e.g., "lexer" from "lexer" or "stdlib::files")
            if let Some(last) = name.split("::").last() {
                if !target_modules.contains(&last.to_string()) {
                    target_modules.push(last.to_string());
                }
            }
        }
        
        if target_modules.is_empty() {
            return CheckResult::Error("Could not determine target module names".to_string());
        }

        // Check if any file in scope_a references modules from scope_b
        if scope_a.paths.is_empty() {
            return CheckResult::Error(format!("Source scope has no paths"));
        }
        
        for path_a in &scope_a.paths {
            if let Ok(content) = fs::read_to_string(path_a) {
                for target in &target_modules {
                    // Check for various dependency patterns:
                    // - use crate::module::
                    // - use crate::module;
                    // - use crate::module::{
                    // - use super::module
                    // - mod module;
                    // - module::Type
                    // - crate::module::
                    // - mod module;
                    // - module::Type
                    // - crate::module::
                    let patterns = [
                        format!("use crate::{}", target),
                        format!("use super::{}", target),
                        format!("mod {};", target),
                        format!("{}::", target),
                        format!("crate::{}::", target),
                        format!("super::{}::", target),
                        format!("use {}::", target),
                    ];
                    
                    for pattern in &patterns {
                        if content.contains(pattern) {
                            return CheckResult::Pass;
                        }
                    }
                }
            } else {
                return CheckResult::Error(format!("Could not read file: {}", path_a));
            }
        }
        
        let target_names = target_modules.join(", ");
        CheckResult::Fail(format!(
            "No dependency found: source does not use '{}'", 
            target_names
        ))
    }

    /// Check that scope_a does NOT depend on scope_b.
    /// This is the inverse of depends_on - verifies architectural boundaries.
    fn check_no_dependency(&self, scope_a: &Scope, scope_b: &Scope, workspace: &str) -> CheckResult {
        match self.check_depends_on(scope_a, scope_b, workspace) {
            CheckResult::Pass => {
                // If depends_on passes, that means there IS a dependency - which we don't want
                CheckResult::Fail("Forbidden dependency detected".to_string())
            }
            CheckResult::Fail(_) => {
                // If depends_on fails, there's no dependency - which is what we want
                CheckResult::Pass
            }
            CheckResult::Error(e) => CheckResult::Error(e),
        }
    }

    /// Extract the primary type from a scope (struct, enum, or trait name).
    fn extract_primary_type(&self, scope: &Scope) -> Option<String> {
        // Check scope kind for type hints
        if let ScopeKind::File(ref name) = scope.kind {
            // Handle "struct:Name", "enum:Name", "trait:Name" patterns
            if let Some(type_name) = name.strip_prefix("struct:") {
                return Some(type_name.to_string());
            }
            if let Some(type_name) = name.strip_prefix("enum:") {
                return Some(type_name.to_string());
            }
            if let Some(type_name) = name.strip_prefix("trait:") {
                return Some(type_name.to_string());
            }
        }
        
        // Try to extract from file content
        for path in &scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                // Look for pub struct/enum/type declarations
                let re_struct = regex::Regex::new(r"pub\s+struct\s+(\w+)").ok();
                let re_enum = regex::Regex::new(r"pub\s+enum\s+(\w+)").ok();
                
                if let Some(re) = re_struct {
                    if let Some(caps) = re.captures(&content) {
                        return caps.get(1).map(|m| m.as_str().to_string());
                    }
                }
                if let Some(re) = re_enum {
                    if let Some(caps) = re.captures(&content) {
                        return caps.get(1).map(|m| m.as_str().to_string());
                    }
                }
            }
        }
        
        None
    }

    /// Check that the output type from scope_a can connect to the input of scope_b.
    /// This verifies that the types at the interface boundaries are compatible.
    fn check_pipeline_connects(&self, output_scope: &Scope, input_scope: &Scope, workspace: &str) -> CheckResult {
        // Get the output type from the first scope
        let output_type = match self.extract_primary_type(output_scope) {
            Some(t) => t,
            None => return CheckResult::Error("Could not determine output type".to_string()),
        };
        
        // Check if input scope's files reference the output type
        for path in &input_scope.paths {
            if let Ok(content) = fs::read_to_string(path) {
                // Check if the input module uses/references the output type
                if content.contains(&output_type) {
                    return CheckResult::Pass;
                }
            }
        }
        
        // Also check workspace-wide if the consumer module uses the producer type
        for entry in find_rust_files(workspace) {
            // Check if this file is related to the input scope
            let entry_str = entry.to_string_lossy();
            let is_input_related = input_scope.paths.iter().any(|p| {
                // Check if paths share the same module
                let p_module = Self::extract_module_name(p);
                let e_module = Self::extract_module_name(&entry_str);
                p_module.is_some() && p_module == e_module
            });
            
            if is_input_related {
                if let Ok(content) = fs::read_to_string(&entry) {
                    if content.contains(&output_type) {
                        return CheckResult::Pass;
                    }
                }
            }
        }
        
        CheckResult::Fail(format!(
            "Pipeline not connected: '{}' is not used by input scope",
            output_type
        ))
    }

    /// Check that two scopes have compatible types (one uses the other's types).
    fn check_type_compatible(&self, scope_a: &Scope, scope_b: &Scope, _workspace: &str) -> CheckResult {
        let type_a = self.extract_primary_type(scope_a);
        let type_b = self.extract_primary_type(scope_b);
        
        match (type_a, type_b) {
            (Some(a), Some(b)) => {
                // Same type is always compatible
                if a == b {
                    return CheckResult::Pass;
                }
                
                // Check if type_a is used where type_b is expected in any file
                // For now, check if they're referenced together
                for path in &scope_b.paths {
                    if let Ok(content) = fs::read_to_string(path) {
                        if content.contains(&a) {
                            return CheckResult::Pass;
                        }
                    }
                }
                
                // Also check the reverse
                for path in &scope_a.paths {
                    if let Ok(content) = fs::read_to_string(path) {
                        if content.contains(&b) {
                            return CheckResult::Pass;
                        }
                    }
                }
                
                CheckResult::Fail(format!("Types '{}' and '{}' are not compatible", a, b))
            }
            (None, _) => CheckResult::Error("Could not determine type from first scope".to_string()),
            (_, None) => CheckResult::Error("Could not determine type from second scope".to_string()),
        }
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
            // Behavioral / Dependency checks
            "depends_on" => {
                let scope_a = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E323", "depends_on requires a source scope as first argument"))?;
                let scope_b = args.get(1)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E324", "depends_on requires a target scope as second argument"))?;
                Ok(self.check_depends_on(scope_a, scope_b, workspace))
            }
            "no_dependency" => {
                let scope_a = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E325", "no_dependency requires a source scope as first argument"))?;
                let scope_b = args.get(1)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E326", "no_dependency requires a target scope as second argument"))?;
                Ok(self.check_no_dependency(scope_a, scope_b, workspace))
            }
            "pipeline_connects" => {
                let output_scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E327", "pipeline_connects requires an output scope as first argument"))?;
                let input_scope = args.get(1)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E328", "pipeline_connects requires an input scope as second argument"))?;
                Ok(self.check_pipeline_connects(output_scope, input_scope, workspace))
            }
            "type_compatible" => {
                let scope_a = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E329", "type_compatible requires a scope as first argument"))?;
                let scope_b = args.get(1)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E330", "type_compatible requires a scope as second argument"))?;
                Ok(self.check_type_compatible(scope_a, scope_b, workspace))
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
