//! Documentation extraction module for Hielements libraries.
//!
//! This module provides types and functions for extracting and generating
//! documentation from Hielements libraries in both human-readable (Markdown)
//! and agent-readable (JSON) formats.

use serde::{Deserialize, Serialize};

/// Documentation for a library function parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDoc {
    /// Parameter name
    pub name: String,
    /// Parameter type (e.g., "string", "Scope", "integer")
    pub param_type: String,
    /// Human-readable description
    pub description: String,
}

impl ParameterDoc {
    /// Create a new parameter documentation entry.
    pub fn new(name: impl Into<String>, param_type: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: param_type.into(),
            description: description.into(),
        }
    }
}

/// Documentation for a library function or check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDoc {
    /// Function name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Parameter documentation
    pub parameters: Vec<ParameterDoc>,
    /// Return type description
    pub return_type: String,
    /// Optional usage example
    pub example: Option<String>,
}

impl FunctionDoc {
    /// Create a new function documentation entry.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: Vec::new(),
            return_type: String::new(),
            example: None,
        }
    }

    /// Add a parameter.
    pub fn with_param(mut self, name: impl Into<String>, param_type: impl Into<String>, description: impl Into<String>) -> Self {
        self.parameters.push(ParameterDoc::new(name, param_type, description));
        self
    }

    /// Set the return type.
    pub fn with_return_type(mut self, return_type: impl Into<String>) -> Self {
        self.return_type = return_type.into();
        self
    }

    /// Set an example.
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }
}

/// Complete documentation for a library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDoc {
    /// Library name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Library version
    pub version: String,
    /// Selector functions documentation
    pub functions: Vec<FunctionDoc>,
    /// Check functions documentation
    pub checks: Vec<FunctionDoc>,
}

impl LibraryDoc {
    /// Create a new library documentation entry.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            version: String::from("0.1.0"),
            functions: Vec::new(),
            checks: Vec::new(),
        }
    }

    /// Set description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Add a selector function.
    pub fn with_function(mut self, func: FunctionDoc) -> Self {
        self.functions.push(func);
        self
    }

    /// Add a check function.
    pub fn with_check(mut self, check: FunctionDoc) -> Self {
        self.checks.push(check);
        self
    }
}

/// Full documentation catalog containing all libraries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationCatalog {
    /// Catalog format version
    pub version: String,
    /// Documentation for each library
    pub libraries: Vec<LibraryDoc>,
}

impl DocumentationCatalog {
    /// Create a new empty catalog.
    pub fn new() -> Self {
        Self {
            version: String::from("1.0"),
            libraries: Vec::new(),
        }
    }

    /// Add a library to the catalog.
    pub fn add_library(&mut self, lib: LibraryDoc) {
        self.libraries.push(lib);
    }

    /// Generate JSON representation of the catalog.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Generate Markdown representation of the catalog.
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        
        md.push_str("# Hielements Library Documentation\n\n");
        md.push_str("This catalog documents all available Hielements libraries, their selectors, and checks.\n\n");
        md.push_str("---\n\n");
        
        md.push_str("## Table of Contents\n\n");
        for lib in &self.libraries {
            md.push_str(&format!("- [{}](#{})\n", lib.name, lib.name.to_lowercase().replace(' ', "-")));
        }
        md.push_str("\n---\n\n");
        
        for lib in &self.libraries {
            md.push_str(&generate_library_markdown(lib));
        }
        
        md
    }
}

impl Default for DocumentationCatalog {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate Markdown documentation for a single library.
fn generate_library_markdown(lib: &LibraryDoc) -> String {
    let mut md = String::new();
    
    md.push_str(&format!("## {}\n\n", lib.name));
    
    if !lib.description.is_empty() {
        md.push_str(&format!("{}\n\n", lib.description));
    }
    
    if !lib.version.is_empty() {
        md.push_str(&format!("**Version:** {}\n\n", lib.version));
    }
    
    // Selectors section
    if !lib.functions.is_empty() {
        md.push_str("### Selectors\n\n");
        for func in &lib.functions {
            md.push_str(&generate_function_markdown(func, &lib.name));
        }
    }
    
    // Checks section
    if !lib.checks.is_empty() {
        md.push_str("### Checks\n\n");
        for check in &lib.checks {
            md.push_str(&generate_function_markdown(check, &lib.name));
        }
    }
    
    md.push_str("---\n\n");
    md
}

/// Generate Markdown documentation for a single function/check.
fn generate_function_markdown(func: &FunctionDoc, lib_name: &str) -> String {
    let mut md = String::new();
    
    // Function signature
    let params: Vec<String> = func.parameters.iter()
        .map(|p| format!("{}: {}", p.name, p.param_type))
        .collect();
    let return_type = if func.return_type.is_empty() {
        String::new()
    } else {
        format!(" -> {}", func.return_type)
    };
    md.push_str(&format!("#### `{}.{}({}){}`\n\n", lib_name, func.name, params.join(", "), return_type));
    
    // Description
    if !func.description.is_empty() {
        md.push_str(&format!("{}\n\n", func.description));
    }
    
    // Parameters table
    if !func.parameters.is_empty() {
        md.push_str("**Parameters:**\n\n");
        for param in &func.parameters {
            md.push_str(&format!("- `{}` ({}): {}\n", param.name, param.param_type, param.description));
        }
        md.push_str("\n");
    }
    
    // Example
    if let Some(ref example) = func.example {
        md.push_str("**Example:**\n\n");
        md.push_str("```hielements\n");
        md.push_str(example);
        if !example.ends_with('\n') {
            md.push_str("\n");
        }
        md.push_str("```\n\n");
    }
    
    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_doc_builder() {
        let func = FunctionDoc::new("file_selector", "Select a file by path")
            .with_param("path", "string", "Path to the file")
            .with_return_type("Scope")
            .with_example("scope main = files.file_selector('main.rs')");
        
        assert_eq!(func.name, "file_selector");
        assert_eq!(func.parameters.len(), 1);
        assert_eq!(func.return_type, "Scope");
        assert!(func.example.is_some());
    }

    #[test]
    fn test_library_doc_builder() {
        let lib = LibraryDoc::new("files")
            .with_description("File system operations")
            .with_version("1.0.0")
            .with_function(FunctionDoc::new("file_selector", "Select a file"));
        
        assert_eq!(lib.name, "files");
        assert_eq!(lib.functions.len(), 1);
    }

    #[test]
    fn test_catalog_json() {
        let mut catalog = DocumentationCatalog::new();
        catalog.add_library(LibraryDoc::new("test"));
        
        let json = catalog.to_json();
        assert!(json.contains("\"name\": \"test\""));
    }

    #[test]
    fn test_catalog_markdown() {
        let mut catalog = DocumentationCatalog::new();
        catalog.add_library(
            LibraryDoc::new("files")
                .with_description("File operations")
                .with_function(
                    FunctionDoc::new("file_selector", "Select files")
                        .with_param("path", "string", "File path")
                        .with_return_type("Scope")
                )
        );
        
        let md = catalog.to_markdown();
        assert!(md.contains("# Hielements Library Documentation"));
        assert!(md.contains("## files"));
        assert!(md.contains("file_selector"));
    }
}
