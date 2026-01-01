//! Parser for the Hielements language.

use crate::ast::*;
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::span::Span;

/// Expected tokens in element body for error messages.
/// Note: 'requires', 'allows', 'forbids' are only allowed in templates, not elements.
const EXPECTED_ELEMENT_BODY_TOKENS: &str = "'scope', 'connection_point', 'check', or 'element'";

/// Expected tokens in template body for error messages.
const EXPECTED_TEMPLATE_BODY_TOKENS: &str = "'scope', 'connection_point', 'check', 'element', 'requires', 'allows', or 'forbids'";

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
    fn parse_element(&mut self, doc_comment: Option<String>) -> Result<Element, Diagnostic> {
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
        
        self.expect(TokenKind::Colon)?;
        self.skip_newlines();
        self.expect(TokenKind::Indent)?;

        let mut scopes = Vec::new();
        let mut connection_points = Vec::new();
        let mut checks = Vec::new();
        let mut template_bindings = Vec::new();
        let mut children = Vec::new();

        loop {
            self.skip_newlines();

            if self.check(TokenKind::Dedent) || self.is_at_end() {
                break;
            }

            // Handle doc comments for nested elements
            let child_doc = self.parse_doc_comment();

            if self.check(TokenKind::Scope) {
                scopes.push(self.parse_scope()?);
            } else if self.check(TokenKind::ConnectionPoint) {
                connection_points.push(self.parse_connection_point()?);
            } else if self.check(TokenKind::Check) {
                checks.push(self.parse_check()?);
            } else if self.check(TokenKind::Element) {
                children.push(self.parse_element(child_doc)?);
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
                // Could be a template binding (e.g., template.element.scope = ...)
                // Peek ahead to see if this looks like a template binding (has a dot after the identifier)
                let pos = self.pos;
                self.advance(); // consume identifier
                if self.check(TokenKind::Dot) {
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
                    // Not a template binding (no dot after identifier)
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
            } else if self.check(TokenKind::Dedent) || self.is_at_end() {
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

        // Consume DEDENT if present
        if self.check(TokenKind::Dedent) {
            self.advance();
        }

        let end_span = self.previous_span();

        Ok(Element {
            doc_comment,
            name,
            implements,
            scopes,
            connection_points,
            checks,
            template_bindings,
            component_requirements: Vec::new(), // Always empty - requires/allows/forbids only allowed in templates
            children,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse a template declaration.
    fn parse_template(&mut self, doc_comment: Option<String>) -> Result<Template, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::Template)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        self.skip_newlines();
        self.expect(TokenKind::Indent)?;

        let mut scopes = Vec::new();
        let mut connection_points = Vec::new();
        let mut checks = Vec::new();
        let mut component_requirements = Vec::new();
        let mut elements = Vec::new();

        loop {
            self.skip_newlines();

            if self.check(TokenKind::Dedent) || self.is_at_end() {
                break;
            }

            // Handle doc comments for nested elements
            let child_doc = self.parse_doc_comment();

            if self.check(TokenKind::Scope) {
                scopes.push(self.parse_scope()?);
            } else if self.check(TokenKind::ConnectionPoint) {
                connection_points.push(self.parse_connection_point()?);
            } else if self.check(TokenKind::Check) {
                checks.push(self.parse_check()?);
            } else if self.check(TokenKind::Element) {
                elements.push(self.parse_element(child_doc)?);
            // Unified syntax: requires/allows/forbids [descendant] ...
            } else if self.check(TokenKind::Requires) {
                component_requirements.push(self.parse_component_requirement(RequirementAction::Requires)?);
            } else if self.check(TokenKind::Allows) {
                component_requirements.push(self.parse_component_requirement(RequirementAction::Allows)?);
            } else if self.check(TokenKind::Forbids) {
                component_requirements.push(self.parse_component_requirement(RequirementAction::Forbids)?);
            } else if self.check(TokenKind::Dedent) || self.is_at_end() {
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

        // Consume DEDENT if present
        if self.check(TokenKind::Dedent) {
            self.advance();
        }

        let end_span = self.previous_span();

        Ok(Template {
            doc_comment,
            name,
            scopes,
            connection_points,
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

    /// Parse a scope declaration.
    /// Syntax: `scope <name> [: <language>] = <expression>`
    fn parse_scope(&mut self) -> Result<ScopeDeclaration, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::Scope)?;
        let name = self.parse_identifier()?;
        
        // Check for optional language annotation: `: <language>`
        let language = if self.check(TokenKind::Colon) {
            self.advance(); // consume ':'
            Some(self.parse_identifier()?)
        } else {
            None
        };
        
        self.expect(TokenKind::Equals)?;
        let expression = self.parse_expression()?;
        self.expect_newline()?;
        let end_span = self.previous_span();

        Ok(ScopeDeclaration {
            name,
            language,
            expression,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse a connection point declaration.
    fn parse_connection_point(&mut self) -> Result<ConnectionPointDeclaration, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::ConnectionPoint)?;
        let name = self.parse_identifier()?;
        
        // Parse mandatory type annotation: `: <type>`
        self.expect(TokenKind::Colon)?;
        let type_annotation = self.parse_type_annotation()?;
        
        self.expect(TokenKind::Equals)?;
        let expression = self.parse_expression()?;
        self.expect_newline()?;
        let end_span = self.previous_span();

        Ok(ConnectionPointDeclaration {
            name,
            type_annotation,
            expression,
            span: start_span.merge(&end_span),
        })
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
        } else if self.check(TokenKind::ConnectionPoint) {
            self.parse_connection_point_component_spec()?
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
                    "Expected 'scope', 'check', 'element', 'connection', 'connection_point', 'implements', or 'language' after '{} {}', found {:?}",
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
                let mut connection_points = Vec::new();
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
                    } else if self.check(TokenKind::ConnectionPoint) {
                        connection_points.push(self.parse_connection_point()?);
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
                    connection_points,
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

    /// Parse a connection point component specification.
    /// Syntax: connection_point name: Type [= expression]
    fn parse_connection_point_component_spec(&mut self) -> Result<ComponentSpec, Diagnostic> {
        self.advance(); // consume 'connection_point'
        
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
        
        Ok(ComponentSpec::ConnectionPoint {
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
                TokenKind::ConnectionPoint | TokenKind::Template | TokenKind::Implements |
                TokenKind::To |
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
        assert_eq!(element.connection_points.len(), 3);
        
        // Check first connection point with type
        assert_eq!(element.connection_points[0].name.name, "port");
        assert_eq!(element.connection_points[0].type_annotation.type_name.name, "integer");
        
        // Check second connection point with type
        assert_eq!(element.connection_points[1].name.name, "url");
        assert_eq!(element.connection_points[1].type_annotation.type_name.name, "string");
        
        // Check third connection point with type
        assert_eq!(element.connection_points[2].name.name, "enabled");
        assert_eq!(element.connection_points[2].type_annotation.type_name.name, "boolean");
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
        assert_eq!(lexer.connection_points[0].name.name, "tokens");
        assert_eq!(lexer.connection_points[0].type_annotation.type_name.name, "TokenStream");
        
        // Check parser connection point with custom type
        let parser_elem = &template.elements[1];
        assert_eq!(parser_elem.connection_points[0].name.name, "ast");
        assert_eq!(parser_elem.connection_points[0].type_annotation.type_name.name, "AbstractSyntaxTree");
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
            ComponentSpec::ConnectionPoint { name, type_annotation, .. } => {
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
                assert_eq!(body.connection_points.len(), 1);
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
}
