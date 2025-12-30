//! Hielements CLI
//!
//! Command-line interface for the Hielements language.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use colored::Colorize;
use hielements_core::diagnostics::{DiagnosticSeverity, DiagnosticsOutput};
use hielements_core::stdlib::CheckResult;
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
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { file, format } => cmd_check(&file, &format),
        Commands::Run { file, workspace, format, dry_run, verbose, filter, limit } => {
            cmd_run(&file, workspace.as_deref(), &format, dry_run, verbose, filter.as_deref(), limit)
        }
        Commands::Parse { file } => cmd_parse(&file),
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

    for cp in &element.connection_points {
        let type_info = format!(": {}", cp.type_annotation.type_name.name);
        println!("{}  connection_point {}{} = ...", prefix, cp.name.name.magenta(), type_info.yellow());
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
