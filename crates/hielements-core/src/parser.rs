//! Parser for the Hielements language.

use crate::ast::*;
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::span::Span;

/// Expected tokens in element body for error messages.
const EXPECTED_ELEMENT_BODY_TOKENS: &str = "'scope', 'connection_point', 'check', 'element', 'requires_descendant', 'allows_connection', 'forbids_connection', or 'requires_connection'";

/// Expected tokens in template body for error messages.
const EXPECTED_TEMPLATE_BODY_TOKENS: &str = "'scope', 'connection_point', 'check', 'element', 'requires_descendant', 'allows_connection', 'forbids_connection', or 'requires_connection'";

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

        // Parse templates and top-level elements
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
            } else if !self.is_at_end() {
                let token = self.current();
                self.diagnostics.push(
                    Diagnostic::error("E001", format!("Expected 'template' or 'element', found {:?}", token.kind))
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
        let mut hierarchical_requirements = Vec::new();
        let mut connection_boundaries = Vec::new();
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
            } else if self.check(TokenKind::RequiresDescendant) {
                hierarchical_requirements.push(self.parse_hierarchical_requirement()?);
            } else if self.check(TokenKind::AllowsConnection) {
                connection_boundaries.push(self.parse_connection_boundary(ConnectionBoundaryKind::Allows)?);
            } else if self.check(TokenKind::ForbidsConnection) {
                connection_boundaries.push(self.parse_connection_boundary(ConnectionBoundaryKind::Forbids)?);
            } else if self.check(TokenKind::RequiresConnection) {
                connection_boundaries.push(self.parse_connection_boundary(ConnectionBoundaryKind::Requires)?);
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
                            "Expected 'scope', 'connection_point', 'check', 'element', 'requires_descendant', 'allows_connection', or 'forbids_connection', found {:?}",
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
            hierarchical_requirements,
            connection_boundaries,
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
        let mut hierarchical_requirements = Vec::new();
        let mut connection_boundaries = Vec::new();
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
            } else if self.check(TokenKind::RequiresDescendant) {
                hierarchical_requirements.push(self.parse_hierarchical_requirement()?);
            } else if self.check(TokenKind::AllowsConnection) {
                connection_boundaries.push(self.parse_connection_boundary(ConnectionBoundaryKind::Allows)?);
            } else if self.check(TokenKind::ForbidsConnection) {
                connection_boundaries.push(self.parse_connection_boundary(ConnectionBoundaryKind::Forbids)?);
            } else if self.check(TokenKind::RequiresConnection) {
                connection_boundaries.push(self.parse_connection_boundary(ConnectionBoundaryKind::Requires)?);
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
            hierarchical_requirements,
            connection_boundaries,
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
    fn parse_scope(&mut self) -> Result<ScopeDeclaration, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::Scope)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::Equals)?;
        let expression = self.parse_expression()?;
        self.expect_newline()?;
        let end_span = self.previous_span();

        Ok(ScopeDeclaration {
            name,
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

    /// Parse a hierarchical requirement (requires_descendant ...).
    fn parse_hierarchical_requirement(&mut self) -> Result<HierarchicalRequirement, Diagnostic> {
        let start_span = self.current_span();
        self.expect(TokenKind::RequiresDescendant)?;

        // Next token determines the kind: scope, check, or element
        let kind = if self.check(TokenKind::Scope) {
            HierarchicalRequirementKind::Scope(self.parse_scope()?)
        } else if self.check(TokenKind::Check) {
            HierarchicalRequirementKind::Check(self.parse_check()?)
        } else if self.check(TokenKind::Element) {
            let child_doc = self.parse_doc_comment();
            HierarchicalRequirementKind::Element(Box::new(self.parse_element(child_doc)?))
        } else {
            let token = self.current();
            return Err(Diagnostic::error(
                "E010",
                format!(
                    "Expected 'scope', 'check', or 'element' after 'requires_descendant', found {:?}",
                    token.kind
                ),
            )
            .with_file(&self.file_path)
            .with_span(token.span)
            .build());
        };

        let end_span = self.previous_span();
        Ok(HierarchicalRequirement {
            kind,
            span: start_span.merge(&end_span),
        })
    }

    /// Parse a connection boundary (allows_connection/forbids_connection/requires_connection to ...).
    fn parse_connection_boundary(&mut self, kind: ConnectionBoundaryKind) -> Result<ConnectionBoundary, Diagnostic> {
        let start_span = self.current_span();
        
        // Consume the keyword (AllowsConnection, ForbidsConnection, or RequiresConnection)
        self.advance();
        
        // Expect 'to' keyword
        self.expect(TokenKind::To)?;
        
        // Parse the connection pattern (e.g., api_gateway.public_api or database.*)
        let target_pattern = self.parse_connection_pattern()?;
        
        self.expect_newline()?;
        let end_span = self.previous_span();

        Ok(ConnectionBoundary {
            kind,
            target_pattern,
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
                TokenKind::To | TokenKind::RequiresDescendant | 
                TokenKind::AllowsConnection | TokenKind::ForbidsConnection |
                TokenKind::RequiresConnection => {
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

    #[test]
    fn test_parse_requires_descendant_scope() {
        let source = r#"element app:
    requires_descendant scope dockerfile = docker.file_selector('Dockerfile')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.hierarchical_requirements.len(), 1);
        
        match &element.hierarchical_requirements[0].kind {
            HierarchicalRequirementKind::Scope(scope) => {
                assert_eq!(scope.name.name, "dockerfile");
            }
            _ => panic!("Expected scope requirement"),
        }
    }

    #[test]
    fn test_parse_requires_descendant_check() {
        let source = r#"element app:
    requires_descendant check docker.has_healthcheck(dockerfile)
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.hierarchical_requirements.len(), 1);
        
        match &element.hierarchical_requirements[0].kind {
            HierarchicalRequirementKind::Check(_) => (),
            _ => panic!("Expected check requirement"),
        }
    }

    #[test]
    fn test_parse_requires_descendant_element() {
        let source = r#"element app:
    requires_descendant element metrics:
        scope module = rust.module_selector('metrics')
        connection_point prometheus: MetricsHandler = rust.function_selector(module, 'handler')
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.hierarchical_requirements.len(), 1);
        
        match &element.hierarchical_requirements[0].kind {
            HierarchicalRequirementKind::Element(elem) => {
                assert_eq!(elem.name.name, "metrics");
                assert_eq!(elem.scopes.len(), 1);
                assert_eq!(elem.connection_points.len(), 1);
            }
            _ => panic!("Expected element requirement"),
        }
    }

    #[test]
    fn test_parse_allows_connection() {
        let source = r#"element frontend:
    allows_connection to api_gateway.public_api
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.connection_boundaries.len(), 1);
        
        let boundary = &element.connection_boundaries[0];
        assert!(matches!(boundary.kind, ConnectionBoundaryKind::Allows));
        assert_eq!(boundary.target_pattern.path.len(), 2);
        assert_eq!(boundary.target_pattern.path[0].name, "api_gateway");
        assert_eq!(boundary.target_pattern.path[1].name, "public_api");
        assert!(!boundary.target_pattern.wildcard);
    }

    #[test]
    fn test_parse_forbids_connection_with_wildcard() {
        let source = r#"element secure_zone:
    forbids_connection to external.*
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.connection_boundaries.len(), 1);
        
        let boundary = &element.connection_boundaries[0];
        assert!(matches!(boundary.kind, ConnectionBoundaryKind::Forbids));
        assert_eq!(boundary.target_pattern.path.len(), 1);
        assert_eq!(boundary.target_pattern.path[0].name, "external");
        assert!(boundary.target_pattern.wildcard);
    }

    #[test]
    fn test_parse_template_with_hierarchical_checks() {
        let source = r#"template dockerized:
    requires_descendant scope dockerfile = docker.file_selector('Dockerfile')
    requires_descendant check docker.has_healthcheck(dockerfile)
    forbids_connection to external.*
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.templates.len(), 1);
        let template = &program.templates[0];
        assert_eq!(template.name.name, "dockerized");
        assert_eq!(template.hierarchical_requirements.len(), 2);
        assert_eq!(template.connection_boundaries.len(), 1);
    }

    #[test]
    fn test_parse_element_with_multiple_boundaries() {
        let source = r#"element secure_service:
    allows_connection to api.endpoint
    forbids_connection to database.*
    forbids_connection to external.network
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.connection_boundaries.len(), 3);
        
        assert!(matches!(element.connection_boundaries[0].kind, ConnectionBoundaryKind::Allows));
        assert!(matches!(element.connection_boundaries[1].kind, ConnectionBoundaryKind::Forbids));
        assert!(matches!(element.connection_boundaries[2].kind, ConnectionBoundaryKind::Forbids));
    }

    #[test]
    fn test_parse_requires_connection() {
        let source = r#"element service:
    requires_connection to logging.*
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.connection_boundaries.len(), 1);
        
        let boundary = &element.connection_boundaries[0];
        assert!(matches!(boundary.kind, ConnectionBoundaryKind::Requires));
        assert_eq!(boundary.target_pattern.path.len(), 1);
        assert_eq!(boundary.target_pattern.path[0].name, "logging");
        assert!(boundary.target_pattern.wildcard);
    }

    #[test]
    fn test_parse_all_connection_boundary_types() {
        let source = r#"element zone:
    allows_connection to api.*
    forbids_connection to database.*
    requires_connection to logging.output
"#;
        let parser = Parser::new(source, "test.hie");
        let (program, diagnostics) = parser.parse();

        assert!(!diagnostics.has_errors(), "Errors: {:?}", diagnostics);
        let program = program.unwrap();
        assert_eq!(program.elements.len(), 1);
        let element = &program.elements[0];
        assert_eq!(element.connection_boundaries.len(), 3);
        
        assert!(matches!(element.connection_boundaries[0].kind, ConnectionBoundaryKind::Allows));
        assert!(matches!(element.connection_boundaries[1].kind, ConnectionBoundaryKind::Forbids));
        assert!(matches!(element.connection_boundaries[2].kind, ConnectionBoundaryKind::Requires));
        
        // The requires_connection should not have wildcard
        assert!(!element.connection_boundaries[2].target_pattern.wildcard);
    }
}
