//! Hielements CLI
//!
//! Command-line interface for the Hielements language.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use colored::Colorize;
use hielements_core::diagnostics::{DiagnosticSeverity, DiagnosticsOutput};
use hielements_core::stdlib::{CheckResult, LibraryRegistry};
use hielements_core::{Interpreter, RunOptions};

#[derive(Parser)]
#[command(name = "hielements")]
#[command(author, version, about = "A language for describing and enforcing software structure", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a Hielements specification (syntax and semantic checks only)
    Check {
        /// Path to the .hie file
        file: String,

        /// Output format (human, json)
        #[arg(short, long, default_value = "human")]
        format: String,
    },

    /// Run checks defined in a Hielements specification
    Run {
        /// Path to the .hie file
        file: String,

        /// Workspace directory (defaults to current directory)
        #[arg(short, long)]
        workspace: Option<String>,

        /// Output format (human, json)
        #[arg(short, long, default_value = "human")]
        format: String,

        /// Dry run - show what would be checked without running
        #[arg(long)]
        dry_run: bool,

        /// Verbose mode - show progress as each check runs
        #[arg(short, long)]
        verbose: bool,

        /// Filter checks by element path pattern (e.g., "core.lexer" or "stdlib")
        #[arg(long)]
        filter: Option<String>,

        /// Limit the number of checks to run
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Parse a file and print the AST (for debugging)
    Parse {
        /// Path to the .hie file
        file: String,
    },

    /// Generate documentation for available libraries
    Doc {
        /// Workspace directory (defaults to current directory)
        #[arg(short, long)]
        workspace: Option<String>,

        /// Output format (markdown, json)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Filter to specific libraries (comma-separated)
        #[arg(short, long)]
        library: Option<String>,
    },

    /// Initialize a new Hielements project
    Init {
        /// Project name (used to name the initial element and .hie file)
        project_name: String,

        /// Target directory (defaults to current directory)
        #[arg(short, long)]
        directory: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { file, format } => cmd_check(&file, &format),
        Commands::Run { file, workspace, format, dry_run, verbose, filter, limit } => {
            cmd_run(&file, workspace.as_deref(), &format, dry_run, verbose, filter.as_deref(), limit)
        }
        Commands::Parse { file } => cmd_parse(&file),
        Commands::Doc { workspace, format, output, library } => {
            cmd_doc(workspace.as_deref(), &format, output.as_deref(), library.as_deref())
        }
        Commands::Init { project_name, directory } => {
            cmd_init(&project_name, directory.as_deref())
        }
    }
}

