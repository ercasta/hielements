//! Interpreter for the Hielements language.

use std::collections::HashMap;

use crate::ast::*;
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::parser::Parser;
use crate::stdlib::{CheckResult, LibraryRegistry, Value};

/// Result of running checks.
#[derive(Debug)]
pub struct CheckOutput {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
    pub results: Vec<SingleCheckResult>,
    pub skipped: usize,
}

/// Result of a single check.
#[derive(Debug)]
pub struct SingleCheckResult {
    pub element_path: String,
    pub check_expr: String,
    pub result: CheckResult,
}

/// Options for running checks.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Filter checks by element path pattern
    pub filter: Option<String>,
    /// Limit the number of checks to run
    pub limit: Option<usize>,
    /// Callback for progress reporting (element_path, check_expr, is_starting)
    pub verbose: bool,
}

/// The Hielements interpreter.
pub struct Interpreter {
    libraries: LibraryRegistry,
    workspace: String,
    scopes: HashMap<String, Value>,
    diagnostics: Diagnostics,
    /// Current element path for scope resolution context
    current_element_path: String,
}

impl Interpreter {
    /// Create a new interpreter with built-in libraries only.
    pub fn new(workspace: impl Into<String>) -> Self {
        let workspace_str = workspace.into();
        Self {
            libraries: LibraryRegistry::with_workspace(&workspace_str),
            workspace: workspace_str,
            scopes: HashMap::new(),
            diagnostics: Diagnostics::new(),
            current_element_path: String::new(),
        }
    }

    /// Register an external library manually.
    pub fn register_library(&mut self, library: Box<dyn super::stdlib::Library>) {
        self.libraries.register(library);
    }

    /// Parse and validate a Hielements file.
    pub fn validate(&mut self, source: &str, file_path: &str) -> (Option<Program>, Diagnostics) {
        let parser = Parser::new(source, file_path);
        let (program, mut diagnostics) = parser.parse();


        if let Some(ref prog) = program {
            // Perform semantic validation
            let semantic_diagnostics = self.validate_semantics(prog, file_path);
            diagnostics.extend(semantic_diagnostics);
        }

        (program, diagnostics)
    }

    /// Perform semantic validation on the AST.
    fn validate_semantics(&self, program: &Program, file_path: &str) -> Diagnostics {
        let mut diagnostics = Diagnostics::new();

        // Validate imports
        for import in &program.imports {
            if let ImportPath::Identifier(parts) = &import.path {
                if let Some(first) = parts.first() {
                    if self.libraries.get(&first.name).is_none() {
                        diagnostics.push(
                            Diagnostic::warning(
                                "W001",
                                format!("Unknown library '{}' (will be resolved at runtime)", first.name),
                            )
                            .with_file(file_path)
                            .with_span(first.span)
                            .build(),
                        );
                    }
                }
            }
        }

        // Validate templates
        for template in &program.templates {
            self.validate_template(template, file_path, &mut diagnostics);
        }

        // Validate elements
        for element in &program.elements {
            self.validate_element(element, file_path, &mut diagnostics, &[]);
        }

        diagnostics
    }

    /// Validate a template and its children.
    fn validate_template(
        &self,
        template: &Template,
        file_path: &str,
        diagnostics: &mut Diagnostics,
    ) {
        // Validate scopes
        for scope in &template.scopes {
            self.validate_expression(&scope.expression, file_path, diagnostics);
        }

        // Validate connection points
        for cp in &template.connection_points {
            self.validate_expression(&cp.expression, file_path, diagnostics);
        }

        // Validate checks
        for check in &template.checks {
            self.validate_expression(&check.expression, file_path, diagnostics);
        }

        // Validate hierarchical requirements
        for req in &template.hierarchical_requirements {
            self.validate_hierarchical_requirement(req, file_path, diagnostics);
        }

        // Connection boundaries don't need expression validation

        // Validate child elements
        for element in &template.elements {
            self.validate_element(element, file_path, diagnostics, &[]);
        }
    }

