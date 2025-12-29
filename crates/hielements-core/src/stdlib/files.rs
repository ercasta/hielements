//! Files and folders library for Hielements.
//!
//! Provides selectors and checks for file system operations.

use std::path::{Path, PathBuf};

use glob::glob;
use walkdir::WalkDir;

use super::{CheckResult, Library, LibraryError, LibraryResult, Scope, ScopeKind, Value};

/// The files library.
pub struct FilesLibrary;

impl FilesLibrary {
    pub fn new() -> Self {
        Self
    }

    /// Create a file selector.
    fn file_selector(&self, path: &str, workspace: &str) -> LibraryResult<Value> {
        let full_path = Path::new(workspace).join(path);
        let scope = Scope::new(ScopeKind::File(path.to_string()));
        
        if full_path.exists() && full_path.is_file() {
            Ok(Value::Scope(scope.with_paths(vec![full_path.to_string_lossy().to_string()])))
        } else {
            Ok(Value::Scope(scope.with_paths(vec![])))
        }
    }

    /// Create a folder selector.
    fn folder_selector(&self, path: &str, workspace: &str) -> LibraryResult<Value> {
        let full_path = Path::new(workspace).join(path);
        let scope = Scope::new(ScopeKind::Folder(path.to_string()));
        
        if full_path.exists() && full_path.is_dir() {
            // Collect all files in the folder
            let mut paths = Vec::new();
            for entry in WalkDir::new(&full_path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    paths.push(entry.path().to_string_lossy().to_string());
                }
            }
            Ok(Value::Scope(scope.with_paths(paths)))
        } else {
            Ok(Value::Scope(scope.with_paths(vec![])))
        }
    }

    /// Create a glob selector.
    fn glob_selector(&self, pattern: &str, workspace: &str) -> LibraryResult<Value> {
        let full_pattern = Path::new(workspace).join(pattern);
        let scope = Scope::new(ScopeKind::Glob(pattern.to_string()));
        
        let mut paths = Vec::new();
        if let Ok(entries) = glob(&full_pattern.to_string_lossy()) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.is_file() {
                    paths.push(entry.to_string_lossy().to_string());
                }
            }
        }
        
        Ok(Value::Scope(scope.with_paths(paths)))
    }

    /// Check if a file exists in a scope.
    fn check_exists(&self, scope: &Scope, filename: &str, workspace: &str) -> CheckResult {
        // For folder scopes, check if the file exists within
        match &scope.kind {
            ScopeKind::File(path) => {
                let file_path = Path::new(workspace).join(path);
                if file_path.exists() {
                    CheckResult::Pass
                } else {
                    CheckResult::Fail(format!("File '{}' does not exist", path))
                }
            }
            ScopeKind::Folder(path) => {
                let file_path = Path::new(workspace).join(path).join(filename);
                if file_path.exists() {
                    CheckResult::Pass
                } else {
                    CheckResult::Fail(format!("File '{}' does not exist in folder '{}'", filename, path))
                }
            }
            ScopeKind::Glob(_) => {
                // Check if any matched file has the given name
                for p in &scope.paths {
                    if Path::new(p).file_name().map(|n| n.to_string_lossy()) == Some(filename.into()) {
                        return CheckResult::Pass;
                    }
                }
                CheckResult::Fail(format!("No file named '{}' found in scope", filename))
            }
        }
    }

    /// Check if a scope contains a file.
    fn check_contains(&self, scope: &Scope, filename: &str, workspace: &str) -> CheckResult {
        match &scope.kind {
            ScopeKind::Folder(path) => {
                let file_path = Path::new(workspace).join(path).join(filename);
                if file_path.exists() {
                    CheckResult::Pass
                } else {
                    CheckResult::Fail(format!("Folder '{}' does not contain '{}'", path, filename))
                }
            }
            _ => {
                // For other scope types, check the resolved paths
                for p in &scope.paths {
                    if p.ends_with(filename) || Path::new(p).file_name().map(|n| n.to_string_lossy()) == Some(filename.into()) {
                        return CheckResult::Pass;
                    }
                }
                CheckResult::Fail(format!("Scope does not contain '{}'", filename))
            }
        }
    }

    /// Check that no files match a pattern.
    fn check_no_files_matching(&self, scope: &Scope, pattern: &str, workspace: &str) -> CheckResult {
        let scope_path = match &scope.kind {
            ScopeKind::Folder(path) => Path::new(workspace).join(path),
            ScopeKind::File(path) => Path::new(workspace).join(path).parent().unwrap_or(Path::new(workspace)).to_path_buf(),
            ScopeKind::Glob(_) => PathBuf::from(workspace),
        };

        let full_pattern = scope_path.join(pattern);
        if let Ok(entries) = glob(&full_pattern.to_string_lossy()) {
            let matches: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            if matches.is_empty() {
                CheckResult::Pass
            } else {
                CheckResult::Fail(format!(
                    "Found {} files matching pattern '{}': {:?}",
                    matches.len(),
                    pattern,
                    matches.iter().take(5).collect::<Vec<_>>()
                ))
            }
        } else {
            CheckResult::Error(format!("Invalid glob pattern: {}", pattern))
        }
    }

    /// Check file size.
    fn check_max_size(&self, scope: &Scope, max_bytes: i64, _workspace: &str) -> CheckResult {
        for path in &scope.paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() > max_bytes as u64 {
                    return CheckResult::Fail(format!(
                        "File '{}' exceeds maximum size ({} > {} bytes)",
                        path,
                        metadata.len(),
                        max_bytes
                    ));
                }
            }
        }
        CheckResult::Pass
    }
}