fn cmd_check(file: &str, format: &str) -> ExitCode {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} Failed to read file '{}': {}", "error:".red().bold(), file, e);
            return ExitCode::from(2);
        }
    };

    let mut interpreter = Interpreter::new(".");
    let (program, diagnostics) = interpreter.validate(&source, file);

    match format {
        "json" => {
            let output = DiagnosticsOutput::from_diagnostics(&diagnostics);
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            // Human-readable output
            for diag in diagnostics.iter() {
                let severity_str = match diag.severity {
                    DiagnosticSeverity::Error => "error".red().bold(),
                    DiagnosticSeverity::Warning => "warning".yellow().bold(),
                    DiagnosticSeverity::Info => "info".blue().bold(),
                    DiagnosticSeverity::Hint => "hint".cyan().bold(),
                };

                println!(
                    "{}{} {} {}",
                    severity_str,
                    format!("[{}]", diag.code).dimmed(),
                    ":".bold(),
                    diag.message
                );

                println!(
                    "  {} {}:{}:{}",
                    "-->".blue().bold(),
                    diag.file,
                    diag.span.start.line,
                    diag.span.start.column
                );

                if let Some(ref context) = diag.context {
                    println!("   {}", "|".blue().bold());
                    println!(
                        "{:>3} {} {}",
                        diag.span.start.line.to_string().blue().bold(),
                        "|".blue().bold(),
                        context
                    );
                    println!("   {}", "|".blue().bold());
                }

                if let Some(ref help) = diag.help {
                    println!("   {} {}: {}", "=".blue().bold(), "help".bold(), help);
                }

                println!();
            }

            if diagnostics.has_errors() {
                let error_count = diagnostics.errors().count();
                let warning_count = diagnostics.warnings().count();
                eprintln!(
                    "{}: could not validate `{}` due to {} previous error{}{}",
                    "error".red().bold(),
                    file,
                    error_count,
                    if error_count == 1 { "" } else { "s" },
                    if warning_count > 0 {
                        format!("; {} warning{} emitted", warning_count, if warning_count == 1 { "" } else { "s" })
                    } else {
                        String::new()
                    }
                );
            } else if program.is_some() {
                let warning_count = diagnostics.warnings().count();
                if warning_count > 0 {
                    println!(
                        "{} `{}` validated with {} warning{}",
                        "Finished".green().bold(),
                        file,
                        warning_count,
                        if warning_count == 1 { "" } else { "s" }
                    );
                } else {
                    println!("{} `{}` validated successfully", "Finished".green().bold(), file);
                }
            }
        }
    }

    if diagnostics.has_errors() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn cmd_run(file: &str, workspace: Option<&str>, format: &str, dry_run: bool, verbose: bool, filter: Option<&str>, limit: Option<usize>) -> ExitCode {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} Failed to read file '{}': {}", "error:".red().bold(), file, e);
            return ExitCode::from(2);
        }
    };

    // Determine workspace directory
    let workspace_dir = workspace
        .map(|w| w.to_string())
        .or_else(|| {
            Path::new(file).parent()
                .and_then(|p| {
                    let s = p.to_string_lossy().to_string();
                    if s.is_empty() { None } else { Some(s) }
                })
        })
        .unwrap_or_else(|| ".".to_string());

    if verbose {
        eprintln!("[verbose] Workspace directory: {}", workspace_dir);
        if let Some(f) = filter {
            eprintln!("[verbose] Filter: {}", f);
        }
        if let Some(l) = limit {
            eprintln!("[verbose] Limit: {} checks", l);
        }
    }

    let mut interpreter = Interpreter::new(&workspace_dir);
    let (program, diagnostics) = interpreter.validate(&source, file);

    // Check for validation errors
    if diagnostics.has_errors() {
        match format {
            "json" => {
                let output = DiagnosticsOutput::from_diagnostics(&diagnostics);
                println!("{}", serde_json::to_string_pretty(&output).unwrap());
            }
            _ => {
                for diag in diagnostics.errors() {
                    eprintln!(
                        "{}{}: {}",
                        "error".red().bold(),
                        format!("[{}]", diag.code).dimmed(),
                        diag.message
                    );
                    eprintln!(
                        "  {} {}:{}:{}",
                        "-->".blue().bold(),
                        diag.file,
                        diag.span.start.line,
                        diag.span.start.column
                    );
                }
            }
        }
        return ExitCode::from(1);
    }

    let program = match program {
        Some(p) => p,
        None => {
            eprintln!("{} Failed to parse file", "error:".red().bold());
            return ExitCode::from(1);
        }
    };

    if dry_run {
        println!("{} Dry run - showing checks that would be executed:", "Info".blue().bold());
        println!();
        print_checks_dry_run(&program, 0);
        return ExitCode::SUCCESS;
    }

    // Run the checks with options
    let run_options = RunOptions {
        filter: filter.map(|s| s.to_string()),
        limit,
        verbose,
    };
    let output = interpreter.run_with_options(&program, &run_options);

    match format {
        "json" => {
            let json_output = serde_json::json!({
                "version": "1.0",
                "status": if output.failed == 0 && output.errors == 0 { "ok" } else { "error" },
                "summary": {
                    "total": output.total,
                    "passed": output.passed,
                    "failed": output.failed,
                    "errors": output.errors
                },
                "results": output.results.iter().map(|r| {
                    serde_json::json!({
                        "element": r.element_path,
                        "check": r.check_expr,
                        "status": match &r.result {
                            CheckResult::Pass => "pass",
                            CheckResult::Fail(_) => "fail",
                            CheckResult::Error(_) => "error"
                        },
                        "message": match &r.result {
                            CheckResult::Pass => None,
                            CheckResult::Fail(msg) => Some(msg.clone()),
                            CheckResult::Error(msg) => Some(msg.clone())
                        }
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        }
        _ => {
            // Human-readable output
            println!("{} Running checks in `{}`...", "Starting".green().bold(), file);
            println!();

            for result in &output.results {
                let status = match &result.result {
                    CheckResult::Pass => "PASS".green().bold(),
                    CheckResult::Fail(_) => "FAIL".red().bold(),
                    CheckResult::Error(_) => "ERROR".yellow().bold(),
                };

                println!("  {} {} :: {}", status, result.element_path.dimmed(), result.check_expr);

                match &result.result {
                    CheckResult::Fail(msg) => {
                        println!("        {} {}", "-->".red(), msg);
                    }
                    CheckResult::Error(msg) => {
                        println!("        {} {}", "-->".yellow(), msg);
                    }
                    _ => {}
                }
            }

            println!();
            let skipped_str = if output.skipped > 0 {
                format!(", {} skipped", output.skipped)
            } else {
                String::new()
            };
            println!(
                "{}: {} total, {} passed, {} failed, {} errors{}",
                "Summary".bold(),
                output.total,
                output.passed.to_string().green(),
                if output.failed > 0 {
                    output.failed.to_string().red().to_string()
                } else {
                    output.failed.to_string()
                },
                if output.errors > 0 {
                    output.errors.to_string().yellow().to_string()
                } else {
                    output.errors.to_string()
                },
                skipped_str
            );
        }
    }

    if output.failed > 0 || output.errors > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_checks_dry_run(program: &hielements_core::Program, indent: usize) {
    for element in &program.elements {
        print_element_checks(&element, indent);
    }
}

fn print_element_checks(element: &hielements_core::Element, indent: usize) {
    let prefix = "  ".repeat(indent);
    println!("{}element {}:", prefix, element.name.name.cyan());

    for scope in &element.scopes {
        println!("{}  scope {} = ...", prefix, scope.name.name.blue());
    }

    for r in &element.refs {
        let type_info = format!(": {}", r.type_annotation.type_name.name);
        println!("{}  ref {}{} = ...", prefix, r.name.name.magenta(), type_info.yellow());
    }

    for u in &element.uses {
        let target = u.target.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join(".");
        println!("{}  {} uses {}", prefix, u.source.name.blue(), target.cyan());
    }

    for _check in &element.checks {
        println!("{}  {} check ...", prefix, "→".green());
    }

    for child in &element.children {
        print_element_checks(child, indent + 1);
    }
}

fn cmd_parse(file: &str) -> ExitCode {
    let source = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} Failed to read file '{}': {}", "error:".red().bold(), file, e);
            return ExitCode::from(2);
        }
    };

    let mut interpreter = Interpreter::new(".");
    let (program, diagnostics) = interpreter.validate(&source, file);

    if diagnostics.has_errors() {
        for diag in diagnostics.errors() {
            eprintln!("{}: {}", "error".red().bold(), diag.message);
        }
        return ExitCode::from(1);
    }

    if let Some(program) = program {
        println!("{}", serde_json::to_string_pretty(&program).unwrap());
    }

    ExitCode::SUCCESS
}

fn cmd_doc(workspace: Option<&str>, format: &str, output: Option<&str>, library_filter: Option<&str>) -> ExitCode {
    // Use workspace or current directory
    let workspace_dir = workspace.unwrap_or(".");
    
    // Create library registry with workspace libraries
    let registry = LibraryRegistry::with_workspace(workspace_dir);
    
    // Generate documentation
    let mut catalog = registry.generate_documentation();
    
    // Filter libraries if specified
    if let Some(filter) = library_filter {
        let filter_libs: Vec<&str> = filter.split(',').map(|s| s.trim()).collect();
        catalog.libraries.retain(|lib| filter_libs.contains(&lib.name.as_str()));
    }
    
    // Sort libraries by name for consistent output
    catalog.libraries.sort_by(|a, b| a.name.cmp(&b.name));
    
    // Generate output based on format
    let output_content = match format {
        "json" => catalog.to_json(),
        "markdown" | "md" | _ => catalog.to_markdown(),
    };
    
    // Write output
    if let Some(output_file) = output {
        // Validate output path: ensure it's a relative path or within current directory
        let output_path = Path::new(output_file);
        
        // Reject absolute paths for safety
        if output_path.is_absolute() {
            eprintln!("{} Absolute paths are not allowed for output files. Use a relative path.", "error:".red().bold());
            return ExitCode::from(2);
        }
        
        // Reject paths with parent directory references (directory traversal)
        if output_path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            eprintln!("{} Output path cannot contain parent directory references (..).", "error:".red().bold());
            return ExitCode::from(2);
        }
        
        match fs::write(output_file, &output_content) {
            Ok(_) => {
                println!("{} Documentation written to '{}'", "Success".green().bold(), output_file);
            }
            Err(e) => {
                eprintln!("{} Failed to write to '{}': {}", "error:".red().bold(), output_file, e);
                return ExitCode::from(2);
            }
        }
    } else {
        println!("{}", output_content);
    }
    
    ExitCode::SUCCESS
}

fn cmd_init(project_name: &str, directory: Option<&str>) -> ExitCode {
    let target_dir = Path::new(directory.unwrap_or("."));
    
    // Validate project name (alphanumeric and underscores)
    if !project_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        eprintln!("{} Project name must contain only alphanumeric characters and underscores", "error:".red().bold());
        return ExitCode::from(2);
    }
    
    // Create target directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(target_dir) {
        eprintln!("{} Failed to create directory '{}': {}", "error:".red().bold(), target_dir.display(), e);
        return ExitCode::from(2);
    }
    
    // Use Path::join for safe path construction
    let hie_file = target_dir.join(format!("{}.hie", project_name));
    let config_file = target_dir.join("hielements.toml");
    let guide_file = target_dir.join("USAGE_GUIDE.md");
    
    // Check if files already exist
    if hie_file.exists() {
        eprintln!("{} File '{}' already exists", "error:".red().bold(), hie_file.display());
        return ExitCode::from(2);
    }
    
    // Generate initial .hie file
    let hie_content = format!(
r#"# {} Architecture Specification
#
# This file describes the structure of the {} project using Hielements.
# Learn more at: https://github.com/ercasta/hielements
#
# For AI agents and quick reference:
# - See USAGE_GUIDE.md for language syntax and available commands
# - Run 'hielements check {}.hie' to validate this specification
# - Run 'hielements run {}.hie' to execute checks

import files

## The {} project
element {} {{
    # Define the root scope
    scope root = files.folder_selector('.')
    
    # Basic checks
    check files.exists(root, 'README.md')
    
    # Add more elements, scopes, and checks here to describe your architecture
}}
"#,
        project_name, project_name, project_name, project_name, project_name, project_name
    );
    
    // Generate hielements.toml
    let config_content = r#"# Hielements Configuration File
#
# This file configures external library plugins for the Hielements interpreter.
# Place this file in your project root (next to your .hie files).

# External Libraries
# Each entry defines a library that can be imported in .hie files.
# 
# Supports two types of plugins:
#   1. External process plugins (JSON-RPC over stdio)
#   2. WASM plugins (sandboxed, near-native performance)
#
# Format:
#   [libraries]
#   # External process plugin
#   library_name = { executable = "path/to/executable", args = ["arg1", "arg2"] }
#   # or shorthand when no args:
#   library_name = { executable = "path/to/executable" }
#   
#   # WASM plugin (explicit type)
#   library_name = { type = "wasm", path = "path/to/plugin.wasm" }
#   # or inferred from .wasm extension:
#   library_name = { path = "path/to/plugin.wasm" }
#
# Example:
#   [libraries]
#   mylib = { executable = "python3", args = ["scripts/mylib_plugin.py"] }
#   typescript = { type = "wasm", path = "lib/typescript.wasm" }

[libraries]
# Add your custom libraries here
# Example:
# mylib = { executable = "python3", args = ["scripts/mylib_plugin.py"] }
"#;
    
    // Generate USAGE_GUIDE.md
    let guide_content = r#"# Hielements Quick Reference

This guide provides a brief introduction to using Hielements in your project.

## Commands

### Check Syntax and Semantics
```bash
hielements check <file>.hie
```
Validates the syntax and semantics of your Hielements specification without running checks.

### Run Checks
```bash
hielements run <file>.hie
```
Executes all checks defined in your specification against your actual codebase.

Options:
- `--verbose` - Show progress as each check runs
- `--filter <pattern>` - Run only checks matching the pattern
- `--limit <n>` - Limit the number of checks to run
- `--dry-run` - Show what would be checked without actually running

### Generate Documentation
```bash
hielements doc --output library_docs.md
```
Generate documentation for available libraries (built-in and custom).

## Language Basics

### Elements
Elements represent logical components of your system:
```hielements
element my_component {
    # Element content
}
```

### Scopes
Scopes define what code/artifacts belong to an element:
```hielements
scope src = files.folder_selector('src/')
scope config = files.file_selector('config.yaml')
```

### Checks
Checks verify properties of your system:
```hielements
check files.exists(src, 'main.py')
check files.no_files_matching(src, '*.pyc')
```

### Hierarchical Elements
Elements can contain child elements:
```hielements
element parent {
    element child {
        scope src = files.folder_selector('child/src')
    }
}
```

### Connection Points (refs)
Connection points expose interfaces with explicit types:
```hielements
ref api: HttpHandler = python.public_functions(module)
ref config: Config = files.file_selector('config.yaml')
```

### Patterns (Templates)
Define reusable architectural blueprints:
```hielements
pattern microservice {
    element api {
        scope module<python>
    }
    element database {
        ref connection: DatabaseConnection
    }
}

element orders_service implements microservice {
    # Bind pattern to actual implementation
}
```

## Built-in Libraries

### files
- `files.folder_selector(path)` - Select a folder
- `files.file_selector(path)` - Select a file
- `files.exists(scope, name)` - Check if file/folder exists
- `files.no_files_matching(scope, pattern)` - Check no files match pattern

### rust
- `rust.module_selector(name)` - Select a Rust module
- `rust.crate_selector(name)` - Select a Rust crate
- `rust.struct_exists(name)` - Check if struct exists
- `rust.function_exists(name)` - Check if function exists

### python
- `python.module_selector(name)` - Select a Python module
- `python.class_selector(module, name)` - Select a class
- `python.function_exists(module, name)` - Check if function exists

## Custom Libraries

You can extend Hielements with custom libraries written in any language.
Configure them in `hielements.toml`:

```toml
[libraries]
mylib = { executable = "python3", args = ["scripts/mylib_plugin.py"] }
```

Then import and use in your .hie files:
```hielements
import mylib

element my_component {
    scope src = mylib.my_selector('src/')
    check mylib.my_check(src)
}
```

## Learn More

- Documentation: https://github.com/ercasta/hielements/blob/main/README.md
- Usage Guide: https://github.com/ercasta/hielements/blob/main/USAGE.md
- Pattern Catalog: https://github.com/ercasta/hielements/blob/main/doc/patterns_catalog.md
- Examples: https://github.com/ercasta/hielements/tree/main/examples
"#;
    
    // Write files
    if let Err(e) = fs::write(&hie_file, hie_content) {
        eprintln!("{} Failed to write '{}': {}", "error:".red().bold(), hie_file.display(), e);
        return ExitCode::from(2);
    }
    
    let config_existed = config_file.exists();
    if !config_existed {
        if let Err(e) = fs::write(&config_file, config_content) {
            eprintln!("{} Failed to write '{}': {}", "error:".red().bold(), config_file.display(), e);
            return ExitCode::from(2);
        }
    } else {
        println!("{} '{}' already exists, skipping", "Info".blue().bold(), config_file.display());
    }
    
    let guide_existed = guide_file.exists();
    if !guide_existed {
        if let Err(e) = fs::write(&guide_file, guide_content) {
            eprintln!("{} Failed to write '{}': {}", "error:".red().bold(), guide_file.display(), e);
            return ExitCode::from(2);
        }
    } else {
        println!("{} '{}' already exists, skipping", "Info".blue().bold(), guide_file.display());
    }
    
    // Success message
    println!("{} Initialized Hielements project '{}'", "Success".green().bold(), project_name);
    println!();
    println!("Created files:");
    println!("  {} - Initial architecture specification", hie_file.display().to_string().cyan());
    if !config_existed {
        println!("  {} - Configuration for custom libraries", config_file.display().to_string().cyan());
    }
    if !guide_existed {
        println!("  {} - Quick reference guide", guide_file.display().to_string().cyan());
    }
    println!();
    println!("Next steps:");
    println!("  1. Edit {} to describe your architecture", hie_file.display().to_string().cyan());
    println!("  2. Run {} to validate", format!("hielements check {}", hie_file.display()).yellow());
    println!("  3. Run {} to execute checks", format!("hielements run {}", hie_file.display()).yellow());
    println!();
    println!("For AI agents: See {} for language syntax and available commands", guide_file.display().to_string().cyan());
    
    ExitCode::SUCCESS
}
