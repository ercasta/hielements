//! MCP Tool Handlers for Hielements
//!
//! Tools are callable functions (check, run, generate) with typed parameters.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use hielements_core::{Interpreter, RunOptions};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, Tool, ToolInputSchema};
use rust_mcp_sdk::schema::schema_utils::CallToolError;
use serde_json::{json, Map, Value};
use tracing::debug;

/// Handler for MCP tools
pub struct ToolHandler {
    workspace: PathBuf,
}

impl ToolHandler {
    /// Create a new tool handler
    pub fn new(workspace: PathBuf) -> Self {
        Self { workspace }
    }

    /// Get workspace path as string, with proper error handling
    fn workspace_str(&self) -> &str {
        self.workspace.to_str().unwrap_or_else(|| {
            tracing::warn!("Workspace path contains non-UTF8 characters, using '.'");
            "."
        })
    }

    /// Helper to create a successful JSON result response
    fn json_result(result: Value) -> Result<CallToolResult, CallToolError> {
        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&result).unwrap_or_else(|e| {
                format!("{{\"error\": \"Failed to serialize result: {}\"}}", e)
            })
        )]))
    }

    /// Create a property map for a tool parameter
    fn make_prop(description: &str, param_type: &str) -> Map<String, Value> {
        let mut prop = Map::new();
        prop.insert("type".to_string(), json!(param_type));
        prop.insert("description".to_string(), json!(description));
        prop
    }

    /// Create a ToolInputSchema
    fn make_schema(properties: HashMap<String, Map<String, Value>>, required: Vec<String>) -> ToolInputSchema {
        ToolInputSchema::new(required, Some(properties), None)
    }

    /// Create a Tool
    fn make_tool(name: &str, description: &str, input_schema: ToolInputSchema) -> Tool {
        Tool {
            name: name.to_string(),
            description: Some(description.to_string()),
            input_schema,
            annotations: None,
            execution: None,
            icons: vec![],
            meta: None,
            output_schema: None,
            title: None,
        }
    }

    /// List all available tools
    pub fn list_tools(&self) -> Vec<Tool> {
        vec![
            self.tool_check_specification(),
            self.tool_check_file(),
            self.tool_run_checks(),
            self.tool_list_patterns(),
            self.tool_get_pattern(),
            self.tool_list_libraries(),
            self.tool_explain_error(),
            self.tool_generate_element(),
        ]
    }

    fn tool_check_specification(&self) -> Tool {
        let mut properties = HashMap::new();
        properties.insert("content".to_string(), Self::make_prop("The Hielements specification content to validate", "string"));
        properties.insert("filename".to_string(), Self::make_prop("Optional filename for error reporting (default: 'input.hie')", "string"));
        
        Self::make_tool(
            "check_specification",
            "Validate a Hielements specification for syntax and semantic errors",
            Self::make_schema(properties, vec!["content".to_string()])
        )
    }

    fn tool_check_file(&self) -> Tool {
        let mut properties = HashMap::new();
        properties.insert("filename".to_string(), Self::make_prop("The .hie file to validate (relative to workspace)", "string"));
        
        Self::make_tool(
            "check_file",
            "Validate a Hielements specification file in the workspace",
            Self::make_schema(properties, vec!["filename".to_string()])
        )
    }

    fn tool_run_checks(&self) -> Tool {
        let mut properties = HashMap::new();
        properties.insert("content".to_string(), Self::make_prop("The Hielements specification content", "string"));
        properties.insert("filename".to_string(), Self::make_prop("Or specify a file in the workspace to run", "string"));
        properties.insert("filter".to_string(), Self::make_prop("Optional filter pattern for checks (e.g., 'core.lexer')", "string"));
        properties.insert("limit".to_string(), Self::make_prop("Maximum number of checks to run", "integer"));
        
        Self::make_tool(
            "run_checks",
            "Execute checks defined in a Hielements specification",
            Self::make_schema(properties, vec![])
        )
    }

    fn tool_list_patterns(&self) -> Tool {
        let mut properties = HashMap::new();
        properties.insert("category".to_string(), Self::make_prop("Filter by category (structural, behavioral, etc.)", "string"));
        
        Self::make_tool(
            "list_patterns",
            "List available architectural patterns",
            Self::make_schema(properties, vec![])
        )
    }

    fn tool_get_pattern(&self) -> Tool {
        let mut properties = HashMap::new();
        properties.insert("category".to_string(), Self::make_prop("Pattern category", "string"));
        properties.insert("name".to_string(), Self::make_prop("Pattern name", "string"));
        
        Self::make_tool(
            "get_pattern",
            "Get details of a specific pattern",
            Self::make_schema(properties, vec!["category".to_string(), "name".to_string()])
        )
    }

    fn tool_list_libraries(&self) -> Tool {
        let mut properties = HashMap::new();
        properties.insert("library".to_string(), Self::make_prop("Filter to a specific library name", "string"));
        
        Self::make_tool(
            "list_libraries",
            "List available Hielements libraries and their functions",
            Self::make_schema(properties, vec![])
        )
    }

    fn tool_explain_error(&self) -> Tool {
        let mut properties = HashMap::new();
        properties.insert("code".to_string(), Self::make_prop("The error code (e.g., 'E001', 'W001')", "string"));
        
        Self::make_tool(
            "explain_error",
            "Get detailed explanation of an error code",
            Self::make_schema(properties, vec!["code".to_string()])
        )
    }

    fn tool_generate_element(&self) -> Tool {
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), Self::make_prop("Element name", "string"));
        properties.insert("pattern".to_string(), Self::make_prop("Optional pattern to implement", "string"));
        properties.insert("language".to_string(), Self::make_prop("Primary language (rust, python, etc.)", "string"));
        
        Self::make_tool(
            "generate_element",
            "Generate a basic Hielements element structure",
            Self::make_schema(properties, vec!["name".to_string()])
        )
    }

    /// Call a tool with the given arguments
    pub fn call_tool(&self, name: &str, arguments: Map<String, Value>) -> Result<CallToolResult, CallToolError> {
        debug!("Calling tool: {} with args: {:?}", name, arguments);

        match name {
            "check_specification" => self.check_specification(arguments),
            "check_file" => self.check_file(arguments),
            "run_checks" => self.run_checks(arguments),
            "list_patterns" => self.list_patterns(arguments),
            "get_pattern" => self.get_pattern(arguments),
            "list_libraries" => self.list_libraries(arguments),
            "explain_error" => self.explain_error(arguments),
            "generate_element" => self.generate_element(arguments),
            _ => Err(CallToolError::unknown_tool(name)),
        }
    }

    /// Check a specification for syntax and semantic errors
    fn check_specification(&self, args: Map<String, Value>) -> Result<CallToolResult, CallToolError> {
        let content = args.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CallToolError::from_message("Missing 'content' parameter"))?;
        
        let filename = args.get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("input.hie");

        let mut interpreter = Interpreter::new(self.workspace_str());
        let (program, diagnostics) = interpreter.validate(content, filename);

        let result = json!({
            "status": if diagnostics.has_errors() { "error" } else { "ok" },
            "has_program": program.is_some(),
            "diagnostics": {
                "errors": diagnostics.errors().count(),
                "warnings": diagnostics.warnings().count(),
                "messages": diagnostics.iter().map(|d| {
                    json!({
                        "severity": format!("{:?}", d.severity),
                        "code": d.code,
                        "message": d.message,
                        "file": d.file,
                        "line": d.span.start.line,
                        "column": d.span.start.column,
                        "help": d.help
                    })
                }).collect::<Vec<_>>()
            }
        });

        Self::json_result(result)
    }

    /// Check a file in the workspace
    fn check_file(&self, args: Map<String, Value>) -> Result<CallToolResult, CallToolError> {
        let filename = args.get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CallToolError::from_message("Missing 'filename' parameter"))?;

        // Security: prevent directory traversal
        if filename.contains("..") {
            return Err(CallToolError::from_message("Invalid filename"));
        }

        let file_path = self.workspace.join(filename);
        
        if !file_path.exists() {
            return Err(CallToolError::from_message(format!("File not found: {}", filename)));
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| CallToolError::from_message(format!("Failed to read file: {}", e)))?;

        let mut args_with_content = args.clone();
        args_with_content.insert("content".to_string(), Value::String(content));
        args_with_content.insert("filename".to_string(), Value::String(filename.to_string()));

        self.check_specification(args_with_content)
    }

    /// Run checks in a specification
    fn run_checks(&self, args: Map<String, Value>) -> Result<CallToolResult, CallToolError> {
        // Get content either directly or from file
        let content = if let Some(content) = args.get("content").and_then(|v| v.as_str()) {
            content.to_string()
        } else if let Some(filename) = args.get("filename").and_then(|v| v.as_str()) {
            // Security: prevent directory traversal
            if filename.contains("..") {
                return Err(CallToolError::from_message("Invalid filename"));
            }
            let file_path = self.workspace.join(filename);
            fs::read_to_string(&file_path)
                .map_err(|e| CallToolError::from_message(format!("Failed to read file: {}", e)))?
        } else {
            return Err(CallToolError::from_message("Either 'content' or 'filename' is required"));
        };

        let filename = args.get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("input.hie");

        let mut interpreter = Interpreter::new(self.workspace_str());
        let (program, diagnostics) = interpreter.validate(&content, filename);

        if diagnostics.has_errors() {
            let result = json!({
                "status": "validation_error",
                "message": "Specification has validation errors",
                "diagnostics": diagnostics.errors().map(|d| {
                    json!({
                        "code": d.code,
                        "message": d.message,
                        "line": d.span.start.line
                    })
                }).collect::<Vec<_>>()
            });

            return Ok(CallToolResult::text_content(vec![TextContent::from(
                serde_json::to_string_pretty(&result).unwrap_or_default()
            )]));
        }

        let program = match program {
            Some(p) => p,
            None => {
                return Ok(CallToolResult::text_content(vec![TextContent::from(
                    json!({"status": "error", "message": "Failed to parse specification"}).to_string()
                )]));
            }
        };

        let options = RunOptions {
            filter: args.get("filter").and_then(|v| v.as_str()).map(String::from),
            limit: args.get("limit").and_then(|v| v.as_i64()).map(|n| n as usize),
            verbose: false,
        };

        let output = interpreter.run_with_options(&program, &options);

        let result = json!({
            "status": if output.failed == 0 && output.errors == 0 { "ok" } else { "failed" },
            "summary": {
                "total": output.total,
                "passed": output.passed,
                "failed": output.failed,
                "errors": output.errors,
                "skipped": output.skipped
            },
            "results": output.results.iter().map(|r| {
                json!({
                    "element": r.element_path,
                    "check": r.check_expr,
                    "status": match &r.result {
                        hielements_core::stdlib::CheckResult::Pass => "pass",
                        hielements_core::stdlib::CheckResult::Fail(_) => "fail",
                        hielements_core::stdlib::CheckResult::Error(_) => "error"
                    },
                    "message": match &r.result {
                        hielements_core::stdlib::CheckResult::Pass => None,
                        hielements_core::stdlib::CheckResult::Fail(msg) => Some(msg.clone()),
                        hielements_core::stdlib::CheckResult::Error(msg) => Some(msg.clone())
                    }
                })
            }).collect::<Vec<_>>()
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&result).unwrap_or_default()
        )]))
    }

    /// List available patterns
    fn list_patterns(&self, args: Map<String, Value>) -> Result<CallToolResult, CallToolError> {
        let category_filter = args.get("category").and_then(|v| v.as_str());
        let patterns_dir = self.workspace.join("patterns");
        
        let mut categories = Vec::new();

        if patterns_dir.exists() {
            for entry in fs::read_dir(&patterns_dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(category_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Apply category filter if provided
                        if let Some(filter) = category_filter {
                            if category_name != filter {
                                continue;
                            }
                        }

                        let mut patterns = Vec::new();
                        if let Ok(pattern_files) = fs::read_dir(&path) {
                            for pf in pattern_files.flatten() {
                                let pf_path = pf.path();
                                if pf_path.extension().map_or(false, |ext| ext == "hie") {
                                    if let Some(name) = pf_path.file_stem().and_then(|n| n.to_str()) {
                                        patterns.push(json!({
                                            "name": name,
                                            "file": pf_path.display().to_string()
                                        }));
                                    }
                                }
                            }
                        }
                        
                        if !patterns.is_empty() {
                            categories.push(json!({
                                "category": category_name,
                                "patterns": patterns
                            }));
                        }
                    }
                }
            }
        }

        let result = json!({
            "pattern_library": categories,
            "total_patterns": categories.iter()
                .flat_map(|c| c.get("patterns").and_then(|p| p.as_array()))
                .flatten()
                .count()
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&result).unwrap_or_default()
        )]))
    }

    /// Get a specific pattern
    fn get_pattern(&self, args: Map<String, Value>) -> Result<CallToolResult, CallToolError> {
        let category = args.get("category")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CallToolError::from_message("Missing 'category' parameter"))?;
        
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CallToolError::from_message("Missing 'name' parameter"))?;

        // Security: prevent directory traversal
        if category.contains("..") || name.contains("..") {
            return Err(CallToolError::from_message("Invalid path"));
        }

        let pattern_path = self.workspace.join("patterns").join(category).join(format!("{}.hie", name));
        
        if !pattern_path.exists() {
            return Err(CallToolError::from_message(format!("Pattern not found: {}/{}", category, name)));
        }

        let content = fs::read_to_string(&pattern_path)
            .map_err(|e| CallToolError::from_message(format!("Failed to read pattern: {}", e)))?;

        let result = json!({
            "category": category,
            "name": name,
            "content": content,
            "path": pattern_path.display().to_string()
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&result).unwrap_or_default()
        )]))
    }

    /// List available libraries
    fn list_libraries(&self, args: Map<String, Value>) -> Result<CallToolResult, CallToolError> {
        use hielements_core::LibraryRegistry;

        let library_filter = args.get("library").and_then(|v| v.as_str());
        
        let registry = LibraryRegistry::with_workspace(
            self.workspace_str()
        );
        let mut catalog = registry.generate_documentation();

        // Apply filter if provided
        if let Some(filter) = library_filter {
            catalog.libraries.retain(|lib| lib.name == filter);
        }

        let result = json!({
            "libraries": catalog.libraries.iter().map(|lib| {
                json!({
                    "name": lib.name,
                    "description": lib.description,
                    "version": lib.version,
                    "functions": lib.functions.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
                    "checks": lib.checks.iter().map(|f| f.name.clone()).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>(),
            "total": catalog.libraries.len()
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&result).unwrap_or_default()
        )]))
    }

    /// Explain an error code
    fn explain_error(&self, args: Map<String, Value>) -> Result<CallToolResult, CallToolError> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CallToolError::from_message("Missing 'code' parameter"))?;

        let explanation = match code {
            "E001" => "Unknown element type. Use 'element' keyword to declare elements.",
            "E002" => "Syntax error in element declaration. Check brackets and colons.",
            "E003" => "Invalid scope expression. Scopes must be assigned selector function calls.",
            "E004" => "Invalid check expression. Checks must be function calls.",
            "E005" => "Unknown pattern. The referenced pattern has not been defined.",
            "E100" => "Unknown library. The imported library is not available.",
            "E101" => "Unknown function. The called function does not exist in the library.",
            "E200" => "Undefined identifier. The referenced name has not been declared.",
            "E201" => "Cannot evaluate member access directly.",
            "E202" => "Undefined reference. The referenced scope or element does not exist.",
            "E203" => "Unknown library in function call.",
            "E204" => "Check must be a function call.",
            "E205" => "Expected library.function format for function calls.",
            "W001" => "Unknown library warning. The library will be resolved at runtime.",
            _ => "Unknown error code. Check the Hielements documentation for more information.",
        };

        let result = json!({
            "code": code,
            "explanation": explanation,
            "severity": if code.starts_with('E') { "error" } else { "warning" }
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&result).unwrap_or_default()
        )]))
    }

    /// Generate a basic element structure
    fn generate_element(&self, args: Map<String, Value>) -> Result<CallToolResult, CallToolError> {
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CallToolError::from_message("Missing 'name' parameter"))?;
        
        let pattern = args.get("pattern").and_then(|v| v.as_str());
        let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("files");

        let imports = match language {
            "rust" => "import files\nimport rust\n",
            "python" => "import files\nimport python\n",
            _ => "import files\n",
        };

        let implements_clause = pattern.map(|p| format!(" implements {}", p)).unwrap_or_default();
        
        let scope_line = match language {
            "rust" => format!("    scope src<rust> = rust.module_selector('{}')", name),
            "python" => format!("    scope src<python> = python.module_selector('{}')", name),
            _ => format!("    scope src = files.folder_selector('src/{}')", name),
        };

        let check_lines = match language {
            "rust" => vec![
                "    check rust.struct_exists('Main')".to_string(),
                "    check rust.has_tests(src)".to_string(),
            ],
            "python" => vec![
                "    check python.function_exists(src, '__init__')".to_string(),
            ],
            _ => vec![
                "    check files.exists(src, 'main.py')".to_string(),
            ],
        };

        let generated = format!(
            "{imports}\nelement {name}{implements_clause} {{\n{scope}\n{checks}\n}}\n",
            imports = imports,
            name = name,
            implements_clause = implements_clause,
            scope = scope_line,
            checks = check_lines.join("\n")
        );

        let result = json!({
            "element_name": name,
            "pattern": pattern,
            "language": language,
            "generated": generated
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&result).unwrap_or_default()
        )]))
    }
}
