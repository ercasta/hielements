//! MCP Resource Handlers for Hielements
//!
//! Resources expose data (specifications, patterns, documentation) for agents to read.

use std::fs;
use std::path::PathBuf;

use rust_mcp_sdk::schema::{Resource, ReadResourceContent, TextResourceContents};
use tracing::debug;

/// Handler for MCP resources
pub struct ResourceHandler {
    workspace: PathBuf,
}

impl ResourceHandler {
    /// Create a new resource handler
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }

    /// Helper to create a Resource
    fn make_resource(uri: &str, name: &str, mime_type: &str, description: &str) -> Resource {
        Resource {
            uri: uri.to_string(),
            name: name.to_string(),
            mime_type: Some(mime_type.to_string()),
            description: Some(description.to_string()),
            annotations: None,
            icons: vec![],
            meta: None,
            size: None,
            title: None,
        }
    }

    /// List all available resources
    pub fn list_resources(&self) -> Vec<Resource> {
        let mut resources = Vec::new();

        // Add workspace specification resources
        resources.push(Self::make_resource(
            "hielements://workspace/specifications",
            "Workspace Specifications",
            "application/json",
            "List of all .hie files in the workspace"
        ));

        // Add pattern library resource
        resources.push(Self::make_resource(
            "hielements://patterns/catalog",
            "Pattern Library",
            "application/json",
            "Available architectural patterns that can be implemented"
        ));

        // Add library documentation resource
        resources.push(Self::make_resource(
            "hielements://libraries/docs",
            "Library Documentation",
            "application/json",
            "Documentation for all available Hielements libraries"
        ));

        // Add language reference resource
        resources.push(Self::make_resource(
            "hielements://docs/language-reference",
            "Language Reference",
            "text/markdown",
            "Hielements language syntax and semantics reference"
        ));

        // Add specific .hie files in the workspace
        if let Ok(entries) = fs::read_dir(&self.workspace) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "hie") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        resources.push(Self::make_resource(
                            &format!("hielements://workspace/file/{}", name),
                            name,
                            "text/x-hielements",
                            &format!("Hielements specification: {}", name)
                        ));
                    }
                }
            }
        }

        resources
    }

    /// Read a resource by URI
    pub fn read_resource(&self, uri: &str) -> Result<Vec<ReadResourceContent>, String> {
        debug!("Reading resource: {}", uri);

        match uri {
            "hielements://workspace/specifications" => {
                self.read_specifications_list()
            }
            "hielements://patterns/catalog" => {
                self.read_pattern_catalog()
            }
            "hielements://libraries/docs" => {
                self.read_library_docs()
            }
            "hielements://docs/language-reference" => {
                self.read_language_reference()
            }
            _ if uri.starts_with("hielements://workspace/file/") => {
                let filename = uri.strip_prefix("hielements://workspace/file/").unwrap();
                self.read_specification_file(filename)
            }
            _ => {
                Err(format!("Resource not found: {}", uri))
            }
        }
    }

    /// Helper to create a TextResourceContents
    fn make_text_content(uri: &str, mime_type: &str, text: String) -> ReadResourceContent {
        ReadResourceContent::TextResourceContents(TextResourceContents {
            uri: uri.to_string(),
            mime_type: Some(mime_type.to_string()),
            text,
            meta: None,
        })
    }

    /// Read the list of specifications in the workspace
    fn read_specifications_list(&self) -> Result<Vec<ReadResourceContent>, String> {
        let mut specs = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.workspace) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "hie") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        specs.push(serde_json::json!({
                            "name": name,
                            "path": path.display().to_string(),
                            "uri": format!("hielements://workspace/file/{}", name)
                        }));
                    }
                }
            }
        }

        let content = serde_json::json!({
            "specifications": specs,
            "workspace": self.workspace.display().to_string()
        });

        Ok(vec![Self::make_text_content(
            "hielements://workspace/specifications",
            "application/json",
            serde_json::to_string_pretty(&content).unwrap_or_default()
        )])
    }

    /// Read the pattern catalog
    fn read_pattern_catalog(&self) -> Result<Vec<ReadResourceContent>, String> {
        let patterns_dir = self.workspace.join("patterns");
        let mut categories = Vec::new();

        if patterns_dir.exists() {
            for entry in fs::read_dir(&patterns_dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(category_name) = path.file_name().and_then(|n| n.to_str()) {
                        let mut patterns = Vec::new();
                        if let Ok(pattern_files) = fs::read_dir(&path) {
                            for pf in pattern_files.flatten() {
                                let pf_path = pf.path();
                                if pf_path.extension().map_or(false, |ext| ext == "hie") {
                                    if let Some(name) = pf_path.file_stem().and_then(|n| n.to_str()) {
                                        patterns.push(serde_json::json!({
                                            "name": name,
                                            "file": pf_path.display().to_string()
                                        }));
                                    }
                                }
                            }
                        }
                        categories.push(serde_json::json!({
                            "category": category_name,
                            "patterns": patterns
                        }));
                    }
                }
            }
        }

        let content = serde_json::json!({
            "pattern_library": categories,
            "description": "Reusable architectural patterns for Hielements"
        });

        Ok(vec![Self::make_text_content(
            "hielements://patterns/catalog",
            "application/json",
            serde_json::to_string_pretty(&content).unwrap_or_default()
        )])
    }

    /// Read library documentation
    fn read_library_docs(&self) -> Result<Vec<ReadResourceContent>, String> {
        use hielements_core::LibraryRegistry;

        let workspace_str = self.workspace.to_str().unwrap_or_else(|| {
            tracing::warn!("Workspace path contains non-UTF8 characters, using '.'");
            "."
        });
        let registry = LibraryRegistry::with_workspace(workspace_str);
        let catalog = registry.generate_documentation();
        
        Ok(vec![Self::make_text_content(
            "hielements://libraries/docs",
            "application/json",
            catalog.to_json()
        )])
    }

    /// Read the language reference documentation
    fn read_language_reference(&self) -> Result<Vec<ReadResourceContent>, String> {
        let lang_ref_path = self.workspace.join("doc/language_reference.md");
        
        let content = if lang_ref_path.exists() {
            fs::read_to_string(&lang_ref_path).unwrap_or_else(|_| self.default_language_reference())
        } else {
            self.default_language_reference()
        };

        Ok(vec![Self::make_text_content(
            "hielements://docs/language-reference",
            "text/markdown",
            content
        )])
    }

    /// Read a specific specification file
    fn read_specification_file(&self, filename: &str) -> Result<Vec<ReadResourceContent>, String> {
        // Security: prevent directory traversal
        if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
            return Err("Invalid filename".to_string());
        }

        let file_path = self.workspace.join(filename);
        
        if !file_path.exists() {
            return Err(format!("File not found: {}", filename));
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        Ok(vec![Self::make_text_content(
            &format!("hielements://workspace/file/{}", filename),
            "text/x-hielements",
            content
        )])
    }

    /// Default language reference if file not found
    fn default_language_reference(&self) -> String {
        r#"# Hielements Language Reference

## Overview
Hielements is a language for describing and enforcing software architecture.

## Basic Syntax

### Elements
Elements represent logical components:
```hielements
element my_component {
    # Element content
}
```

### Scopes
Scopes define what code/artifacts belong to an element:
```hielements
scope src = files.folder_selector('src/')
```

### Checks
Checks verify properties of your system:
```hielements
check files.exists(src, 'main.py')
```

### Patterns
Define reusable architectural blueprints:
```hielements
pattern microservice {
    element api {
        scope module<python>
    }
}
```

For full documentation, see: https://github.com/ercasta/hielements
"#.to_string()
    }
}