    /// Validate an element and its children.
    fn validate_element(
        &self,
        element: &Element,
        file_path: &str,
        diagnostics: &mut Diagnostics,
        path: &[String],
    ) {
        let mut current_path = path.to_vec();
        current_path.push(element.name.name.clone());

        // Validate template implementations (just check syntax for now)
        // Full validation would require resolving template definitions
        for template_impl in &element.implements {
            // Could add validation that template exists, but that requires
            // building a template registry first
            let _ = template_impl; // Acknowledge the field
        }

        // Validate scopes
        for scope in &element.scopes {
            self.validate_expression(&scope.expression, file_path, diagnostics);
        }

        // Validate connection points
        for cp in &element.connection_points {
            self.validate_expression(&cp.expression, file_path, diagnostics);
        }

        // Validate checks
        for check in &element.checks {
            self.validate_expression(&check.expression, file_path, diagnostics);
        }

        // Validate template bindings
        for binding in &element.template_bindings {
            self.validate_expression(&binding.expression, file_path, diagnostics);
        }

        // Validate hierarchical requirements
        for req in &element.hierarchical_requirements {
            self.validate_hierarchical_requirement(req, file_path, diagnostics);
        }

        // Connection boundaries don't need expression validation, just structural
        // The target patterns are already parsed

        // Validate children
        for child in &element.children {
            self.validate_element(child, file_path, diagnostics, &current_path);
        }
    }

    /// Validate a hierarchical requirement.
    fn validate_hierarchical_requirement(
        &self,
        req: &HierarchicalRequirement,
        file_path: &str,
        diagnostics: &mut Diagnostics,
    ) {
        match &req.kind {
            HierarchicalRequirementKind::Scope(scope) => {
                self.validate_expression(&scope.expression, file_path, diagnostics);
            }
            HierarchicalRequirementKind::Check(check) => {
                self.validate_expression(&check.expression, file_path, diagnostics);
            }
            HierarchicalRequirementKind::Element(element) => {
                self.validate_element(element, file_path, diagnostics, &[]);
            }
            HierarchicalRequirementKind::ImplementsTemplate(_template_name) => {
                // Template name validation - just ensure it's a valid identifier
                // Actual template existence check happens during element implementation validation
                // No further validation needed here during parse-time validation
            }
        }
    }

    /// Validate an expression.
    fn validate_expression(&self, expr: &Expression, file_path: &str, diagnostics: &mut Diagnostics) {
        match expr {
            Expression::FunctionCall { function, arguments, .. } => {
                self.validate_expression(function, file_path, diagnostics);
                for arg in arguments {
                    self.validate_expression(arg, file_path, diagnostics);
                }
            }
            Expression::MemberAccess { object, .. } => {
                self.validate_expression(object, file_path, diagnostics);
            }
            Expression::List { elements, .. } => {
                for elem in elements {
                    self.validate_expression(elem, file_path, diagnostics);
                }
            }
            _ => {}
        }
    }

    /// Run all checks in a program.
    pub fn run(&mut self, program: &Program) -> CheckOutput {
        self.run_with_options(program, &RunOptions::default())
    }

    /// Run all checks in a program with options.
    pub fn run_with_options(&mut self, program: &Program, options: &RunOptions) -> CheckOutput {
        let mut output = CheckOutput {
            total: 0,
            passed: 0,
            failed: 0,
            errors: 0,
            results: Vec::new(),
            skipped: 0,
        };

        // Process imports (for now, we just use built-in libraries)
        // TODO: Load external libraries

        // Run checks for each element
        for element in &program.elements {
            self.run_element_with_options(element, &[], &mut output, options);
        }

        output
    }

