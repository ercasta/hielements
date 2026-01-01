//! Lexer for the Hielements language.
//!
//! Uses the `logos` crate for efficient tokenization.

use logos::Logos;

use crate::span::{Position, Span};

/// Token kinds for the Hielements language.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]  // Skip spaces and tabs (not newlines)
pub enum TokenKind {
    // Keywords
    #[token("element")]
    Element,

    #[token("template")]
    Template,

    #[token("implements")]
    Implements,

    #[token("scope")]
    Scope,

    #[token("connection_point")]
    ConnectionPoint,

    #[token("check")]
    Check,

    #[token("import")]
    Import,

    #[token("from")]
    From,

    #[token("as")]
    As,

    #[token("true")]
    True,

    #[token("false")]
    False,

    // Transitivity keywords
    #[token("requires_descendant")]
    RequiresDescendant,

    #[token("allows_connection")]
    AllowsConnection,

    #[token("forbids_connection")]
    ForbidsConnection,

    #[token("requires_connection")]
    RequiresConnection,

    #[token("to")]
    To,

    // Punctuation
    #[token(":")]
    Colon,

    #[token("=")]
    Equals,

    #[token(".")]
    Dot,

    #[token(",")]
    Comma,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("*")]
    Star,

    // Newline (significant for indentation-based syntax)
    #[regex(r"\r?\n")]
    Newline,

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    // String literals
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringDouble,

    #[regex(r#"'([^'\\]|\\.)*'"#)]
    StringSingle,

    // Number literals
    #[regex(r"[0-9]+(\.[0-9]+)?")]
    Number,

    // Comments
    #[regex(r"###[^#]*###")]
    MultiLineComment,

    #[regex(r"##[^\n]*")]
    DocComment,

    #[regex(r"#[^\n]*")]
    Comment,

    // Indentation (handled specially during lexing)
    Indent,
    Dedent,

    // End of file
    Eof,
}

impl TokenKind {
    pub fn is_trivia(&self) -> bool {
        matches!(self, TokenKind::Comment | TokenKind::MultiLineComment)
    }
}

/// A token with its kind, text, and source span.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, text: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            text: text.into(),
            span,
        }
    }
}