impl Library for FilesLibrary {
    fn name(&self) -> &str {
        "files"
    }

    fn call(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<Value> {
        match function {
            "file_selector" => {
                let path = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E100", "file_selector requires a string path argument"))?;
                self.file_selector(path, workspace)
            }
            "folder_selector" => {
                let path = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E101", "folder_selector requires a string path argument"))?;
                self.folder_selector(path, workspace)
            }
            "glob_selector" => {
                let pattern = args.get(0)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E102", "glob_selector requires a string pattern argument"))?;
                self.glob_selector(pattern, workspace)
            }
            _ => Err(LibraryError::new("E199", format!("Unknown function: files.{}", function)))
        }
    }

    fn check(&mut self, function: &str, args: Vec<Value>, workspace: &str) -> LibraryResult<CheckResult> {
        match function {
            "exists" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E110", "exists requires a scope as first argument"))?;
                let filename = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E111", "exists requires a filename as second argument"))?;
                Ok(self.check_exists(scope, filename, workspace))
            }
            "contains" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E112", "contains requires a scope as first argument"))?;
                let filename = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E113", "contains requires a filename as second argument"))?;
                Ok(self.check_contains(scope, filename, workspace))
            }
            "no_files_matching" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E114", "no_files_matching requires a scope as first argument"))?;
                let pattern = args.get(1)
                    .and_then(|v| v.as_string())
                    .ok_or_else(|| LibraryError::new("E115", "no_files_matching requires a pattern as second argument"))?;
                Ok(self.check_no_files_matching(scope, pattern, workspace))
            }
            "max_size" => {
                let scope = args.get(0)
                    .and_then(|v| v.as_scope())
                    .ok_or_else(|| LibraryError::new("E116", "max_size requires a scope as first argument"))?;
                let max_bytes = args.get(1)
                    .and_then(|v| v.as_int())
                    .ok_or_else(|| LibraryError::new("E117", "max_size requires a number as second argument"))?;
                Ok(self.check_max_size(scope, max_bytes, workspace))
            }
            _ => Err(LibraryError::new("E199", format!("Unknown check function: files.{}", function)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_file_selector() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "content").unwrap();

        let lib = FilesLibrary::new();
        let result = lib.file_selector("test.txt", dir.path().to_str().unwrap()).unwrap();

        if let Value::Scope(scope) = result {
            assert!(!scope.paths.is_empty());
        } else {
            panic!("Expected Scope value");
        }
    }

    #[test]
    fn test_folder_selector() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("src");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("main.py"), "print('hello')").unwrap();

        let lib = FilesLibrary::new();
        let result = lib.folder_selector("src", dir.path().to_str().unwrap()).unwrap();

        if let Value::Scope(scope) = result {
            assert!(!scope.paths.is_empty());
        } else {
            panic!("Expected Scope value");
        }
    }

    #[test]
    fn test_check_exists() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("src");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("main.py"), "print('hello')").unwrap();

        let lib = FilesLibrary::new();
        let scope = Scope::new(ScopeKind::Folder("src".to_string())).with_paths(vec![]);
        
        let result = lib.check_exists(&scope, "main.py", dir.path().to_str().unwrap());
        assert!(result.is_pass());

        let result = lib.check_exists(&scope, "nonexistent.py", dir.path().to_str().unwrap());
        assert!(result.is_fail());
    }
}