    /// Run checks for an element with options.
    fn run_element_with_options(&mut self, element: &Element, path: &[String], output: &mut CheckOutput, options: &RunOptions) {
        let mut current_path = path.to_vec();
        current_path.push(element.name.name.clone());
        let path_str = current_path.join(".");
        
        // Set current element path for scope resolution context
        self.current_element_path = path_str.clone();

        // Check if we've hit the limit
        if let Some(limit) = options.limit {
            if output.results.len() >= limit {
                return;
            }
        }

        // Check filter
        let matches_filter = if let Some(ref filter) = options.filter {
            path_str.contains(filter)
        } else {
            true
        };

        // Evaluate and store scopes (always do this to ensure scope resolution works)
        for scope in &element.scopes {
            let scope_name = format!("{}.{}", path_str, scope.name.name);
            if options.verbose {
                eprintln!("[verbose] Evaluating scope: {}", scope_name);
            }
            match self.evaluate_expression(&scope.expression) {
                Ok(value) => {
                    if options.verbose {
                        eprintln!("[verbose]   -> resolved {} paths", 
                            if let Value::Scope(ref s) = value { s.paths.len() } else { 0 });
                    }
                    self.scopes.insert(scope_name, value);
                }
                Err(e) => {
                    if options.verbose {
                        eprintln!("[verbose]   -> ERROR: {}", e.message);
                    }
                    self.diagnostics.push(e);
                }
            }
        }

        // Run checks
        for check in &element.checks {
            // Check limit again
            if let Some(limit) = options.limit {
                if output.results.len() >= limit {
                    output.skipped += 1;
                    continue;
                }
            }

            let check_expr = self.expression_to_string(&check.expression);

            // Skip if doesn't match filter
            if !matches_filter {
                output.skipped += 1;
                continue;
            }

            output.total += 1;

            if options.verbose {
                eprint!("[verbose] Running: {} :: {} ... ", path_str, check_expr);
                use std::io::Write;
                let _ = std::io::stderr().flush();
            }

            match self.run_check(&check.expression) {
                Ok(result) => {
                    if options.verbose {
                        match &result {
                            CheckResult::Pass => eprintln!("PASS"),
                            CheckResult::Fail(msg) => eprintln!("FAIL: {}", msg),
                            CheckResult::Error(msg) => eprintln!("ERROR: {}", msg),
                        }
                    }
                    match &result {
                        CheckResult::Pass => output.passed += 1,
                        CheckResult::Fail(_) => output.failed += 1,
                        CheckResult::Error(_) => output.errors += 1,
                    }
                    output.results.push(SingleCheckResult {
                        element_path: path_str.clone(),
                        check_expr,
                        result,
                    });
                }
                Err(e) => {
                    if options.verbose {
                        eprintln!("ERROR: {}", e.message);
                    }
                    output.errors += 1;
                    output.results.push(SingleCheckResult {
                        element_path: path_str.clone(),
                        check_expr,
                        result: CheckResult::Error(e.message),
                    });
                }
            }
        }

        // Process children
        for child in &element.children {
            self.run_element_with_options(child, &current_path, output, options);
        }
    }

