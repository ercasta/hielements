//! Hielements Core Library
//!
//! This crate provides the core functionality for the Hielements language,
//! including lexing, parsing, semantic analysis, and evaluation.

pub mod ast;
pub mod diagnostics;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod stdlib;

pub use ast::*;
pub use diagnostics::{Diagnostic, DiagnosticSeverity, Diagnostics};
pub use interpreter::Interpreter;
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::Parser;
pub use span::Span;