/// Lexer for the Hielements language.
pub struct Lexer<'a> {
    source: &'a str,
    inner: logos::Lexer<'a, TokenKind>,
    indent_stack: Vec<usize>,
    pending_tokens: Vec<Token>,
    at_line_start: bool,
    current_line: usize,
    line_start_offset: usize,
    finished: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            inner: TokenKind::lexer(source),
            indent_stack: vec![0],
            pending_tokens: Vec::new(),
            at_line_start: true,
            current_line: 1,
            line_start_offset: 0,
            finished: false,
        }
    }

    /// Get the current position in the source.
    fn current_position(&self, offset: usize) -> Position {
        Position::new(
            self.current_line,
            offset - self.line_start_offset + 1,
            offset,
        )
    }

    /// Process indentation at the start of a line.
    fn process_indentation(&mut self, offset: usize) {
        // Count leading whitespace
        let mut indent = 0;
        let mut pos = offset;

        for ch in self.source[offset..].chars() {
            match ch {
                ' ' => {
                    indent += 1;
                    pos += 1;
                }
                '\t' => {
                    // Treat tabs as 4 spaces
                    indent += 4;
                    pos += 1;
                }
                '\n' | '\r' => {
                    // Empty line, skip indentation changes
                    return;
                }
                '#' => {
                    // Comment line - still track indentation for INDENT tokens
                    // but only if we're increasing indentation
                    let current_indent = *self.indent_stack.last().unwrap();
                    if indent > current_indent {
                        // Emit INDENT token
                        self.indent_stack.push(indent);
                        let span = Span::new(
                            self.current_position(offset),
                            self.current_position(pos),
                        );
                        self.pending_tokens.push(Token::new(TokenKind::Indent, "", span));
                    }
                    // Don't process DEDENT on comment-only lines
                    return;
                }
                _ => break,
            }
        }

        let current_indent = *self.indent_stack.last().unwrap();

        if indent > current_indent {
            // Emit INDENT token
            self.indent_stack.push(indent);
            let span = Span::new(
                self.current_position(offset),
                self.current_position(pos),
            );
            self.pending_tokens.push(Token::new(TokenKind::Indent, "", span));
        } else if indent < current_indent {
            // Emit DEDENT tokens for each level we're leaving
            while let Some(&top) = self.indent_stack.last() {
                if top <= indent {
                    break;
                }
                self.indent_stack.pop();
                let span = Span::new(
                    self.current_position(offset),
                    self.current_position(offset),
                );
                self.pending_tokens.push(Token::new(TokenKind::Dedent, "", span));
            }
        }
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> Token {
        // Return pending tokens first (INDENT/DEDENT)
        if let Some(token) = self.pending_tokens.pop() {
            return token;
        }

        if self.finished {
            let pos = Position::new(self.current_line, 1, self.source.len());
            return Token::new(TokenKind::Eof, "", Span::new(pos, pos));
        }

        loop {
            // Check for indentation at line start
            if self.at_line_start {
                self.at_line_start = false;
                self.process_indentation(self.line_start_offset);
                if let Some(token) = self.pending_tokens.pop() {
                    return token;
                }
            }

            match self.inner.next() {
                Some(Ok(kind)) => {
                    let span_range = self.inner.span();
                    let text = self.inner.slice();
                    let start_pos = self.current_position(span_range.start);
                    let end_pos = self.current_position(span_range.end);
                    let span = Span::new(start_pos, end_pos);

                    match kind {
                        TokenKind::Newline => {
                            self.current_line += 1;
                            self.line_start_offset = span_range.end;
                            self.at_line_start = true;
                            return Token::new(TokenKind::Newline, text, span);
                        }
                        TokenKind::Comment | TokenKind::MultiLineComment => {
                            // Skip comments
                            continue;
                        }
                        TokenKind::DocComment => {
                            return Token::new(TokenKind::DocComment, text, span);
                        }
                        _ => {
                            return Token::new(kind, text, span);
                        }
                    }
                }
                Some(Err(_)) => {
                    // Invalid token, skip and continue
                    continue;
                }
                None => {
                    // End of input - emit remaining DEDENTs
                    self.finished = true;
                    while self.indent_stack.len() > 1 {
                        self.indent_stack.pop();
                        let pos = Position::new(self.current_line, 1, self.source.len());
                        self.pending_tokens.push(Token::new(
                            TokenKind::Dedent,
                            "",
                            Span::new(pos, pos),
                        ));
                    }
                    if let Some(token) = self.pending_tokens.pop() {
                        return token;
                    }
                    let pos = Position::new(self.current_line, 1, self.source.len());
                    return Token::new(TokenKind::Eof, "", Span::new(pos, pos));
                }
            }
        }
    }

    /// Tokenize the entire source.
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_element() {
        let source = "element test:\n    scope x = files.folder('src')";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Element));
        assert!(kinds.contains(&&TokenKind::Identifier));
        assert!(kinds.contains(&&TokenKind::Colon));
        assert!(kinds.contains(&&TokenKind::Indent));
        assert!(kinds.contains(&&TokenKind::Scope));
    }

    #[test]
    fn test_string_literals() {
        let source = r#"'single' "double""#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert!(tokens.iter().any(|t| t.kind == TokenKind::StringSingle));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::StringDouble));
    }

    #[test]
    fn test_keywords() {
        let source = "element template implements scope connection_point check import from as true false";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Element));
        assert!(kinds.contains(&&TokenKind::Template));
        assert!(kinds.contains(&&TokenKind::Implements));
        assert!(kinds.contains(&&TokenKind::Scope));
        assert!(kinds.contains(&&TokenKind::ConnectionPoint));
        assert!(kinds.contains(&&TokenKind::Check));
        assert!(kinds.contains(&&TokenKind::Import));
        assert!(kinds.contains(&&TokenKind::From));
        assert!(kinds.contains(&&TokenKind::As));
        assert!(kinds.contains(&&TokenKind::True));
        assert!(kinds.contains(&&TokenKind::False));
    }

    #[test]
    fn test_template_keyword() {
        let source = "template compiler:\n    element lexer";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Template));
        assert!(kinds.contains(&&TokenKind::Identifier));
        assert!(kinds.contains(&&TokenKind::Colon));
        assert!(kinds.contains(&&TokenKind::Element));
    }

    #[test]
    fn test_implements_keyword() {
        let source = "element service implements microservice";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Element));
        assert!(kinds.contains(&&TokenKind::Implements));
    }

    #[test]
    fn test_transitivity_keywords() {
        let source = "requires_descendant allows_connection forbids_connection requires_connection to";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::RequiresDescendant));
        assert!(kinds.contains(&&TokenKind::AllowsConnection));
        assert!(kinds.contains(&&TokenKind::ForbidsConnection));
        assert!(kinds.contains(&&TokenKind::RequiresConnection));
        assert!(kinds.contains(&&TokenKind::To));
    }

    #[test]
    fn test_star_token() {
        let source = "database.*";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
        assert!(kinds.contains(&&TokenKind::Identifier));
        assert!(kinds.contains(&&TokenKind::Dot));
        assert!(kinds.contains(&&TokenKind::Star));
    }
}