    /// Evaluate an expression and return a value.
    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, Diagnostic> {
        match expr {
            Expression::String(s) => Ok(Value::String(s.value.clone())),
            Expression::Number(n) => {
                if n.value.fract() == 0.0 {
                    Ok(Value::Int(n.value as i64))
                } else {
                    Ok(Value::Float(n.value))
                }
            }
            Expression::Boolean(b) => Ok(Value::Bool(b.value)),
            Expression::List { elements, .. } => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(self.evaluate_expression(elem)?);
                }
                Ok(Value::List(values))
            }
            Expression::Identifier(id) => {
                // Look up in scopes with priority for current element's scopes
                // First try to find in current element's path (e.g., "hielements.core.parser.module")
                let current_scope_key = format!("{}.{}", self.current_element_path, id.name);
                if let Some(value) = self.scopes.get(&current_scope_key) {
                    return Ok(value.clone());
                }
                
                // Then try exact suffix match, prioritizing closer scopes
                // The key is "parent.name", so we check if it ends with ".name"
                let lookup_suffix = format!(".{}", id.name);
                for (name, value) in &self.scopes {
                    // Exact suffix match: "parent.module" ends with ".module"
                    // But NOT "parent.lexer_module" for lookup of "module"
                    if name.ends_with(&lookup_suffix) || name == &id.name {
                        return Ok(value.clone());
                    }
                }
                Err(Diagnostic::error("E200", format!("Undefined identifier: {}", id.name))
                    .with_span(id.span)
                    .build())
            }
            Expression::MemberAccess { object, member, span } => {
                // Check if it's a library call
                if let Expression::Identifier(lib_id) = object.as_ref() {
                    // Could be library.function - return as-is for function call handling
                    return Err(Diagnostic::error(
                        "E201",
                        format!("Member access {}.{} cannot be evaluated directly", lib_id.name, member.name),
                    )
                    .with_span(*span)
                    .build());
                }
                
                // Otherwise, look up scope
                let scope_name = self.expression_to_string(expr);
                for (name, value) in &self.scopes {
                    if name.ends_with(&scope_name) {
                        return Ok(value.clone());
                    }
                }
                Err(Diagnostic::error("E202", format!("Undefined reference: {}", scope_name))
                    .with_span(*span)
                    .build())
            }
            Expression::FunctionCall { function, arguments, span } => {
                // Get library and function name
                let (lib_name, func_name) = self.get_library_function(function)?;
                
                // Evaluate arguments
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.evaluate_expression(arg)?);
                }

                // Call the library function
                if let Some(library) = self.libraries.get_mut(&lib_name) {
                    library.call(&func_name, args, &self.workspace).map_err(|e| {
                        Diagnostic::error(e.code, e.message)
                            .with_span(*span)
                            .build()
                    })
                } else {
                    Err(Diagnostic::error("E203", format!("Unknown library: {}", lib_name))
                        .with_span(*span)
                        .build())
                }
            }
        }
    }

    /// Run a check expression.
    fn run_check(&mut self, expr: &Expression) -> Result<CheckResult, Diagnostic> {
        if let Expression::FunctionCall { function, arguments, span } = expr {
            let (lib_name, func_name) = self.get_library_function(function)?;
            
            // Evaluate arguments
            let mut args = Vec::new();
            for arg in arguments {
                args.push(self.evaluate_expression(arg)?);
            }

            // Call the library check function
            if let Some(library) = self.libraries.get_mut(&lib_name) {
                library.check(&func_name, args, &self.workspace).map_err(|e| {
                    Diagnostic::error(e.code, e.message)
                        .with_span(*span)
                        .build()
                })
            } else {
                Err(Diagnostic::error("E203", format!("Unknown library: {}", lib_name))
                    .with_span(*span)
                    .build())
            }
        } else {
            Err(Diagnostic::error("E204", "Check must be a function call")
                .with_span(expr.span())
                .build())
        }
    }

    /// Extract library and function name from a function expression.
    fn get_library_function(&self, expr: &Expression) -> Result<(String, String), Diagnostic> {
        if let Expression::MemberAccess { object, member, span } = expr {
            if let Expression::Identifier(lib_id) = object.as_ref() {
                return Ok((lib_id.name.clone(), member.name.clone()));
            }
            Err(Diagnostic::error("E205", "Expected library.function")
                .with_span(*span)
                .build())
        } else {
            Err(Diagnostic::error("E205", "Expected library.function")
                .with_span(expr.span())
                .build())
        }
    }

    /// Convert an expression to a string representation.
    fn expression_to_string(&self, expr: &Expression) -> String {
        match expr {
            Expression::Identifier(id) => id.name.clone(),
            Expression::String(s) => format!("'{}'", s.value),
            Expression::Number(n) => n.value.to_string(),
            Expression::Boolean(b) => b.value.to_string(),
            Expression::MemberAccess { object, member, .. } => {
                format!("{}.{}", self.expression_to_string(object), member.name)
            }
            Expression::FunctionCall { function, arguments, .. } => {
                let args: Vec<_> = arguments.iter().map(|a| self.expression_to_string(a)).collect();
                format!("{}({})", self.expression_to_string(function), args.join(", "))
            }
            Expression::List { elements, .. } => {
                let elems: Vec<_> = elements.iter().map(|e| self.expression_to_string(e)).collect();
                format!("[{}]", elems.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_validate_simple() {
        let source = r#"import files

element test:
    scope src = files.folder_selector('src')
"#;
        let mut interpreter = Interpreter::new(".");
        let (program, _diagnostics) = interpreter.validate(source, "test.hie");
        
        assert!(program.is_some());
        // Should have warning about 'files' library (though it exists)
    }

    #[test]
    fn test_run_file_check() {
        let dir = tempdir().unwrap();
        let src_dir = dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("main.py"), "print('hello')").unwrap();

        let source = r#"import files

element test:
    scope src = files.folder_selector('src')
    check files.exists(src, 'main.py')
"#;
        let mut interpreter = Interpreter::new(dir.path().to_str().unwrap());
        let (program, _diagnostics) = interpreter.validate(source, "test.hie");
        
        assert!(program.is_some());
        let output = interpreter.run(&program.unwrap());
        
        assert_eq!(output.total, 1);
        assert_eq!(output.passed, 1);
    }
}
