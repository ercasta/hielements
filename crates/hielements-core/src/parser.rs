//! Parser for the Hielements language.

use crate::ast::*;
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::span::Span;

/// Expected tokens in element body for error messages.
/// Note: 'requires', 'allows', 'forbids' are only allowed in templates, not elements.
const EXPECTED_ELEMENT_BODY_TOKENS: &str = "'scope', 'ref', 'uses', 'check', or 'element'";

/// Expected tokens in template body for error messages.
const EXPECTED_TEMPLATE_BODY_TOKENS: &str = "'scope', 'ref', 'check', 'element', 'requires', 'allows', or 'forbids'";

/// Parser for the Hielements language.
pub struct Parser<'a> {
    #[allow(dead_code)]
    source: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Diagnostics,
    file_path: String,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, file_path: impl Into<String>) -> Self {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        Self {
            source,
            tokens,
            pos: 0,
            diagnostics: Diagnostics::new(),
            file_path: file_path.into(),
        }
    }

    /// Parse the entire program.
    pub fn parse(mut self) -> (Option<Program>, Diagnostics) {
        let start_span = self.current_span();

        let mut imports = Vec::new();
        let mut languages = Vec::new();
        let mut templates = Vec::new();
        let mut elements = Vec::new();

        // Skip leading newlines and doc comments
        self.skip_newlines_and_comments();

        // Parse imports
        while self.check(TokenKind::Import) || self.check(TokenKind::From) {
            match self.parse_import() {
                Ok(import) => imports.push(import),
                Err(diag) => {
                    self.diagnostics.push(diag);
                    self.recover_to_newline();
                }
            }
            self.skip_newlines_and_comments();
        }

        // Parse templates, elements, and language declarations
        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            // Skip doc comments before templates/elements
            let doc_comment = self.parse_doc_comment();

            if self.check(TokenKind::Template) {
                match self.parse_template(doc_comment) {
                    Ok(template) => templates.push(template),
                    Err(diag) => {
                        self.diagnostics.push(diag);
                        self.recover_to_element();
                    }
                }
            } else if self.check(TokenKind::Element) {
                match self.parse_element(doc_comment) {
                    Ok(element) => elements.push(element),
                    Err(diag) => {
                        self.diagnostics.push(diag);
                        self.recover_to_element();
                    }
                }
            } else if self.check(TokenKind::Language) {
                match self.parse_language_declaration() {
                    Ok(lang) => languages.push(lang),
                    Err(diag) => {
                        self.diagnostics.push(diag);
                        self.recover_to_newline();
                    }
                }
            } else if !self.is_at_end() {
                let token = self.current();
                self.diagnostics.push(
                    Diagnostic::error("E001", format!("Expected 'template', 'element', or 'language', found {:?}", token.kind))
                        .with_file(&self.file_path)
                        .with_span(token.span)
                        .build(),
                );
                self.advance();
            }
        }

        let end_span = self.previous_span();
        let program = Program {
            imports,
            languages,
            templates,
            elements,
            span: start_span.merge(&end_span),
        };

        (Some(program), self.diagnostics)
    }

    /// Parse a doc comment if present.
    fn parse_doc_comment(&mut self) -> Option<String> {
        let mut doc_lines = Vec::new();
        while self.check(TokenKind::DocComment) {
            let token = self.advance();
            // Remove the '## ' prefix
            let text = token.text.trim_start_matches('#').trim();
            doc_lines.push(text.to_string());
            self.skip_newlines();
        }
        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join("\n"))
        }
    }

    /// Parse an import statement.
    fn parse_import(&mut self) -> Result<ImportStatement, Diagnostic> {
        let start_span = self.current_span();

        if self.check(TokenKind::From) {
            // from X import Y, Z
            self.advance();
            let path = self.parse_import_path()?;
            self.expect(TokenKind::Import)?;
            let mut selective = Vec::new();
            selective.push(self.parse_identifier()?);
            while self.check(TokenKind::Comma) {
                self.advance();
                selective.push(self.parse_identifier()?);
            }
            self.expect_newline()?;
            let end_span = self.previous_span();

            Ok(ImportStatement {
                path,
                alias: None,
                selective,
                span: start_span.merge(&end_span),
            })
        } else {
            // import X [as Y]
            self.expect(TokenKind::Import)?;
            let path = self.parse_import_path()?;
            let alias = if self.check(TokenKind::As) {
                self.advance();
                Some(self.parse_identifier()?)
            } else {
                None
            };
            self.expect_newline()?;
            let end_span = self.previous_span();

            Ok(ImportStatement {
                path,
                alias,
                selective: Vec::new(),
                span: start_span.merge(&end_span),
            })
        }
    }

    /// Parse an import path.
    fn parse_import_path(&mut self) -> Result<ImportPath, Diagnostic> {
        if self.check(TokenKind::StringSingle) || self.check(TokenKind::StringDouble) {
            let string = self.parse_string_literal()?;
            Ok(ImportPath::String(string))
        } else {
            let mut parts = vec![self.parse_identifier()?];
            while self.check(TokenKind::Dot) {
                self.advance();
                parts.push(self.parse_identifier()?);
            }
            Ok(ImportPath::Identifier(parts))
        }
    }

    /// Parse a language declaration.
    /// Syntax: `language <name>` or `language <name>:` followed by connection_check definitions
    fn parse_language_declaration(&mut self) -> Result<LanguageDeclaration, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::Language)?;
        let name = self.parse_identifier()?;
        
        // Check if this is a simple declaration or has a body
        if self.check(TokenKind::Colon) {
            self.advance(); // consume ':'
            self.skip_newlines();
            self.expect(TokenKind::Indent)?;
            
            let mut connection_checks = Vec::new();
            
            loop {
                self.skip_newlines();
                
                // Skip doc comments before connection checks
                let _ = self.parse_doc_comment();
                
                if self.check(TokenKind::Dedent) || self.is_at_end() {
                    break;
                }
                
                if self.check(TokenKind::ConnectionCheck) {
                    connection_checks.push(self.parse_connection_check()?);
                } else {
                    let token = self.current();
                    return Err(Diagnostic::error(
                        "E013",
                        format!("Expected 'connection_check' in language body, found {:?}", token.kind),
                    )
                    .with_file(&self.file_path)
                    .with_span(token.span)
                    .build());
                }
            }
            
            // Consume DEDENT if present
            if self.check(TokenKind::Dedent) {
                self.advance();
            }
            
            let end_span = self.previous_span();
            Ok(LanguageDeclaration {
                name,
                connection_checks,
                span: start_span.merge(&end_span),
            })
        } else {
            // Simple declaration without body
            self.expect_newline()?;
            let end_span = self.previous_span();
            Ok(LanguageDeclaration {
                name,
                connection_checks: Vec::new(),
                span: start_span.merge(&end_span),
            })
        }
    }

    /// Parse a connection check declaration.
    /// Syntax: `connection_check <name>(<params>):` followed by indented body
    fn parse_connection_check(&mut self) -> Result<ConnectionCheckDeclaration, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::ConnectionCheck)?;
        let name = self.parse_identifier()?;
        
        // Parse parameters
        self.expect(TokenKind::LParen)?;
        let mut parameters = Vec::new();
        
        if !self.check(TokenKind::RParen) {
            parameters.push(self.parse_connection_check_parameter()?);
            while self.check(TokenKind::Comma) {
                self.advance();
                if self.check(TokenKind::RParen) {
                    break; // Allow trailing comma
                }
                parameters.push(self.parse_connection_check_parameter()?);
            }
        }
        
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Colon)?;
        self.skip_newlines();
        self.expect(TokenKind::Indent)?;
        
        // Parse body - for now we expect a single expression (like `return expr`)
        self.skip_newlines();
        let body = self.parse_expression()?;
        self.expect_newline()?;
        
        // Consume DEDENT
        self.skip_newlines();
        if self.check(TokenKind::Dedent) {
            self.advance();
        }
        
        let end_span = self.previous_span();
        Ok(ConnectionCheckDeclaration {
            name,
            parameters,
            body,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse a connection check parameter.
    /// Syntax: `<name>: scope[]`
    fn parse_connection_check_parameter(&mut self) -> Result<ConnectionCheckParameter, Diagnostic> {
        let start_span = self.current_span();
        let name = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        
        // Expect "scope[]"
        self.expect(TokenKind::Scope)?;
        self.expect(TokenKind::LBracket)?;
        self.expect(TokenKind::RBracket)?;
        
        let end_span = self.previous_span();
        Ok(ConnectionCheckParameter {
            name,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse an element declaration.
    /// `in_template` indicates whether this element is inside a template (allows unbounded scopes).
    /// Supports both curly bracket syntax `element name { ... }` and indentation syntax `element name:\n    ...`
    fn parse_element_with_context(&mut self, doc_comment: Option<String>, in_template: bool) -> Result<Element, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::Element)?;
        let name = self.parse_identifier()?;
        
        // Parse optional template implementation
        let mut implements = Vec::new();
        if self.check(TokenKind::Implements) {
            self.advance();
            loop {
                let template_start = self.current_span();
                let template_name = self.parse_identifier()?;
                implements.push(TemplateImplementation {
                    template_name,
                    span: template_start.merge(&self.previous_span()),
                });
                
                if !self.check(TokenKind::Comma) {
                    break;
                }
                self.advance(); // consume comma
            }
        }
        
        // Support both curly bracket syntax `{ ... }` and colon/indent syntax `: ...`
        let use_braces = self.check(TokenKind::LBrace);
        if use_braces {
            self.advance(); // consume '{'
            self.skip_newlines_and_indents();
        } else {
            self.expect(TokenKind::Colon)?;
            self.skip_newlines();
            self.expect(TokenKind::Indent)?;
        }

        let mut scopes = Vec::new();
        let mut refs = Vec::new();
        let mut uses = Vec::new();
        let mut checks = Vec::new();
        let mut template_bindings = Vec::new();
        let mut children = Vec::new();

        loop {
            // Use different skip strategy based on syntax
            if use_braces {
                self.skip_newlines_and_indents();
            } else {
                self.skip_newlines();
            }

            // Check for end of block (depends on syntax used)
            if use_braces {
                if self.check(TokenKind::RBrace) || self.is_at_end() {
                    break;
                }
            } else {
                if self.check(TokenKind::Dedent) || self.is_at_end() {
                    break;
                }
            }

            // Handle doc comments for nested elements
            let child_doc = self.parse_doc_comment();

            if self.check(TokenKind::Scope) {
                scopes.push(self.parse_scope()?);
            } else if self.check(TokenKind::Ref) || self.check(TokenKind::ConnectionPoint) {
                // Support both 'ref' and 'connection_point' keywords
                refs.push(self.parse_ref()?);
            } else if self.check(TokenKind::Uses) {
                // Parse uses declaration: `source uses target`
                uses.push(self.parse_uses()?);
            } else if self.check(TokenKind::Check) {
                checks.push(self.parse_check()?);
            } else if self.check(TokenKind::Element) {
                children.push(self.parse_element_with_context(child_doc, in_template)?);
            // Note: requires/allows/forbids are NOT allowed in regular elements
            // They are only allowed in templates. Provide helpful error message.
            } else if self.check(TokenKind::Requires) || self.check(TokenKind::Allows) || self.check(TokenKind::Forbids) {
                let token = self.current();
                return Err(Diagnostic::error(
                    "E012",
                    format!(
                        "'{}' is only allowed in templates, not in regular elements. Define a template with this constraint and have the element implement it.",
                        token.text
                    ),
                )
                .with_file(&self.file_path)
                .with_span(token.span)
                .build());
            } else if self.check(TokenKind::Identifier) {
                // Could be:
                // 1. A uses declaration: `identifier uses target`
                // 2. A template binding: `template.element.scope = ...`
                let pos = self.pos;
                self.advance(); // consume identifier
                
                if self.check(TokenKind::Uses) {
                    // This is a uses declaration with the identifier as the source
                    self.pos = pos; // restore position
                    uses.push(self.parse_uses()?);
                } else if self.check(TokenKind::Dot) {
                    // Looks like a template binding, restore and try to parse it
                    self.pos = pos;
                    match self.try_parse_template_binding() {
                        Ok(binding) => template_bindings.push(binding),
                        Err(err) => {
                            // Failed to parse as template binding
                            return Err(err);
                        }
                    }
                } else {
                    // Not a template binding or uses (no dot or uses keyword after identifier)
                    self.pos = pos; // restore
                    let token = self.current();
                    return Err(Diagnostic::error(
                        "E002",
                        format!(
                            "Expected {}, found {:?}",
                            EXPECTED_ELEMENT_BODY_TOKENS,
                            token.kind
                        ),
                    )
                    .with_file(&self.file_path)
                    .with_span(token.span)
                    .build());
                }
            } else if (use_braces && self.check(TokenKind::RBrace)) || 
                      (!use_braces && self.check(TokenKind::Dedent)) || 
                      self.is_at_end() {
                break;
            } else {
                let token = self.current();
                return Err(Diagnostic::error(
                    "E002",
                    format!(
                        "Expected {}, found {:?}",
                        EXPECTED_ELEMENT_BODY_TOKENS,
                        token.kind
                    ),
                )
                .with_file(&self.file_path)
                .with_span(token.span)
                .build());
            }
        }

        // Consume end of block (depends on syntax used)
        if use_braces {
            if self.check(TokenKind::RBrace) {
                self.advance();
            }
        } else {
            if self.check(TokenKind::Dedent) {
                self.advance();
            }
        }

        let end_span = self.previous_span();

        // Validate: unbounded scopes (no expression) are NOT allowed in regular elements
        // They are only allowed in templates. Provide helpful error message.
        // Skip validation if we're inside a template (in_template = true)
        if !in_template {
            for scope in &scopes {
                if scope.expression.is_none() {
                    return Err(Diagnostic::error(
                        "E014",
                        format!(
                            "Unbounded scope '{}' is only allowed in templates, not in regular elements. \
                            Provide an expression: `scope {} = <expression>`",
                            scope.name.name, scope.name.name
                        ),
                    )
                    .with_file(&self.file_path)
                    .with_span(scope.span)
                    .build());
                }
            }

            // Validate: unbounded refs (no expression) are NOT allowed in regular elements
            for r in &refs {
                if r.expression.is_none() {
                    return Err(Diagnostic::error(
                        "E015",
                        format!(
                            "Unbounded ref '{}' is only allowed in templates, not in regular elements. \
                            Provide an expression: `ref {}: {} = <expression>`",
                            r.name.name, r.name.name, r.type_annotation.type_name.name
                        ),
                    )
                    .with_file(&self.file_path)
                    .with_span(r.span)
                    .build());
                }
            }
        }

        Ok(Element {
            doc_comment,
            name,
            implements,
            scopes,
            refs,
            uses,
            checks,
            template_bindings,
            component_requirements: Vec::new(), // Always empty - requires/allows/forbids only allowed in templates
            children,
            span: start_span.merge(&end_span),
        })
    }

    /// Convenience wrapper to parse element at top level (not in template)
    fn parse_element(&mut self, doc_comment: Option<String>) -> Result<Element, Diagnostic> {
        self.parse_element_with_context(doc_comment, false)
    }

    /// Parse a template declaration.
    /// Supports both curly bracket syntax `template name { ... }` and indentation syntax `template name:\n    ...`
    fn parse_template(&mut self, doc_comment: Option<String>) -> Result<Template, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::Template)?;
        let name = self.parse_identifier()?;
        
        // Support both curly bracket syntax `{ ... }` and colon/indent syntax `: ...`
        let use_braces = self.check(TokenKind::LBrace);
        if use_braces {
            self.advance(); // consume '{'
            self.skip_newlines_and_indents();
        } else {
            self.expect(TokenKind::Colon)?;
            self.skip_newlines();
            self.expect(TokenKind::Indent)?;
        }

        let mut scopes = Vec::new();
        let mut refs = Vec::new();
        let mut checks = Vec::new();
        let mut component_requirements = Vec::new();
        let mut elements = Vec::new();

        loop {
            // Use different skip strategy based on syntax
            if use_braces {
                self.skip_newlines_and_indents();
            } else {
                self.skip_newlines();
            }

            // Check for end of block (depends on syntax used)
            if use_braces {
                if self.check(TokenKind::RBrace) || self.is_at_end() {
                    break;
                }
            } else {
                if self.check(TokenKind::Dedent) || self.is_at_end() {
                    break;
                }
            }

            // Handle doc comments for nested elements
            let child_doc = self.parse_doc_comment();

            if self.check(TokenKind::Scope) {
                scopes.push(self.parse_scope()?);
            } else if self.check(TokenKind::Ref) || self.check(TokenKind::ConnectionPoint) {
                // Support both 'ref' and 'connection_point' keywords
                refs.push(self.parse_ref()?);
            } else if self.check(TokenKind::Check) {
                checks.push(self.parse_check()?);
            } else if self.check(TokenKind::Element) {
                // Elements inside templates can have unbounded scopes
                elements.push(self.parse_element_with_context(child_doc, true)?);
            // Unified syntax: requires/allows/forbids [descendant] ...
            } else if self.check(TokenKind::Requires) {
                component_requirements.push(self.parse_component_requirement(RequirementAction::Requires)?);
            } else if self.check(TokenKind::Allows) {
                component_requirements.push(self.parse_component_requirement(RequirementAction::Allows)?);
            } else if self.check(TokenKind::Forbids) {
                component_requirements.push(self.parse_component_requirement(RequirementAction::Forbids)?);
            } else if (use_braces && self.check(TokenKind::RBrace)) || 
                      (!use_braces && self.check(TokenKind::Dedent)) || 
                      self.is_at_end() {
                break;
            } else {
                let token = self.current();
                return Err(Diagnostic::error(
                    "E002",
                    format!(
                        "Expected {} in template, found {:?}",
                        EXPECTED_TEMPLATE_BODY_TOKENS,
                        token.kind
                    ),
                )
                .with_file(&self.file_path)
                .with_span(token.span)
                .build());
            }
        }

        // Consume end of block (depends on syntax used)
        if use_braces {
            if self.check(TokenKind::RBrace) {
                self.advance();
            }
        } else {
            if self.check(TokenKind::Dedent) {
                self.advance();
            }
        }

        let end_span = self.previous_span();

        Ok(Template {
            doc_comment,
            name,
            scopes,
            refs,
            checks,
            component_requirements,
            elements,
            span: start_span.merge(&end_span),
        })
    }

    /// Try to parse a template binding (e.g., template.element.scope = expression).
    fn try_parse_template_binding(&mut self) -> Result<TemplateBinding, Diagnostic> {
        let start_span = self.current_span();
        let start_pos = self.pos;
        
        // Parse the qualified path (template.element.property)
        let mut path = vec![self.parse_identifier()?];
        
        // Must have at least one dot for it to be a template binding
        if !self.check(TokenKind::Dot) {
            // Restore position and fail
            self.pos = start_pos;
            return Err(Diagnostic::error("E003", "Not a template binding")
                .with_file(&self.file_path)
                .with_span(start_span)
                .build());
        }
        
        while self.check(TokenKind::Dot) {
            self.advance(); // consume dot
            path.push(self.parse_identifier()?);
        }
        
        // Template bindings must have at least 2 parts (template.property) and use =
        if path.len() < 2 || !self.check(TokenKind::Equals) {
            // Restore position and fail
            self.pos = start_pos;
            return Err(Diagnostic::error("E003", "Not a template binding")
                .with_file(&self.file_path)
                .with_span(start_span)
                .build());
        }
        
        self.expect(TokenKind::Equals)?;
        let expression = self.parse_expression()?;
        self.expect_newline()?;
        let end_span = self.previous_span();
        
        Ok(TemplateBinding {
            path,
            expression,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse a scope declaration (V2 syntax).
    /// Syntax: `scope <name> [<language>] [binds <path>] [= <expression>]`
    /// Unbounded scopes (no `=`) are allowed in templates.
    fn parse_scope(&mut self) -> Result<ScopeDeclaration, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::Scope)?;
        let name = self.parse_identifier()?;
        
        // Check for optional language annotation with angular brackets: `<language>`
        let language = if self.check(TokenKind::LAngle) {
            self.advance(); // consume '<'
            let lang = self.parse_identifier()?;
            self.expect(TokenKind::RAngle)?; // consume '>'
            Some(lang)
        } else if self.check(TokenKind::Colon) {
            // Also support legacy colon syntax for backward compatibility during migration
            self.advance(); // consume ':'
            Some(self.parse_identifier()?)
        } else {
            None
        };
        
        // Check for optional binds clause: `binds template.element.scope`
        let binds = if self.check(TokenKind::Binds) {
            self.advance(); // consume 'binds'
            Some(self.parse_qualified_path()?)
        } else {
            None
        };
        
        // Expression is optional for unbounded scopes in templates
        let expression = if self.check(TokenKind::Equals) {
            self.expect(TokenKind::Equals)?;
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect_newline()?;
        let end_span = self.previous_span();

        Ok(ScopeDeclaration {
            name,
            language,
            binds,
            expression,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse a ref declaration (V2 syntax - renamed from connection_point).
    /// Syntax: `ref <name> : <type> [binds <path>] [= <expression>]`
    /// Also accepts `connection_point` for backward compatibility.
    /// Unbounded refs (no `=`) are allowed in templates.
    fn parse_ref(&mut self) -> Result<RefDeclaration, Diagnostic> {
        let start_span = self.current_span();
        // Accept both 'ref' and 'connection_point' keywords
        if self.check(TokenKind::Ref) {
            self.advance();
        } else {
            self.expect(TokenKind::ConnectionPoint)?;
        }
        let name = self.parse_identifier()?;
        
        // Parse mandatory type annotation: `: <type>`
        self.expect(TokenKind::Colon)?;
        let type_annotation = self.parse_type_annotation()?;
        
        // Check for optional binds clause: `binds template.element.ref`
        let binds = if self.check(TokenKind::Binds) {
            self.advance(); // consume 'binds'
            Some(self.parse_qualified_path()?)
        } else {
            None
        };
        
        // Expression is optional for unbounded refs in templates
        let expression = if self.check(TokenKind::Equals) {
            self.expect(TokenKind::Equals)?;
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect_newline()?;
        let end_span = self.previous_span();

        Ok(RefDeclaration {
            name,
            type_annotation,
            binds,
            expression,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse a uses declaration.
    /// Syntax: `<source> uses <target>` where target is a qualified identifier (e.g., `lexer` or `core.lexer`)
    /// This declares that the source scope/element has a dependency on the target element/scope.
    fn parse_uses(&mut self) -> Result<UsesDeclaration, Diagnostic> {
        let start_span = self.current_span();
        
        // Parse the source identifier
        let source = if self.check(TokenKind::Identifier) {
            self.parse_identifier()?
        } else {
            let token = self.current();
            return Err(Diagnostic::error(
                "E016",
                format!("Expected identifier for uses source, found {:?}", token.kind),
            )
            .with_file(&self.file_path)
            .with_span(token.span)
            .build());
        };
        
        // Expect 'uses' keyword
        self.expect(TokenKind::Uses)?;
        
        // Parse the target (qualified identifier like 'lexer' or 'core.lexer')
        let target = self.parse_qualified_path()?;
        
        self.expect_newline()?;
        let end_span = self.previous_span();

        Ok(UsesDeclaration {
            source,
            target,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse a qualified path for binds clauses: `template.element.scope`
    fn parse_qualified_path(&mut self) -> Result<Vec<Identifier>, Diagnostic> {
        let mut path = vec![self.parse_identifier()?];
        while self.check(TokenKind::Dot) {
            self.advance(); // consume '.'
            path.push(self.parse_identifier()?);
        }
        Ok(path)
    }

    /// Parse a check declaration.
    fn parse_check(&mut self) -> Result<CheckDeclaration, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::Check)?;
        let expression = self.parse_expression()?;
        self.expect_newline()?;
        let end_span = self.previous_span();

        Ok(CheckDeclaration {
            expression,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse a connection pattern (e.g., api_gateway.public_api or database.*).
    fn parse_connection_pattern(&mut self) -> Result<ConnectionPattern, Diagnostic> {
        let start_span = self.current_span();
        let mut path = vec![self.parse_identifier()?];
        let mut wildcard = false;

        while self.check(TokenKind::Dot) {
            self.advance(); // consume dot
            
            // Check for wildcard (*)
            if self.check(TokenKind::Star) {
                self.advance(); // consume *
                wildcard = true;
                break; // * must be last
            } else if self.check(TokenKind::Identifier) {
                path.push(self.parse_identifier()?);
            } else {
                // End of pattern
                break;
            }
        }

        let end_span = self.previous_span();
        Ok(ConnectionPattern {
            path,
            wildcard,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse a component requirement.
    /// Syntax: (requires | allows | forbids) [descendant] (scope | check | element | connection | connection_point)
    fn parse_component_requirement(&mut self, action: RequirementAction) -> Result<ComponentRequirement, Diagnostic> {
        let start_span = self.current_span();
        
        // Consume the action keyword (requires, allows, or forbids)
        self.advance();
        
        // Check for optional 'descendant' modifier
        let is_descendant = if self.check(TokenKind::Descendant) {
            self.advance();
            true
        } else {
            false
        };
        
        // Parse the component specification
        let component = if self.check(TokenKind::Scope) {
            ComponentSpec::Scope(self.parse_scope()?)
        } else if self.check(TokenKind::Check) {
            ComponentSpec::Check(self.parse_check()?)
        } else if self.check(TokenKind::Element) {
            self.parse_element_component_spec()?
        } else if self.check(TokenKind::Connection) {
            self.advance(); // consume 'connection'
            // Optional 'to' keyword
            if self.check(TokenKind::To) {
                self.advance();
            }
            let pattern = self.parse_connection_pattern()?;
            self.expect_newline()?;
            ComponentSpec::Connection(pattern)
        } else if self.check(TokenKind::Ref) || self.check(TokenKind::ConnectionPoint) {
            // Support both 'ref' and 'connection_point' keywords
            self.parse_ref_component_spec()?
        } else if self.check(TokenKind::Implements) {
            // Support for shorthand: `requires descendant implements template_name`
            self.advance(); // consume 'implements'
            let template_name = self.parse_identifier()?;
            self.expect_newline()?;
            // Create an element spec with just implements (anonymous element)
            ComponentSpec::Element {
                name: Identifier::new("_anonymous", template_name.span),
                type_annotation: None,
                implements: Some(template_name),
                body: None,
            }
        } else if self.check(TokenKind::Language) {
            // Language constraint: `requires language <name>`
            self.advance(); // consume 'language'
            let lang_name = self.parse_identifier()?;
            self.expect_newline()?;
            ComponentSpec::Language(lang_name)
        } else {
            let token = self.current();
            return Err(Diagnostic::error(
                "E011",
                format!(
                    "Expected 'scope', 'check', 'element', 'connection', 'ref', 'implements', or 'language' after '{} {}', found {:?}",
                    match action {
                        RequirementAction::Requires => "requires",
                        RequirementAction::Allows => "allows",
                        RequirementAction::Forbids => "forbids",
                    },
                    if is_descendant { "descendant" } else { "" },
                    token.kind
                ),
            )
            .with_file(&self.file_path)
            .with_span(token.span)
            .build());
        };

        let end_span = self.previous_span();
        Ok(ComponentRequirement {
            action,
            is_descendant,
            component,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse an element component specification.
    /// Syntax: element name [: Type] [implements template] [: body]
    fn parse_element_component_spec(&mut self) -> Result<ComponentSpec, Diagnostic> {
        self.advance(); // consume 'element'
        
        let name = self.parse_identifier()?;
        
        // Optional type annotation: `: Type`
        let type_annotation = if self.check(TokenKind::Colon) {
            // Peek ahead to see if this is a type annotation or element body
            // A type annotation is followed by an identifier (the type name)
            // An element body is followed by NEWLINE/INDENT
            let pos = self.pos;
            self.advance(); // consume ':'
            
            if self.check(TokenKind::Identifier) {
                // This could be either a type annotation or the start of a body with scope/element
                // We need to check if the identifier is followed by NEWLINE (body) or something else (type)
                let id = self.parse_identifier()?;
                
                // If next is newline and then indent, this identifier might be a type
                // But we also need to handle `implements` after the type
                if self.check(TokenKind::Implements) || self.check(TokenKind::Newline) || self.check(TokenKind::Colon) {
                    Some(TypeAnnotation {
                        type_name: id.clone(),
                        span: id.span,
                    })
                } else {
                    // Restore and treat as no type annotation
                    self.pos = pos;
                    None
                }
            } else {
                // Not a type annotation, restore position
                self.pos = pos;
                None
            }
        } else {
            None
        };
        
        // Optional implements: `implements template_name`
        let implements = if self.check(TokenKind::Implements) {
            self.advance();
            Some(self.parse_identifier()?)
        } else {
            None
        };
        
        // Check for element body (colon followed by newline and indent)
        let body = if self.check(TokenKind::Colon) {
            let doc_comment = None;
            // We've already parsed 'element name [: Type] [implements ...]', now we parse the body
            // by consuming the colon and parsing the indented block manually
            self.expect(TokenKind::Colon)?;
            self.skip_newlines();
            
            if self.check(TokenKind::Indent) {
                self.expect(TokenKind::Indent)?;
                
                // Parse element body contents
                // Note: requires/allows/forbids are NOT allowed in element bodies
                let mut scopes = Vec::new();
                let mut refs = Vec::new();
                let uses = Vec::new(); // Uses not yet parsed in element_component_spec body
                let mut checks = Vec::new();
                let mut children = Vec::new();

                loop {
                    self.skip_newlines();

                    if self.check(TokenKind::Dedent) || self.is_at_end() {
                        break;
                    }

                    let child_doc = self.parse_doc_comment();

                    if self.check(TokenKind::Scope) {
                        scopes.push(self.parse_scope()?);
                    } else if self.check(TokenKind::Ref) || self.check(TokenKind::ConnectionPoint) {
                        refs.push(self.parse_ref()?);
                    } else if self.check(TokenKind::Check) {
                        checks.push(self.parse_check()?);
                    } else if self.check(TokenKind::Element) {
                        children.push(self.parse_element(child_doc)?);
                    } else if self.check(TokenKind::Requires) || self.check(TokenKind::Allows) || self.check(TokenKind::Forbids) {
                        let token = self.current();
                        return Err(Diagnostic::error(
                            "E012",
                            format!(
                                "'{}' is only allowed in templates, not in regular elements.",
                                token.text
                            ),
                        )
                        .with_file(&self.file_path)
                        .with_span(token.span)
                        .build());
                    } else {
                        break;
                    }
                }

                if self.check(TokenKind::Dedent) {
                    self.advance();
                }

                let span = name.span.merge(&self.previous_span());
                Some(Box::new(Element {
                    doc_comment,
                    name: name.clone(),
                    implements: if let Some(ref impl_name) = implements {
                        vec![TemplateImplementation {
                            template_name: impl_name.clone(),
                            span: impl_name.span,
                        }]
                    } else {
                        Vec::new()
                    },
                    scopes,
                    refs,
                    uses,
                    checks,
                    template_bindings: Vec::new(),
                    component_requirements: Vec::new(), // Always empty - requires/allows/forbids only allowed in templates
                    children,
                    span,
                }))
            } else {
                None
            }
        } else {
            self.expect_newline()?;
            None
        };
        
        Ok(ComponentSpec::Element {
            name,
            type_annotation,
            implements,
            body,
        })
    }

    /// Parse a ref component specification (formerly connection_point).
    /// Syntax: ref name: Type [= expression]
    /// Also accepts connection_point for backward compatibility.
    fn parse_ref_component_spec(&mut self) -> Result<ComponentSpec, Diagnostic> {
        self.advance(); // consume 'ref' or 'connection_point'
        
        let name = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        let type_annotation = self.parse_type_annotation()?;
        
        // Optional expression: `= expression`
        let expression = if self.check(TokenKind::Equals) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect_newline()?;
        
        Ok(ComponentSpec::Ref {
            name,
            type_annotation,
            expression,
        })
    }

    /// Parse a type annotation.
    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, Diagnostic> {
        let type_name = self.parse_identifier()?;
        let span = type_name.span;
        
        Ok(TypeAnnotation {
            type_name,
            span,
        })
    }

    /// Parse an expression.
    fn parse_expression(&mut self) -> Result<Expression, Diagnostic> {
        self.parse_postfix()
    }

    /// Parse postfix expressions (member access and function calls).
    fn parse_postfix(&mut self) -> Result<Expression, Diagnostic> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(TokenKind::Dot) {
                self.advance();
                let member = self.parse_identifier()?;
                let span = expr.span().merge(&member.span);
                expr = Expression::MemberAccess {
                    object: Box::new(expr),
                    member,
                    span,
                };
            } else if self.check(TokenKind::LParen) {
                self.advance();
                let mut arguments = Vec::new();
                if !self.check(TokenKind::RParen) {
                    arguments.push(self.parse_expression()?);
                    while self.check(TokenKind::Comma) {
                        self.advance();
                        arguments.push(self.parse_expression()?);
                    }
                }
                let end_span = self.current_span();
                self.expect(TokenKind::RParen)?;
                let span = expr.span().merge(&end_span);
                expr = Expression::FunctionCall {
                    function: Box::new(expr),
                    arguments,
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse a primary expression.
    fn parse_primary(&mut self) -> Result<Expression, Diagnostic> {
        if self.check(TokenKind::Identifier) {
            let id = self.parse_identifier()?;
            Ok(Expression::Identifier(id))
        } else if self.check(TokenKind::StringSingle) || self.check(TokenKind::StringDouble) {
            let string = self.parse_string_literal()?;
            Ok(Expression::String(string))
        } else if self.check(TokenKind::Number) {
            let number = self.parse_number_literal()?;
            Ok(Expression::Number(number))
        } else if self.check(TokenKind::True) {
            let span = self.current_span();
            self.advance();
            Ok(Expression::Boolean(BooleanLiteral::new(true, span)))
        } else if self.check(TokenKind::False) {
            let span = self.current_span();
            self.advance();
            Ok(Expression::Boolean(BooleanLiteral::new(false, span)))
        } else if self.check(TokenKind::LBracket) {
            self.parse_list()
        } else {
            let token = self.current();
            Err(Diagnostic::error("E003", format!("Expected expression, found {:?}", token.kind))
                .with_file(&self.file_path)
                .with_span(token.span)
                .build())
        }
    }

    /// Parse a list literal.
    fn parse_list(&mut self) -> Result<Expression, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::LBracket)?;
        let mut elements = Vec::new();
        if !self.check(TokenKind::RBracket) {
            elements.push(self.parse_expression()?);
            while self.check(TokenKind::Comma) {
                self.advance();
                if self.check(TokenKind::RBracket) {
                    break; // Allow trailing comma
                }
                elements.push(self.parse_expression()?);
            }
        }
        let end_span = self.current_span();
        self.expect(TokenKind::RBracket)?;

        Ok(Expression::List {
            elements,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse an identifier (or keyword used as identifier in some contexts).
    fn parse_identifier(&mut self) -> Result<Identifier, Diagnostic> {
        if self.check(TokenKind::Identifier) {
            let token = self.advance();
            Ok(Identifier::new(token.text, token.span))
        } else {
            // In some contexts (like template binding paths), keywords can be used as identifiers
            // Allow certain keywords to be treated as identifiers
            let token = self.current();
            match token.kind {
                TokenKind::Scope | TokenKind::Element | TokenKind::Check | 
                TokenKind::ConnectionPoint | TokenKind::Ref | TokenKind::Uses |
                TokenKind::Template | TokenKind::Implements |
                TokenKind::Binds | TokenKind::To |
                // Unified keywords can also be used as identifiers in some contexts
                TokenKind::Requires | TokenKind::Allows | TokenKind::Forbids |
                TokenKind::Descendant | TokenKind::Connection |
                // Language keywords can also be used as identifiers
                TokenKind::Language | TokenKind::ConnectionCheck => {
                    let token = self.advance();
                    Ok(Identifier::new(token.text, token.span))
                }
                _ => {
                    Err(Diagnostic::error("E004", format!("Expected identifier, found {:?}", token.kind))
                        .with_file(&self.file_path)
                        .with_span(token.span)
                        .build())
                }
            }
        }
    }

    /// Parse a string literal.
    fn parse_string_literal(&mut self) -> Result<StringLiteral, Diagnostic> {
        if self.check(TokenKind::StringSingle) || self.check(TokenKind::StringDouble) {
            let token = self.advance();
            let value = self.unescape_string(&token.text);
            Ok(StringLiteral::new(value, token.span))
        } else {
            let token = self.current();
            Err(Diagnostic::error("E005", format!("Expected string, found {:?}", token.kind))
                .with_file(&self.file_path)
                .with_span(token.span)
                .build())
        }
    }

    /// Parse a number literal.
    fn parse_number_literal(&mut self) -> Result<NumberLiteral, Diagnostic> {
        if self.check(TokenKind::Number) {
            let token = self.advance();
            let value: f64 = token.text.parse().unwrap_or(0.0);
            Ok(NumberLiteral::new(value, token.span))
        } else {
            let token = self.current();
            Err(Diagnostic::error("E006", format!("Expected number, found {:?}", token.kind))
                .with_file(&self.file_path)
                .with_span(token.span)
                .build())
        }
    }

    /// Unescape a string literal.
    fn unescape_string(&self, text: &str) -> String {
        let inner = &text[1..text.len() - 1];
        let mut result = String::new();
        let mut chars = inner.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(&next) = chars.peek() {
                    chars.next();
                    match next {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '\'' => result.push('\''),
                        '"' => result.push('"'),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                } else {
                    result.push('\\');
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    // Helper methods

    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn current_span(&self) -> Span {
        self.current().span
    }

    fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            self.current_span()
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current().kind == kind
    }

    fn is_at_end(&self) -> bool {
        self.current().kind == TokenKind::Eof
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.tokens[self.pos - 1].clone()
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, Diagnostic> {
        if self.check(kind.clone()) {
            Ok(self.advance())
        } else {
            let token = self.current();
            Err(Diagnostic::error("E007", format!("Expected {:?}, found {:?}", kind, token.kind))
                .with_file(&self.file_path)
                .with_span(token.span)
                .build())
        }
    }

    fn expect_newline(&mut self) -> Result<(), Diagnostic> {
        if self.check(TokenKind::Newline) || self.check(TokenKind::Eof) || self.check(TokenKind::Dedent) {
            if self.check(TokenKind::Newline) {
                self.advance();
            }
            Ok(())
        } else {
            let token = self.current();
            Err(Diagnostic::error("E008", format!("Expected newline, found {:?}", token.kind))
                .with_file(&self.file_path)
                .with_span(token.span)
                .build())
        }
    }

    fn skip_newlines(&mut self) {
        while self.check(TokenKind::Newline) {
            self.advance();
        }
    }

    /// Skip newlines, indents, and dedents - used when parsing inside curly brackets
    fn skip_newlines_and_indents(&mut self) {
        while self.check(TokenKind::Newline) || self.check(TokenKind::Indent) || self.check(TokenKind::Dedent) {
            self.advance();
        }
    }

    fn skip_newlines_and_comments(&mut self) {
        while self.check(TokenKind::Newline) || self.check(TokenKind::DocComment) {
            self.advance();
        }
    }

    fn recover_to_newline(&mut self) {
        while !self.is_at_end() && !self.check(TokenKind::Newline) {
            self.advance();
        }
        if self.check(TokenKind::Newline) {
            self.advance();
        }
    }

    fn recover_to_element(&mut self) {
        while !self.is_at_end() {
            if self.check(TokenKind::Element) {
                return;
            }
            if self.check(TokenKind::Dedent) {
                self.advance();
            } else {
                self.advance();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_element() {
        let source = r#"element test:
    scope src = files.folder_selector('src')
    check files.exists(src, 'main.py')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        assert_eq!(program.elements[0].name.name, "test");
        assert_eq!(program.elements[0].scopes.len(), 1);
        assert_eq!(program.elements[0].checks.len(), 1);
    }

    #[test]
    fn test_parse_with_imports() {
        let source = r#"import python
import docker as d

element service:
    scope module = python.module_selector('main')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.imports.len(), 2);
        assert_eq!(program.elements.len(), 1);
    }

    #[test]
    fn test_parse_nested_elements() {
        let source = r#"element parent:
    element child:
        scope src = files.folder_selector('src')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        assert_eq!(program.elements[0].children.len(), 1);
        assert_eq!(program.elements[0].children[0].name.name, "child");
    }

    #[test]
    fn test_parse_template_declaration() {
        let source = r#"template compiler:
    element lexer:
        connection_point tokens: TokenStream = rust.function_selector('tokenize')
    element parser:
        connection_point ast: AbstractSyntaxTree = rust.function_selector('parse')
    check compiler.lexer.tokens.compatible_with(compiler.parser.input)
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        assert_eq!(program.templates[0].name.name, "compiler");
        assert_eq!(program.templates[0].elements.len(), 2);
        assert_eq!(program.templates[0].elements[0].name.name, "lexer");
        assert_eq!(program.templates[0].elements[1].name.name, "parser");
        assert_eq!(program.templates[0].checks.len(), 1);
    }

    #[test]
    fn test_parse_template_implementation() {
        let source = r#"element my_compiler implements compiler:
    scope src = files.folder_selector('src')
    compiler.lexer.scope = rust.module_selector('lexer')
    compiler.parser.scope = rust.module_selector('parser')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        assert_eq!(program.elements[0].name.name, "my_compiler");
        assert_eq!(program.elements[0].implements.len(), 1);
        assert_eq!(program.elements[0].implements[0].template_name.name, "compiler");
        assert_eq!(program.elements[0].template_bindings.len(), 2);
        
        // Check first binding
        let binding1 = &program.elements[0].template_bindings[0];
        assert_eq!(binding1.path.len(), 3);
        assert_eq!(binding1.path[0].name, "compiler");
        assert_eq!(binding1.path[1].name, "lexer");
        assert_eq!(binding1.path[2].name, "scope");
        
        // Check second binding
        let binding2 = &program.elements[0].template_bindings[1];
        assert_eq!(binding2.path.len(), 3);
        assert_eq!(binding2.path[0].name, "compiler");
        assert_eq!(binding2.path[1].name, "parser");
        assert_eq!(binding2.path[2].name, "scope");
    }

    #[test]
    fn test_parse_multiple_template_implementation() {
        let source = r#"element my_service implements microservice, observable:
    scope config = files.file_selector('config.yaml')
    microservice.api.scope = rust.module_selector('api')
    observable.metrics.scope = rust.module_selector('metrics')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        assert_eq!(program.elements[0].implements.len(), 2);
        assert_eq!(program.elements[0].implements[0].template_name.name, "microservice");
        assert_eq!(program.elements[0].implements[1].template_name.name, "observable");
        assert_eq!(program.elements[0].template_bindings.len(), 2);
    }

    #[test]
    fn test_parse_template_with_scopes_and_checks() {
        let source = r#"template microservice:
    scope config = files.file_selector('config.yaml')
    
    element api:
        connection_point endpoint: HttpHandler = rust.function_selector('api_handler')
    
    check microservice.api.endpoint.is_valid()
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        assert_eq!(program.templates[0].scopes.len(), 1);
        assert_eq!(program.templates[0].elements.len(), 1);
        assert_eq!(program.templates[0].checks.len(), 1);
    }

    #[test]
    fn test_parse_connection_point_with_type() {
        let source = r#"element api_service:
    connection_point port: integer = docker.exposed_port(dockerfile)
    connection_point url: string = config.get_api_url()
    connection_point enabled: boolean = config.get_flag('api_enabled')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.refs.len(), 3);
        
        // Check first connection point with type
        assert_eq!(element.refs[0].name.name, "port");
        assert_eq!(element.refs[0].type_annotation.type_name.name, "integer");
        
        // Check second connection point with type
        assert_eq!(element.refs[1].name.name, "url");
        assert_eq!(element.refs[1].type_annotation.type_name.name, "string");
        
        // Check third connection point with type
        assert_eq!(element.refs[2].name.name, "enabled");
        assert_eq!(element.refs[2].type_annotation.type_name.name, "boolean");
    }

    #[test]
    fn test_parse_connection_point_without_type_fails() {
        let source = r#"element api_service:
    connection_point endpoint = python.public_functions(module)
"#;
        let parser = Parser::new(source, "test.hie");
        let (_program, diagnostics) = parser.parse();

        // Should have errors because type annotation is mandatory
        assert!(diagnostics.has_errors(), "Expected error for missing type annotation");
    }

    #[test]
    fn test_parse_connection_point_with_custom_type() {
        let source = r#"template compiler:
    element lexer:
        connection_point tokens: TokenStream = rust.struct_selector('Token')
    element parser:
        connection_point ast: AbstractSyntaxTree = rust.struct_selector('Program')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.elements.len(), 2);
        
        // Check lexer connection point with custom type
        let lexer = &template.elements[0];
        assert_eq!(lexer.refs[0].name.name, "tokens");
        assert_eq!(lexer.refs[0].type_annotation.type_name.name, "TokenStream");
        
        // Check parser connection point with custom type
        let parser_elem = &template.elements[1];
        assert_eq!(parser_elem.refs[0].name.name, "ast");
        assert_eq!(parser_elem.refs[0].type_annotation.type_name.name, "AbstractSyntaxTree");
    }

    // ========================================================================
    // Tests for unified syntax: requires/allows/forbids [descendant] ...
    // ========================================================================

    #[test]
    fn test_parse_requires_descendant_scope() {
        let source = r#"template dockerized:
    requires descendant scope dockerfile = docker.file_selector('Dockerfile')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(matches!(req.action, RequirementAction::Requires));
        assert!(req.is_descendant);
        
        match &req.component {
            ComponentSpec::Scope(scope) => {
                assert_eq!(scope.name.name, "dockerfile");
            }
            _ => panic!("Expected scope component"),
        }
    }

    #[test]
    fn test_parse_requires_descendant_element() {
        let source = r#"template observable:
    requires descendant element metrics_service implements metrics_provider
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(matches!(req.action, RequirementAction::Requires));
        assert!(req.is_descendant);
        
        match &req.component {
            ComponentSpec::Element { name, implements, .. } => {
                assert_eq!(name.name, "metrics_service");
                assert_eq!(implements.as_ref().unwrap().name, "metrics_provider");
            }
            _ => panic!("Expected element component"),
        }
    }

    #[test]
    fn test_parse_forbids_descendant_connection_point() {
        let source = r#"template secure_zone:
    forbids descendant connection_point external_api: HttpHandler
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(matches!(req.action, RequirementAction::Forbids));
        assert!(req.is_descendant);
        
        match &req.component {
            ComponentSpec::Ref { name, type_annotation, .. } => {
                assert_eq!(name.name, "external_api");
                assert_eq!(type_annotation.type_name.name, "HttpHandler");
            }
            _ => panic!("Expected connection_point component"),
        }
    }

    #[test]
    fn test_parse_allows_connection() {
        let source = r#"template frontend_zone:
    allows connection to api_gateway.public_api
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(matches!(req.action, RequirementAction::Allows));
        assert!(!req.is_descendant);
        
        match &req.component {
            ComponentSpec::Connection(pattern) => {
                assert_eq!(pattern.path.len(), 2);
                assert_eq!(pattern.path[0].name, "api_gateway");
                assert_eq!(pattern.path[1].name, "public_api");
                assert!(!pattern.wildcard);
            }
            _ => panic!("Expected connection component"),
        }
    }

    #[test]
    fn test_parse_forbids_connection_with_wildcard() {
        let source = r#"template secure_zone:
    forbids connection to external.*
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(matches!(req.action, RequirementAction::Forbids));
        assert!(!req.is_descendant);
        
        match &req.component {
            ComponentSpec::Connection(pattern) => {
                assert_eq!(pattern.path.len(), 1);
                assert_eq!(pattern.path[0].name, "external");
                assert!(pattern.wildcard);
            }
            _ => panic!("Expected connection component"),
        }
    }

    #[test]
    fn test_parse_requires_element_immediate() {
        // Without 'descendant' modifier - requires immediate child
        let source = r#"template microservice:
    requires element api implements api_handler
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(matches!(req.action, RequirementAction::Requires));
        assert!(!req.is_descendant); // Not descendant - immediate child
        
        match &req.component {
            ComponentSpec::Element { name, implements, .. } => {
                assert_eq!(name.name, "api");
                assert_eq!(implements.as_ref().unwrap().name, "api_handler");
            }
            _ => panic!("Expected element component"),
        }
    }

    #[test]
    fn test_parse_requires_descendant_implements_shorthand() {
        // Shorthand: requires descendant implements template_name
        let source = r#"template production_ready:
    requires descendant implements dockerized
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(matches!(req.action, RequirementAction::Requires));
        assert!(req.is_descendant);
        
        match &req.component {
            ComponentSpec::Element { implements, .. } => {
                assert_eq!(implements.as_ref().unwrap().name, "dockerized");
            }
            _ => panic!("Expected element component with implements"),
        }
    }

    #[test]
    fn test_parse_all_requirement_types() {
        let source = r#"template complete:
    requires descendant scope config = files.file_selector('config.yaml')
    requires descendant check files.exists(config, 'required.txt')
    allows connection to api.*
    forbids connection to external.*
    requires connection to logging.output
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 5);
    }

    #[test]
    fn test_parse_element_with_body() {
        let source = r#"template observable:
    requires descendant element metrics:
        scope module = rust.module_selector('metrics')
        connection_point handler: MetricsHandler = rust.function_selector(module, 'handler')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(req.is_descendant);
        
        match &req.component {
            ComponentSpec::Element { name, body, .. } => {
                assert_eq!(name.name, "metrics");
                let body = body.as_ref().expect("Expected element body");
                assert_eq!(body.scopes.len(), 1);
                assert_eq!(body.refs.len(), 1);
            }
            _ => panic!("Expected element component with body"),
        }
    }

    #[test]
    fn test_parse_requires_check() {
        let source = r#"template validated:
    requires descendant check files.exists(src, 'README.md')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(matches!(req.action, RequirementAction::Requires));
        assert!(req.is_descendant);
        
        match &req.component {
            ComponentSpec::Check(_) => (),
            _ => panic!("Expected check component"),
        }
    }

    #[test]
    fn test_parse_requires_in_element_fails() {
        // requires/allows/forbids should only be allowed in templates, not elements
        let source = r#"element test_element:
    scope src = files.folder_selector('src')
    requires connection to logging.*
"#;
        let parser = Parser::new(source, "test.hie");
        let (_program, diagnostics) = parser.parse();

        // Should have errors because requires is not allowed in elements
        assert!(diagnostics.has_errors(), "Expected error for 'requires' in element");
        // Check that the error message mentions templates
        let error_msg = diagnostics.iter().next().unwrap().message.clone();
        assert!(error_msg.contains("template"), "Error message should mention templates: {}", error_msg);
    }

    #[test]
    fn test_parse_allows_in_element_fails() {
        // requires/allows/forbids should only be allowed in templates, not elements
        let source = r#"element test_element:
    allows connection to api.*
"#;
        let parser = Parser::new(source, "test.hie");
        let (_program, diagnostics) = parser.parse();

        // Should have errors because allows is not allowed in elements
        assert!(diagnostics.has_errors(), "Expected error for 'allows' in element");
    }

    #[test]
    fn test_parse_forbids_in_element_fails() {
        // requires/allows/forbids should only be allowed in templates, not elements
        let source = r#"element test_element:
    forbids connection to external.*
"#;
        let parser = Parser::new(source, "test.hie");
        let (_program, diagnostics) = parser.parse();

        // Should have errors because forbids is not allowed in elements
        assert!(diagnostics.has_errors(), "Expected error for 'forbids' in element");
    }

    // ========================================================================
    // Tests for language declarations and connection checks
    // ========================================================================

    #[test]
    fn test_parse_simple_language_declaration() {
        let source = r#"language python
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.languages.len(), 1);
        assert_eq!(program.languages[0].name.name, "python");
        assert!(program.languages[0].connection_checks.is_empty());
    }

    #[test]
    fn test_parse_multiple_language_declarations() {
        let source = r#"language python
language rust
language java
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.languages.len(), 3);
        assert_eq!(program.languages[0].name.name, "python");
        assert_eq!(program.languages[1].name.name, "rust");
        assert_eq!(program.languages[2].name.name, "java");
    }

    #[test]
    fn test_parse_language_with_connection_check() {
        let source = r#"language python:
    connection_check can_import(source: scope[], target: scope[]):
        python.imports_allowed(source, target)
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.languages.len(), 1);
        assert_eq!(program.languages[0].name.name, "python");
        assert_eq!(program.languages[0].connection_checks.len(), 1);
        
        let check = &program.languages[0].connection_checks[0];
        assert_eq!(check.name.name, "can_import");
        assert_eq!(check.parameters.len(), 2);
        assert_eq!(check.parameters[0].name.name, "source");
        assert_eq!(check.parameters[1].name.name, "target");
    }

    #[test]
    fn test_parse_scope_with_language() {
        let source = r#"element test:
    scope src : python = python.module_selector('test')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        assert_eq!(program.elements[0].scopes.len(), 1);
        
        let scope = &program.elements[0].scopes[0];
        assert_eq!(scope.name.name, "src");
        assert!(scope.language.is_some());
        assert_eq!(scope.language.as_ref().unwrap().name, "python");
    }

    #[test]
    fn test_parse_scope_without_language() {
        // Should still work without language annotation
        let source = r#"element test:
    scope src = files.folder_selector('src')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        assert_eq!(program.elements[0].scopes.len(), 1);
        
        let scope = &program.elements[0].scopes[0];
        assert_eq!(scope.name.name, "src");
        assert!(scope.language.is_none());
    }

    #[test]
    fn test_parse_requires_language() {
        let source = r#"template python_only:
    requires language python
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(matches!(req.action, RequirementAction::Requires));
        assert!(!req.is_descendant);
        
        match &req.component {
            ComponentSpec::Language(lang) => {
                assert_eq!(lang.name, "python");
            }
            _ => panic!("Expected language component"),
        }
    }

    #[test]
    fn test_parse_forbids_language() {
        let source = r#"template no_rust:
    forbids language rust
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 1);
        
        let req = &template.component_requirements[0];
        assert!(matches!(req.action, RequirementAction::Forbids));
        
        match &req.component {
            ComponentSpec::Language(lang) => {
                assert_eq!(lang.name, "rust");
            }
            _ => panic!("Expected language component"),
        }
    }

    #[test]
    fn test_parse_allows_language() {
        let source = r#"template multilingual:
    allows language python
    allows language rust
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.component_requirements.len(), 2);
        
        // Check first requirement
        assert!(matches!(template.component_requirements[0].action, RequirementAction::Allows));
        match &template.component_requirements[0].component {
            ComponentSpec::Language(lang) => assert_eq!(lang.name, "python"),
            _ => panic!("Expected language component"),
        }
        
        // Check second requirement
        assert!(matches!(template.component_requirements[1].action, RequirementAction::Allows));
        match &template.component_requirements[1].component {
            ComponentSpec::Language(lang) => assert_eq!(lang.name, "rust"),
            _ => panic!("Expected language component"),
        }
    }

    #[test]
    fn test_parse_complete_language_example() {
        let source = r#"language python:
    connection_check can_import(source: scope[], target: scope[]):
        python.imports_allowed(source, target)
    connection_check no_circular(scopes: scope[]):
        python.no_circular_imports(scopes)

template python_service:
    requires language python
    forbids language rust

element my_api implements python_service:
    scope src : python = python.module_selector('my_api')
    check python.has_docstrings(src)
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        
        // Check language declaration
        assert_eq!(program.languages.len(), 1);
        assert_eq!(program.languages[0].name.name, "python");
        assert_eq!(program.languages[0].connection_checks.len(), 2);
        
        // Check template
        assert_eq!(program.templates.len(), 1);
        assert_eq!(program.templates[0].component_requirements.len(), 2);
        
        // Check element
        assert_eq!(program.elements.len(), 1);
        assert_eq!(program.elements[0].scopes.len(), 1);
        assert!(program.elements[0].scopes[0].language.is_some());
    }

    // ========================================================================
    // Tests for V2 syntax: angular brackets, binds keyword, unbounded scopes
    // ========================================================================

    #[test]
    fn test_parse_v2_angular_bracket_language() {
        let source = r#"element test:
    scope src<rust> = rust.module_selector('test')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        assert_eq!(program.elements[0].scopes.len(), 1);
        
        let scope = &program.elements[0].scopes[0];
        assert_eq!(scope.name.name, "src");
        assert!(scope.language.is_some());
        assert_eq!(scope.language.as_ref().unwrap().name, "rust");
        assert!(scope.expression.is_some());
    }

    #[test]
    fn test_parse_v2_unbounded_scope_in_template() {
        let source = r#"template observable:
    element metrics:
        scope module<rust>
        connection_point prometheus: MetricsHandler
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.elements.len(), 1);
        
        let metrics = &template.elements[0];
        assert_eq!(metrics.scopes.len(), 1);
        
        let scope = &metrics.scopes[0];
        assert_eq!(scope.name.name, "module");
        assert!(scope.language.is_some());
        assert_eq!(scope.language.as_ref().unwrap().name, "rust");
        // Unbounded scope - no expression
        assert!(scope.expression.is_none());
        
        // Connection point - unbounded
        assert_eq!(metrics.refs.len(), 1);
        let cp = &metrics.refs[0];
        assert_eq!(cp.name.name, "prometheus");
        assert_eq!(cp.type_annotation.type_name.name, "MetricsHandler");
        assert!(cp.expression.is_none());
    }

    #[test]
    fn test_parse_v2_scope_with_binds() {
        let source = r#"element component implements observable:
    scope main_module<rust> binds observable.metrics.module = rust.module_selector('api')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        
        let element = &program.elements[0];
        assert_eq!(element.scopes.len(), 1);
        
        let scope = &element.scopes[0];
        assert_eq!(scope.name.name, "main_module");
        assert!(scope.language.is_some());
        assert_eq!(scope.language.as_ref().unwrap().name, "rust");
        
        // Check binds path
        assert!(scope.binds.is_some());
        let binds = scope.binds.as_ref().unwrap();
        assert_eq!(binds.len(), 3);
        assert_eq!(binds[0].name, "observable");
        assert_eq!(binds[1].name, "metrics");
        assert_eq!(binds[2].name, "module");
        
        // Check expression
        assert!(scope.expression.is_some());
    }

    #[test]
    fn test_parse_v2_connection_point_with_binds() {
        let source = r#"element component implements observable:
    connection_point handler: MetricsHandler binds observable.metrics.prometheus = rust.function_selector(module, 'handler')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        
        let element = &program.elements[0];
        assert_eq!(element.refs.len(), 1);
        
        let cp = &element.refs[0];
        assert_eq!(cp.name.name, "handler");
        assert_eq!(cp.type_annotation.type_name.name, "MetricsHandler");
        
        // Check binds path
        assert!(cp.binds.is_some());
        let binds = cp.binds.as_ref().unwrap();
        assert_eq!(binds.len(), 3);
        assert_eq!(binds[0].name, "observable");
        assert_eq!(binds[1].name, "metrics");
        assert_eq!(binds[2].name, "prometheus");
        
        // Check expression
        assert!(cp.expression.is_some());
    }

    #[test]
    fn test_parse_v2_complete_template_and_implementation() {
        let source = r#"template observable:
    allows language rust
    element metrics:
        scope module<rust>
        connection_point prometheus: MetricsHandler

element observable_component implements observable:
    scope main_module<rust> binds observable.metrics.module = rust.module_selector('payments::api')
    connection_point main_handler: MetricsHandler binds observable.metrics.prometheus = rust.function_selector(main_module, 'handler')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        
        // Check template
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.name.name, "observable");
        assert_eq!(template.elements.len(), 1);
        assert_eq!(template.component_requirements.len(), 1); // allows language rust
        
        // Check template element has unbounded scope
        let template_metrics = &template.elements[0];
        assert!(template_metrics.scopes[0].expression.is_none());
        assert!(template_metrics.refs[0].expression.is_none());
        
        // Check element implementation
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.name.name, "observable_component");
        assert_eq!(element.implements.len(), 1);
        assert_eq!(element.implements[0].template_name.name, "observable");
        
        // Check element has bound scope
        assert_eq!(element.scopes.len(), 1);
        let scope = &element.scopes[0];
        assert!(scope.binds.is_some());
        assert!(scope.expression.is_some());
        
        // Check element has bound connection point
        assert_eq!(element.refs.len(), 1);
        let cp = &element.refs[0];
        assert!(cp.binds.is_some());
        assert!(cp.expression.is_some());
    }

    #[test]
    fn test_parse_v2_unbounded_scope_in_element_fails() {
        // Unbounded scopes are only allowed in templates, not in regular elements
        let source = r#"element test:
    scope src<rust>
"#;
        let parser = Parser::new(source, "test.hie");
        let (_program, diagnostics) = parser.parse();

        // Should have errors because unbounded scope is not allowed in elements
        assert!(diagnostics.has_errors(), "Expected error for unbounded scope in element");
        let error_msg = diagnostics.iter().next().unwrap().message.clone();
        assert!(error_msg.contains("only allowed in templates"), "Error message should mention templates: {}", error_msg);
    }

    #[test]
    fn test_parse_v2_unbounded_connection_point_in_element_fails() {
        // Unbounded connection points are only allowed in templates, not in regular elements
        let source = r#"element test:
    connection_point api: HttpHandler
"#;
        let parser = Parser::new(source, "test.hie");
        let (_program, diagnostics) = parser.parse();

        // Should have errors because unbounded connection point is not allowed in elements
        assert!(diagnostics.has_errors(), "Expected error for unbounded connection point in element");
        let error_msg = diagnostics.iter().next().unwrap().message.clone();
        assert!(error_msg.contains("only allowed in templates"), "Error message should mention templates: {}", error_msg);
    }

    // ========================================================================
    // Tests for V3 syntax: curly brackets, ref keyword, uses keyword
    // ========================================================================

    #[test]
    fn test_parse_curly_brackets_element() {
        let source = r#"element test {
    scope src = files.folder_selector('src')
    check files.exists(src, 'main.py')
}
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        assert_eq!(program.elements[0].name.name, "test");
        assert_eq!(program.elements[0].scopes.len(), 1);
        assert_eq!(program.elements[0].checks.len(), 1);
    }

    #[test]
    fn test_parse_curly_brackets_template() {
        let source = r#"template observable {
    element metrics {
        scope module<rust>
        ref prometheus: MetricsHandler
    }
}
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        assert_eq!(program.templates[0].name.name, "observable");
        assert_eq!(program.templates[0].elements.len(), 1);
        
        let metrics = &program.templates[0].elements[0];
        assert_eq!(metrics.scopes.len(), 1);
        assert_eq!(metrics.refs.len(), 1);
        assert_eq!(metrics.refs[0].name.name, "prometheus");
    }

    #[test]
    fn test_parse_ref_keyword() {
        let source = r#"element api_service {
    ref port: integer = docker.exposed_port(dockerfile)
    ref url: string = config.get_api_url()
}
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.refs.len(), 2);
        
        assert_eq!(element.refs[0].name.name, "port");
        assert_eq!(element.refs[0].type_annotation.type_name.name, "integer");
        
        assert_eq!(element.refs[1].name.name, "url");
        assert_eq!(element.refs[1].type_annotation.type_name.name, "string");
    }

    #[test]
    fn test_parse_uses_declaration() {
        let source = r#"element core {
    element lexer {
        scope module = rust.module_selector('lexer')
    }
    element parser {
        scope module = rust.module_selector('parser')
        scope lexer_module = rust.module_selector('lexer')
        lexer_module uses lexer
    }
}
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        
        let core = &program.elements[0];
        assert_eq!(core.children.len(), 2);
        
        let parser_elem = &core.children[1];
        assert_eq!(parser_elem.uses.len(), 1);
        assert_eq!(parser_elem.uses[0].source.name, "lexer_module");
        assert_eq!(parser_elem.uses[0].target.len(), 1);
        assert_eq!(parser_elem.uses[0].target[0].name, "lexer");
    }

    #[test]
    fn test_parse_uses_qualified_target() {
        let source = r#"element parser {
    scope module = rust.module_selector('parser')
    module uses core.lexer
}
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        
        let parser_elem = &program.elements[0];
        assert_eq!(parser_elem.uses.len(), 1);
        assert_eq!(parser_elem.uses[0].source.name, "module");
        assert_eq!(parser_elem.uses[0].target.len(), 2);
        assert_eq!(parser_elem.uses[0].target[0].name, "core");
        assert_eq!(parser_elem.uses[0].target[1].name, "lexer");
    }

    #[test]
    fn test_parse_mixed_syntax() {
        // Test that curly brackets work within indentation-based templates
        let source = r#"element outer:
    element inner {
        scope src = files.folder_selector('src')
    }
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        assert_eq!(program.elements[0].children.len(), 1);
        assert_eq!(program.elements[0].children[0].name.name, "inner");
    }

    #[test]
    fn test_parse_nested_curly_brackets() {
        let source = r#"element system {
    element frontend {
        scope src = files.folder_selector('frontend')
    }
    element backend {
        scope src = files.folder_selector('backend')
        element api {
            scope module = rust.module_selector('api')
        }
    }
}
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        
        let system = &program.elements[0];
        assert_eq!(system.children.len(), 2);
        
        let backend = &system.children[1];
        assert_eq!(backend.children.len(), 1);
        assert_eq!(backend.children[0].name.name, "api");
    }
}
