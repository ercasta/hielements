//! Parser for the Hielements language.

use crate::ast::*;
use crate::diagnostics::{Diagnostic, Diagnostics};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::span::Span;

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
        let mut elements = Vec::new();

        // Skip leading newlines
        self.skip_newlines();

        // Parse imports
        while self.check(TokenKind::Import) || self.check(TokenKind::From) {
            match self.parse_import() {
                Ok(import) => imports.push(import),
                Err(diag) => {
                    self.diagnostics.push(diag);
                    self.recover_to_newline();
                }
            }
            self.skip_newlines();
        }

        // Parse top-level elements
        while !self.is_at_end() {
            self.skip_newlines();
            if self.is_at_end() {
                break;
            }

            // Skip doc comments before elements
            let doc_comment = self.parse_doc_comment();

            if self.check(TokenKind::Element) {
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
                    Diagnostic::error("E001", format!("Expected 'element', found {:?}", token.kind))
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
        self.expect(TokenKind::Colon)?;
        self.skip_newlines();
        self.expect(TokenKind::Indent)?;

        let mut scopes = Vec::new();
        let mut connection_points = Vec::new();
        let mut checks = Vec::new();
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
            } else if self.check(TokenKind::Dedent) || self.is_at_end() {
                break;
            } else {
                let token = self.current();
                return Err(Diagnostic::error(
                    "E002",
                    format!(
                        "Expected 'scope', 'connection_point', 'check', or 'element', found {:?}",
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
            scopes,
            connection_points,
            checks,
            children,
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
        self.expect(TokenKind::Equals)?;
        let expression = self.parse_expression()?;
        self.expect_newline()?;
        let end_span = self.previous_span();

        Ok(ConnectionPointDeclaration {
            name,
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

    /// Parse an identifier.
    fn parse_identifier(&mut self) -> Result<Identifier, Diagnostic> {
        if self.check(TokenKind::Identifier) {
            let token = self.advance();
            Ok(Identifier::new(token.text, token.span))
        } else {
            let token = self.current();
            Err(Diagnostic::error("E004", format!("Expected identifier, found {:?}", token.kind))
                .with_file(&self.file_path)
                .with_span(token.span)
                .build())
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
}
